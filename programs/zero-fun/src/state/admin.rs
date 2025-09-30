use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    /// - Admin: They control the global state, they can also whitelist tokens
    /// or removed whitelisted tokens
    pub admin: Pubkey,
    /// - Message signer: They sign messages mark an action as approved by the admin
    /// and be executed by anyone
    pub message_signer: Pubkey,
    pub max_deposit: u8, // In bps
    pub max_payout: u8, // In bps
    pub game_state: GameState,
    vault_bump:u8,
}

impl GlobalState {
    
    pub fn new(
        admin: Pubkey,
        message_signer: Pubkey,
        max_deposit: u8,
        max_payout: u8,
        game_state: GameState,
        vault_bump:u8,
    ) -> Self {
        Self {
            admin,
            message_signer,
            max_deposit,
            max_payout,
            game_state,
            vault_bump,
        }
    }

    pub fn is_admin(&self, admin:&Pubkey)->bool{
        self.admin.eq(admin)
    }

    pub fn is_active(&self) -> bool{
        self.game_state.eq(&crate::GameState::Active)
    }

    pub fn get_vault_bump(&self) -> u8 {
        self.vault_bump
    }
}

#[derive(InitSpace, Clone, Copy, AnchorDeserialize, AnchorSerialize, PartialEq)]
pub enum GameState{
    Active,
    Locked
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub enum GlobalStateUpdate {
    Admin(Pubkey),
    MessageSigner(Pubkey),
    MaxDeposit(u8),
    MaxPayout(u8),
    GameState(GameState)
}
