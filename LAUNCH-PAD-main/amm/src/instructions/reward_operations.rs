use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::{constants::*, state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*};

#[derive(Accounts)]
#[instruction(reward_index: u8)]
pub struct InitializeReward<'info> {
    #[account(
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// Reward token mint
    pub reward_mint: Account<'info, Mint>,

    /// Reward vault (multi-sig protected)
    #[account(
        init,
        payer = reward_authority,
        token::mint = reward_mint,
        token::authority = pool,
        seeds = [POOL_REWARD_VAULT_SEED, pool.key().as_ref(), &reward_index.to_le_bytes()],
        bump
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == amm_global.admin_authority 
            
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for reward initialization)
    #[account(
        constraint = multisig_authority.key() == amm_global.multisig_authority 
            
    )]
    pub multisig_authority: Signer<'info>,

    /// Reward authority (who can set emissions)
    #[account(mut)]
    pub reward_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetPoolReward<'info> {
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

    /// Multi-sig authority (required for reward settings)
    #[account(
        constraint = multisig_authority.key() == amm_global.multisig_authority 
            
    )]
    pub multisig_authority: Signer<'info>,
}

pub fn initialize_reward(
    ctx: Context<InitializeReward>,
    reward_index: u8,
) -> Result<()> {
    let amm_global = &ctx.accounts.amm_global;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for reward initialization
    amm_global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Validate reward index
    require!(reward_index < REWARD_NUM as u8, AmmError::InvalidRewardIndex);

    // Check if reward is already initialized
    require!(
        pool.reward_infos[reward_index as usize].mint == Pubkey::default(),
        AmmError::RewardAlreadyInitialized
    );

    // Initialize reward info
    pool.reward_infos[reward_index as usize] = RewardInfo {
        mint: ctx.accounts.reward_mint.key(),
        vault: ctx.accounts.reward_vault.key(),
        authority: ctx.accounts.reward_authority.key(),
        emissions_per_second_x64: 0,
        growth_global_x64: 0,
        last_update_time: clock.unix_timestamp as u64,
        total_amount_owed: 0,
    };

    // Update pool timestamp
    pool.updated_at = clock.unix_timestamp;

    // Emit reward initialized event
    emit!(RewardInitializedEvent {
        pool_id: pool.key(),
        reward_index,
        reward_mint: ctx.accounts.reward_mint.key(),
        reward_vault: ctx.accounts.reward_vault.key(),
        authority: ctx.accounts.reward_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigAmmOperationEvent {
        operation: "REWARD_INITIALIZED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: pool.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🎁 Reward initialized successfully");
    msg!("Pool: {}", pool.key());
    msg!("Reward Index: {}", reward_index);
    msg!("Reward Mint: {}", ctx.accounts.reward_mint.key());
    msg!("Reward Vault: {}", ctx.accounts.reward_vault.key());
    msg!("Reward Authority: {}", ctx.accounts.reward_authority.key());

    Ok(())
}

pub fn set_pool_reward(
    ctx: Context<SetPoolReward>,
    reward_index: u8,
    emissions_per_second_x64: u128,
) -> Result<()> {
    let amm_global = &ctx.accounts.amm_global;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for reward emission settings
    amm_global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Validate reward index
    require!(reward_index < REWARD_NUM as u8, AmmError::InvalidRewardIndex);

    // Check if reward is initialized
    require!(
        pool.reward_infos[reward_index as usize].mint != Pubkey::default(),
        AmmError::RewardNotInitialized
    );

    // Update reward emissions
    // Extract pool liquidity before mutable borrow to avoid borrow checker issues
    let pool_liquidity = pool.liquidity;
    let reward_info = &mut pool.reward_infos[reward_index as usize];
    
    // Update growth before changing emissions
    update_reward_growth(reward_info, pool_liquidity, clock.unix_timestamp as u64)?;
    
    // Set new emissions rate
    reward_info.emissions_per_second_x64 = emissions_per_second_x64;
    reward_info.last_update_time = clock.unix_timestamp as u64;

    // Update pool timestamp
    pool.updated_at = clock.unix_timestamp;

    // Emit reward emission updated event
    emit!(RewardEmissionUpdatedEvent {
        pool_id: pool.key(),
        reward_index,
        emissions_per_second_x64,
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigAmmOperationEvent {
        operation: "REWARD_EMISSION_UPDATED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: pool.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🎁 Reward emissions updated successfully");
    msg!("Pool: {}", pool.key());
    msg!("Reward Index: {}", reward_index);
    msg!("Emissions per Second: {}", emissions_per_second_x64);

    Ok(())
}

fn update_reward_growth(
    reward_info: &mut RewardInfo,
    pool_liquidity: u128,
    current_time: u64,
) -> Result<()> {
    if pool_liquidity == 0 {
        reward_info.last_update_time = current_time;
        return Ok(());
    }

    let time_delta = current_time
        .checked_sub(reward_info.last_update_time)
        .ok_or(AmmError::Underflow)?;

    if time_delta == 0 {
        return Ok(());
    }

    let reward_growth_delta = reward_info.emissions_per_second_x64
        .checked_mul(time_delta as u128)
        .and_then(|x| x.checked_div(pool_liquidity))
        .ok_or(AmmError::Overflow)?;

    reward_info.growth_global_x64 = reward_info.growth_global_x64
        .checked_add(reward_growth_delta)
        .ok_or(AmmError::Overflow)?;

    reward_info.last_update_time = current_time;

    Ok(())
}