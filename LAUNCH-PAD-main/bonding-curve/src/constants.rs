use anchor_lang::prelude::*;

// Multi-sig authority public keys (Fort Knox security)
#[constant]
pub const ADMIN_WALLET_PUBLIC_KEY: &str = "4XRqKaastzwzQk6pmHkGkeswzwDm77BJQ5koxEFVQF3Z";

#[constant]
pub const MULTISIG_WALLET_PUBLIC_KEY: &str = "86ThFX5j5Dvg3ficaQTD3XkAEHdHZ5YjeMdbMRhkN5KY";

// Fee collection wallets
#[constant]
pub const PLATFORM_WALLET_PUBLIC_KEY: &str = "DnQbCNpWyR6k2k1mJY6wX7mYxXR3Dh6SrcTMhF2ZoGZv";

#[constant]
pub const CREATOR_WALLET_PUBLIC_KEY: &str = "9SgdP17rkWdDpxPobyemMmGzqTg3yytVTuKpDjx78di2";

// Bonding curve constants
pub const VIRTUAL_SOL_RESERVES: u64 = 30_000_000_000; // 30 SOL
pub const VIRTUAL_TOKEN_RESERVES: u64 = 1_000_000_000_000_000; // 1B tokens (with decimals)
pub const MIGRATION_THRESHOLD: u64 = 70_000_000_000; // 70 SOL
pub const TOTAL_SUPPLY: u64 = 1_000_000_000_000_000; // 1B tokens
pub const LP_RESERVE_PERCENTAGE: u64 = 20; // 20% for LP reserves

// Fee constants
pub const PLATFORM_FEE_BASIS_POINTS: u16 = 300; // 3%
pub const CREATOR_FEE_BASIS_POINTS: u16 = 100; // 1%
pub const MIGRATION_FEE_BASIS_POINTS: u16 = 500; // 5%
pub const MAX_SLIPPAGE_BASIS_POINTS: u16 = 1000; // 10%
pub const BASIS_POINTS_DENOMINATOR: u64 = 10000;

// Seeds for PDAs
pub const GLOBAL_SEED: &[u8] = b"global";
pub const BONDING_CURVE_SEED: &[u8] = b"bonding_curve";
pub const USER_VOLUME_SEED: &[u8] = b"user_volume";
pub const LP_RESERVE_SEED: &[u8] = b"lp_reserve";
pub const SOL_VAULT_SEED: &[u8] = b"sol_vault";
pub const TOKEN_VAULT_SEED: &[u8] = b"token_vault";

// Multi-sig constants
pub const REQUIRED_SIGNATURES: u8 = 2; // Require both admin and multisig

// Compile-time validation constants for efficiency
pub const ADMIN_WALLET_PUBKEY: [u8; 32] = [
    // 4XRqKaastzwzQk6pmHkGkeswzwDm77BJQ5koxEFVQF3Z
     52,  94, 144,  94,  49, 196, 242, 133, 138,  58,  68,  74, 139,  85,  24, 178,
    106,  52,  61,  65,  79,  79,  36, 104,  70,   9,  61,  10,   4, 239,   5, 100
];

pub const MULTISIG_WALLET_PUBKEY: [u8; 32] = [
    // 86ThFX5j5Dvg3ficaQTD3XkAEHdHZ5YjeMdbMRhkN5KY
    105, 103, 229,  63, 110, 146, 135,  14, 110, 209,  32, 242,  87, 119, 123,  80,
    185, 244, 171,  72,  75,  94,  63, 192,  90, 113, 155, 160, 142, 141,  58, 187
];

pub const PLATFORM_WALLET_PUBKEY: [u8; 32] = [
    // DnQbCNpWyR6k2k1mJY6wX7mYxXR3Dh6SrcTMhF2ZoGZv
    189, 238,  13, 103,  77,  99, 173, 242, 216, 203, 163, 196, 164, 206,  68, 172,
     25,  73, 161,  98,  23,  84, 208, 247,  33,  25, 212, 170, 245, 117,  14, 225
];

pub const CREATOR_WALLET_PUBKEY: [u8; 32] = [
    // 9SgdP17rkWdDpxPobyemMmGzqTg3yytVTuKpDjx78di2
    125, 113, 210, 129, 146,  98, 186,  40, 155,  20, 171, 167, 211, 167, 137, 188,
    209, 189,  14, 119, 182,  70, 103, 119, 249,  98, 242,  16,  42,  91, 180, 179
];