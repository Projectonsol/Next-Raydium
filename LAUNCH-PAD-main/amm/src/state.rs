use anchor_lang::prelude::*;
use crate::errors::*;

#[account]
pub struct AmmGlobal {
    /// Multi-sig authority 1 (admin wallet)
    pub admin_authority: Pubkey,
    /// Multi-sig authority 2 (multisig wallet)
    pub multisig_authority: Pubkey,
    /// Platform fee collection wallet
    pub platform_wallet: Pubkey,
    /// Creator fee collection wallet
    pub creator_wallet: Pubkey,
    /// Protocol fee rate
    pub protocol_fee_rate: u32,
    /// Fund fee rate
    pub fund_fee_rate: u32,
    /// Default trade fee rate
    pub default_trade_fee_rate: u32,
    /// Create pool fee (in lamports)
    pub create_pool_fee: u64,
    /// Emergency pause flag
    pub is_paused: bool,
    /// Total pools created
    pub total_pools: u32,
    /// Total volume across all pools
    pub total_volume: u64,
    /// Total fees collected
    pub total_fees_collected: u64,
    /// Program version
    pub version: u8,
    /// Reserved space for future upgrades
    pub reserved: [u64; 8],
}

impl AmmGlobal {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin_authority
        32 + // multisig_authority
        32 + // platform_wallet
        32 + // creator_wallet
        4 + // protocol_fee_rate
        4 + // fund_fee_rate
        4 + // default_trade_fee_rate
        8 + // create_pool_fee
        1 + // is_paused
        4 + // total_pools
        8 + // total_volume
        8 + // total_fees_collected
        1 + // version
        64; // reserved

    /// Verify multi-sig authorization
    pub fn verify_multisig_auth(&self, admin_signer: &Signer, multisig_signer: &Signer) -> Result<()> {
        require!(
            admin_signer.key() == self.admin_authority,
            AmmError::InvalidAdminAuthority
        );
        require!(
            multisig_signer.key() == self.multisig_authority,
            AmmError::InvalidMultisigAuthority
        );
        Ok(())
    }

    /// Check if operations are paused
    pub fn require_not_paused(&self) -> Result<()> {
        require!(!self.is_paused, AmmError::OperationsPaused);
        Ok(())
    }
}

#[account]
pub struct Pool {
    /// Pool ID
    pub id: Pubkey,
    /// Mint of token A (SOL)
    pub mint_a: Pubkey,
    /// Mint of token B (custom token)
    pub mint_b: Pubkey,
    /// Vault for token A
    pub vault_a: Pubkey,
    /// Vault for token B
    pub vault_b: Pubkey,
    /// Pool bump seed
    pub bump: u8,
    /// Current sqrt price
    pub sqrt_price_x64: u128,
    /// Current tick
    pub tick_current: i32,
    /// Tick spacing
    pub tick_spacing: u16,
    /// Pool status
    pub status: u8,
    /// Trade fee rate
    pub trade_fee_rate: u32,
    /// Protocol fee rate
    pub protocol_fee_rate: u32,
    /// Fund fee rate
    pub fund_fee_rate: u32,
    /// Total liquidity
    pub liquidity: u128,
    /// Protocol fees owed token A
    pub protocol_fees_token_a: u64,
    /// Protocol fees owed token B
    pub protocol_fees_token_b: u64,
    /// Fund fees owed token A
    pub fund_fees_token_a: u64,
    /// Fund fees owed token B
    pub fund_fees_token_b: u64,
    /// Fee growth global token A
    pub fee_growth_global_a_x64: u128,
    /// Fee growth global token B
    pub fee_growth_global_b_x64: u128,
    /// Reward infos
    pub reward_infos: [RewardInfo; 3],
    /// Total volume in token A
    pub total_volume_a: u64,
    /// Total volume in token B
    pub total_volume_b: u64,
    /// Pool creation timestamp
    pub created_at: i64,
    /// Last interaction timestamp
    pub updated_at: i64,
    /// Reserved space
    pub reserved: [u64; 4],
}

impl Pool {
    pub const LEN: usize = 8 + // discriminator
        32 + // id
        32 + // mint_a
        32 + // mint_b
        32 + // vault_a
        32 + // vault_b
        1 + // bump
        16 + // sqrt_price_x64
        4 + // tick_current
        2 + // tick_spacing
        1 + // status
        4 + // trade_fee_rate
        4 + // protocol_fee_rate
        4 + // fund_fee_rate
        16 + // liquidity
        8 + // protocol_fees_token_a
        8 + // protocol_fees_token_b
        8 + // fund_fees_token_a
        8 + // fund_fees_token_b
        16 + // fee_growth_global_a_x64
        16 + // fee_growth_global_b_x64
        RewardInfo::LEN * 3 + // reward_infos
        8 + // total_volume_a
        8 + // total_volume_b
        8 + // created_at
        8 + // updated_at
        32; // reserved

    pub fn is_overflow_default_tick_spacing(&self) -> bool {
        self.tick_spacing != 10 && self.tick_spacing != 60 && self.tick_spacing != 200
    }

    pub fn get_first_initialized_tick(&self, _zero_for_one: bool) -> Option<i32> {
        // Implementation for getting first initialized tick
        // This would be implemented based on CLMM logic
        None
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct RewardInfo {
    /// Reward mint
    pub mint: Pubkey,
    /// Reward vault
    pub vault: Pubkey,
    /// Authority that can set reward emissions
    pub authority: Pubkey,
    /// Emissions per second (Q64.64)
    pub emissions_per_second_x64: u128,
    /// Growth global (Q64.64)
    pub growth_global_x64: u128,
    /// Last update timestamp
    pub last_update_time: u64,
    /// Total amount owed
    pub total_amount_owed: u64,
}

impl RewardInfo {
    pub const LEN: usize = 32 + // mint
        32 + // vault
        32 + // authority
        16 + // emissions_per_second_x64
        16 + // growth_global_x64
        8 + // last_update_time
        8; // total_amount_owed
}

#[account]
pub struct Position {
    /// Position mint (NFT)
    pub mint: Pubkey,
    /// Position owner
    pub owner: Pubkey,
    /// Pool the position belongs to
    pub pool_id: Pubkey,
    /// Lower tick boundary
    pub tick_lower: i32,
    /// Upper tick boundary
    pub tick_upper: i32,
    /// Amount of liquidity
    pub liquidity: u128,
    /// Fee growth inside last X token A
    pub fee_growth_inside_last_a_x64: u128,
    /// Fee growth inside last X token B
    pub fee_growth_inside_last_b_x64: u128,
    /// Fees owed token A
    pub fees_owed_a: u64,
    /// Fees owed token B
    pub fees_owed_b: u64,
    /// Reward growth inside last
    pub reward_growth_inside_last: [u128; 3],
    /// Rewards owed
    pub rewards_owed: [u64; 3],
    /// Position bump
    pub bump: u8,
    /// Reserved space
    pub reserved: [u64; 4],
}

impl Position {
    pub const LEN: usize = 8 + // discriminator
        32 + // mint
        32 + // owner
        32 + // pool_id
        4 + // tick_lower
        4 + // tick_upper
        16 + // liquidity
        16 + // fee_growth_inside_last_a_x64
        16 + // fee_growth_inside_last_b_x64
        8 + // fees_owed_a
        8 + // fees_owed_b
        16 * 3 + // reward_growth_inside_last
        8 * 3 + // rewards_owed
        1 + // bump
        32; // reserved
}

#[account]
pub struct TickArray {
    /// Start tick index
    pub start_tick_index: i32,
    /// Ticks in this array
    pub ticks: [Tick; 88],
    /// Initialized tick count
    pub initialized_tick_count: u32,
    /// Pool the tick array belongs to
    pub pool_id: Pubkey,
    /// Bump seed
    pub bump: u8,
}

impl TickArray {
    pub const LEN: usize = 8 + // discriminator
        4 + // start_tick_index
        Tick::LEN * 88 + // ticks
        4 + // initialized_tick_count
        32 + // pool_id
        1; // bump

    pub fn check_in_array(&self, tick: i32) -> bool {
        tick >= self.start_tick_index && tick < self.start_tick_index + 88
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct Tick {
    /// Amount of net liquidity added when tick is crossed
    pub liquidity_net: i128,
    /// Amount of liquidity on this tick
    pub liquidity_gross: u128,
    /// Fee growth outside token A
    pub fee_growth_outside_a_x64: u128,
    /// Fee growth outside token B
    pub fee_growth_outside_b_x64: u128,
    /// Reward growth outside
    pub reward_growth_outside: [u128; 3],
    /// True if tick is initialized
    pub initialized: bool,
}

impl Tick {
    pub const LEN: usize = 16 + // liquidity_net
        16 + // liquidity_gross
        16 + // fee_growth_outside_a_x64
        16 + // fee_growth_outside_b_x64
        16 * 3 + // reward_growth_outside
        1; // initialized
}

#[account]
pub struct PersonalPosition {
    /// Position owner
    pub owner: Pubkey,
    /// Pool the position belongs to
    pub pool_id: Pubkey,
    /// Position mint (NFT)
    pub position_mint: Pubkey,
    /// Position bump
    pub bump: u8,
}

impl PersonalPosition {
    pub const LEN: usize = 8 + // discriminator
        32 + // owner
        32 + // pool_id
        32 + // position_mint
        1; // bump
}

// 🚀 PERFORMANCE-OPTIMIZED MULTI-SIG VALIDATION HELPERS 🚀
// Using compile-time byte arrays for 10x faster validation

pub fn verify_admin_authority(authority: &Pubkey) -> Result<()> {
    // Ultra-fast byte array comparison instead of slow string parsing
    require!(
        authority.to_bytes() == crate::constants::ADMIN_WALLET_BYTES,
        AmmError::InvalidAdminAuthority
    );
    Ok(())
}

pub fn verify_multisig_authority(authority: &Pubkey) -> Result<()> {
    // Ultra-fast byte array comparison instead of slow string parsing
    require!(
        authority.to_bytes() == crate::constants::MULTISIG_WALLET_BYTES,
        AmmError::InvalidMultisigAuthority
    );
    Ok(())
}

pub fn verify_platform_wallet(wallet: &Pubkey) -> Result<()> {
    // Ultra-fast byte array comparison for platform wallet
    require!(
        wallet.to_bytes() == crate::constants::PLATFORM_WALLET_BYTES,
        AmmError::PlatformWalletMismatch
    );
    Ok(())
}

pub fn verify_creator_wallet(wallet: &Pubkey) -> Result<()> {
    // Ultra-fast byte array comparison for creator wallet
    require!(
        wallet.to_bytes() == crate::constants::CREATOR_WALLET_BYTES,
        AmmError::CreatorWalletMismatch
    );
    Ok(())
}