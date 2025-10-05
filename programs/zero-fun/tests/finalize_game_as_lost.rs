use anchor_lang::InstructionData;
use anchor_lang::solana_program::hash::hashv;
use litesvm::LiteSVM;
use anyhow::Result;
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
    transaction::Transaction,
};

mod common;
use common::utils::{
    add_zero_fun_program,
    create_game_session_account,
    create_vault_account,
    create_global_state_account,
};

use zero_fun::{
    instruction::FinalizeGameAsLost,
    FinalizeGameAsLostArgs,
    GameSession,
    GameSessionStatus,
    GlobalState,
    GameState,
    MAX_MOVE_COUNT,
    MAX_MOVE_TYPE_COUNT,
    PUBLIC_SEED,
    ID as ZERO_FUN_PROGRAM_ID,
};

struct TestSetup {}

impl TestSetup {
    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;

    pub fn builder(
        svm: &mut LiteSVM,
        state_player: Pubkey,
        instruction_player: Keypair,
        state_vault: Pubkey,
        instruction_vault: Pubkey,
        game_session_status: GameSessionStatus,
        correct_public_config: bool,
        matching_move: bool,
    ) -> Result<([Instruction; 1], Vec<Keypair>)> {

        // Create the player
        svm.airdrop(&instruction_player.pubkey(), 1_000_000_000)
            .expect("Could not airdrop to player");

        // Create a random seed and hash it to form the private_config_seed
        let random_seed = Pubkey::new_unique().to_bytes();
        let private_config_seed = hashv(&[random_seed.as_ref()]).to_bytes();

        // Derive public_config_seed from the private_config_seed
        let derived_public_config_seed = hashv(&[
            PUBLIC_SEED.as_ref(),
            private_config_seed.as_ref(),
        ])
        .to_bytes();

        // Generate a fail position
        let fail_position = u8::try_from(rand::random_range(0..MAX_MOVE_COUNT)).unwrap();

        // Compute public_config_seed_for_move and number of move types for the round
        let public_config_seed_for_move = hashv(&[
            &[fail_position],
            derived_public_config_seed.as_ref(),
        ])
        .to_bytes();

        let move_type_count_for_round: u8 =
            (public_config_seed_for_move[0] % (u8::try_from(MAX_MOVE_TYPE_COUNT).unwrap() - 1)) + 2;

        // Compute private_config_seed_for_move and fail_move
        let private_config_seed_for_move =
            hashv(&[&[fail_position], private_config_seed.as_ref()]).to_bytes();

        let fail_move: u8 = private_config_seed_for_move[0] % move_type_count_for_round;

        // Prepare the GameSession.player_moves array and the stored public_config_seed
        let mut player_moves = [0u8; MAX_MOVE_COUNT];

        // If matching_move is true, record the real fail_move. Otherwise record a different move.
        let recorded_move = if matching_move {
            fail_move
        } else {
            // Choose a different move by adding 1 mod move_type_count_for_round
            (fail_move + 1) % move_type_count_for_round
        };

        player_moves[usize::from(fail_position)] = recorded_move;

        // Ensure next_player_move_position > fail_position
        let next_player_move_position = fail_position + 1;

        // Decide what public_config_seed to store in GameSession
        let pub_config_seed_to_store = if correct_public_config {
            derived_public_config_seed
        } else {
            // Store a wrong seed
            Pubkey::new_unique().to_bytes()
        };

        // Create game session
        let (game_session_pda, _) = Pubkey::find_program_address(
            &[
                b"game-session",
                pub_config_seed_to_store.as_ref(),
                state_player.as_ref(),
            ],
            &Self::ZERO_FUN_PROGRAM_ID,
        );

        let game_session_account = GameSession {
            last_action_time: 0,
            player: state_player,
            deposit: 1_000_000u64,
            status: game_session_status,
            public_config_seed: pub_config_seed_to_store,
            game_metadata: "metadata".to_string(),
            player_moves,
            vault: state_vault,
            next_player_move_position,
        };

        create_game_session_account(svm, game_session_pda, &game_session_account);

        // Create global state & main vault
        let (vault_pda, vault_bump) =
            Pubkey::find_program_address(&[b"vault"], &Self::ZERO_FUN_PROGRAM_ID);

        let (global_state, _) =
            Pubkey::find_program_address(&[b"global-state"], &Self::ZERO_FUN_PROGRAM_ID);

        let global_state_account = GlobalState {
            admin: Pubkey::new_unique(),
            message_signer: Pubkey::new_unique(),
            max_deposit: 10u8,
            max_payout: 100u8,
            game_state: GameState::Active,
            vault_bump: vault_bump,
        };

        create_global_state_account(svm, global_state, global_state_account);

        let rent = svm.minimum_balance_for_rent_exemption(0);

        create_vault_account(svm, vault_pda, rent);
        
        // Create the user vault
        let deposit_amount = 1_000_000u64;

        create_vault_account(svm, instruction_vault, 
                rent + deposit_amount); // user vault holds deposit

        // Build the instruction
        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(game_session_pda, false),        
            AccountMeta::new(instruction_player.pubkey(), false),
            AccountMeta::new(instruction_vault, false),      
            AccountMeta::new(vault_pda, false),              
            AccountMeta::new(global_state, false),       
        ];

        let args = FinalizeGameAsLostArgs {
            private_config_seed,
            fail_position,
        };

        let instruction = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: FinalizeGameAsLost { args }.data(),
        };

        Ok(([instruction], vec![instruction_player]))
    }

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let vault = Pubkey::new_unique();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            GameSessionStatus::Active,
            true,
            true,
        )
    }

    pub fn with_invalid_player(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new(); // Unrecognized player
        let state_player = Pubkey::new_unique(); 

        let vault = Pubkey::new_unique();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            GameSessionStatus::Active,
            true,
            true,
        )
    }

    pub fn with_invalid_vault(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let state_vault = Pubkey::new_unique(); // Unrecognized vault
        let instruction_vault = Pubkey::new_unique();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            state_vault,
            instruction_vault,
            GameSessionStatus::Active,
            true,
            true,
        )
    }

    pub fn with_invalid_public_config(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let vault = Pubkey::new_unique();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            GameSessionStatus::Active,
            false, // Incorrect public config
            true,
        )
    }

    pub fn with_invalid_fail_position(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();
        
        let vault = Pubkey::new_unique();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            GameSessionStatus::Active,
            true,
            false, // recorded move != computed move
        )
    }
}


#[test]
fn test_finalize_game_as_lost_success() {
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
fn test_finalize_game_as_lost_fails_with_invalid_player() {
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
fn test_finalize_game_as_lost_fails_with_invalid_vault() {
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
fn test_finalize_game_as_lost_fails_with_invalid_public_config() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_public_config(&mut svm);

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
            panic!("This transaction should have failed - Invalid game seed (public config mismatch)");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_lost_fails_with_invalid_fail_position() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_fail_position(&mut svm);

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
            panic!("This transaction should have failed - Invalid fail position / move mismatch");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}
