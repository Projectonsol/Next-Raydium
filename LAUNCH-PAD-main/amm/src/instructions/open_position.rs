use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, MintTo},
    metadata::{
        create_metadata_accounts_v3,
        mpl_token_metadata::types::{Creator, DataV2, CollectionDetails},
        CreateMetadataAccountsV3, Metadata,
    },
};
use crate::{constants::*, state::{AmmGlobal, Pool, RewardInfo, Position, TickArray, Tick, PersonalPosition}, events::*, errors::*};

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(
        constraint = !amm_global.is_paused 
    )]
    pub amm_global: Account<'info, AmmGlobal>,

    #[account(
        constraint = pool.status == POOL_STATUS_INITIALIZED 
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = position_owner,
        space = Position::LEN,
        seeds = [POSITION_SEED, position_mint.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,

    #[account(
        init,
        payer = position_owner,
        mint::decimals = 0,
        mint::authority = position,
        mint::freeze_authority = position,
    )]
    pub position_mint: Account<'info, Mint>,

    /// Position metadata account (NFT)
    /// CHECK: Created via CPI to metadata program
    #[account(
        mut,
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            position_mint.key().as_ref()
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// Position NFT token account
    #[account(
        init,
        payer = position_owner,
        associated_token::mint = position_mint,
        associated_token::authority = position_owner
    )]
    pub position_token_account: Account<'info, TokenAccount>,

    /// Personal position tracking
    #[account(
        init,
        payer = position_owner,
        space = PersonalPosition::LEN,
        seeds = [PERSONAL_POSITION_SEED, position_owner.key().as_ref(), pool.key().as_ref()],
        bump
    )]
    pub personal_position: Account<'info, PersonalPosition>,

    #[account(mut)]
    pub position_owner: Signer<'info>,

    pub metadata_program: Program<'info, Metadata>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn open_position(
    ctx: Context<OpenPosition>,
    tick_lower: i32,
    tick_upper: i32,
) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let position = &mut ctx.accounts.position;
    let personal_position = &mut ctx.accounts.personal_position;
    let clock = Clock::get()?;

    // Validate tick range
    require!(tick_lower < tick_upper, AmmError::InvalidTickRange);
    require!(
        tick_lower >= MIN_TICK && tick_lower <= MAX_TICK,
        AmmError::TickOutOfBounds
    );
    require!(
        tick_upper >= MIN_TICK && tick_upper <= MAX_TICK,
        AmmError::TickOutOfBounds
    );

    // Check tick spacing alignment
    require!(
        tick_lower % pool.tick_spacing as i32 == 0,
        AmmError::InvalidTickSpacing
    );
    require!(
        tick_upper % pool.tick_spacing as i32 == 0,
        AmmError::InvalidTickSpacing
    );

    // Initialize position state
    position.mint = ctx.accounts.position_mint.key();
    position.owner = ctx.accounts.position_owner.key();
    position.pool_id = pool.key();
    position.tick_lower = tick_lower;
    position.tick_upper = tick_upper;
    position.liquidity = 0;
    position.fee_growth_inside_last_a_x64 = 0;
    position.fee_growth_inside_last_b_x64 = 0;
    position.fees_owed_a = 0;
    position.fees_owed_b = 0;
    position.reward_growth_inside_last = [0; 3];
    position.rewards_owed = [0; 3];
    position.bump = ctx.bumps.position;

    // Initialize personal position tracking
    personal_position.owner = ctx.accounts.position_owner.key();
    personal_position.pool_id = pool.key();
    personal_position.position_mint = ctx.accounts.position_mint.key();
    personal_position.bump = ctx.bumps.personal_position;

    // Mint position NFT using position authority
    let position_mint_key = ctx.accounts.position_mint.key();
    let seeds = &[
        POSITION_SEED,
        position_mint_key.as_ref(),
        &[position.bump],
    ];
    let signer = &[&seeds[..]];

    let mint_to_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.position_mint.to_account_info(),
            to: ctx.accounts.position_token_account.to_account_info(),
            authority: position.to_account_info(),
        },
        signer,
    );
    token::mint_to(mint_to_ctx, 1)?; // Mint 1 NFT

    // Create position NFT metadata
    let metadata_ctx = CpiContext::new_with_signer(
        ctx.accounts.metadata_program.to_account_info(),
        CreateMetadataAccountsV3 {
            metadata: ctx.accounts.metadata_account.to_account_info(),
            mint: ctx.accounts.position_mint.to_account_info(),
            mint_authority: position.to_account_info(),
            update_authority: position.to_account_info(),
            payer: ctx.accounts.position_owner.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        },
        signer,
    );

    let metadata_data = DataV2 {
        name: format!("CLMM Position #{}", position.mint.to_string()[..8].to_uppercase()),
        symbol: "CLMM-POS".to_string(),
        uri: "https://api.example.com/position-metadata".to_string(), // Would be dynamic
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator {
            address: ctx.accounts.position_owner.key(),
            verified: true,
            share: 100,
        }]),
        collection: None,
        uses: None,
    };

    create_metadata_accounts_v3(
        metadata_ctx,
        metadata_data,
        true, // is_mutable
        true, // update_authority_is_signer
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    // Emit position opened event
    emit!(PositionOpenedEvent {
        position_mint: position.mint,
        pool_id: position.pool_id,
        owner: position.owner,
        tick_lower: position.tick_lower,
        tick_upper: position.tick_upper,
        timestamp: clock.unix_timestamp,
    });

    msg!("🎯 CLMM Position opened successfully");
    msg!("Position Mint: {}", position.mint);
    msg!("Pool: {}", position.pool_id);
    msg!("Owner: {}", position.owner);
    msg!("Tick Range: {} to {}", tick_lower, tick_upper);
    msg!("Position NFT minted to owner");

    Ok(())
}