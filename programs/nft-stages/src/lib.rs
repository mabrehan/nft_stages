use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use {anchor_spl::token::Mint,
    anchor_spl::metadata::MetadataAccount,
    mpl_token_metadata::{instruction::update_metadata_accounts_v2, state::{Metadata, DataV2}}};

declare_id!("3fmuFJf2auxKMBJsx5YrUyRosi9yBGUz4vyPhLZTCY6P");

#[program]
pub mod nft_stages {
    use std::fs::ReadDir;

    use super::*;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        msg!("Initialize");
        let nft_pda = ctx.accounts.nft_pda.as_mut();

        nft_pda.is_initialized = true;  

        nft_pda.bump = *ctx.bumps.get("nft_pda").ok_or(ProgramError::InvalidSeeds)?;
        nft_pda.nft_authority_bump = *ctx.bumps.get("nft_update_authority").ok_or(ProgramError::InvalidSeeds)?;

        nft_pda.level = 1;
        nft_pda.mint = ctx.accounts.mint.key();

        Ok(())
    }

    pub fn level_up(ctx: Context<LevelUp>) -> Result<()> {
        msg!("Level Up");
        let nft_pda = ctx.accounts.nft_pda.as_mut();
        let metadata_account = ctx.accounts.metadata_account.as_mut();

        if nft_pda.level >= 6 {
            msg!("Already at Max level");
            return Ok(())
        }

        nft_pda.level = nft_pda.level + 1;
        // nft_pda.level = 2;
        // msg!("metadata.uri {:?}", metadata_account.data.uri);

        let str = metadata_account.data.uri.clone();
        let str = str.as_str().trim_end_matches('\0');
        msg!("new uri 1: {:?}", str);
        let mut new_uri: String = str[0..(str.len()-6)].to_string();
        msg!("new uri 2: {:?}", new_uri);

        match nft_pda.level {
            1_u8 => new_uri.push_str(&"1.json"),
            2_u8 => new_uri.push_str(&"2.json"),
            3_u8 => new_uri.push_str(&"3.json"),
            4_u8 => new_uri.push_str(&"4.json"),
            5_u8 => new_uri.push_str(&"5.json"),
            6_u8 => new_uri.push_str(&"6.json"),
            _ => new_uri.push_str(&"1.json"),
        };
        msg!("new uri 3: {:?}", new_uri);

        let account_info = vec![
            metadata_account.to_account_info(),
            ctx.accounts.nft_update_authority.to_account_info(),
        ];

        invoke_signed(
            &update_metadata_accounts_v2(
                ctx.accounts.metadata_program.key(),
                metadata_account.key(),
                ctx.accounts.nft_update_authority.key(),
                None,
                Some (DataV2 { 
                    name: metadata_account.data.name.clone(), 
                    symbol: metadata_account.data.symbol.clone(), 
                    uri:  new_uri,
                    seller_fee_basis_points: metadata_account.data.seller_fee_basis_points, 
                    creators: metadata_account.data.creators.clone(), 
                    collection: metadata_account.collection.clone(), 
                    uses: metadata_account.uses.clone() 
                }),
                Some(metadata_account.primary_sale_happened),
                Some(metadata_account.is_mutable)
            ),
            account_info.as_slice(),
            &[&[b"update_authority", &[nft_pda.nft_authority_bump]]]
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub metadata_account: Box<Account<'info, MetadataAccount>>,

    #[account(
        init_if_needed,
        payer = owner,
        space = NftPda::LEN,
        seeds = [b"DEFIxNFT", mint.key().as_ref()],
        bump 
    )]
    pub nft_pda: Box<Account<'info, NftPda>>,

    /// CHECK:` doc comment explaining why no checks through types are necessary.
    #[account(
        init_if_needed,
        payer = owner,
        space = 0,
        seeds = [b"update_authority"],
        bump
    )]
    pub nft_update_authority: AccountInfo<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct LevelUp<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub metadata_account: Box<Account<'info, MetadataAccount>>,

    #[account(
        mut,
        seeds = [b"DEFIxNFT",
                mint.key().as_ref()],
        bump = nft_pda.bump
    )]
    pub nft_pda: Box<Account<'info, NftPda>>,

    #[account(
        mut,
        seeds = [b"update_authority"],
        bump = nft_pda.nft_authority_bump // match with the stored bump
    )]
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    pub nft_update_authority: AccountInfo<'info>,

    #[account(address = mpl_token_metadata::ID)]
    /// CHECK:` doc comment explaining why no checks through types are necessary.
    pub metadata_program: UncheckedAccount<'info>, 
}

#[account]
#[derive(Default, Debug)]
pub struct NftPda {
    pub is_initialized: bool,
    pub bump: u8,
    pub nft_authority_bump: u8,

    pub level: u8,

    pub mint: Pubkey,
}

impl NftPda {
    pub const LEN: usize = 8 + std::mem::size_of::<NftPda>();
    
}