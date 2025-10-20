use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Token, TokenAccount, Transfer},
};
use crate::{constants::*, state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*};

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        constraint = position.pool_id == pool.key() ,
        constraint = position.owner == position_owner.key() 
    )]
    pub position: Account<'info, Position>,

    /// Pool vault for token A (multi-sig protected)
    #[account(
        mut,
        constraint = vault_a.key() == pool.vault_a 
    )]
    pub vault_a: Account<'info, TokenAccount>,

    /// Pool vault for token B (multi-sig protected)
    #[account(
        mut,
        constraint = vault_b.key() == pool.vault_b 
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// User's token A account
    #[account(
        mut,
        constraint = user_token_a.owner == position_owner.key() ,
        constraint = user_token_a.mint == vault_a.mint 
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    /// User's token B account
    #[account(
        mut,
        constraint = user_token_b.owner == position_owner.key() ,
        constraint = user_token_b.mint == vault_b.mint 
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub position_owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CollectProtocolFees<'info> {
    #[account(
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(
        mut,
        constraint = pool.protocol_fees_token_a > 0 || pool.protocol_fees_token_b > 0 
            
    )]
    pub pool: Account<'info, Pool>,

    /// Pool vault for token A (multi-sig protected)
    #[account(
        mut,
        constraint = vault_a.key() == pool.vault_a 
    )]
    pub vault_a: Account<'info, TokenAccount>,

    /// Pool vault for token B (multi-sig protected)
    #[account(
        mut,
        constraint = vault_b.key() == pool.vault_b 
    )]
    pub vault_b: Account<'info, TokenAccount>,

    /// Platform wallet for protocol fees
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = platform_wallet.key() == amm_global.platform_wallet 
            
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == amm_global.admin_authority 
            
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for fee collection)
    #[account(
        constraint = multisig_authority.key() == amm_global.multisig_authority 
            
    )]
    pub multisig_authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn collect_fees(
    ctx: Context<CollectFees>,
    amount0_requested: u64,
    amount1_requested: u64,
) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let position = &mut ctx.accounts.position;
    let clock = Clock::get()?;

    // Calculate fees owed to this position
    let (fees_owed_a, fees_owed_b) = calculate_fees_owed(pool, position)?;

    // Determine actual amounts to collect
    let amount0_to_collect = if amount0_requested == u64::MAX {
        fees_owed_a
    } else {
        amount0_requested.min(fees_owed_a)
    };

    let amount1_to_collect = if amount1_requested == u64::MAX {
        fees_owed_b
    } else {
        amount1_requested.min(fees_owed_b)
    };

    // Verify there are fees to collect
    require!(
        amount0_to_collect > 0 || amount1_to_collect > 0,
        AmmError::InsufficientFees
    );

    // Use pool authority to transfer fees from vaults to user
    let pool_seeds = &[
        POOL_SEED,
        pool.mint_a.as_ref(),
        pool.mint_b.as_ref(),
        &[pool.bump],
    ];
    let pool_signer = &[&pool_seeds[..]];

    // Transfer token A fees
    if amount0_to_collect > 0 {
        let transfer_a_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_a_ctx, amount0_to_collect)?;
    }

    // Transfer token B fees
    if amount1_to_collect > 0 {
        let transfer_b_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_b_ctx, amount1_to_collect)?;
    }

    // Update position fees owed
    position.fees_owed_a = position.fees_owed_a
        .checked_sub(amount0_to_collect)
        .ok_or(AmmError::Underflow)?;

    position.fees_owed_b = position.fees_owed_b
        .checked_sub(amount1_to_collect)
        .ok_or(AmmError::Underflow)?;

    // Emit fees collected event
    emit!(FeesCollectedEvent {
        position_mint: position.mint,
        pool_id: position.pool_id,
        amount0: amount0_to_collect,
        amount1: amount1_to_collect,
        collector: ctx.accounts.position_owner.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("💰 Position fees collected successfully");
    msg!("Position: {}", position.mint);
    msg!("Amount0 Collected: {} tokens", amount0_to_collect);
    msg!("Amount1 Collected: {} tokens", amount1_to_collect);

    Ok(())
}

pub fn collect_protocol_fees(
    ctx: Context<CollectProtocolFees>,
    amount0: u64,
    amount1: u64,
) -> Result<()> {
    let amm_global = &ctx.accounts.amm_global;
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for protocol fee collection
    amm_global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Determine actual amounts to collect
    let amount0_to_collect = amount0.min(pool.protocol_fees_token_a);
    let amount1_to_collect = amount1.min(pool.protocol_fees_token_b);

    // Verify there are fees to collect
    require!(
        amount0_to_collect > 0 || amount1_to_collect > 0,
        AmmError::InsufficientFees
    );

    // Use pool authority to transfer protocol fees
    let pool_seeds = &[
        POOL_SEED,
        pool.mint_a.as_ref(),
        pool.mint_b.as_ref(),
        &[pool.bump],
    ];
    let pool_signer = &[&pool_seeds[..]];

    // Transfer token A protocol fees to platform wallet
    if amount0_to_collect > 0 {
        let transfer_a_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.platform_wallet.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_a_ctx, amount0_to_collect)?;
    }

    // Transfer token B protocol fees to platform wallet
    if amount1_to_collect > 0 {
        let transfer_b_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.platform_wallet.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_b_ctx, amount1_to_collect)?;
    }

    // Update pool protocol fees
    pool.protocol_fees_token_a = pool.protocol_fees_token_a
        .checked_sub(amount0_to_collect)
        .ok_or(AmmError::Underflow)?;

    pool.protocol_fees_token_b = pool.protocol_fees_token_b
        .checked_sub(amount1_to_collect)
        .ok_or(AmmError::Underflow)?;

    // Emit protocol fees collected event
    emit!(ProtocolFeesCollectedEvent {
        pool_id: pool.key(),
        amount0: amount0_to_collect,
        amount1: amount1_to_collect,
        collector: ctx.accounts.admin_authority.key(),
        destination: ctx.accounts.platform_wallet.key(),
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigAmmOperationEvent {
        operation: "PROTOCOL_FEES_COLLECTED".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: pool.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("💰 Protocol fees collected successfully");
    msg!("Pool: {}", pool.key());
    msg!("Amount0 Collected: {} tokens", amount0_to_collect);
    msg!("Amount1 Collected: {} tokens", amount1_to_collect);

    Ok(())
}

fn calculate_fees_owed(_pool: &Pool, position: &Position) -> Result<(u64, u64)> {
    // Simplified fee calculation
    // In production, this would involve complex fee growth calculations
    
    // For now, return the fees already tracked in the position
    Ok((position.fees_owed_a, position.fees_owed_b))
}