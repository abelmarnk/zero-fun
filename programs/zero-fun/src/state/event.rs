use anchor_lang::prelude::*;

use crate::{GameSession, GlobalStateUpdate, HASH_LENGTH};

#[event]
pub struct MarkGameAsWonEvent {
    pub game_session: Pubkey,
}

#[event]
pub struct InitializeGameEvent {
    pub game_session:Pubkey,
    pub game_session_account: GameSession,
}

#[event]
pub struct DefaultGameEvent {
    pub game_session: Pubkey,
}

#[event]
pub struct FinalizeGameAsWonForPlayerEvent {
    pub admin: Pubkey,
    pub payout: u64,
    pub game_session: Pubkey,
}

#[event]
pub struct FinalizeGameAsWonEvent {
    pub payout: u64,
    pub game_session: Pubkey,
}

#[event]
pub struct FinalizeGameAsLostEvent {
    pub game_session: Pubkey,
    pub private_config_seed: [u8; HASH_LENGTH]
}

#[event]
pub struct UpdateGlobalStateEvent {
    pub admin_at_time_of_update: Pubkey,
    pub update: GlobalStateUpdate,
}

#[event]
pub struct WithdrawEvent {
    pub admin: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
}
