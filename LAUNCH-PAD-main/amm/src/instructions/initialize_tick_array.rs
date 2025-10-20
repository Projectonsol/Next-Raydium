use anchor_lang::prelude::*;
use crate::{constants::*, state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*};

#[derive(Accounts)]
#[instruction(start_tick_index: i32)]
pub struct InitializeTickArray<'info> {
    #[account(
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = payer,
        space = TickArray::LEN,
        seeds = [TICK_ARRAY_SEED, pool.key().as_ref(), &start_tick_index.to_le_bytes()],
        bump
    )]
    pub tick_array: Account<'info, TickArray>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_tick_array(
    ctx: Context<InitializeTickArray>,
    start_tick_index: i32,
) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let tick_array = &mut ctx.accounts.tick_array;
    let clock = Clock::get()?;

    // Validate start tick index alignment
    require!(
        start_tick_index % (TICK_ARRAY_SIZE * pool.tick_spacing as i32) == 0,
        AmmError::InvalidTickArray
    );

    // Validate tick index bounds
    require!(
        start_tick_index >= MIN_TICK && start_tick_index <= MAX_TICK,
        AmmError::TickOutOfBounds
    );

    // Initialize tick array
    tick_array.start_tick_index = start_tick_index;
    tick_array.pool_id = pool.key();
    tick_array.bump = ctx.bumps.tick_array;
    tick_array.initialized_tick_count = 0;
    
    // Initialize all ticks as uninitialized
    tick_array.ticks = [Default::default(); TICK_ARRAY_SIZE as usize];

    // Emit tick array initialized event
    emit!(TickArrayInitializedEvent {
        pool_id: pool.key(),
        tick_array: tick_array.key(),
        start_tick_index,
        timestamp: clock.unix_timestamp,
    });

    msg!("📊 Tick array initialized successfully");
    msg!("Pool: {}", pool.key());
    msg!("Tick Array: {}", tick_array.key());
    msg!("Start Tick Index: {}", start_tick_index);
    msg!("End Tick Index: {}", start_tick_index + TICK_ARRAY_SIZE - 1);

    Ok(())
}