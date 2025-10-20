use anchor_lang::prelude::*;
// SPL imports removed - not needed for core AMM functionality

declare_id!("B1NJQNWgWRG1A3N1nAkE7YGFeinnxwhLAPqVSXXKtB5R");

pub mod constants;
pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;
pub mod math;

use instructions::*;

#[program]
pub mod amm {
    use super::*;

    /// Initialize AMM global configuration with multi-sig authority
    pub fn initialize_amm_global(ctx: Context<InitializeAmmGlobal>) -> Result<()> {
        instructions::initialize_amm_global(ctx)
    }

    /// Create concentrated liquidity pool (requires multi-sig)
    pub fn create_pool(
        ctx: Context<CreatePool>,
        sqrt_price_x64: u128,
        tick_spacing: u16,
    ) -> Result<()> {
        instructions::create_pool(ctx, sqrt_price_x64, tick_spacing)
    }

    /// Initialize liquidity position NFT
    pub fn open_position(
        ctx: Context<OpenPosition>,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<()> {
        instructions::open_position(ctx, tick_lower, tick_upper)
    }

    /// Add liquidity to position
    pub fn increase_liquidity(
        ctx: Context<IncreaseLiquidity>,
        liquidity_delta: u128,
        amount0_max: u64,
        amount1_max: u64,
    ) -> Result<()> {
        instructions::increase_liquidity(ctx, liquidity_delta, amount0_max, amount1_max)
    }

    /// Remove liquidity from position
    pub fn decrease_liquidity(
        ctx: Context<DecreaseLiquidity>,
        liquidity_delta: u128,
        amount0_min: u64,
        amount1_min: u64,
    ) -> Result<()> {
        instructions::decrease_liquidity(ctx, liquidity_delta, amount0_min, amount1_min)
    }

    /// Swap tokens in the pool
    pub fn swap(
        ctx: Context<Swap>,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit_x64: u128,
        is_base_input: bool,
    ) -> Result<()> {
        instructions::swap(ctx, amount, other_amount_threshold, sqrt_price_limit_x64, is_base_input)
    }

    /// Collect fees from position
    pub fn collect_fees(
        ctx: Context<CollectFees>,
        amount0_requested: u64,
        amount1_requested: u64,
    ) -> Result<()> {
        instructions::collect_fees(ctx, amount0_requested, amount1_requested)
    }

    /// Collect protocol fees (multi-sig required)
    pub fn collect_protocol_fees(
        ctx: Context<CollectProtocolFees>,
        amount0: u64,
        amount1: u64,
    ) -> Result<()> {
        instructions::collect_protocol_fees(ctx, amount0, amount1)
    }

    /// Update pool fees (multi-sig required)
    pub fn update_pool_fees(
        ctx: Context<UpdatePoolFees>,
        trade_fee_rate: u32,
        protocol_fee_rate: u32,
        fund_fee_rate: u32,
    ) -> Result<()> {
        instructions::update_pool_fees(ctx, trade_fee_rate, protocol_fee_rate, fund_fee_rate)
    }

    /// Initialize tick array for price ranges
    pub fn initialize_tick_array(
        ctx: Context<InitializeTickArray>,
        start_tick_index: i32,
    ) -> Result<()> {
        instructions::initialize_tick_array(ctx, start_tick_index)
    }

    /// Emergency pause (multi-sig required)
    pub fn emergency_pause_amm(ctx: Context<EmergencyPauseAmm>) -> Result<()> {
        instructions::emergency_pause_amm(ctx)
    }

    /// Resume operations (multi-sig required)
    pub fn resume_amm_operations(ctx: Context<ResumeAmmOperations>) -> Result<()> {
        instructions::resume_amm_operations(ctx)
    }

    /// Set pool reward (multi-sig required)
    pub fn set_pool_reward(
        ctx: Context<SetPoolReward>,
        reward_index: u8,
        emissions_per_second_x64: u128,
    ) -> Result<()> {
        instructions::set_pool_reward(ctx, reward_index, emissions_per_second_x64)
    }

    /// Initialize reward for pool
    pub fn initialize_reward(
        ctx: Context<InitializeReward>,
        reward_index: u8,
    ) -> Result<()> {
        instructions::initialize_reward(ctx, reward_index)
    }
}