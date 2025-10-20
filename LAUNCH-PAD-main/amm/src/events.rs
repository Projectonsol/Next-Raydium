use anchor_lang::prelude::*;

#[event]
pub struct AmmGlobalInitializedEvent {
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub platform_wallet: Pubkey,
    pub creator_wallet: Pubkey,
    pub protocol_fee_rate: u32,
    pub fund_fee_rate: u32,
    pub default_trade_fee_rate: u32,
    pub create_pool_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct PoolCreatedEvent {
    pub pool_id: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub sqrt_price_x64: u128,
    pub tick_current: i32,
    pub tick_spacing: u16,
    pub trade_fee_rate: u32,
    pub protocol_fee_rate: u32,
    pub fund_fee_rate: u32,
    pub created_by: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PositionOpenedEvent {
    pub position_mint: Pubkey,
    pub pool_id: Pubkey,
    pub owner: Pubkey,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityIncreasedEvent {
    pub position_mint: Pubkey,
    pub pool_id: Pubkey,
    pub liquidity_delta: u128,
    pub amount0: u64,
    pub amount1: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityDecreasedEvent {
    pub position_mint: Pubkey,
    pub pool_id: Pubkey,
    pub liquidity_delta: u128,
    pub amount0: u64,
    pub amount1: u64,
    pub timestamp: i64,
}

#[event]
pub struct SwapEvent {
    pub pool_id: Pubkey,
    pub user: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub output_amount: u64,
    pub fee_amount: u64,
    pub sqrt_price_x64: u128,
    pub tick_current: i32,
    pub timestamp: i64,
}

#[event]
pub struct FeesCollectedEvent {
    pub position_mint: Pubkey,
    pub pool_id: Pubkey,
    pub amount0: u64,
    pub amount1: u64,
    pub collector: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ProtocolFeesCollectedEvent {
    pub pool_id: Pubkey,
    pub amount0: u64,
    pub amount1: u64,
    pub collector: Pubkey,
    pub destination: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PoolFeesUpdatedEvent {
    pub pool_id: Pubkey,
    pub trade_fee_rate: u32,
    pub protocol_fee_rate: u32,
    pub fund_fee_rate: u32,
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TickArrayInitializedEvent {
    pub pool_id: Pubkey,
    pub tick_array: Pubkey,
    pub start_tick_index: i32,
    pub timestamp: i64,
}

#[event]
pub struct RewardInitializedEvent {
    pub pool_id: Pubkey,
    pub reward_index: u8,
    pub reward_mint: Pubkey,
    pub reward_vault: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct RewardEmissionUpdatedEvent {
    pub pool_id: Pubkey,
    pub reward_index: u8,
    pub emissions_per_second_x64: u128,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPauseAmmEvent {
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct AmmOperationsResumedEvent {
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MultisigAmmOperationEvent {
    pub operation: String,
    pub admin_signer: Pubkey,
    pub multisig_signer: Pubkey,
    pub target_account: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SecurityAmmAlertEvent {
    pub alert_type: String,
    pub details: String,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct PoolMigrationEvent {
    pub bonding_curve_program: Pubkey,
    pub bonding_curve: Pubkey,
    pub token_mint: Pubkey,
    pub pool_id: Pubkey,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub initial_liquidity: u128,
    pub timestamp: i64,
}