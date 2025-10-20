use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Token, TokenAccount, Transfer},
};
use crate::{constants::*, state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*, math::MathUtil};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        constraint = !amm_global.is_paused
            @ AmmError::OperationsPaused
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(
        mut,
        constraint = pool.status == POOL_STATUS_INITIALIZED || pool.status == POOL_STATUS_SWAP_ONLY
            @ AmmError::PoolDisabled
    )]
    pub pool: Account<'info, Pool>,

    /// Pool vault for input token (multi-sig protected)
    #[account(
        mut,
        constraint = input_vault.mint == input_token_account.mint
            @ AmmError::InvalidTokenAccount,
        constraint = (input_vault.key() == pool.vault_a || input_vault.key() == pool.vault_b)
            @ AmmError::InvalidTokenAccount
    )]
    pub input_vault: Account<'info, TokenAccount>,

    /// Pool vault for output token (multi-sig protected)
    #[account(
        mut,
        constraint = output_vault.mint == output_token_account.mint
            @ AmmError::InvalidTokenAccount,
        constraint = (output_vault.key() == pool.vault_a || output_vault.key() == pool.vault_b)
            @ AmmError::InvalidTokenAccount,
        constraint = input_vault.key() != output_vault.key()
            @ AmmError::InvalidTokenAccount
    )]
    pub output_vault: Account<'info, TokenAccount>,

    /// User's input token account
    #[account(
        mut,
        constraint = input_token_account.owner == user.key()
            @ AmmError::InvalidAccountOwner
    )]
    pub input_token_account: Account<'info, TokenAccount>,

    /// User's output token account
    #[account(
        mut,
        constraint = output_token_account.owner == user.key()
            @ AmmError::InvalidAccountOwner
    )]
    pub output_token_account: Account<'info, TokenAccount>,

    /// Tick array for current price range
    #[account(
        mut,
        constraint = tick_array.pool_id == pool.key()
            @ AmmError::InvalidTickArray
    )]
    pub tick_array: Account<'info, TickArray>,

    /// Platform fee collection wallet (multi-sig controlled)
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = platform_wallet.key() == amm_global.platform_wallet
            @ AmmError::PlatformWalletMismatch
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// Creator fee collection wallet (multi-sig controlled)
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = creator_wallet.key() == amm_global.creator_wallet
            @ AmmError::CreatorWalletMismatch
    )]
    pub creator_wallet: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn swap(
    ctx: Context<Swap>,
    amount: u64,
    other_amount_threshold: u64,
    sqrt_price_limit_x64: u128,
    is_base_input: bool,
) -> Result<()> {
    let amm_global = &mut ctx.accounts.amm_global;
    let pool = &mut ctx.accounts.pool;
    let tick_array = &mut ctx.accounts.tick_array;
    let clock = Clock::get()?;

    // Validate input amount
    require!(amount > 0, AmmError::InvalidTokenAmount);

    // Validate sqrt price limit
    require!(
        sqrt_price_limit_x64 >= MIN_SQRT_PRICE_X64 && sqrt_price_limit_x64 <= MAX_SQRT_PRICE_X64,
        AmmError::InvalidSqrtPrice
    );

    // Determine if this is a zero-for-one swap (token A for token B)
    let zero_for_one = if is_base_input {
        ctx.accounts.input_vault.key() == pool.vault_a
    } else {
        ctx.accounts.output_vault.key() == pool.vault_a
    };

    // Validate price limit direction
    if zero_for_one {
        require!(
            sqrt_price_limit_x64 < pool.sqrt_price_x64,
            AmmError::InvalidSqrtPrice
        );
    } else {
        require!(
            sqrt_price_limit_x64 > pool.sqrt_price_x64,
            AmmError::InvalidSqrtPrice
        );
    }

    // Check if user has sufficient input tokens
    require!(
        ctx.accounts.input_token_account.amount >= amount,
        AmmError::InsufficientTokenBalance
    );

    // Perform the swap calculation
    let (amount_in, amount_out, new_sqrt_price, new_tick) = calculate_swap(
        pool,
        tick_array,
        amount,
        sqrt_price_limit_x64,
        zero_for_one,
        is_base_input,
    )?;

    // Check slippage protection
    if is_base_input {
        require!(amount_out >= other_amount_threshold, AmmError::SlippageExceeded);
    } else {
        require!(amount_in <= other_amount_threshold, AmmError::SlippageExceeded);
    }

    // Calculate fees
    let trade_fee = amount_in
        .checked_mul(pool.trade_fee_rate as u64)
        .and_then(|x| x.checked_div(FEE_RATE_DENOMINATOR_VALUE))
        .ok_or(AmmError::Overflow)?;

    let protocol_fee = trade_fee
        .checked_mul(pool.protocol_fee_rate as u64)
        .and_then(|x| x.checked_div(FEE_RATE_DENOMINATOR_VALUE))
        .ok_or(AmmError::Overflow)?;

    let platform_fee = trade_fee
        .checked_mul(PLATFORM_FEE_BASIS_POINTS as u64)
        .and_then(|x| x.checked_div(BASIS_POINTS_DENOMINATOR))
        .ok_or(AmmError::Overflow)?;

    let creator_fee = trade_fee
        .checked_mul(CREATOR_FEE_BASIS_POINTS as u64)
        .and_then(|x| x.checked_div(BASIS_POINTS_DENOMINATOR))
        .ok_or(AmmError::Overflow)?;

    let net_amount_in = amount_in
        .checked_sub(trade_fee)
        .ok_or(AmmError::Underflow)?;

    // Transfer input tokens from user to pool
    let transfer_input_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.input_token_account.to_account_info(),
            to: ctx.accounts.input_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    );
    token::transfer(transfer_input_ctx, net_amount_in)?;

    // Transfer fees to respective wallets using pool authority
    let pool_seeds = &[
        POOL_SEED,
        pool.mint_a.as_ref(),
        pool.mint_b.as_ref(),
        &[pool.bump],
    ];
    let pool_signer = &[&pool_seeds[..]];

    // Transfer protocol fee to platform wallet
    if protocol_fee > 0 {
        let transfer_protocol_fee_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.input_vault.to_account_info(),
                to: ctx.accounts.platform_wallet.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_protocol_fee_ctx, protocol_fee)?;
    }

    // Transfer platform fee
    if platform_fee > 0 {
        let transfer_platform_fee_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.input_vault.to_account_info(),
                to: ctx.accounts.platform_wallet.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_platform_fee_ctx, platform_fee)?;
    }

    // Transfer creator fee
    if creator_fee > 0 {
        let transfer_creator_fee_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.input_vault.to_account_info(),
                to: ctx.accounts.creator_wallet.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_creator_fee_ctx, creator_fee)?;
    }

    // Transfer output tokens from pool to user
    let transfer_output_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.output_vault.to_account_info(),
            to: ctx.accounts.output_token_account.to_account_info(),
            authority: pool.to_account_info(),
        },
        pool_signer,
    );
    token::transfer(transfer_output_ctx, amount_out)?;

    // Update pool state
    pool.sqrt_price_x64 = new_sqrt_price;
    pool.tick_current = new_tick;
    pool.updated_at = clock.unix_timestamp;

    // Update protocol fees
    if zero_for_one {
        pool.protocol_fees_token_a = pool.protocol_fees_token_a
            .checked_add(protocol_fee)
            .ok_or(AmmError::Overflow)?;
        pool.total_volume_a = pool.total_volume_a
            .checked_add(amount_in)
            .ok_or(AmmError::Overflow)?;
    } else {
        pool.protocol_fees_token_b = pool.protocol_fees_token_b
            .checked_add(protocol_fee)
            .ok_or(AmmError::Overflow)?;
        pool.total_volume_b = pool.total_volume_b
            .checked_add(amount_in)
            .ok_or(AmmError::Overflow)?;
    }

    // Update global volume tracking
    amm_global.total_volume = amm_global.total_volume
        .checked_add(amount_in)
        .ok_or(AmmError::Overflow)?;

    amm_global.total_fees_collected = amm_global.total_fees_collected
        .checked_add(trade_fee)
        .ok_or(AmmError::Overflow)?;

    // Emit swap event
    emit!(SwapEvent {
        pool_id: pool.key(),
        user: ctx.accounts.user.key(),
        input_mint: ctx.accounts.input_token_account.mint,
        output_mint: ctx.accounts.output_token_account.mint,
        input_amount: amount_in,
        output_amount: amount_out,
        fee_amount: trade_fee,
        sqrt_price_x64: pool.sqrt_price_x64,
        tick_current: pool.tick_current,
        timestamp: clock.unix_timestamp,
    });

    msg!("🔄 Swap executed successfully");
    msg!("Input Amount: {} tokens", amount_in);
    msg!("Output Amount: {} tokens", amount_out);
    msg!("Trade Fee: {} tokens", trade_fee);
    msg!("Protocol Fee: {} tokens", protocol_fee);
    msg!("Platform Fee: {} tokens", platform_fee);
    msg!("Creator Fee: {} tokens", creator_fee);
    msg!("New Price: {}", new_sqrt_price);
    msg!("New Tick: {}", new_tick);

    Ok(())
}

// Simplified swap calculation (would be more complex in production)
fn calculate_swap(
    pool: &Pool,
    _tick_array: &TickArray,
    amount: u64,
    sqrt_price_limit_x64: u128,
    zero_for_one: bool,
    is_base_input: bool,
) -> Result<(u64, u64, u128, i32)> {
    // This is a simplified calculation
    // In production, this would involve complex CLMM math with tick arrays
    
    let current_sqrt_price = pool.sqrt_price_x64;
    
    // Simple constant product approximation for demo
    let amount_in = if is_base_input { amount } else { amount * 99 / 100 }; // Approximate input needed
    let amount_out = if is_base_input { amount * 99 / 100 } else { amount }; // Approximate output
    
    // Calculate new price (simplified)
    let price_impact = (amount_in as u128 * 100) / (pool.liquidity + 1); // Prevent division by zero
    let new_sqrt_price = if zero_for_one {
        current_sqrt_price.saturating_sub(price_impact)
    } else {
        current_sqrt_price.saturating_add(price_impact)
    };
    
    // Clamp to price limit
    let final_sqrt_price = if zero_for_one {
        new_sqrt_price.max(sqrt_price_limit_x64)
    } else {
        new_sqrt_price.min(sqrt_price_limit_x64)
    };
    
    // Calculate new tick
    let new_tick = MathUtil::sqrt_price_x64_to_tick(final_sqrt_price)?;
    
    Ok((amount_in, amount_out, final_sqrt_price, new_tick))
}