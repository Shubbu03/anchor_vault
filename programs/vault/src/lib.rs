#![allow(unexpected_cfgs)]
pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

pub use constants::*;
pub use instructions::*;

declare_id!("Fb9Eh4d5SB6nQ3F3Pgg5vE9EdFteepkDjmwLHSrKCLoG");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"state", signer.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // the one who deposits
    #[account(
        mut, // prev was not mut , but now it is , as now we'll deposit and change the state
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>, // place where it will be deposited
    #[account(
        seeds = [b"state", signer.key().as_ref()], // init will also not come as it has been initialised before, if we use here, it will fail
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer {
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_account);

        transfer(cpi_context, amount)?;

        Ok(())
        //No signing happens here as the initial transaction was already signed, so our sign is inherited here and will continue to be inherited for as long as the transaction occurs.
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // the one who deposits
    #[account(
        mut, // prev was not mut , but now it is , as now we'll deposit and change the state
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>, // place where it will be deposited
    #[account(
        seeds = [b"state", signer.key().as_ref()], // init will also not come as it has been initialised before, if we use here, it will fail
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info(),
        };

        let signer_seeds = [
            b"vault",
            self.vault_state.to_account_info().key.as_ref(), // we use key simply and not key() because we dont need the actual pubkey here which will be returned by key()
            &[self.vault_state.vault_bump],
        ];

        let seeds = &[&signer_seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_account, seeds);

        transfer(cpi_context, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // the one who deposits
    #[account(
        mut, // prev was not mut , but now it is , as now we'll deposit and change the state
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>, // place where it will be deposited
    #[account(
        mut,
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump,
        close = signer
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();

        let cpi_account = Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info(),
        };

        let signer_seeds = [
            b"vault",
            self.vault_state.to_account_info().key.as_ref(), // we use key simply and not key() because we dont need the actual pubkey here which will be returned by key()
            &[self.vault_state.vault_bump],
        ];

        let seeds = &[&signer_seeds[..]];

        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_account, seeds);

        transfer(cpi_context, self.vault.lamports())?;

        Ok(())
    }
}
#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}

impl Space for VaultState {
    const INIT_SPACE: usize = 1 + 1; // 8 is the anchor discriminator , one for each value in VaultState
}
