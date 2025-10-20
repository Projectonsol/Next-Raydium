use anchor_lang::prelude::*;
use crate::{constants::*, state::{AmmGlobal, Pool, Position, TickArray, PersonalPosition, PoolReward, ObservationState}, events::*, errors::*};

#[derive(Accounts)]
pub struct InitializeAmmGlobal<'info> {
    #[account(
        init,
        payer = admin_authority,
        space = AmmGlobal::LEN,
        seeds = [GLOBAL_SEED],
        bump
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(
        mut,
        constraint = admin_authority.key().to_bytes() == ADMIN_WALLET_BYTES
            @ AmmError::InvalidAdminAuthority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for Fort Knox security)
    #[account(
        constraint = multisig_authority.key().to_bytes() == MULTISIG_WALLET_BYTES
            @ AmmError::InvalidMultisigAuthority
    )]
    pub multisig_authority: Signer<'info>,

    /// Platform fee collection wallet
    /// CHECK: Validated against constants
    #[account(
        constraint = platform_wallet.key().to_bytes() == PLATFORM_WALLET_BYTES
            @ AmmError::PlatformWalletMismatch
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// Creator fee collection wallet
    /// CHECK: Validated against constants
    #[account(
        constraint = creator_wallet.key().to_bytes() == CREATOR_WALLET_BYTES
            @ AmmError::CreatorWalletMismatch
    )]
    pub creator_wallet: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_amm_global(ctx: Context<InitializeAmmGlobal>) -> Result<()> {
    let amm_global = &mut ctx.accounts.amm_global;
    let clock = Clock::get()?;

    // Set multi-sig authorities
    amm_global.admin_authority = ctx.accounts.admin_authority.key();
    amm_global.multisig_authority = ctx.accounts.multisig_authority.key();
    
    // Set fee collection wallets
    amm_global.platform_wallet = ctx.accounts.platform_wallet.key();
    amm_global.creator_wallet = ctx.accounts.creator_wallet.key();

    // Initialize fee settings
    amm_global.protocol_fee_rate = DEFAULT_PROTOCOL_FEE_RATE;
    amm_global.fund_fee_rate = DEFAULT_FUND_FEE_RATE;
    amm_global.default_trade_fee_rate = DEFAULT_TRADE_FEE_RATE;
    amm_global.create_pool_fee = 1_000_000_000; // 1 SOL

    // Initialize flags and counters
    amm_global.is_paused = false;
    amm_global.total_pools = 0;
    amm_global.total_volume = 0;
    amm_global.total_fees_collected = 0;
    amm_global.version = 1;

    // Emit initialization event
    emit!(AmmGlobalInitializedEvent {
        admin_authority: amm_global.admin_authority,
        multisig_authority: amm_global.multisig_authority,
        platform_wallet: amm_global.platform_wallet,
        creator_wallet: amm_global.creator_wallet,
        protocol_fee_rate: amm_global.protocol_fee_rate,
        fund_fee_rate: amm_global.fund_fee_rate,
        default_trade_fee_rate: amm_global.default_trade_fee_rate,
        create_pool_fee: amm_global.create_pool_fee,
        timestamp: clock.unix_timestamp,
    });

    // Security audit log
    emit!(SecurityAmmAlertEvent {
        alert_type: "AMM_GLOBAL_INITIALIZED".to_string(),
        details: "Multi-sig AMM global configuration initialized with Fort Knox security".to_string(),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🔒 AMM Global configuration initialized with multi-sig security");
    msg!("Admin Authority: {}", amm_global.admin_authority);
    msg!("Multisig Authority: {}", amm_global.multisig_authority);
    msg!("Platform Wallet: {}", amm_global.platform_wallet);
    msg!("Creator Wallet: {}", amm_global.creator_wallet);
    msg!("Protocol Fee Rate: {}%", amm_global.protocol_fee_rate as f64 / 10000.0);
    msg!("Trade Fee Rate: {}%", amm_global.default_trade_fee_rate as f64 / 10000.0);

    Ok(())
}