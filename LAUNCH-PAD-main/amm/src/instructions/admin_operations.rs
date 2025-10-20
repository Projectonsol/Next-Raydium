use anchor_lang::prelude::*;
use crate::{state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*};

#[derive(Accounts)]
pub struct UpdatePoolFees<'info> {
    #[account(
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == amm_global.admin_authority 
            
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for fee updates)
    #[account(
        constraint = multisig_authority.key() == amm_global.multisig_authority 
            
    )]
    pub multisig_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct EmergencyPauseAmm<'info> {
    #[account(mut)]
    pub amm_global: Account<'info, AmmGlobal>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == amm_global.admin_authority 
            
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for emergency operations)
    #[account(
        constraint = multisig_authority.key() == amm_global.multisig_authority 
            
    )]
    pub multisig_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ResumeAmmOperations<'info> {
    #[account(mut)]
    pub amm_global: Account<'info, AmmGlobal>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == amm_global.admin_authority 
            
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for resume operations)
    #[account(
        constraint = multisig_authority.key() == amm_global.multisig_authority 
            
    )]
    pub multisig_authority: Signer<'info>,
}

pub fn update_pool_fees(
    ctx: Context<UpdatePoolFees>,
    trade_fee_rate: u32,
    protocol_fee_rate: u32,
    fund_fee_rate: u32,
) -> Result<()> {
    let amm_global = &ctx.accounts.amm_global;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for critical fee updates
    amm_global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Validate fee rates
    require!(trade_fee_rate <= 100000, AmmError::FeeTooHigh); // Max 10%
    require!(protocol_fee_rate <= 200000, AmmError::FeeTooHigh); // Max 20%
    require!(fund_fee_rate <= 200000, AmmError::FeeTooHigh); // Max 20%

    // Update pool fee rates
    pool.trade_fee_rate = trade_fee_rate;
    pool.protocol_fee_rate = protocol_fee_rate;
    pool.fund_fee_rate = fund_fee_rate;
    pool.updated_at = clock.unix_timestamp;

    // Emit pool fees updated event
    emit!(PoolFeesUpdatedEvent {
        pool_id: pool.key(),
        trade_fee_rate,
        protocol_fee_rate,
        fund_fee_rate,
        admin_authority: ctx.accounts.admin_authority.key(),
        multisig_authority: ctx.accounts.multisig_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigAmmOperationEvent {
        operation: "POOL_FEES_UPDATED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: pool.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🔧 Pool fees updated with multi-sig authorization");
    msg!("Pool: {}", pool.key());
    msg!("Trade Fee: {}%", trade_fee_rate as f64 / 10000.0);
    msg!("Protocol Fee: {}%", protocol_fee_rate as f64 / 10000.0);
    msg!("Fund Fee: {}%", fund_fee_rate as f64 / 10000.0);

    Ok(())
}

pub fn emergency_pause_amm(ctx: Context<EmergencyPauseAmm>) -> Result<()> {
    let amm_global = &mut ctx.accounts.amm_global;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for emergency pause
    amm_global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Set pause flag
    amm_global.is_paused = true;

    // Emit emergency pause event
    emit!(EmergencyPauseAmmEvent {
        admin_authority: ctx.accounts.admin_authority.key(),
        multisig_authority: ctx.accounts.multisig_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Security alert
    emit!(SecurityAmmAlertEvent {
        alert_type: "EMERGENCY_PAUSE_AMM".to_string(),
        details: "All AMM operations have been paused by multi-sig authorities".to_string(),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigAmmOperationEvent {
        operation: "EMERGENCY_PAUSE_AMM".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: amm_global.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🚨 EMERGENCY PAUSE ACTIVATED - All AMM operations suspended");

    Ok(())
}

pub fn resume_amm_operations(ctx: Context<ResumeAmmOperations>) -> Result<()> {
    let amm_global = &mut ctx.accounts.amm_global;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for resume
    amm_global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Clear pause flag
    amm_global.is_paused = false;

    // Emit operations resumed event
    emit!(AmmOperationsResumedEvent {
        admin_authority: ctx.accounts.admin_authority.key(),
        multisig_authority: ctx.accounts.multisig_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Security alert
    emit!(SecurityAmmAlertEvent {
        alert_type: "AMM_OPERATIONS_RESUMED".to_string(),
        details: "All AMM operations have been resumed by multi-sig authorities".to_string(),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigAmmOperationEvent {
        operation: "AMM_OPERATIONS_RESUMED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: amm_global.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("✅ AMM Operations resumed - Platform is operational");

    Ok(())
}