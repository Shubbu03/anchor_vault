use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Amount can't be zero")]
    ZeroAmount,
}
