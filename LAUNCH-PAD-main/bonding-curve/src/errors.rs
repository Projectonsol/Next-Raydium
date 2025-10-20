use anchor_lang::prelude::*;

#[error_code]
pub enum BondingCurveError {
    #[msg("Invalid admin authority - multi-sig required")]
    InvalidAdminAuthority,
    
    #[msg("Invalid multisig authority - multi-sig required")]
    InvalidMultisigAuthority,
    
    #[msg("Multi-sig authorization required for this operation")]
    MultisigRequired,
    
    #[msg("Operations are currently paused")]
    OperationsPaused,
    
    #[msg("Migration is currently disabled")]
    MigrationDisabled,
    
    #[msg("Migration threshold not met")]
    MigrationThresholdNotMet,
    
    #[msg("Token already migrated")]
    AlreadyMigrated,
    
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    
    #[msg("Insufficient SOL reserves")]
    InsufficientSolReserves,
    
    #[msg("Insufficient token reserves")]
    InsufficientTokenReserves,
    
    #[msg("Mathematical overflow")]
    Overflow,
    
    #[msg("Mathematical underflow")]
    Underflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    #[msg("Invalid token amount - must be greater than zero")]
    InvalidTokenAmount,
    
    #[msg("Invalid SOL amount - must be greater than zero")]
    InvalidSolAmount,
    
    #[msg("Fee too high - exceeds maximum allowed")]
    FeeTooHigh,
    
    #[msg("Insufficient fees available for collection")]
    InsufficientFees,
    
    #[msg("Invalid price calculation")]
    InvalidPrice,
    
    #[msg("Token creation failed")]
    TokenCreationFailed,
    
    #[msg("Metadata creation failed")]
    MetadataCreationFailed,
    
    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,
    
    #[msg("Invalid token name")]
    InvalidTokenName,
    
    #[msg("Invalid token symbol")]
    InvalidTokenSymbol,
    
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
    
    #[msg("Invalid AMM program")]
    InvalidAmmProgram,
    
    #[msg("AMM pool creation failed")]
    AmmPoolCreationFailed,
    
    #[msg("Reserve vault access denied - multi-sig required")]
    ReserveVaultAccessDenied,
    
    #[msg("LP token access denied - multi-sig required")]
    LPTokenAccessDenied,
    
    #[msg("Critical operation requires both admin and multisig signatures")]
    CriticalOperationRequiresMultisig,
    
    #[msg("Unauthorized access to protected resources")]
    UnauthorizedAccess,
    
    #[msg("Security check failed")]
    SecurityCheckFailed,
    
    #[msg("Invalid lamports transfer amount")]
    InvalidLamportsAmount,
    
    #[msg("CPI transfer failed")]
    CPITransferFailed,
    
    #[msg("Account lamports insufficient")]
    InsufficientLamports,
    
    #[msg("Zero amount transfer not allowed")]
    ZeroAmountTransfer,
}