use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::token::{close_account, CloseAccount};

use crate::Vault;

#[derive(Accounts)]
pub struct Close<'info> {
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

impl<'info> Close<'info> {
    pub fn close_vault(&mut self) -> Result<()> {
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

        transfer(cpi_context, self.vault.lamports())?;

        //close vault account

        let close_cpi_account = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.vault.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let close_cpi_ctx =
            CpiContext::new(self.system_program.to_account_info(), close_cpi_account);

        close_account(close_cpi_ctx)?;

        Ok(())
    }
}
