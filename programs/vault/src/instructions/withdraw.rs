use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};

use crate::{error::VaultError, Vault};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"vault",
            vault_state.key().as_ref()
        ],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [
            b"vault_state",
            user.key().as_ref()
        ],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, Vault>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw_from_vault_to_user(&mut self, amount: u64) -> Result<()> {
        require!(amount.gt(&0), VaultError::ZeroAmount);

        //transfer from vault to user by cpi signer seeds
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let signer_seeds = [
            b"vault_state",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let seeds = &[&signer_seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);

        transfer(cpi_context, amount)?;
        
        Ok(())
    }
}
