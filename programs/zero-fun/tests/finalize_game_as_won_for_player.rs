use anchor_lang::InstructionData;
use litesvm::LiteSVM;
use anyhow::Result;
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
    transaction::Transaction
};

mod common;
use common::utils::{
    add_zero_fun_program,
    create_game_session_account,
    create_vault_account,
    create_global_state_account,
};

use zero_fun::{
    instruction::FinalizeGameAsWonForPlayer,
    FinalizeGameAsWonForPlayerArgs,
    GameSession,
    GameSessionStatus,
    GlobalState,
    GameState,
    HASH_LENGTH,
    ID as ZERO_FUN_PROGRAM_ID,
};

struct TestSetup {}

impl TestSetup {
    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;

    pub fn builder(
        svm: &mut LiteSVM,
        state_player: Pubkey,
        instruction_player: Keypair,
        state_player_vault: Pubkey,
        instruction_player_vault: Pubkey,
        state_admin: Pubkey,
        instruction_admin: Keypair,
        game_session_status: GameSessionStatus,
    ) -> Result<([Instruction; 1], Vec<Keypair>)> {
        
        // Create the admin
        svm.airdrop(&instruction_admin.pubkey(), 1_000_000_000)
            .expect("Could not airdrop to admin");

        // Create the game session
        let (game_session_pda, _) = Pubkey::find_program_address(
            &[
                b"game-session",
                [0u8; HASH_LENGTH].as_ref(),
                state_player.as_ref(),
            ],
            &Self::ZERO_FUN_PROGRAM_ID,
        );

        let game_session_account = GameSession {
            last_action_time: 0,
            player: state_player,
            deposit: 1_000_000u64,
            status: game_session_status,
            public_config_seed:[0;HASH_LENGTH],
            game_metadata: "metadata".to_string(),
            player_moves: [0u8; zero_fun::MAX_MOVE_COUNT],
            vault: state_player_vault,
            next_player_move_position: 0u8,
        };

        create_game_session_account(svm, game_session_pda, 
            &game_session_account);

        // Create global state & main vault
        let (vault, vault_bump) =
            Pubkey::find_program_address(&[b"vault"], &Self::ZERO_FUN_PROGRAM_ID);

        let (global_state, _) =
            Pubkey::find_program_address(&[b"global-state"], &Self::ZERO_FUN_PROGRAM_ID);

        let global_state_account = GlobalState {
            admin: state_admin,
            message_signer: Pubkey::new_unique(),
            max_deposit: 10u8,
            max_payout: 100u8,
            game_state: GameState::Active,
            vault_bump: vault_bump,
        };

        create_global_state_account(svm, global_state, global_state_account);

        // Create program vault and user vault
        let rent = svm.minimum_balance_for_rent_exemption(0);

        let payout = 1_000_000;

        // Create the user vault
        create_vault_account(svm, vault, rent + payout);

        // Create the 
        create_vault_account(svm, instruction_player_vault, 
            rent + game_session_account.deposit);

        // Build instruction
        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(game_session_pda, false),           
            AccountMeta::new(instruction_player.pubkey(), false), 
            AccountMeta::new(instruction_player_vault, false),          
            AccountMeta::new(vault, false),                  
            AccountMeta::new(global_state, false),               
            AccountMeta::new(instruction_admin.pubkey(), true),  
        ];


        let args = 
            FinalizeGameAsWonForPlayerArgs { payout };

        let instruction = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: FinalizeGameAsWonForPlayer { args }.data(),
        };

        Ok(([instruction], vec![instruction_admin]))
    }

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let vault = Pubkey::new_unique();

        let instruction_admin = Keypair::new();
        let state_admin = instruction_admin.pubkey();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            state_admin,
            instruction_admin,
            GameSessionStatus::Won,
        )
    }

    pub fn with_invalid_player(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new(); // Unrecognized player
        let state_player = Pubkey::new_unique(); 

        let vault = Pubkey::new_unique();

        let instruction_admin = Keypair::new();
        let state_admin = instruction_admin.pubkey();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            state_admin,
            instruction_admin,
            GameSessionStatus::Won,
        )
    }

    pub fn with_invalid_vault(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let state_player_vault = Pubkey::new_unique();
        let instruction_player_vault = Pubkey::new_unique(); // Unrecognized player vault

        let instruction_admin = Keypair::new();
        let state_admin = instruction_admin.pubkey();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            state_player_vault,
            instruction_player_vault,
            state_admin,
            instruction_admin,
            GameSessionStatus::Won,
        )
    }

    pub fn with_invalid_admin(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let vault = Pubkey::new_unique();
        let state_admin = Keypair::new(); 

        let instruction_admin = Keypair::new(); // Unrecognized admin

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            state_admin.pubkey(),
            instruction_admin,
            GameSessionStatus::Won,
        )
    }

    pub fn with_not_won(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let vault = Pubkey::new_unique();
        
        let instruction_admin = Keypair::new();
        let state_admin = instruction_admin.pubkey();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            state_admin,
            instruction_admin,
            GameSessionStatus::Active, // not Won
        )
    }
}

#[test]
fn test_finalize_game_as_won_for_player_success() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_default(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();

    let payer = signers[0].pubkey();

    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            panic!("Expected success but transaction failed");
        }
    }
}

#[test]
fn test_finalize_game_as_won_for_player_fails_with_invalid_player() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_player(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();

    let payer = signers[0].pubkey();

    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Invalid player");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_for_player_fails_with_invalid_vault() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_vault(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();

    let payer = signers[0].pubkey();

    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Invalid vault");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_for_player_fails_with_invalid_admin() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_admin(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();

    let payer = signers[0].pubkey();

    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Invalid admin");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_for_player_fails_when_not_won() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_not_won(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();
    
    let payer = signers[0].pubkey();

    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Game session not marked Won");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}
