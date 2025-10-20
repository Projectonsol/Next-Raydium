use anchor_lang::prelude::*;
use crate::{constants::*, state::Global, events::*};

#[derive(Accounts)]
pub struct InitializeGlobal<'info> {
    #[account(
        init,
        payer = admin_authority,
        space = Global::LEN,
        seeds = [GLOBAL_SEED],
        bump
    )]
    pub global: Account<'info, Global>,

    #[account(
        mut,
        constraint = admin_authority.key().to_bytes() == ADMIN_WALLET_PUBKEY
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for Fort Knox security)
    #[account(
        constraint = multisig_authority.key().to_bytes() == MULTISIG_WALLET_PUBKEY
    )]
    pub multisig_authority: Signer<'info>,

    /// Platform fee collection wallet
    /// CHECK: Validated against constants
    #[account(
        constraint = platform_wallet.key().to_bytes() == PLATFORM_WALLET_PUBKEY
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// Creator fee collection wallet
    /// CHECK: Validated against constants
    #[account(
        constraint = creator_wallet.key().to_bytes() == CREATOR_WALLET_PUBKEY
    )]
    pub creator_wallet: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_global(ctx: Context<InitializeGlobal>) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let clock = Clock::get()?;

    // Set multi-sig authorities
    global.admin_authority = ctx.accounts.admin_authority.key();
    global.multisig_authority = ctx.accounts.multisig_authority.key();
    
    // Set fee collection wallets
    global.platform_wallet = ctx.accounts.platform_wallet.key();
    global.creator_wallet = ctx.accounts.creator_wallet.key();

    // Initialize fee settings
    global.platform_fee_basis_points = PLATFORM_FEE_BASIS_POINTS;
    global.creator_fee_basis_points = CREATOR_FEE_BASIS_POINTS;
    global.migration_fee_basis_points = MIGRATION_FEE_BASIS_POINTS;
    global.max_slippage_basis_points = MAX_SLIPPAGE_BASIS_POINTS;

    // Initialize flags
    global.migration_enabled = true;
    global.is_paused = false;

    // Initialize counters
    global.total_volume_sol = 0;
    global.total_fees_collected = 0;
    global.tokens_created = 0;
    global.successful_migrations = 0;
    global.version = 1;

    // Emit initialization event
    emit!(GlobalInitializedEvent {
        admin_authority: global.admin_authority,
        multisig_authority: global.multisig_authority,
        platform_wallet: global.platform_wallet,
        creator_wallet: global.creator_wallet,
        platform_fee: global.platform_fee_basis_points,
        creator_fee: global.creator_fee_basis_points,
        migration_fee: global.migration_fee_basis_points,
        timestamp: clock.unix_timestamp,
    });

    // Security audit log
    emit!(SecurityAlertEvent {
        alert_type: "GLOBAL_INITIALIZED".to_string(),
        details: "Multi-sig global configuration initialized with Fort Knox security".to_string(),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🔒 Global configuration initialized with multi-sig security");
    msg!("Admin Authority: {}", global.admin_authority);
    msg!("Multisig Authority: {}", global.multisig_authority);
    msg!("Platform Wallet: {}", global.platform_wallet);
    msg!("Creator Wallet: {}", global.creator_wallet);

    Ok(())
}