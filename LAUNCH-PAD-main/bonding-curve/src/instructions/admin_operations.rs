use anchor_lang::prelude::*;
use crate::{state::{Global, BondingCurve}, events::*, errors::*};

#[derive(Accounts)]
pub struct UpdateGlobalSettings<'info> {
    #[account(mut)]
    pub global: Account<'info, Global>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == global.admin_authority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for critical settings)
    #[account(
        constraint = multisig_authority.key() == global.multisig_authority
    )]
    pub multisig_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CollectPlatformFees<'info> {
    #[account(mut)]
    pub global: Account<'info, Global>,

    /// Platform fee collection wallet (multi-sig controlled)
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = platform_wallet.key() == global.platform_wallet
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == global.admin_authority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for fee collection)
    #[account(
        constraint = multisig_authority.key() == global.multisig_authority
    )]
    pub multisig_authority: Signer<'info>,

    /// Treasury account to receive fees
    /// CHECK: Destination account for fee collection
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CollectCreatorFees<'info> {
    #[account(
        constraint = !global.is_paused
    )]
    pub global: Account<'info, Global>,

    #[account(mut)]
    pub bonding_curve: Account<'info, BondingCurve>,

    /// Creator fee collection wallet (multi-sig controlled)
    /// CHECK: Validated against global configuration  
    #[account(
        mut,
        constraint = creator_wallet.key() == global.creator_wallet
    )]
    pub creator_wallet: UncheckedAccount<'info>,

    /// Token creator
    #[account(
        constraint = creator.key() == bonding_curve.creator 
            @ BondingCurveError::InvalidAccountOwner
    )]
    pub creator: Signer<'info>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == global.admin_authority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for fee collection)
    #[account(
        constraint = multisig_authority.key() == global.multisig_authority
    )]
    pub multisig_authority: Signer<'info>,

    /// Destination account for creator fees
    /// CHECK: Creator's designated fee collection account
    #[account(mut)]
    pub creator_fee_destination: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EmergencyPause<'info> {
    #[account(mut)]
    pub global: Account<'info, Global>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == global.admin_authority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for emergency operations)
    #[account(
        constraint = multisig_authority.key() == global.multisig_authority
    )]
    pub multisig_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumeOperations<'info> {
    #[account(mut)]
    pub global: Account<'info, Global>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == global.admin_authority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for resume operations)
    #[account(
        constraint = multisig_authority.key() == global.multisig_authority
    )]
    pub multisig_authority: Signer<'info>,
}

pub fn update_global_settings(
    ctx: Context<UpdateGlobalSettings>,
    platform_fee_basis_points: Option<u16>,
    creator_fee_basis_points: Option<u16>,
    migration_fee_basis_points: Option<u16>,
    migration_enabled: Option<bool>,
) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for critical settings
    global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Update platform fee if provided
    if let Some(platform_fee) = platform_fee_basis_points {
        require!(platform_fee <= 1000, BondingCurveError::FeeTooHigh); // Max 10%
        global.platform_fee_basis_points = platform_fee;
    }

    // Update creator fee if provided
    if let Some(creator_fee) = creator_fee_basis_points {
        require!(creator_fee <= 1000, BondingCurveError::FeeTooHigh); // Max 10%
        global.creator_fee_basis_points = creator_fee;
    }

    // Update migration fee if provided
    if let Some(migration_fee) = migration_fee_basis_points {
        require!(migration_fee <= 2000, BondingCurveError::FeeTooHigh); // Max 20%
        global.migration_fee_basis_points = migration_fee;
    }

    // Update migration enabled flag if provided
    if let Some(migration_flag) = migration_enabled {
        global.migration_enabled = migration_flag;
    }

    // Emit settings update event
    emit!(GlobalSettingsUpdatedEvent {
        admin_authority: global.admin_authority,
        multisig_authority: global.multisig_authority,
        platform_fee: global.platform_fee_basis_points,
        creator_fee: global.creator_fee_basis_points,
        migration_fee: global.migration_fee_basis_points,
        migration_enabled: global.migration_enabled,
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigOperationEvent {
        operation: "GLOBAL_SETTINGS_UPDATED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: global.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🔧 Global settings updated with multi-sig authorization");

    Ok(())
}

pub fn collect_platform_fees(ctx: Context<CollectPlatformFees>, amount: u64) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for fee collection
    global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Verify sufficient fees available
    require!(amount <= global.total_fees_collected, BondingCurveError::InsufficientFees);

    // Transfer fees from platform wallet to treasury
    **ctx.accounts.platform_wallet.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.treasury.to_account_info().try_borrow_mut_lamports()? += amount;

    // Update global fee tracking
    global.total_fees_collected = global.total_fees_collected
        .checked_sub(amount)
        .ok_or(BondingCurveError::Underflow)?;

    // Emit fee collection event
    emit!(PlatformFeesCollectedEvent {
        collector: ctx.accounts.admin_authority.key(),
        amount,
        destination: ctx.accounts.treasury.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigOperationEvent {
        operation: "PLATFORM_FEES_COLLECTED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: ctx.accounts.platform_wallet.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("💰 Platform fees collected: {} SOL", amount);

    Ok(())
}

pub fn collect_creator_fees(ctx: Context<CollectCreatorFees>, amount: u64) -> Result<()> {
    let global = &ctx.accounts.global;
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for fee collection
    global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Verify sufficient creator fees available
    require!(amount <= bonding_curve.creator_fees_collected, BondingCurveError::InsufficientFees);

    // Transfer fees from creator wallet to destination
    **ctx.accounts.creator_wallet.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.creator_fee_destination.to_account_info().try_borrow_mut_lamports()? += amount;

    // Update bonding curve fee tracking
    bonding_curve.creator_fees_collected = bonding_curve.creator_fees_collected
        .checked_sub(amount)
        .ok_or(BondingCurveError::Underflow)?;

    // Emit creator fee collection event
    emit!(CreatorFeesCollectedEvent {
        token_mint: bonding_curve.token_mint,
        creator: ctx.accounts.creator.key(),
        collector: ctx.accounts.admin_authority.key(),
        amount,
        destination: ctx.accounts.creator_fee_destination.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigOperationEvent {
        operation: "CREATOR_FEES_COLLECTED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: bonding_curve.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("💰 Creator fees collected for token {}: {} SOL", bonding_curve.token_mint, amount);

    Ok(())
}

pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for emergency pause
    global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Set pause flag
    global.is_paused = true;

    // Emit emergency pause event
    emit!(EmergencyPauseEvent {
        admin_authority: ctx.accounts.admin_authority.key(),
        multisig_authority: ctx.accounts.multisig_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Security alert
    emit!(SecurityAlertEvent {
        alert_type: "EMERGENCY_PAUSE".to_string(),
        details: "All operations have been paused by multi-sig authorities".to_string(),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🚨 EMERGENCY PAUSE ACTIVATED - All operations suspended");

    Ok(())
}

pub fn resume_operations(ctx: Context<ResumeOperations>) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for resume
    global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Clear pause flag
    global.is_paused = false;

    // Emit operations resumed event
    emit!(OperationsResumedEvent {
        admin_authority: ctx.accounts.admin_authority.key(),
        multisig_authority: ctx.accounts.multisig_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Security alert
    emit!(SecurityAlertEvent {
        alert_type: "OPERATIONS_RESUMED".to_string(),
        details: "All operations have been resumed by multi-sig authorities".to_string(),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ Operations resumed - Platform is operational");

    Ok(())
}