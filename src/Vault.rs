use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("1dNP51Nojqs772ez9BtqfAYhsRpkB7uc2iSa7gUfuE5");

#[program]
pub mod asset_manager_vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let (vault_pda, vault_bump) = Pubkey::find_program_address(
            &[b"vault", ctx.accounts.manager.key().as_ref()],
            ctx.program_id,
        );

        require!(
            vault_pda == ctx.accounts.vault.key(),
            ErrorCode::PDASeedsMismatch
        );

        let vault = &mut ctx.accounts.vault;
        vault.manager = ctx.accounts.manager.key();
        vault.bump = vault_bump; // store the bump for later use

        Ok(())
    }

    pub fn deposit_tokens(ctx: Context<DepositTokens>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }


    pub fn withdraw_tokens(ctx: Context<WithdrawTokens>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;
        require!(ctx.accounts.user.key() == vault.manager, ErrorCode::Unauthorized);

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct InitializeVault<'info> {
    #[account(init, seeds = [b"vault", manager.key().as_ref()], bump, payer = manager, space = 8 + 32 + 1)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub manager: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositTokens<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawTokens<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Vault {
    pub manager: Pubkey,
    pub bump: u8, // Store the bump for future PDA signing
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized action.")]
    Unauthorized,
    #[msg("PDA seeds mismatch.")]
    PDASeedsMismatch,
}
