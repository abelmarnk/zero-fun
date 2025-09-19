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

    pub fn initialize_global_state(
        ctx: Context<InitializeGlobalStateCtx>,
        args: InitializeGlobalStateArgs
    ) -> Result<()> {
        initialize_global_state_handler(ctx, args)
    }

    pub fn update_global_state(
        ctx: Context<UpdateGlobalStateCtx>,
        args: UpdateGlobalStateArgs,
    ) -> Result<()> {
        update_global_state_handler(ctx, args)
    }

    pub fn initialize_game(
        ctx: Context<InitializeGameCtx>,
        args: InitializeGameArgs,
    ) -> Result<()> {
        initialize_game_handler(ctx, args)
    }

    pub fn finalize_win(
        ctx: Context<FinalizeWinCtx>,
        args: FinalizeWinArgs,
    ) -> Result<()> {
        finalize_win_handler(ctx, args)
    }

    pub fn finalize_loss(
        ctx: Context<FinalizeLossCtx>,
        args: FinalizeLossArgs,
    ) -> Result<()> {
        finalize_loss_handler(ctx, args)
    }

    pub fn withdraw(
        ctx: Context<WithdrawCtx>,
        args: WithdrawArgs,
    ) -> Result<()> {
        withdraw_handler(ctx, args)
    }
}