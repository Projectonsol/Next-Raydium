use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Token, TokenAccount, Transfer},
};
use crate::{constants::*, state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*, math::MathUtil};

#[derive(Accounts)]
pub struct DecreaseLiquidity<'info> {
    #[account(
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(
        mut,
        constraint = pool.status == POOL_STATUS_INITIALIZED || pool.status == POOL_STATUS_WITHDRAW_ONLY 
            
    )]
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

    /// Tick array for lower tick
    #[account(
        mut,
        constraint = tick_array_lower.pool_id == pool.key() ,
        constraint = tick_array_lower.check_in_array(position.tick_lower) 
    )]
    pub tick_array_lower: Account<'info, TickArray>,

    /// Tick array for upper tick
    #[account(
        mut,
        constraint = tick_array_upper.pool_id == pool.key() ,
        constraint = tick_array_upper.check_in_array(position.tick_upper) 
    )]
    pub tick_array_upper: Account<'info, TickArray>,

    #[account(mut)]
    pub position_owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn decrease_liquidity(
    ctx: Context<DecreaseLiquidity>,
    liquidity_delta: u128,
    amount0_min: u64,
    amount1_min: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let position = &mut ctx.accounts.position;
    let clock = Clock::get()?;

    // Validate liquidity amount
    require!(liquidity_delta > 0, AmmError::InvalidLiquidityAmount);
    require!(liquidity_delta <= position.liquidity, AmmError::InsufficientLiquidity);

    // Calculate token amounts to withdraw
    let sqrt_price_lower_x64 = MathUtil::tick_to_sqrt_price_x64(position.tick_lower)?;
    let sqrt_price_upper_x64 = MathUtil::tick_to_sqrt_price_x64(position.tick_upper)?;
    let sqrt_price_current_x64 = pool.sqrt_price_x64;

    let (amount0_to_withdraw, amount1_to_withdraw) = calculate_amounts_for_liquidity_withdrawal(
        sqrt_price_current_x64,
        sqrt_price_lower_x64,
        sqrt_price_upper_x64,
        liquidity_delta,
    )?;

    // Check slippage protection
    require!(amount0_to_withdraw >= amount0_min, AmmError::SlippageExceeded);
    require!(amount1_to_withdraw >= amount1_min, AmmError::SlippageExceeded);

    // Verify pool has sufficient tokens
    require!(
        ctx.accounts.vault_a.amount >= amount0_to_withdraw,
        AmmError::InsufficientTokenBalance
    );
    require!(
        ctx.accounts.vault_b.amount >= amount1_to_withdraw,
        AmmError::InsufficientTokenBalance
    );

    // Use pool authority to transfer tokens from vaults to user
    let pool_seeds = &[
        POOL_SEED,
        pool.mint_a.as_ref(),
        pool.mint_b.as_ref(),
        &[pool.bump],
    ];
    let pool_signer = &[&pool_seeds[..]];

    // Transfer token A from vault to user
    if amount0_to_withdraw > 0 {
        let transfer_a_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_a_ctx, amount0_to_withdraw)?;
    }

    // Transfer token B from vault to user
    if amount1_to_withdraw > 0 {
        let transfer_b_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: pool.to_account_info(),
            },
            pool_signer,
        );
        token::transfer(transfer_b_ctx, amount1_to_withdraw)?;
    }

    // Update position liquidity
    position.liquidity = position.liquidity
        .checked_sub(liquidity_delta)
        .ok_or(AmmError::Underflow)?;

    // Update pool liquidity if position is in range
    if pool.tick_current >= position.tick_lower && pool.tick_current < position.tick_upper {
        pool.liquidity = pool.liquidity
            .checked_sub(liquidity_delta)
            .ok_or(AmmError::Underflow)?;
    }

    // Update tick arrays
    update_ticks_for_liquidity_decrease(
        &mut ctx.accounts.tick_array_lower,
        &mut ctx.accounts.tick_array_upper,
        position.tick_lower,
        position.tick_upper,
        liquidity_delta,
    )?;

    // Update pool timestamp
    pool.updated_at = clock.unix_timestamp;

    // Emit liquidity decreased event
    emit!(LiquidityDecreasedEvent {
        position_mint: position.mint,
        pool_id: position.pool_id,
        liquidity_delta,
        amount0: amount0_to_withdraw,
        amount1: amount1_to_withdraw,
        timestamp: clock.unix_timestamp,
    });

    msg!("💧 Liquidity decreased successfully");
    msg!("Position: {}", position.mint);
    msg!("Liquidity Delta: {}", liquidity_delta);
    msg!("Amount0 Withdrawn: {} tokens", amount0_to_withdraw);
    msg!("Amount1 Withdrawn: {} tokens", amount1_to_withdraw);
    msg!("Remaining Position Liquidity: {}", position.liquidity);

    Ok(())
}

fn calculate_amounts_for_liquidity_withdrawal(
    sqrt_price_current_x64: u128,
    sqrt_price_lower_x64: u128,
    sqrt_price_upper_x64: u128,
    liquidity_delta: u128,
) -> Result<(u64, u64)> {
    let (amount0, amount1) = if sqrt_price_current_x64 <= sqrt_price_lower_x64 {
        // All amount0
        let amount0 = MathUtil::get_amount0_from_liquidity(
            sqrt_price_lower_x64,
            sqrt_price_upper_x64,
            liquidity_delta,
        )?;
        (amount0, 0)
    } else if sqrt_price_current_x64 < sqrt_price_upper_x64 {
        // Both amounts
        let amount0 = MathUtil::get_amount0_from_liquidity(
            sqrt_price_current_x64,
            sqrt_price_upper_x64,
            liquidity_delta,
        )?;
        let amount1 = MathUtil::get_amount1_from_liquidity(
            sqrt_price_lower_x64,
            sqrt_price_current_x64,
            liquidity_delta,
        )?;
        (amount0, amount1)
    } else {
        // All amount1
        let amount1 = MathUtil::get_amount1_from_liquidity(
            sqrt_price_lower_x64,
            sqrt_price_upper_x64,
            liquidity_delta,
        )?;
        (0, amount1)
    };

    Ok((amount0, amount1))
}

fn update_ticks_for_liquidity_decrease(
    tick_array_lower: &mut TickArray,
    tick_array_upper: &mut TickArray,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: u128,
) -> Result<()> {
    // Simplified tick update logic for liquidity decrease
    
    // Update lower tick
    let lower_index = ((tick_lower - tick_array_lower.start_tick_index) / 1) as usize;
    if lower_index < tick_array_lower.ticks.len() {
        let tick = &mut tick_array_lower.ticks[lower_index];
        tick.liquidity_net = tick.liquidity_net
            .checked_sub(liquidity_delta as i128)
            .ok_or(AmmError::Underflow)?;
        tick.liquidity_gross = tick.liquidity_gross
            .checked_sub(liquidity_delta)
            .ok_or(AmmError::Underflow)?;
        
        // If no liquidity left, mark as uninitialized
        if tick.liquidity_gross == 0 {
            tick.initialized = false;
        }
    }

    // Update upper tick
    let upper_index = ((tick_upper - tick_array_upper.start_tick_index) / 1) as usize;
    if upper_index < tick_array_upper.ticks.len() {
        let tick = &mut tick_array_upper.ticks[upper_index];
        tick.liquidity_net = tick.liquidity_net
            .checked_add(liquidity_delta as i128)
            .ok_or(AmmError::Overflow)?;
        tick.liquidity_gross = tick.liquidity_gross
            .checked_sub(liquidity_delta)
            .ok_or(AmmError::Underflow)?;
        
        // If no liquidity left, mark as uninitialized
        if tick.liquidity_gross == 0 {
            tick.initialized = false;
        }
    }

    Ok(())
}