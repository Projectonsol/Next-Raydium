use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Invalid admin authority - multi-sig required")]
    InvalidAdminAuthority,
    
    #[msg("Invalid multisig authority - multi-sig required")]
    InvalidMultisigAuthority,
    
    #[msg("Multi-sig authorization required for this operation")]
    MultisigRequired,
    
    #[msg("Operations are currently paused")]
    OperationsPaused,
    
    #[msg("Invalid tick range")]
    InvalidTickRange,
    
    #[msg("Tick out of bounds")]
    TickOutOfBounds,
    
    #[msg("Invalid tick spacing")]
    InvalidTickSpacing,
    
    #[msg("Tick not initialized")]
    TickNotInitialized,
    
    #[msg("Tick already initialized")]
    TickAlreadyInitialized,
    
    #[msg("Invalid sqrt price")]
    InvalidSqrtPrice,
    
    #[msg("Invalid liquidity amount")]
    InvalidLiquidityAmount,
    
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[msg("Mathematical overflow")]
    Overflow,
    
    #[msg("Mathematical underflow")]
    Underflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Invalid token amount")]
    InvalidTokenAmount,
    
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    
    #[msg("Insufficient token balance")]
    InsufficientTokenBalance,
    
    #[msg("Pool already initialized")]
    PoolAlreadyInitialized,
    
    #[msg("Pool not initialized")]
    PoolNotInitialized,
    
    #[msg("Pool is disabled")]
    PoolDisabled,
    
    #[msg("Position not found")]
    PositionNotFound,
    
    #[msg("Position already exists")]
    PositionAlreadyExists,
    
    #[msg("Invalid position")]
    InvalidPosition,
    
    #[msg("Fee rate too high")]
    FeeTooHigh,
    
    #[msg("Invalid fee rate")]
    InvalidFeeRate,
    
    #[msg("Insufficient fees")]
    InsufficientFees,
    
    #[msg("Invalid price calculation")]
    InvalidPriceCalculation,
    
    #[msg("Invalid tick array")]
    InvalidTickArray,
    
    #[msg("Tick array not initialized")]
    TickArrayNotInitialized,
    
    #[msg("Invalid reward index")]
    InvalidRewardIndex,
    
    #[msg("Reward not initialized")]
    RewardNotInitialized,
    
    #[msg("Reward already initialized")]
    RewardAlreadyInitialized,
    
    #[msg("Invalid reward authority")]
    InvalidRewardAuthority,
    
    #[msg("Invalid reward amount")]
    InvalidRewardAmount,
    
    #[msg("Oracle not updated")]
    OracleNotUpdated,
    
    #[msg("Invalid oracle data")]
    InvalidOracleData,
    
    #[msg("Platform wallet mismatch")]
    PlatformWalletMismatch,
    
    #[msg("Creator wallet mismatch")]
    CreatorWalletMismatch,
    
    #[msg("Invalid account owner")]
    InvalidAccountOwner,
    
    #[msg("Account not initialized")]
    AccountNotInitialized,
    
    #[msg("Account already initialized")]
    AccountAlreadyInitialized,
    
    #[msg("Invalid PDA derivation")]
    InvalidPDA,
    
    #[msg("Cross-program invocation failed")]
    CPIFailed,
    
    #[msg("Vault access denied - multi-sig required")]
    VaultAccessDenied,
    
    #[msg("LP token access denied - multi-sig required")]
    LPTokenAccessDenied,
    
    #[msg("Critical operation requires both admin and multisig signatures")]
    CriticalOperationRequiresMultisig,
    
    #[msg("Unauthorized access to protected resources")]
    UnauthorizedAccess,
    
    #[msg("Security check failed")]
    SecurityCheckFailed,
    
    #[msg("Pool creation fee not paid")]
    PoolCreationFeeNotPaid,
    
    #[msg("Invalid bonding curve program")]
    InvalidBondingCurveProgram,
    
    #[msg("Migration not authorized")]
    MigrationNotAuthorized,
    
    #[msg("Pool configuration invalid")]
    PoolConfigurationInvalid,
    
    #[msg("Tick calculation failed")]
    TickCalculationFailed,
    
    #[msg("Liquidity calculation failed")]
    LiquidityCalculationFailed,
    
    #[msg("Price calculation failed")]
    PriceCalculationFailed,
    
    #[msg("Fee calculation failed")]
    FeeCalculationFailed,
}