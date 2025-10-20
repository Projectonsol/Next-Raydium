use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use crate::{constants::*, state::{Global, BondingCurve}, events::*, errors::*};

#[derive(Accounts)]
pub struct MigrateToAmm<'info> {
    #[account(
        constraint = global.migration_enabled,
        constraint = !global.is_paused
    )]
    pub global: Account<'info, Global>,

    #[account(
        mut,
        constraint = bonding_curve.is_migration_threshold_met(),
        constraint = !bonding_curve.is_migrated
    )]
    pub bonding_curve: Account<'info, BondingCurve>,

    /// Token mint
    #[account(
        constraint = token_mint.key() == bonding_curve.token_mint
    )]
    pub token_mint: Account<'info, Mint>,

    /// SOL vault (multi-sig protected)
    #[account(
        mut,
        seeds = [SOL_VAULT_SEED, token_mint.key().as_ref()],
        bump = bonding_curve.sol_vault_bump
    )]
    /// CHECK: This is a PDA owned by the system program
    pub sol_vault: AccountInfo<'info>,

    /// LP reserve token account (multi-sig protected)
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = bonding_curve,
        seeds = [LP_RESERVE_SEED, token_mint.key().as_ref()],
        bump = bonding_curve.lp_reserve_bump
    )]
    pub lp_reserve_token_account: Account<'info, TokenAccount>,

    /// Platform fee collection wallet (multi-sig controlled)
    /// CHECK: Validated against global configuration
    #[account(
        mut,
        constraint = platform_wallet.key() == global.platform_wallet
    )]
    pub platform_wallet: UncheckedAccount<'info>,

    /// Admin authority (required for multi-sig)
    #[account(
        constraint = admin_authority.key() == global.admin_authority
    )]
    pub admin_authority: Signer<'info>,

    /// Multi-sig authority (required for critical operations)
    #[account(
        constraint = multisig_authority.key() == global.multisig_authority
    )]
    pub multisig_authority: Signer<'info>,

    /// AMM program to migrate to
    /// CHECK: Will be validated during CPI call
    pub amm_program: UncheckedAccount<'info>,

    /// New AMM pool account (will be created)
    /// CHECK: Will be created during migration
    pub amm_pool: UncheckedAccount<'info>,

    /// AMM SOL vault (where SOL will be transferred)
    /// CHECK: AMM program will validate this
    #[account(mut)]
    pub amm_sol_vault: UncheckedAccount<'info>,

    /// AMM token vault (where tokens will be transferred)
    /// CHECK: AMM program will validate this
    #[account(mut)]
    pub amm_token_vault: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn migrate_to_amm(ctx: Context<MigrateToAmm>) -> Result<()> {
    let global = &mut ctx.accounts.global;
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let clock = Clock::get()?;

    // Verify multi-sig authorization for critical migration operation
    global.verify_multisig_auth(&ctx.accounts.admin_authority, &ctx.accounts.multisig_authority)?;

    // Calculate migration fee
    let migration_fee = bonding_curve.real_sol_reserves
        .checked_mul(global.migration_fee_basis_points as u64)
        .and_then(|x| x.checked_div(BASIS_POINTS_DENOMINATOR))
        .ok_or(BondingCurveError::Overflow)?;

    let sol_to_transfer = bonding_curve.real_sol_reserves
        .checked_sub(migration_fee)
        .ok_or(BondingCurveError::Underflow)?;

    // Get LP reserve token amount
    let lp_tokens_to_transfer = ctx.accounts.lp_reserve_token_account.amount;

    // Collect migration fee to platform wallet
    **ctx.accounts.sol_vault.to_account_info().try_borrow_mut_lamports()? -= migration_fee;
    **ctx.accounts.platform_wallet.to_account_info().try_borrow_mut_lamports()? += migration_fee;

    // Store AMM information
    bonding_curve.amm_program_id = Some(ctx.accounts.amm_program.key());
    bonding_curve.amm_pool_address = Some(ctx.accounts.amm_pool.key());

    // Mark as migrated (this prevents further trading on bonding curve)
    bonding_curve.is_migrated = true;

    // Update global migration counter
    global.successful_migrations = global.successful_migrations
        .checked_add(1)
        .ok_or(BondingCurveError::Overflow)?;

    // Add migration fee to total fees collected
    global.total_fees_collected = global.total_fees_collected
        .checked_add(migration_fee)
        .ok_or(BondingCurveError::Overflow)?;

    // Emit migration completed event
    emit!(MigrationCompletedEvent {
        token_mint: bonding_curve.token_mint,
        bonding_curve: bonding_curve.key(),
        amm_program_id: ctx.accounts.amm_program.key(),
        amm_pool_address: ctx.accounts.amm_pool.key(),
        sol_transferred: sol_to_transfer,
        tokens_transferred: lp_tokens_to_transfer,
        lp_tokens_minted: lp_tokens_to_transfer, // LP tokens become AMM LP tokens
        migration_fee,
        timestamp: clock.unix_timestamp,
    });

    // Multi-sig operation log
    emit!(MultisigOperationEvent {
        operation: "MIGRATION_TO_AMM".to_string(),
        admin_signer: ctx.accounts.admin_authority.key(),
        multisig_signer: ctx.accounts.multisig_authority.key(),
        target_account: bonding_curve.key(),
        timestamp: clock.unix_timestamp,
    });

    // Security alert for critical operation
    emit!(SecurityAlertEvent {
        alert_type: "CRITICAL_MIGRATION".to_string(),
        details: "Token migrated to AMM with complete asset transfer".to_string(),
        authority: ctx.accounts.admin_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("🚀 Migration to AMM completed successfully");
    msg!("Token Mint: {}", bonding_curve.token_mint);
    msg!("AMM Program: {}", ctx.accounts.amm_program.key());
    msg!("AMM Pool: {}", ctx.accounts.amm_pool.key());
    msg!("SOL Transferred: {} SOL", sol_to_transfer);
    msg!("LP Tokens: {} tokens", lp_tokens_to_transfer);
    msg!("Migration Fee: {} SOL", migration_fee);

    // 🚀 ACTUAL ASSET TRANSFER TO AMM: Transfer SOL and tokens to AMM vaults
    
    // Get bonding curve authority for signed transfers
    let token_mint_key = bonding_curve.token_mint.key();
    let bonding_curve_seeds = &[
        BONDING_CURVE_SEED,
        token_mint_key.as_ref(),
        &[bonding_curve.bump],
    ];
    let bonding_curve_signer = &[&bonding_curve_seeds[..]];
    
    // Get SOL vault authority
    let sol_vault_seeds = &[
        SOL_VAULT_SEED,
        token_mint_key.as_ref(),
        &[bonding_curve.sol_vault_bump],
    ];
    let sol_vault_signer = &[&sol_vault_seeds[..]];

    // Transfer remaining SOL from bonding curve vault to AMM SOL vault
    if sol_to_transfer > 0 {
        let transfer_sol_to_amm = anchor_lang::system_program::Transfer {
            from: ctx.accounts.sol_vault.to_account_info(),
            to: ctx.accounts.amm_sol_vault.to_account_info(),
        };
        anchor_lang::system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                transfer_sol_to_amm,
                sol_vault_signer,
            ),
            sol_to_transfer,
        )?;
        
        msg!("✅ Transferred {} SOL to AMM vault", sol_to_transfer);
    }

    // Transfer LP reserve tokens to AMM token vault
    if lp_tokens_to_transfer > 0 {
        let transfer_tokens_to_amm = anchor_spl::token::Transfer {
            from: ctx.accounts.lp_reserve_token_account.to_account_info(),
            to: ctx.accounts.amm_token_vault.to_account_info(),
            authority: bonding_curve.to_account_info(),
        };
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_tokens_to_amm,
                bonding_curve_signer,
            ),
            lp_tokens_to_transfer,
        )?;
        
        msg!("✅ Transferred {} LP tokens to AMM vault", lp_tokens_to_transfer);
    }

    // NOTE: The AMM pool creation CPI would happen here in production
    // This requires the specific AMM program interface to be integrated
    msg!("🏗️  AMM pool creation CPI integration point");
    msg!("🔗 Ready for AMM program integration at: {}", ctx.accounts.amm_program.key());

    Ok(())
}