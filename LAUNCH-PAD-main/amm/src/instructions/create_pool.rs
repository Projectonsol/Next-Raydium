use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::{constants::*, state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*, math::MathUtil};

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(
        mut,
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(
        init,
        payer = pool_creator,
        space = Pool::LEN,
        seeds = [POOL_SEED, mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// Token A mint (usually SOL or WSOL)
    pub mint_a: Account<'info, Mint>,

    /// Token B mint (custom token from bonding curve)
    pub mint_b: Account<'info, Mint>,

    /// Pool vault for token A (multi-sig protected)
    #[account(
        init,
        payer = pool_creator,
        token::mint = mint_a,
        token::authority = pool,
        seeds = [POOL_VAULT_SEED, pool.key().as_ref(), mint_a.key().as_ref()],
        bump
    )]
    pub vault_a: Account<'info, TokenAccount>,

    /// Pool vault for token B (multi-sig protected)
    #[account(
        init,
        payer = pool_creator,
        token::mint = mint_b,
        token::authority = pool,
        seeds = [POOL_VAULT_SEED, pool.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == amm_global.admin_authority
            @ AmmError::InvalidAdminAuthority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for pool creation)
    #[account(
        constraint = multisig_authority.key() == amm_global.multisig_authority
            @ AmmError::InvalidMultisigAuthority
    )]
    pub multisig_authority: Signer<'info>,

    /// Pool creator (pays for creation)
    #[account(mut)]
    pub pool_creator: Signer<'info>,

    /// Platform wallet for creation fees
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = platform_wallet.key() == amm_global.platform_wallet
            @ AmmError::PlatformWalletMismatch
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_pool(
    ctx: Context<CreatePool>,
    sqrt_price_x64: u128,
    tick_spacing: u16,
) -> Result<()> {
    let amm_global = &mut ctx.accounts.amm_global;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for critical pool creation
    amm_global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Validate tick spacing
    require!(
        tick_spacing == TICK_SPACING_10 || 
        tick_spacing == TICK_SPACING_60 || 
        tick_spacing == TICK_SPACING_200,
        AmmError::InvalidTickSpacing
    );

    // Validate sqrt price
    require!(
        sqrt_price_x64 >= MIN_SQRT_PRICE_X64 && sqrt_price_x64 <= MAX_SQRT_PRICE_X64,
        AmmError::InvalidSqrtPrice
    );

    // Collect pool creation fee
    let creation_fee = amm_global.create_pool_fee;
    require!(
        ctx.accounts.pool_creator.lamports() >= creation_fee,
        AmmError::PoolCreationFeeNotPaid
    );

    // Transfer creation fee to platform wallet
    let transfer_fee_ix = anchor_lang::system_program::Transfer {
        from: ctx.accounts.pool_creator.to_account_info(),
        to: ctx.accounts.platform_wallet.to_account_info(),
    };
    anchor_lang::system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_fee_ix,
        ),
        creation_fee,
    )?;

    // Calculate initial tick from sqrt price
    let tick_current = MathUtil::sqrt_price_x64_to_tick(sqrt_price_x64)?;

    // Initialize pool state
    pool.id = pool.key();
    pool.mint_a = ctx.accounts.mint_a.key();
    pool.mint_b = ctx.accounts.mint_b.key();
    pool.vault_a = ctx.accounts.vault_a.key();
    pool.vault_b = ctx.accounts.vault_b.key();
    pool.bump = ctx.bumps.pool;
    pool.sqrt_price_x64 = sqrt_price_x64;
    pool.tick_current = tick_current;
    pool.tick_spacing = tick_spacing;
    pool.status = POOL_STATUS_INITIALIZED;
    pool.trade_fee_rate = amm_global.default_trade_fee_rate;
    pool.protocol_fee_rate = amm_global.protocol_fee_rate;
    pool.fund_fee_rate = amm_global.fund_fee_rate;
    pool.liquidity = 0;
    pool.protocol_fees_token_a = 0;
    pool.protocol_fees_token_b = 0;
    pool.fund_fees_token_a = 0;
    pool.fund_fees_token_b = 0;
    pool.fee_growth_global_a_x64 = 0;
    pool.fee_growth_global_b_x64 = 0;
    pool.total_volume_a = 0;
    pool.total_volume_b = 0;
    pool.created_at = clock.unix_timestamp;
    pool.updated_at = clock.unix_timestamp;

    // Initialize reward infos (empty initially)
    pool.reward_infos = [Default::default(); 3];

    // Update global counters
    amm_global.total_pools = amm_global.total_pools
        .checked_add(1)
        .ok_or(AmmError::Overflow)?;

    amm_global.total_fees_collected = amm_global.total_fees_collected
        .checked_add(creation_fee)
        .ok_or(AmmError::Overflow)?;

    // Emit pool creation event
    emit!(PoolCreatedEvent {
        pool_id: pool.key(),
        mint_a: pool.mint_a,
        mint_b: pool.mint_b,
        vault_a: pool.vault_a,
        vault_b: pool.vault_b,
        sqrt_price_x64: pool.sqrt_price_x64,
        tick_current: pool.tick_current,
        tick_spacing: pool.tick_spacing,
        trade_fee_rate: pool.trade_fee_rate,
        protocol_fee_rate: pool.protocol_fee_rate,
        fund_fee_rate: pool.fund_fee_rate,
        created_by: ctx.accounts.pool_creator.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigAmmOperationEvent {
        operation: "POOL_CREATED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: pool.key(),
        timestamp: clock.unix_timestamp,
    });

    // Security alert for critical operation
    emit!(SecurityAmmAlertEvent {
        alert_type: "CRITICAL_POOL_CREATION".to_string(),
        details: format!(
            "CLMM pool created with multi-sig authorization: {} / {}",
            pool.mint_a,
            pool.mint_b
        ),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🏊 CLMM Pool created successfully with multi-sig protection");
    msg!("Pool ID: {}", pool.key());
    msg!("Token A: {}", pool.mint_a);
    msg!("Token B: {}", pool.mint_b);
    msg!("Initial Price: {}", sqrt_price_x64);
    msg!("Tick Current: {}", tick_current);
    msg!("Tick Spacing: {}", tick_spacing);
    msg!("Trade Fee: {}%", pool.trade_fee_rate as f64 / 10000.0);
    msg!("Protocol Fee: {}%", pool.protocol_fee_rate as f64 / 10000.0);
    msg!("Creation Fee Paid: {} SOL", creation_fee as f64 / 1_000_000_000.0);

    Ok(())
}