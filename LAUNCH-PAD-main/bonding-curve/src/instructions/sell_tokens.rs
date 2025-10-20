use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::{constants::*, state::{Global, BondingCurve, UserVolumeAccumulator}, events::*, errors::*};

#[derive(Accounts)]
pub struct SellTokens<'info> {
    #[account(
        constraint = !global.is_paused
    )]
    pub global: Account<'info, Global>,

    #[account(
        mut,
        constraint = !bonding_curve.is_migrated
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    /// Token mint
    #[account(
        constraint = token_mint.key() == bonding_curve.token_mint
    )]
    pub token_mint: Account<'info, Mint>,

    /// SOL vault (multi-sig protected)
    #[account(
        mut,
        seeds = [SOL_VAULT_SEED, token_mint.key().as_ref()],
        bump = bonding_curve.sol_vault_bump
    )]
    /// CHECK: This is a PDA owned by the system program
    pub sol_vault: AccountInfo<'info>,

    /// Token vault (multi-sig protected)
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = bonding_curve,
        seeds = [TOKEN_VAULT_SEED, token_mint.key().as_ref()],
        bump = bonding_curve.token_vault_bump
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// User's token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = seller
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// User volume accumulator
    #[account(
        mut,
        seeds = [USER_VOLUME_SEED, seller.key().as_ref()],
        bump = user_volume_accumulator.bump
    )]
    pub user_volume_accumulator: Account<'info, UserVolumeAccumulator>,

    /// Platform fee collection wallet (multi-sig controlled)
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = platform_wallet.key() == global.platform_wallet
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// Creator fee collection wallet (multi-sig controlled)
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = creator_wallet.key() == global.creator_wallet
    )]
    pub creator_wallet: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn sell_tokens(
    ctx: Context<SellTokens>,
    token_amount: u64,
    min_sol_received: u64,
) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let clock = Clock::get()?;

    // Enhanced validation using new security method
    require!(min_sol_received > 0, BondingCurveError::InvalidSolAmount);
    bonding_curve.validate_trade_amounts(token_amount, false)?;
    
    // Check if user has enough tokens
    require!(
        ctx.accounts.user_token_account.amount >= token_amount,
        BondingCurveError::InsufficientTokenReserves
    );

    // Calculate SOL received using constant product formula
    let sol_received = calculate_sell_proceeds(
        token_amount,
        bonding_curve.virtual_sol_reserves,
        bonding_curve.virtual_token_reserves,
        bonding_curve.real_sol_reserves,
        bonding_curve.real_token_reserves,
    )?;

    // Check slippage protection
    require!(
        sol_received >= min_sol_received,
        BondingCurveError::SlippageExceeded
    );

    // Calculate fees
    let platform_fee = sol_received
        .checked_mul(global.platform_fee_basis_points as u64)
        .and_then(|x| x.checked_div(BASIS_POINTS_DENOMINATOR))
        .ok_or(BondingCurveError::Overflow)?;

    let creator_fee = sol_received
        .checked_mul(global.creator_fee_basis_points as u64)
        .and_then(|x| x.checked_div(BASIS_POINTS_DENOMINATOR))
        .ok_or(BondingCurveError::Overflow)?;

    let net_sol_received = sol_received
        .checked_sub(platform_fee)
        .and_then(|x| x.checked_sub(creator_fee))
        .ok_or(BondingCurveError::Underflow)?;

    // Check if sol vault has enough SOL
    require!(
        ctx.accounts.sol_vault.lamports() >= sol_received,
        BondingCurveError::InsufficientSolReserves
    );

    // Transfer tokens from user to vault
    let transfer_tokens_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        },
    );
    token::transfer(transfer_tokens_ctx, token_amount)?;

    // 🔐 SECURE CPI TRANSFERS: Use proper signed transfers instead of dangerous direct manipulation
    
    // Get vault authority for signed transfers
    let token_mint_key = bonding_curve.token_mint.key();
    let vault_seeds = &[
        SOL_VAULT_SEED,
        token_mint_key.as_ref(),
        &[bonding_curve.sol_vault_bump],
    ];
    let vault_signer = &[&vault_seeds[..]];

    // Transfer net SOL to seller from vault using secure CPI
    require!(net_sol_received > 0, BondingCurveError::ZeroAmountTransfer);
    let transfer_to_seller = anchor_lang::system_program::Transfer {
        from: ctx.accounts.sol_vault.to_account_info(),
        to: ctx.accounts.seller.to_account_info(),
    };
    anchor_lang::system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            transfer_to_seller,
            vault_signer,
        ),
        net_sol_received,
    )?;

    // Transfer platform fee from vault to platform wallet using secure CPI
    if platform_fee > 0 {
        let transfer_platform_fee = anchor_lang::system_program::Transfer {
            from: ctx.accounts.sol_vault.to_account_info(),
            to: ctx.accounts.platform_wallet.to_account_info(),
        };
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                transfer_platform_fee,
                vault_signer,
            ),
            platform_fee,
        )?;
    }

    // Transfer creator fee from vault to creator wallet using secure CPI
    if creator_fee > 0 {
        let transfer_creator_fee = anchor_lang::system_program::Transfer {
            from: ctx.accounts.sol_vault.to_account_info(),
            to: ctx.accounts.creator_wallet.to_account_info(),
        };
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                transfer_creator_fee,
                vault_signer,
            ),
            creator_fee,
        )?;
    }

    // Update bonding curve reserves
    bonding_curve.real_sol_reserves = bonding_curve.real_sol_reserves
        .checked_sub(sol_received)
        .ok_or(BondingCurveError::Underflow)?;

    bonding_curve.real_token_reserves = bonding_curve.real_token_reserves
        .checked_add(token_amount)
        .ok_or(BondingCurveError::Overflow)?;

    // Update volume tracking
    bonding_curve.total_volume_sol = bonding_curve.total_volume_sol
        .checked_add(sol_received)
        .ok_or(BondingCurveError::Overflow)?;

    bonding_curve.total_volume_tokens = bonding_curve.total_volume_tokens
        .checked_add(token_amount)
        .ok_or(BondingCurveError::Overflow)?;

    bonding_curve.platform_fees_collected = bonding_curve.platform_fees_collected
        .checked_add(platform_fee)
        .ok_or(BondingCurveError::Overflow)?;

    bonding_curve.creator_fees_collected = bonding_curve.creator_fees_collected
        .checked_add(creator_fee)
        .ok_or(BondingCurveError::Overflow)?;

    bonding_curve.sell_count = bonding_curve.sell_count
        .checked_add(1)
        .ok_or(BondingCurveError::Overflow)?;

    bonding_curve.last_trade_at = clock.unix_timestamp;

    // Update global tracking
    global.total_volume_sol = global.total_volume_sol
        .checked_add(sol_received)
        .ok_or(BondingCurveError::Overflow)?;

    global.total_fees_collected = global.total_fees_collected
        .checked_add(platform_fee)
        .ok_or(BondingCurveError::Overflow)?;

    // Update user volume accumulator
    let user_volume = &mut ctx.accounts.user_volume_accumulator;
    user_volume.volume_sol = user_volume.volume_sol
        .checked_add(sol_received)
        .ok_or(BondingCurveError::Overflow)?;

    user_volume.volume_tokens = user_volume.volume_tokens
        .checked_add(token_amount)
        .ok_or(BondingCurveError::Overflow)?;

    user_volume.trades_count = user_volume.trades_count
        .checked_add(1)
        .ok_or(BondingCurveError::Overflow)?;

    user_volume.last_trade_timestamp = clock.unix_timestamp;

    // Calculate new price for event
    let new_price = bonding_curve.current_price()?;

    // Emit sell event
    emit!(TokensSoldEvent {
        token_mint: bonding_curve.token_mint,
        seller: ctx.accounts.seller.key(),
        token_amount,
        sol_received: net_sol_received,
        platform_fee,
        creator_fee,
        new_sol_reserves: bonding_curve.real_sol_reserves,
        new_token_reserves: bonding_curve.real_token_reserves,
        new_price,
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ Tokens sold successfully");
    msg!("Amount: {} tokens", token_amount);
    msg!("Gross SOL Received: {} SOL", sol_received);
    msg!("Net SOL Received: {} SOL", net_sol_received);
    msg!("Platform Fee: {} SOL", platform_fee);
    msg!("Creator Fee: {} SOL", creator_fee);
    msg!("New Price: {} SOL per token", new_price);

    Ok(())
}

// 🔒 SECURE SOL proceeds calculation with manipulation protection
fn calculate_sell_proceeds(
    token_amount: u64,
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    real_sol_reserves: u64,
    real_token_reserves: u64,
) -> Result<u64> {
    // Anti-manipulation checks
    require!(virtual_sol_reserves > 0, BondingCurveError::InvalidPrice);
    require!(virtual_token_reserves > 0, BondingCurveError::InvalidPrice);
    require!(token_amount > 0, BondingCurveError::InvalidTokenAmount);
    require!(real_sol_reserves > 0, BondingCurveError::InsufficientSolReserves);
    // Use virtual reserves for pricing calculation
    let current_virtual_sol = virtual_sol_reserves
        .checked_add(real_sol_reserves)
        .ok_or(BondingCurveError::Overflow)?;
    
    let current_virtual_tokens = virtual_token_reserves
        .checked_sub(real_token_reserves)
        .ok_or(BondingCurveError::Underflow)?;

    let new_virtual_tokens = current_virtual_tokens
        .checked_add(token_amount)
        .ok_or(BondingCurveError::Overflow)?;

    // k = x * y (constant product)
    let k = current_virtual_sol
        .checked_mul(current_virtual_tokens)
        .ok_or(BondingCurveError::Overflow)?;

    // new_sol = k / new_tokens
    let new_virtual_sol = k
        .checked_div(new_virtual_tokens)
        .ok_or(BondingCurveError::DivisionByZero)?;

    // proceeds = current_sol - new_sol
    let sol_proceeds = current_virtual_sol
        .checked_sub(new_virtual_sol)
        .ok_or(BondingCurveError::Underflow)?;

    Ok(sol_proceeds)
}