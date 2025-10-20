use anchor_lang::prelude::*;

declare_id!("8DV5gyq2Dsy5DW5dMQtLZ5FGw657BUH2h9pZyBDcoSz3");

pub mod constants;
pub mod state;
pub mod instructions;
pub mod errors;
pub mod events;

use instructions::*;

#[program]
pub mod bonding_curve {
    use super::*;

    /// Initialize global configuration with multi-sig authority
    pub fn initialize_global(ctx: Context<InitializeGlobal>) -> Result<()> {
        instructions::initialize_global(ctx)
    }

    /// Initialize bonding curve with multi-sig security
    pub fn initialize_bonding_curve(
        ctx: Context<InitializeBondingCurve>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::initialize_bonding_curve(ctx, name, symbol, uri)
    }

    /// Buy tokens from bonding curve
    pub fn buy_tokens(ctx: Context<BuyTokens>, token_amount: u64, max_sol_cost: u64) -> Result<()> {
        instructions::buy_tokens(ctx, token_amount, max_sol_cost)
    }

    /// Sell tokens to bonding curve
    pub fn sell_tokens(ctx: Context<SellTokens>, token_amount: u64, min_sol_received: u64) -> Result<()> {
        instructions::sell_tokens(ctx, token_amount, min_sol_received)
    }

    /// Initialize user volume accumulator
    pub fn init_user_volume_accumulator(ctx: Context<InitUserVolumeAccumulator>) -> Result<()> {
        instructions::init_user_volume_accumulator(ctx)
    }

    /// Migrate to AMM (requires multi-sig approval)
    pub fn migrate_to_amm(ctx: Context<MigrateToAmm>) -> Result<()> {
        instructions::migrate_to_amm(ctx)
    }

    /// Update global settings (multi-sig required)
    pub fn update_global_settings(
        ctx: Context<UpdateGlobalSettings>,
        platform_fee_basis_points: Option<u16>,
        creator_fee_basis_points: Option<u16>,
        migration_fee_basis_points: Option<u16>,
        migration_enabled: Option<bool>,
    ) -> Result<()> {
        instructions::update_global_settings(
            ctx,
            platform_fee_basis_points,
            creator_fee_basis_points,
            migration_fee_basis_points,
            migration_enabled,
        )
    }

    /// Collect platform fees (multi-sig required)
    pub fn collect_platform_fees(ctx: Context<CollectPlatformFees>, amount: u64) -> Result<()> {
        instructions::collect_platform_fees(ctx, amount)
    }

    /// Collect creator fees (multi-sig required)
    pub fn collect_creator_fees(ctx: Context<CollectCreatorFees>, amount: u64) -> Result<()> {
        instructions::collect_creator_fees(ctx, amount)
    }

    /// Emergency pause (multi-sig required)
    pub fn emergency_pause(ctx: Context<EmergencyPause>) -> Result<()> {
        instructions::emergency_pause(ctx)
    }

    /// Resume operations (multi-sig required)
    pub fn resume_operations(ctx: Context<ResumeOperations>) -> Result<()> {
        instructions::resume_operations(ctx)
    }
}