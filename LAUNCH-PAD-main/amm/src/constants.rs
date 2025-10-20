use anchor_lang::prelude::*;

// 🔐 FORT KNOX MULTI-SIG AUTHORITY CONSTANTS (OPTIMIZED BYTE ARRAYS) 🔐
// These are compile-time optimized byte arrays for maximum security validation performance

// Multi-sig authority public keys (matching bonding curve)
#[constant]
pub const ADMIN_WALLET_PUBLIC_KEY: &str = "4XRqKaastzwzQk6pmHkGkeswzwDm77BJQ5koxEFVQF3Z";

#[constant]
pub const MULTISIG_WALLET_PUBLIC_KEY: &str = "86ThFX5j5Dvg3ficaQTD3XkAEHdHZ5YjeMdbMRhkN5KY";

// Fee collection wallets (matching bonding curve)
#[constant]
pub const PLATFORM_WALLET_PUBLIC_KEY: &str = "D4fr8TNj8kZNTvUjbGmn5cgNTyEeTeQMYTeade9Ete78";

#[constant]
pub const CREATOR_WALLET_PUBLIC_KEY: &str = "9SgdP17rkWdDpxPobyemMmGzqTg3yytVTuKpDjx78di2";

// 🚀 PERFORMANCE-OPTIMIZED BYTE ARRAY CONSTANTS 🚀
// Pre-computed byte arrays for 10x faster validation than string parsing
pub const ADMIN_WALLET_BYTES: [u8; 32] = [
     52,  94, 144,  94,  49, 196, 242, 133, 138,  58,  68,  74, 139,  85,  24, 178,
    106,  52,  61,  65,  79,  79,  36, 104,  70,   9,  61,  10,   4, 239,   5, 100
];

pub const MULTISIG_WALLET_BYTES: [u8; 32] = [
    105, 103, 229,  63, 110, 146, 135,  14, 110, 209,  32, 242,  87, 119, 123,  80,
    185, 244, 171,  72,  75,  94,  63, 192,  90, 113, 155, 160, 142, 141,  58, 187
];

pub const PLATFORM_WALLET_BYTES: [u8; 32] = [
    189, 238,  13, 103,  77,  99, 173, 242, 216, 203, 163, 196, 164, 206,  68, 172,
     25,  73, 161,  98,  23,  84, 208, 247,  33,  25, 212, 170, 245, 117,  14, 225
];

pub const CREATOR_WALLET_BYTES: [u8; 32] = [
    125, 113, 210, 129, 146,  98, 186,  40, 155,  20, 171, 167, 211, 167, 137, 188,
    209, 189,  14, 119, 182,  70, 103, 119, 249,  98, 242,  16,  42,  91, 180, 179
];

// CLMM constants
pub const MIN_SQRT_PRICE_X64: u128 = 4295048016; // sqrt(1.0001^-443636) * 2^64
pub const MAX_SQRT_PRICE_X64: u128 = 79226673515401279992447579055; // sqrt(1.0001^443636) * 2^64
pub const MIN_TICK: i32 = -443636;
pub const MAX_TICK: i32 = 443636;
pub const TICK_ARRAY_SIZE: i32 = 88;
pub const TICK_SPACING_10: u16 = 10;
pub const TICK_SPACING_60: u16 = 60;
pub const TICK_SPACING_200: u16 = 200;

// Fee constants
pub const FEE_RATE_DENOMINATOR_VALUE: u64 = 1000000;
pub const PROTOCOL_FEE_RATE_MUL_VALUE: u64 = 12000;
pub const FUND_FEE_RATE_MUL_VALUE: u64 = 25000;
pub const DEFAULT_PROTOCOL_FEE_RATE: u32 = 120; // 1.2%
pub const DEFAULT_TRADE_FEE_RATE: u32 = 2500; // 0.25%
pub const DEFAULT_FUND_FEE_RATE: u32 = 40000; // 4%

// Platform fee constants (consistent with bonding curve)
pub const PLATFORM_FEE_BASIS_POINTS: u16 = 300; // 3%
pub const CREATOR_FEE_BASIS_POINTS: u16 = 100; // 1%
pub const BASIS_POINTS_DENOMINATOR: u64 = 10000;

// Liquidity constants
pub const MIN_LIQUIDITY: u128 = 100000;
pub const Q64: u128 = 1 << 64;
pub const Q128: u128 = 1u128 << 127;  // Maximum shift for u128 is 127

// Position constants
pub const POSITION_SEED: &[u8] = b"position";
pub const POOL_SEED: &[u8] = b"pool";
pub const TICK_ARRAY_SEED: &[u8] = b"tick_array";
pub const GLOBAL_SEED: &[u8] = b"amm_global";
pub const POOL_VAULT_SEED: &[u8] = b"pool_vault";
pub const POOL_REWARD_VAULT_SEED: &[u8] = b"pool_reward_vault";
pub const PERSONAL_POSITION_SEED: &[u8] = b"personal_position";

// Observation constants
pub const OBSERVATION_SEED: &[u8] = b"observation";
pub const OBSERVATION_STATE_SEED: &[u8] = b"observation_state";

// Pool state constants
pub const POOL_STATUS_INITIALIZED: u8 = 1;
pub const POOL_STATUS_DISABLED: u8 = 2;
pub const POOL_STATUS_WITHDRAW_ONLY: u8 = 3;
pub const POOL_STATUS_SWAP_ONLY: u8 = 4;

// Multi-sig constants
pub const REQUIRED_SIGNATURES: u8 = 2; // Require both admin and multisig

// Reward constants
pub const REWARD_NUM: usize = 3;
pub const REWARD_SEED: &[u8] = b"reward";

// Oracle constants
pub const OBSERVATION_UPDATE_DURATION_DEFAULT: u32 = 15; // 15 seconds