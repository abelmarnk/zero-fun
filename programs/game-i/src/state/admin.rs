use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub admin: Pubkey,
    pub message_signer: Pubkey,
    pub vault: Pubkey,
    pub max_deposit: u8, // In bps
    pub max_payout: u8, // In bps
    bump: u8
}

impl GlobalState {
    
    pub fn new(
        admin: Pubkey,
        message_signer: Pubkey,
        vault: Pubkey,
        max_deposit: u8,
        max_payout: u8,
        bump: u8
    ) -> Self {
        Self {
            admin,
            message_signer,
            vault,
            max_deposit,
            max_payout,
            bump
        }
    }
    
    pub fn get_bump(&self) -> u8 {
        self.bump
    }
}