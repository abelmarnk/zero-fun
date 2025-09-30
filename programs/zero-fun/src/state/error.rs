use anchor_lang::prelude::*;

#[error_code]
pub enum GameError{
    #[msg("The game session has already been finalized.")]
    GameSessionAlreadyFinalized,
    #[msg("The game session not won yet.")]
    GameSessionNotWon,    
    #[msg("The provided vault does not match the expected vault.")]
    InvalidVault,
    #[msg("Expected ED25519 program")]
    InvalidED25519Program,
    #[msg("Invalid account count for ED25519 program")]
    InvalidAccountCountForED25519Program,
    #[msg("Invalid data for ED25519 program")]
    InvalidDataForED25519Program,
    #[msg("Invalid message signer")]
    InvalidMessageSigner,
    #[msg("The deposit exceeds the maximum allowed deposit.")]
    DepositExceedsMaximum,
    #[msg("The payout exceeds the maximum allowed payout.")]
    PayoutExceedsMaximum,
    #[msg("The game session is not active.")]
    GameSessionNotActive,
    #[msg("The provided commitment does not match the expected commitment.")]
    InvalidCommitment,
    #[msg("The deadline for this action has passed.")]
    DeadlinePassed,
    #[msg("The provided game metadata exceeds the maximum allowed length.")]
    MetadataTooLong,
    #[msg("The provided player does not match the game session's player.")]
    InvalidPlayer,
    #[msg("Invalid admin")]
    InvalidAdmin,
    #[msg("Invalid bootstrap key")]
    InvalidBootstrapKey,
    #[msg("The game is not currently active")]
    GameNotActive,
    #[msg("Max moves already reached")]
    MaxMoveReached,
    #[msg("Too soon to default")]
    TooSoonToDefault,
    #[msg("Invalid game seed")]
    InvalidGameSeed,
    #[msg("Invalid fail position")]
    InvalidFailPosition
}