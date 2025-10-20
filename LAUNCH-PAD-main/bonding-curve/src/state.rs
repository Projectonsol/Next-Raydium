use anchor_lang::prelude::*;

#[account]
pub struct Global {
    /// Multi-sig authority 1 (admin wallet)
    pub admin_authority: Pubkey,
    /// Multi-sig authority 2 (multisig wallet)  
    pub multisig_authority: Pubkey,
    /// Platform fee collection wallet
    pub platform_wallet: Pubkey,
    /// Creator fee collection wallet
    pub creator_wallet: Pubkey,
    /// Platform fee in basis points
    pub platform_fee_basis_points: u16,
    /// Creator fee in basis points
    pub creator_fee_basis_points: u16,
    /// Migration fee in basis points
    pub migration_fee_basis_points: u16,
    /// Maximum allowed slippage
    pub max_slippage_basis_points: u16,
    /// Migration enabled flag
    pub migration_enabled: bool,
    /// Emergency pause flag
    pub is_paused: bool,
    /// Total volume across all tokens (in SOL)
    pub total_volume_sol: u64,
    /// Total fees collected
    pub total_fees_collected: u64,
    /// Number of tokens created
    pub tokens_created: u32,
    /// Number of successful migrations
    pub successful_migrations: u32,
    /// Program version
    pub version: u8,
    /// Reserved space for future upgrades
    pub reserved: [u64; 8],
}

impl Global {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin_authority
        32 + // multisig_authority
        32 + // platform_wallet
        32 + // creator_wallet
        2 + // platform_fee_basis_points
        2 + // creator_fee_basis_points
        2 + // migration_fee_basis_points
        2 + // max_slippage_basis_points
        1 + // migration_enabled
        1 + // is_paused
        8 + // total_volume_sol
        8 + // total_fees_collected
        4 + // tokens_created
        4 + // successful_migrations
        1 + // version
        64; // reserved

    /// Verify multi-sig authorization
    pub fn verify_multisig_auth(&self, admin_signer: &Signer, multisig_signer: &Signer) -> Result<()> {
        require!(
            admin_signer.key() == self.admin_authority,
            BondingCurveError::InvalidAdminAuthority
        );
        require!(
            multisig_signer.key() == self.multisig_authority,
            BondingCurveError::InvalidMultisigAuthority
        );
        Ok(())
    }

    /// Check if operations are paused
    pub fn require_not_paused(&self) -> Result<()> {
        require!(!self.is_paused, BondingCurveError::OperationsPaused);
        Ok(())
    }
}

#[account]
pub struct BondingCurve {
    /// Associated token mint
    pub token_mint: Pubkey,
    /// Creator of the token
    pub creator: Pubkey,
    /// Token name (stored directly since no metadata program)
    pub name: String,
    /// Token symbol (stored directly since no metadata program)  
    pub symbol: String,
    /// Virtual SOL reserves for pricing
    pub virtual_sol_reserves: u64,
    /// Virtual token reserves for pricing
    pub virtual_token_reserves: u64,
    /// Actual SOL reserves in vault
    pub real_sol_reserves: u64,
    /// Actual token reserves in vault
    pub real_token_reserves: u64,
    /// LP reserve token supply (20% of total)
    pub lp_reserve_supply: u64,
    /// Migration threshold in SOL
    pub migration_threshold: u64,
    /// Migration ready flag
    pub migration_ready: bool,
    /// Migration completed flag
    pub is_migrated: bool,
    /// AMM program ID (set after migration)
    pub amm_program_id: Option<Pubkey>,
    /// AMM pool address (set after migration)
    pub amm_pool_address: Option<Pubkey>,
    /// Total volume traded
    pub total_volume_sol: u64,
    /// Total volume in tokens
    pub total_volume_tokens: u64,
    /// Platform fees collected
    pub platform_fees_collected: u64,
    /// Creator fees collected
    pub creator_fees_collected: u64,
    /// Number of buy transactions
    pub buy_count: u32,
    /// Number of sell transactions
    pub sell_count: u32,
    /// Creation timestamp
    pub created_at: i64,
    /// Last trade timestamp
    pub last_trade_at: i64,
    /// Bump seeds for PDAs
    pub bump: u8,
    pub sol_vault_bump: u8,
    pub token_vault_bump: u8,
    pub lp_reserve_bump: u8,
    /// Reserved space
    pub reserved: [u64; 4],
}

impl BondingCurve {
    pub const LEN: usize = 8 + // discriminator
        32 + // token_mint
        32 + // creator
        4 + 32 + // name (String)
        4 + 10 + // symbol (String)
        8 + // virtual_sol_reserves
        8 + // virtual_token_reserves
        8 + // real_sol_reserves
        8 + // real_token_reserves
        8 + // lp_reserve_supply
        8 + // migration_threshold
        1 + // migration_ready
        1 + // is_migrated
        33 + // amm_program_id (Option<Pubkey>)
        33 + // amm_pool_address (Option<Pubkey>)
        8 + // total_volume_sol
        8 + // total_volume_tokens
        8 + // platform_fees_collected
        8 + // creator_fees_collected
        4 + // buy_count
        4 + // sell_count
        8 + // created_at
        8 + // last_trade_at
        1 + // bump
        1 + // sol_vault_bump
        1 + // token_vault_bump
        1 + // lp_reserve_bump
        32; // reserved

    /// Check if migration threshold is met
    pub fn is_migration_threshold_met(&self) -> bool {
        self.real_sol_reserves >= self.migration_threshold
    }

    /// Calculate current price in SOL per token
    pub fn current_price(&self) -> Result<u64> {
        let total_sol = self.virtual_sol_reserves
            .checked_add(self.real_sol_reserves)
            .ok_or(BondingCurveError::Overflow)?;
        
        let total_tokens = self.virtual_token_reserves
            .checked_sub(self.real_token_reserves)
            .ok_or(BondingCurveError::Underflow)?;

        if total_tokens == 0 {
            return Err(BondingCurveError::DivisionByZero.into());
        }

        // Enhanced precision scaling with overflow protection
        const PRECISION_SCALE: u64 = 1_000_000_000;
        
        // Check if multiplication would overflow before doing it
        if total_sol > u64::MAX / PRECISION_SCALE {
            return Err(BondingCurveError::Overflow.into());
        }
        
        let scaled_sol = total_sol
            .checked_mul(PRECISION_SCALE)
            .ok_or(BondingCurveError::Overflow)?;
            
        scaled_sol
            .checked_div(total_tokens)
            .ok_or(BondingCurveError::DivisionByZero.into())
    }
    
    /// Enhanced validation for trading operations
    pub fn validate_trade_amounts(&self, token_amount: u64, is_buy: bool) -> Result<()> {
        require!(token_amount > 0, BondingCurveError::InvalidTokenAmount);
        require!(!self.is_migrated, BondingCurveError::AlreadyMigrated);
        
        if is_buy {
            require!(
                token_amount <= self.real_token_reserves,
                BondingCurveError::InsufficientTokenReserves
            );
        } else {
            // For sells, ensure user doesn't try to sell more than circulating supply
            let circulating_supply = self.virtual_token_reserves
                .checked_sub(self.real_token_reserves)
                .ok_or(BondingCurveError::Underflow)?;
            require!(
                token_amount <= circulating_supply,
                BondingCurveError::InvalidTokenAmount
            );
        }
        
        Ok(())
    }
}

#[account]
pub struct UserVolumeAccumulator {
    /// User's public key
    pub user: Pubkey,
    /// Total volume in SOL
    pub volume_sol: u64,
    /// Total volume in tokens
    pub volume_tokens: u64,
    /// Number of trades
    pub trades_count: u32,
    /// Last trade timestamp
    pub last_trade_timestamp: i64,
    /// PDA bump
    pub bump: u8,
    /// Reserved space
    pub reserved: [u64; 2],
}

impl UserVolumeAccumulator {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 + // volume_sol
        8 + // volume_tokens
        4 + // trades_count
        8 + // last_trade_timestamp
        1 + // bump
        16; // reserved
}

// Multi-sig validation helpers
pub fn verify_admin_authority(authority: &Pubkey) -> Result<()> {
    require!(
        authority.to_bytes() == crate::constants::ADMIN_WALLET_PUBKEY,
        BondingCurveError::InvalidAdminAuthority
    );
    Ok(())
}

pub fn verify_multisig_authority(authority: &Pubkey) -> Result<()> {
    require!(
        authority.to_bytes() == crate::constants::MULTISIG_WALLET_PUBKEY,
        BondingCurveError::InvalidMultisigAuthority
    );
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AuthorityType {
    Admin,
    Multisig,
    Platform,
    Creator,
}

