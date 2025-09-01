use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{error::VaultError, Vault, DISCRIMINATOR};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        init,
        payer = user,
        space = DISCRIMINATOR + Vault::INIT_SPACE,
        seeds = [b"state", user.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, Vault>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit_to_vault(&mut self, amount: u64, vault_bump: u8, state_bump: u8) -> Result<()> {
        require!(amount.gt(&0), VaultError::ZeroAmount);

        // Store the bumps in the vault state
        self.vault_state.vault_bump = vault_bump;
        self.vault_state.state_bump = state_bump;

        //simple transfer cpi
        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}
