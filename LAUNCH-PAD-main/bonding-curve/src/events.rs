use anchor_lang::prelude::*;

#[event]
pub struct GlobalInitializedEvent {
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub platform_wallet: Pubkey,
    pub creator_wallet: Pubkey,
    pub platform_fee: u16,
    pub creator_fee: u16,
    pub migration_fee: u16,
    pub timestamp: i64,
}

#[event]
pub struct BondingCurveInitializedEvent {
    pub token_mint: Pubkey,
    pub creator: Pubkey,
    pub bonding_curve: Pubkey,
    pub sol_vault: Pubkey,
    pub token_vault: Pubkey,
    pub lp_reserve: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub total_supply: u64,
    pub lp_reserve_supply: u64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub migration_threshold: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensPurchasedEvent {
    pub token_mint: Pubkey,
    pub buyer: Pubkey,
    pub sol_cost: u64,
    pub token_amount: u64,
    pub platform_fee: u64,
    pub creator_fee: u64,
    pub new_sol_reserves: u64,
    pub new_token_reserves: u64,
    pub new_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct TokensSoldEvent {
    pub token_mint: Pubkey,
    pub seller: Pubkey,
    pub token_amount: u64,
    pub sol_received: u64,
    pub platform_fee: u64,
    pub creator_fee: u64,
    pub new_sol_reserves: u64,
    pub new_token_reserves: u64,
    pub new_price: u64,
    pub timestamp: i64,
}

#[event]
pub struct MigrationReadyEvent {
    pub token_mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub sol_reserves: u64,
    pub token_reserves: u64,
    pub migration_threshold: u64,
    pub timestamp: i64,
}

#[event]
pub struct MigrationCompletedEvent {
    pub token_mint: Pubkey,
    pub bonding_curve: Pubkey,
    pub amm_program_id: Pubkey,
    pub amm_pool_address: Pubkey,
    pub sol_transferred: u64,
    pub tokens_transferred: u64,
    pub lp_tokens_minted: u64,
    pub migration_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct PlatformFeesCollectedEvent {
    pub collector: Pubkey,
    pub amount: u64,
    pub destination: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct CreatorFeesCollectedEvent {
    pub token_mint: Pubkey,
    pub creator: Pubkey,
    pub collector: Pubkey,
    pub amount: u64,
    pub destination: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct GlobalSettingsUpdatedEvent {
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub platform_fee: u16,
    pub creator_fee: u16,
    pub migration_fee: u16,
    pub migration_enabled: bool,
    pub timestamp: i64,
}

#[event]
pub struct EmergencyPauseEvent {
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct OperationsResumedEvent {
    pub admin_authority: Pubkey,
    pub multisig_authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UserVolumeAccumulatorInitializedEvent {
    pub user: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct SecurityAlertEvent {
    pub alert_type: String,
    pub details: String,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MultisigOperationEvent {
    pub operation: String,
    pub admin_signer: Pubkey,
    pub multisig_signer: Pubkey,
    pub target_account: Pubkey,
    pub timestamp: i64,
}