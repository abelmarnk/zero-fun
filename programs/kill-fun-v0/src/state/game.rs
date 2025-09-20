use anchor_lang::prelude::*;

use crate::{HASH_LENGTH, MAX_MOVE_COUNT, state::error::GameError};


#[derive(AnchorDeserialize, AnchorSerialize, InitSpace, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus{
    Active,
    Won,
    Lost
}

#[account]
#[derive(InitSpace)]
/// Represents a game session for a player.
pub struct GameSession{
    creation_time:i64,
    player:Pubkey,
    deposit:u64,
    status:GameStatus,
    payout:u64,
    /// A SHA-256 hash commitment to all the game's relevant configuration.
    /// including both public and private configuration, it is used to ensure
    /// that the details of the game cannot be altered after the start 
    /// of the session.
    commitment:[u8;HASH_LENGTH],
    /// A SHA-256 hash seed used to derive the public configuration of the game,
    /// which is known to both the player and the game system.
    public_config_seed:[u8;HASH_LENGTH],
    /// A SHA-256 hash seed used to derive the private configuration of the game
    /// it can be used to derive the winning sequence which is why it is kept 
    /// secret until the game is over.
    private_config_seed:[u8;HASH_LENGTH],
    /// Arbitrary metadata about the game, such as the algorithm version,
    /// configuration parameters, etc. Limited to 64 bytes.
    #[max_len(64)]
    game_metadata:String,
    /// Stores the set of moves the user made while it was active, it is filled in
    /// when the game is finalized.
    finalized_game_state:[u8;MAX_MOVE_COUNT]
}

impl GameSession{
    pub fn new(
        player:Pubkey,
        deposit:u64,
        commitment:[u8;HASH_LENGTH],
        public_config_seed:[u8;HASH_LENGTH],
        game_metadata:String,
        current_timestamp:i64,
    ) -> Self{
        Self{
            creation_time:current_timestamp,
            player,
            deposit,
            status:GameStatus::Active,
            payout:0,
            commitment,
            public_config_seed:public_config_seed,
            private_config_seed:[0u8;HASH_LENGTH],
            game_metadata,
            finalized_game_state:[0u8;MAX_MOVE_COUNT]
        }
    }

    pub fn get_creation_time(&self) -> i64{
        self.creation_time
    }

    pub fn get_player(&self) -> &Pubkey{
        &self.player
    }

    pub fn get_deposit(&self) -> u64{
        self.deposit
    }

    pub fn get_status(&self) -> GameStatus{
        self.status
    }

    pub fn set_status(&mut self, status:GameStatus)->Result<()>{
        if self.status != GameStatus::Active{
            return Err(GameError::GameAlreadyFinalized.into());
        }
        self.status = status;
        Ok(())
    }

    pub fn set_payout(&mut self, payout:u64){
        self.payout = payout;
    }

    pub fn set_commitment(&mut self, commitment:[u8;HASH_LENGTH]){
        self.commitment = commitment;
    }

    pub fn get_commitment(&self) -> &[u8;HASH_LENGTH]{
        &self.commitment
    }

    pub fn set_public_config_seed(&mut self, public_config_seed:[u8;HASH_LENGTH]){
        self.public_config_seed = public_config_seed;
    }

    pub fn set_private_config_seed(&mut self, private_config_seed:[u8;HASH_LENGTH]){
        self.private_config_seed = private_config_seed;
    }

    pub fn set_game_metadata(&mut self, game_metadata:String){
        self.game_metadata = game_metadata;
    }

    pub fn set_finalized_game_state(&mut self, finalized_game_state:[u8;MAX_MOVE_COUNT]){
        self.finalized_game_state = finalized_game_state;
    }
}