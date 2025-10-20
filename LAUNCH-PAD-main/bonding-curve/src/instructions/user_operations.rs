use anchor_lang::prelude::*;
use crate::{constants::*, state::{Global, UserVolumeAccumulator}, events::*};

#[derive(Accounts)]
pub struct InitUserVolumeAccumulator<'info> {
    #[account(
        constraint = !global.is_paused
    )]
    pub global: Account<'info, Global>,

    #[account(
        init,
        payer = user,
        space = UserVolumeAccumulator::LEN,
        seeds = [USER_VOLUME_SEED, user.key().as_ref()],
        bump
    )]
    pub user_volume_accumulator: Account<'info, UserVolumeAccumulator>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn init_user_volume_accumulator(ctx: Context<InitUserVolumeAccumulator>) -> Result<()> {
    let user_volume = &mut ctx.accounts.user_volume_accumulator;
    let clock = Clock::get()?;

    // Initialize user volume tracking
    user_volume.user = ctx.accounts.user.key();
    user_volume.volume_sol = 0;
    user_volume.volume_tokens = 0;
    user_volume.trades_count = 0;
    user_volume.last_trade_timestamp = 0;
    user_volume.bump = ctx.bumps.user_volume_accumulator;

    // Emit initialization event
    emit!(UserVolumeAccumulatorInitializedEvent {
        user: ctx.accounts.user.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("📊 User volume accumulator initialized for: {}", ctx.accounts.user.key());

    Ok(())
}