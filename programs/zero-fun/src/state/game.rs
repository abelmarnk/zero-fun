use std::ops::{Add};

use anchor_lang::prelude::*;

use crate::{HASH_LENGTH, DEFAULT_OFFSET, MAX_MOVE_COUNT, state::error::GameError};


#[derive(AnchorDeserialize, AnchorSerialize, InitSpace, Clone, Copy, PartialEq, Eq)]
pub enum GameSessionStatus{
    Active,
    Won,
    Lost
}

#[account]
#[derive(InitSpace)]
/// Represents a game session for a player.
pub struct GameSession{
    pub last_action_time:i64,
    pub player:Pubkey,
    pub deposit:u64,
    pub status:GameSessionStatus,
    /// A SHA-256 hash seed used to derive the public configuration of the game,
    /// which is known to both the player and the game system.
    pub public_config_seed:[u8;HASH_LENGTH],
    /// Arbitrary metadata about the game, such as the algorithm version,
    /// configuration parameters, etc. Limited to 64 bytes.
    #[max_len(64)]
    pub game_metadata:String,
    /// Stores the set of moves the user made while it was active, it is filled in
    /// when the user makes a move
    pub player_moves:[u8;MAX_MOVE_COUNT],

    pub vault:Pubkey,

    /// Stores the next position for the player move
    pub next_player_move_position:u8,
}

impl GameSession{
    pub fn new(
        player:Pubkey,
        deposit:u64,
        vault:Pubkey,
        public_config_seed:[u8;HASH_LENGTH],
        game_metadata:String,
        now:i64
    ) -> Self{
        Self{
            last_action_time:now,
            player,
            deposit,
            vault,
            status:GameSessionStatus::Active,
            public_config_seed:public_config_seed,
            game_metadata,
            player_moves:[0;MAX_MOVE_COUNT],
            next_player_move_position:0
        }
    }

    pub fn is_vault_for_game(&self, vault:&Pubkey)->bool{
        self.vault.eq(vault)
    }

    pub fn is_owned_by_player(&self, player:&Pubkey)->bool{
        self.player.eq(player)
    }

    pub fn can_default(&self, now:i64)->bool{
        now.gt(&self.last_action_time.add(DEFAULT_OFFSET))
    }

    pub fn is_active(&self)->bool{
        self.status == crate::GameSessionStatus::Active
    }

    pub fn is_won(&self) -> bool{
        self.status == crate::GameSessionStatus::Won
    }

    pub fn set_next_player_move(&mut self, player_move:u8)->Result<()>{
        let player_move_position = usize::from(self.next_player_move_position);
        
        if player_move_position.ge(&MAX_MOVE_COUNT){
            return Err(GameError::MaxMoveReached.into());
        }

        self.player_moves[player_move_position] = player_move;
        // Overflow not possible it is bounded
        self.next_player_move_position += 1;
        Ok(())
    }
}