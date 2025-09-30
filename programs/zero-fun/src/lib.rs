use anchor_lang::prelude::*;

declare_id!("5e4vTmm5pcUFHPr34rtrpu33kXC5nG4eN7JmkHhJpJsP");

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;

pub mod utils;
pub use utils::*;

#[program]
pub mod game_i {
    use super::*;

    /// Initializes the global program state.
    /// This sets configuration parameters and creates the global vault account.
    pub fn initialize_global_state(
        ctx: Context<InitializeGlobalStateAccounts>,
        args: InitializeGlobalStateArgs
    ) -> Result<()> {
        initialize_global_state_handler(ctx, args)
    }

    /// Updates global configuration values (admin only).
    pub fn update_global_state(
        ctx: Context<UpdateGlobalStateAccounts>,
        args: UpdateGlobalStateArgs,
    ) -> Result<()> {
        update_global_state_handler(ctx, args)
    }

    /// Records a player action (move) on-chain during an active game session.
    pub fn record_action(
        ctx: Context<RecordActionAccounts>,
        args: RecordActionArgs,
    ) -> Result<()> {
        record_action_handler(ctx, args)
    }

    /// Initializes a new game session for a player.
    pub fn initialize_game(
        ctx: Context<InitializeGameAccounts>,
        args: InitializeGameArgs,
    ) -> Result<()> {
        initialize_game_handler(ctx, args)
    }

    /// Allows a player to default (cancel) their game if the session has expired.
    /// Used as a fallback to reclaim deposits in stalled games.
    pub fn default_game(
        ctx: Context<DefaultGameAccounts>
    ) -> Result<()> {
        default_game_handler(ctx)
    }

    /// Finalizes a game as won by the player, it requires the admin’s signature.
    pub fn finalize_game_as_won(
        ctx: Context<FinalizeGameAsWonAccounts>,
        args: FinalizeGameAsWonArgs,
    ) -> Result<()> {
        finalize_game_as_won_handler(ctx, args)
    }

    /// Finalizes a game as won for the player, in cases where the admin refused to provide a signature
    /// they publicly mark the game as won, this then gives the admin the chance of paying it out in
    /// that case.
    pub fn finalize_game_as_won_for_player(
        ctx: Context<FinalizeGameAsWonForPlayerAccounts>,
        args: FinalizeGameAsWonForPlayerArgs,
    ) -> Result<()> {
        finalize_game_as_won_for_player_handler(ctx, args)
    }

    /// Finalizes a game as lost when the player hits the fail move.
    /// This verifies the failing position based on the committed seeds and moves funds to the main vault.
    pub fn finalize_game_as_lost(
        ctx: Context<FinalizeGameAsLostAccounts>,
        args: FinalizeGameAsLostArgs,
    ) -> Result<()> {
        finalize_game_as_lost_handler(ctx, args)
    }

    /// Allows a player to mark the game as won if the admin is unresponsive.
    /// This broadcasts the player's claim on-chain, requiring the admin to later settle it.
    pub fn mark_game_as_won(
        ctx: Context<MarkGameAsWonAccounts>,
    )-> Result<()>{
        mark_game_as_won_handler(ctx)
    }
    
    /// Withdraws funds (admin-only). Used to withdraw accumulated fees from the global vault.
    pub fn withdraw(
        ctx: Context<WithdrawAccounts>,
        args: WithdrawArgs,
    ) -> Result<()> {
        withdraw_handler(ctx, args)
    }
}
