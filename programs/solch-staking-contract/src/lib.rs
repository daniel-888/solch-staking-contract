use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Token, Mint};
use anchor_lang::solana_program::{pubkey::Pubkey,clock, entrypoint::ProgramResult, program_error::ProgramError};
use std::convert::Into;
use crate::constants::*;
declare_id!("6Pf6bCr94Y8UFwDWneMbyWNUDtv9LRowVwGR9DzKUACD");

mod constants {
    use crate::TYPE;
    pub const DECIMAL: u64 = 1000000000;
    pub const DAYTIME: u32 = 86400; //86400 on mainnet
    pub const REWARDS: [u64;5] = [75000 * DECIMAL/2, 150000 * DECIMAL/2, 300000 * DECIMAL/2, 375000 * DECIMAL/2, 600000 * DECIMAL/2];
    pub const MAX: [u64; 5] = [3671328671 * 10000/2, 4368932039 * 10000/2, 7068062827 * 10000/2, 4115853659 * 10000/2, 5357142857 * 10000/2];
    pub const TYPE0: TYPE = TYPE{
        days: 7,
        fee: 1,
        min_amount: 20,
        apy: 2
    };
    pub const TYPE1: TYPE = TYPE{
        days: 30,
        fee: 1,
        min_amount: 100,
        apy: 20
    };
    pub const TYPE2: TYPE = TYPE{
        days: 90,
        fee: 1,
        min_amount: 500,
        apy: 80
    };
    pub const TYPE3: TYPE = TYPE{
        days: 180,
        fee: 1,
        min_amount: 1000,
        apy: 400
    };
    pub const TYPE4: TYPE = TYPE{
        days: 365,
        fee: 1,
        min_amount: 2000,
        apy: 1020
    };
    pub const TYPES: [TYPE; 5] = [TYPE0, TYPE1, TYPE2, TYPE3, TYPE4];
}

#[program]
pub mod solch_staking_contract {
    use super::*;
    use anchor_lang::solana_program::pubkey;
    pub fn create_vault(_ctx: Context<CreateVaultContext>, _bump_vault: u8) -> ProgramResult {
        Ok(())
    }

    pub fn create_data_account(_ctx: Context<CreateDataContext>, _bump_data: u8) -> ProgramResult {
       let data = &mut _ctx.accounts.data;
        data.total_locked_amount = 0;
        data.fees = 0;
        for index in 0..5 {
            data.reward_info.push(REWARD { total_amount: REWARDS[index], available_amount: REWARDS[index], claimed_amount: 0, max_amount: 0 });
        }
        data.mint = pubkey!("EKSM2sjtptnvkqq79kwfAaSfVudNAtFYZSBdPe5jeRSt");
       Ok(())
    }
    pub fn create_pool_signer(_ctx: Context<CreatePoolSignerContext>, _bump_signer: u8) -> ProgramResult {
        Ok(())
    }
    pub fn create_pool(_ctx: Context<CreatePoolContext>, _bump_pool: u8, _index: u8) -> ProgramResult {
        let pool = &mut _ctx.accounts.pool;
        let clock = clock::Clock::get().unwrap();
        pool.user = _ctx.accounts.user.key();
        pool.reward = 0;
        pool.staked_amount = 0;
        pool.index = _index;
        pool.start_time = clock.unix_timestamp as u32;
        pool.fee = TYPES[_index as usize].fee;
        pool.lock_time = TYPES[_index as usize].days as u32 * DAYTIME;
        pool.apy = TYPES[_index as usize].apy;
        pool.min_amount = TYPES[_index as usize].min_amount as u64 * DECIMAL;
        pool.is_staked = false;
        Ok(())
    }
    pub fn stake(_ctx: Context<StakeContext>, _amount: u32, _amount_second: u32 , _index: u8) -> ProgramResult {
        let pool = &mut _ctx.accounts.pool;
        let data = &mut _ctx.accounts.data;

        if pool.index != _index {
            return Err(ProgramError::InvalidArgument);
        }
        let transfer_amount = _amount as u64 * DECIMAL + _amount_second as u64;
        if pool.min_amount > transfer_amount {
            return Err(ProgramError::InsufficientFunds);
        }
        
        pool.staked_amount = transfer_amount * (100 - pool.fee) as u64 / 100;//change as percent
        data.fees += transfer_amount * pool.fee as u64 / 100;//change consider percent
        
        let able_reward = pool.staked_amount * pool.apy as u64 / 100;
        data.reward_info[_index as usize].available_amount -= able_reward;
        data.reward_info[_index as usize].max_amount += pool.staked_amount;
        if data.reward_info[_index as usize].max_amount > MAX[_index as usize] {
            return Err(ProgramError::MaxAccountsDataSizeExceeded);
        }
        let clock = clock::Clock::get().unwrap();
        pool.start_time = clock.unix_timestamp as u32;
        pool.end_time = pool.start_time + pool.lock_time;
        pool.reward = 0;
        let cpi_ctx = CpiContext::new(
            _ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: _ctx.accounts.token_from.to_account_info(),
                    to: _ctx.accounts.token_to.to_account_info(),
                    authority: _ctx.accounts.user.to_account_info() 
                }
        ); 
        token::transfer(cpi_ctx, transfer_amount.into())?;
        data.total_locked_amount += pool.staked_amount;
        pool.is_staked = true;
        Ok(())
    }
    pub fn claim(_ctx: Context<ClaimContext>, _bump_vault: u8) -> ProgramResult {
        let pool =  &mut _ctx.accounts.pool;
        let data = &mut _ctx.accounts.data;
        let clock = clock::Clock::get().unwrap();
        if pool.end_time < clock.unix_timestamp as u32 {
            return Err(ProgramError::InvalidArgument);
        }

        let life_time = clock.unix_timestamp as u32 - pool.start_time;
        pool.reward = life_time as u64 * pool.staked_amount * pool.apy as u64 / 100 / pool.lock_time as u64; 
        let transfer_amount = pool.reward; 
        
        let vault_seeds = &[
            b"rewards vault".as_ref(),
            &[_bump_vault],
        ];

        let vault_signer = &[&vault_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.reward_from.to_account_info(),
                to: _ctx.accounts.reward_to.to_account_info(),
                authority: _ctx.accounts.vault.to_account_info()
            },
            vault_signer
        ); 
        token::transfer(cpi_ctx, transfer_amount.into())?;
        // pool.reward = 0;// to read current claimed amount from backend side;
        pool.start_time = clock.unix_timestamp as u32;
        data.reward_info[pool.index as usize].claimed_amount += transfer_amount;
        data.reward_info[pool.index as usize].total_amount -= transfer_amount;
        Ok(())
    }
    pub fn unstake(_ctx: Context<UnstakeContext>, _bump_signer: u8, _bump_vault: u8) -> ProgramResult {
        let pool = &mut _ctx.accounts.pool;
        let data = &mut _ctx.accounts.data;
        let clock = clock::Clock::get().unwrap();

        if pool.end_time > clock.unix_timestamp as u32 {
            // pool.reward = 0;
            return Err(ProgramError::InvalidArgument);//can't unstake until locktime
        }
        let life_time = pool.end_time - pool.start_time;
        let reward_transfer_amount = life_time as u64 * pool.staked_amount * pool.apy as u64 / 100 / pool.lock_time as u64; 
        
        pool.reward = reward_transfer_amount;

        let vault_seeds = &[
        b"rewards vault".as_ref(),
        &[_bump_vault],
        ];
        let vault_signer = &[&vault_seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.reward_from.to_account_info(),
                to: _ctx.accounts.reward_to.to_account_info(),
                authority: _ctx.accounts.vault.to_account_info()
            },
            vault_signer
        ); 
        token::transfer(cpi_ctx, reward_transfer_amount.into())?;

        data.reward_info[pool.index as usize].claimed_amount += reward_transfer_amount;
        data.reward_info[pool.index as usize].total_amount -= reward_transfer_amount;
        let transfer_amount = pool.staked_amount;
        
        let pool_signer_seeds = &[
            b"pool signer".as_ref(),
            _ctx.accounts.user.to_account_info().key.as_ref(),
            &[_bump_signer],
        ];

        let pool_signer = &[&pool_signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            _ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: _ctx.accounts.token_from.to_account_info(),
                to: _ctx.accounts.token_to.to_account_info(),
                authority: _ctx.accounts.pool_signer.to_account_info()
            },
            pool_signer
        );
        token::transfer(cpi_ctx, transfer_amount.into())?;
        data.total_locked_amount -= pool.staked_amount;
        data.reward_info[pool.index as usize].max_amount -= pool.staked_amount;
        pool.staked_amount = 0;
        pool.start_time = pool.end_time;
        pool.is_staked = false;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct StakeContext<'info> {
    pub user: Signer<'info>,
    #[account(mut, has_one = mint)]
    pub data: Account<'info, Data>,
    #[account(mut, has_one = user)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_from: Box<Account<'info, TokenAccount>>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_to: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimContext<'info> {
    #[account(mut, has_one = user)]
    pub pool: Account<'info, Pool>,
    pub vault: Account<'info, Vault>, // this vault account
    pub user: Signer<'info>,
    #[account(mut)]
    pub data: Account<'info, Data>,
    #[account(mut)]
    pub reward_from: Box<Account<'info, TokenAccount>>, // vault token account
    #[account(mut)]
    pub reward_to: Box<Account<'info, TokenAccount>>, // user token account
    pub token_program: Program<'info, Token>
}
#[derive(Accounts)]
pub struct UnstakeContext<'info> {
    #[account(mut, has_one = user)]
    pub pool: Account<'info, Pool>,
    pub pool_signer: Account<'info, PoolSigner>,
    pub vault: Account<'info, Vault>, // this vault account
    pub user: Signer<'info>,
    #[account(mut)]
    pub data: Account<'info, Data>,
    #[account(mut)]
    pub reward_from: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub reward_to: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub token_from: Box<Account<'info, TokenAccount>>, // vault token account
    #[account(mut)]
    pub token_to: Box<Account<'info, TokenAccount>>, // user token account
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateVaultContext<'info> {
    #[account(init, seeds = [b"rewards vault".as_ref()], bump, payer = admin, space = 8 + 1)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreateDataContext<'info> {
    #[account(init, seeds = [b"pool data".as_ref()], bump, payer = admin, space = 8 + 8 + 8 + 124 + 40 + 32)]
    pub data: Account<'info, Data>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}
#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct CreatePoolSignerContext<'info> {
    #[account(init, seeds = [b"pool signer".as_ref(), user.key.as_ref()], bump, payer = user, space = 8 + 1)]
    pub pool_signer: Account<'info, PoolSigner>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(bump: u8, index: u8)]
pub struct CreatePoolContext<'info> {
    #[account(init, seeds = [format!("{}{}","pool", index).as_bytes().as_ref(), user.to_account_info().key.as_ref()], bump, payer = user, space = 8 + 32 + 8 + 8 + 4 + 4 + 1 + 4 + 1 + 8 + 1 + 1 + 1)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[account]
#[derive(Default, Debug)]
pub struct Data {
    pub total_locked_amount: u64,
    pub fees: u64,
    pub reward_info: Vec<REWARD>,
    pub mint: Pubkey
}

#[account]
pub struct Pool {
    pub user: Pubkey,
    pub reward: u64,
    pub staked_amount: u64,
    pub start_time: u32,
    pub end_time: u32,
    pub index: u8,
    pub lock_time: u32,
    pub fee: u8,
    pub min_amount: u64,
    pub apy: u16,
    pub is_staked: bool,
}
#[account]
pub struct Vault {
    pub bump_vault: u8
}
#[account]
pub struct PoolSigner {
    pub bump_signer: u8
}
#[account]
pub struct TYPE {
    pub days: u16,
    pub fee: u8,
    pub min_amount: u16,
    pub apy: u16
}
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct REWARD {
    pub total_amount: u64,
    pub available_amount: u64,
    pub claimed_amount: u64,
    pub max_amount: u64
}