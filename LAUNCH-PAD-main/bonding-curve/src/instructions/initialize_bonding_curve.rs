use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, MintTo, SetAuthority},
};
use spl_token::instruction::AuthorityType;
use crate::{constants::*, state::{Global, BondingCurve}, events::*, errors::*};

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct InitializeBondingCurve<'info> {
    #[account(
        constraint = !global.is_paused
    )]
    pub global: Account<'info, Global>,

    #[account(
        init,
        payer = creator,
        space = BondingCurve::LEN,
        seeds = [BONDING_CURVE_SEED, token_mint.key().as_ref()],
        bump
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    #[account(
        init,
        payer = creator,
        mint::decimals = 9,
        mint::authority = creator,
        mint::freeze_authority = creator,
    )]
    pub token_mint: Account<'info, Mint>,

    /// SOL vault for bonding curve reserves (multi-sig protected)
    /// CHECK: This is a PDA owned by the system program
    #[account(
        init,
        payer = creator,
        seeds = [SOL_VAULT_SEED, token_mint.key().as_ref()],
        bump,
        space = 0
    )]
    pub sol_vault: AccountInfo<'info>,

    /// Token vault for bonding curve reserves (multi-sig protected)
    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = bonding_curve,
        seeds = [TOKEN_VAULT_SEED, token_mint.key().as_ref()],
        bump
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// LP reserve token account (multi-sig protected)
    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = bonding_curve,
        seeds = [LP_RESERVE_SEED, token_mint.key().as_ref()],
        bump
    )]
    pub lp_reserve_token_account: Account<'info, TokenAccount>,

    // Metadata removed for SolPG compatibility

    #[account(mut)]
    pub creator: Signer<'info>,

    /// Multi-sig authority required for LP reserve creation
    #[account(
        constraint = admin_authority.key() == global.admin_authority
    )]
    pub admin_authority: Signer<'info>,

    /// Second multi-sig authority
    #[account(
        constraint = multisig_authority.key() == global.multisig_authority
    )]
    pub multisig_authority: Signer<'info>,

    // Metadata program removed for SolPG compatibility
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize_bonding_curve(
    ctx: Context<InitializeBondingCurve>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for critical operation
    global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Validate input parameters
    require!(name.len() > 0 && name.len() <= 32, BondingCurveError::InvalidTokenName);
    require!(symbol.len() > 0 && symbol.len() <= 10, BondingCurveError::InvalidTokenSymbol);
    require!(uri.len() > 0 && uri.len() <= 200, BondingCurveError::InvalidMetadataUri);

    // Get bump seeds
    let bonding_curve_bump = ctx.bumps.bonding_curve;
    let sol_vault_bump = ctx.bumps.sol_vault;
    let token_vault_bump = ctx.bumps.token_vault;
    let lp_reserve_bump = ctx.bumps.lp_reserve_token_account;

    // Calculate supplies
    let total_supply = TOTAL_SUPPLY;
    let lp_reserve_supply = total_supply
        .checked_mul(LP_RESERVE_PERCENTAGE)
        .and_then(|x| x.checked_div(100))
        .ok_or(BondingCurveError::Overflow)?;
    let bonding_curve_supply = total_supply
        .checked_sub(lp_reserve_supply)
        .ok_or(BondingCurveError::Underflow)?;

    // Initialize bonding curve state
    bonding_curve.token_mint = ctx.accounts.token_mint.key();
    bonding_curve.creator = ctx.accounts.creator.key();
    bonding_curve.name = name.clone();
    bonding_curve.symbol = symbol.clone();
    bonding_curve.virtual_sol_reserves = VIRTUAL_SOL_RESERVES;
    bonding_curve.virtual_token_reserves = VIRTUAL_TOKEN_RESERVES;
    bonding_curve.real_sol_reserves = 0;
    bonding_curve.real_token_reserves = bonding_curve_supply;
    bonding_curve.lp_reserve_supply = lp_reserve_supply;
    bonding_curve.migration_threshold = MIGRATION_THRESHOLD;
    bonding_curve.migration_ready = false;
    bonding_curve.is_migrated = false;
    bonding_curve.amm_program_id = None;
    bonding_curve.amm_pool_address = None;
    bonding_curve.total_volume_sol = 0;
    bonding_curve.total_volume_tokens = 0;
    bonding_curve.platform_fees_collected = 0;
    bonding_curve.creator_fees_collected = 0;
    bonding_curve.buy_count = 0;
    bonding_curve.sell_count = 0;
    bonding_curve.created_at = clock.unix_timestamp;
    bonding_curve.last_trade_at = 0;
    bonding_curve.bump = bonding_curve_bump;
    bonding_curve.sol_vault_bump = sol_vault_bump;
    bonding_curve.token_vault_bump = token_vault_bump;
    bonding_curve.lp_reserve_bump = lp_reserve_bump;

    // Mint tokens to vaults using bonding curve authority
    let token_mint_key = ctx.accounts.token_mint.key();
    let seeds = &[
        BONDING_CURVE_SEED,
        token_mint_key.as_ref(),
        &[bonding_curve_bump],
    ];
    let signer = &[&seeds[..]];

    // Mint bonding curve supply to token vault
    let mint_to_vault_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.token_vault.to_account_info(),
            authority: bonding_curve.to_account_info(),
        },
        signer,
    );
    token::mint_to(mint_to_vault_ctx, bonding_curve_supply)?;

    // Mint LP reserve supply to LP reserve account (multi-sig protected)
    let mint_lp_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.token_mint.to_account_info(),
            to: ctx.accounts.lp_reserve_token_account.to_account_info(),
            authority: bonding_curve.to_account_info(),
        },
        signer,
    );
    token::mint_to(mint_lp_ctx, lp_reserve_supply)?;

    // 🔥 REVOKE MINT AND FREEZE AUTHORITIES FOR PERMANENT DECENTRALIZATION
    msg!("🔥 Revoking mint authority - making supply permanent...");
    
    // Revoke mint authority (no more tokens can ever be minted)
    let revoke_mint_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        SetAuthority {
            account_or_mint: ctx.accounts.token_mint.to_account_info(),
            current_authority: ctx.accounts.creator.to_account_info(),
        },
    );
    token::set_authority(revoke_mint_ctx, AuthorityType::MintTokens, None)?;

    // Revoke freeze authority (no accounts can ever be frozen)
    msg!("🔥 Revoking freeze authority - making accounts unfreezable...");
    let revoke_freeze_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        SetAuthority {
            account_or_mint: ctx.accounts.token_mint.to_account_info(),
            current_authority: ctx.accounts.creator.to_account_info(),
        },
    );
    token::set_authority(revoke_freeze_ctx, AuthorityType::FreezeAccount, None)?;

    // Token metadata creation removed for SolPG compatibility
    // Name and symbol will be stored in bonding curve state instead
    msg!("Token created: {} ({})", name, symbol);
    msg!("🔒 Mint & freeze authorities permanently revoked - fully decentralized!");

    // Update global counters
    global.tokens_created = global.tokens_created
        .checked_add(1)
        .ok_or(BondingCurveError::Overflow)?;

    // Emit events
    emit!(BondingCurveInitializedEvent {
        token_mint: bonding_curve.token_mint,
        creator: bonding_curve.creator,
        bonding_curve: bonding_curve.key(),
        sol_vault: ctx.accounts.sol_vault.key(),
        token_vault: ctx.accounts.token_vault.key(),
        lp_reserve: ctx.accounts.lp_reserve_token_account.key(),
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        total_supply,
        lp_reserve_supply,
        virtual_sol_reserves: bonding_curve.virtual_sol_reserves,
        virtual_token_reserves: bonding_curve.virtual_token_reserves,
        migration_threshold: bonding_curve.migration_threshold,
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigOperationEvent {
        operation: "BONDING_CURVE_INITIALIZED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: bonding_curve.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🔒 Bonding curve initialized with multi-sig protection");
    msg!("Token Mint: {}", bonding_curve.token_mint);
    msg!("Creator: {}", bonding_curve.creator);
    msg!("Total Supply: {} tokens", total_supply);
    msg!("LP Reserve: {} tokens ({}%)", lp_reserve_supply, LP_RESERVE_PERCENTAGE);
    msg!("Bonding Curve Supply: {} tokens", bonding_curve_supply);
    msg!("Migration Threshold: {} SOL", MIGRATION_THRESHOLD / 1_000_000_000);

    Ok(())
}