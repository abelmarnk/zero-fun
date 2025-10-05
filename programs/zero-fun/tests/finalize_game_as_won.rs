use anchor_lang::InstructionData;
use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ADDRESS;
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
    set_current_time,
    ed25519_instruction_for_parts
};

use zero_fun::{
    instruction::FinalizeGameAsWon,
    FinalizeGameAsWonArgs,
    GameSession,
    GameSessionStatus,
    GlobalState,
    GameState,
    FINALIZE_WIN_ACTION,
    HASH_LENGTH,
    ID as ZERO_FUN_PROGRAM_ID,
    MAX_MOVE_COUNT,
};

struct FinalizeWonTestParams {
    // players & vaults
    pub instruction_player: Keypair,
    pub state_player: Pubkey,
    pub instruction_vault: Pubkey,
    pub state_vault: Pubkey,

    // signed values (what message_signer signs)
    pub signed_payout: u64,
    pub signed_deadline: i64,
    pub signed_public_config_seed: [u8; HASH_LENGTH],

    // instruction values (what is passed into the program)
    pub instruction_payout: u64,
    pub instruction_deadline: i64,
    pub state_public_config_seed: [u8; HASH_LENGTH],

    // time + vault settings
    pub current_time: i64,
    pub vault_balance: u64,
    pub global_state_max_payout_bps: u8,
}

struct TestSetup {}

impl TestSetup {
    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;

    pub fn builder(svm: &mut LiteSVM, params: FinalizeWonTestParams) -> Result<([Instruction; 2], Vec<Keypair>)> {
        
        // Create the player
        svm.airdrop(&params.instruction_player.pubkey(), 1_000_000_000)
            .expect("Could not airdrop to player");

        // Create GameSession account
        let (game_session, _) = Pubkey::find_program_address(
            &[
                b"game-session",
                params.state_public_config_seed.as_ref(),
                params.state_player.as_ref(),
            ],
            &Self::ZERO_FUN_PROGRAM_ID,
        );

        let game_session_account = GameSession {
            last_action_time: 0,
            player: params.state_player,
            deposit: 1_000_000u64,
            status: GameSessionStatus::Active,
            public_config_seed: params.state_public_config_seed,
            game_metadata: "meta".to_string(),
            player_moves:[0u8; MAX_MOVE_COUNT],
            vault: params.state_vault,
            next_player_move_position: 0,
        };

        create_game_session_account(svm, game_session, 
            &game_session_account);

        // Create global state and main vault & message signer
        let message_signer = Keypair::new();

        let (vault, vault_bump) =
            Pubkey::find_program_address(&[b"vault"], &Self::ZERO_FUN_PROGRAM_ID);

        let (global_state, _) =
            Pubkey::find_program_address(&[b"global-state"], &Self::ZERO_FUN_PROGRAM_ID);

        let global_state_account = GlobalState {
            admin: Pubkey::new_unique(),
            message_signer: message_signer.pubkey(),
            max_deposit: 10u8,
            max_payout: params.global_state_max_payout_bps,
            game_state: GameState::Active,
            vault_bump: vault_bump,
        };

        create_global_state_account(svm, global_state, global_state_account);

        let rent = svm.minimum_balance_for_rent_exemption(0);

        create_vault_account(svm, vault, rent + params.vault_balance);

        // Create user vault
        let deposit_amount = 1_000_000u64;
        
        create_vault_account(svm, params.instruction_vault, rent + deposit_amount);

        // Set current time
        set_current_time(svm, params.current_time);

        // Build ed25519 instruction
        let signed_payout_bytes = params.signed_payout.to_le_bytes();
        let signed_deadline_bytes = params.signed_deadline.to_le_bytes();

        let parts: [&[u8];4] = [
            FINALIZE_WIN_ACTION.as_bytes(),
            &signed_payout_bytes,
            &signed_deadline_bytes,
            params.signed_public_config_seed.as_ref(),
        ];

        let ed25519_instruction = ed25519_instruction_for_parts(&message_signer, &parts);

        // Build program instruction
        let args = FinalizeGameAsWonArgs {
            payout: params.instruction_payout,
            deadline: params.instruction_deadline
        };

        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(game_session, false),
            AccountMeta::new(params.instruction_player.pubkey(), true),
            AccountMeta::new(params.instruction_vault, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(global_state, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            AccountMeta::new_readonly(INSTRUCTIONS_SYSVAR_ADDRESS, false),
        ];

        let program_instruction = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: FinalizeGameAsWon { args }.data(),
        };

        Ok(([ed25519_instruction, program_instruction], vec![params.instruction_player]))
    }

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();
        let vault = Pubkey::new_unique();

        let public_config_seed = Pubkey::new_unique().to_bytes();

        let params = FinalizeWonTestParams {
            instruction_player,
            state_player,
            instruction_vault: vault,
            state_vault: vault,
            signed_payout:100u64,
            instruction_payout:100u64,
            signed_deadline:1_750_000_000i64,
            instruction_deadline:1_750_000_000i64,
            state_public_config_seed: public_config_seed,
            signed_public_config_seed: public_config_seed,
            current_time: 1_650_000_000i64,
            vault_balance: 1_000_000_000u64,
            global_state_max_payout_bps: 10u8,
        };

        Self::builder(svm, params)
    }

    pub fn with_invalid_player(svm: &mut LiteSVM) -> Result<([Instruction;2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = Pubkey::new_unique();

        let vault = Pubkey::new_unique();

        let public_config_seed = Pubkey::new_unique().to_bytes();

        let params = FinalizeWonTestParams {
            state_player, // different from instruction
            instruction_player,
            state_vault: vault,
            instruction_vault: vault,
            signed_payout: 100,
            instruction_payout: 100,
            signed_deadline: 1_750_000_000i64,
            instruction_deadline: 1_750_000_000i64,
            signed_public_config_seed: public_config_seed,
            state_public_config_seed: public_config_seed,
            current_time: 1_650_000_000i64,
            vault_balance: 1_000_000_000u64,
            global_state_max_payout_bps: 10u8,
        };

        Self::builder(svm, params)
    }

    pub fn with_invalid_vault(svm: &mut LiteSVM) -> Result<([Instruction;2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let state_vault = Pubkey::new_unique();
        let instruction_vault = Pubkey::new_unique();

        let public_config_seed = Pubkey::new_unique().to_bytes();

        let params = FinalizeWonTestParams {
            state_player,           
            instruction_player,
            state_vault, // different from instruction
            instruction_vault,
            signed_payout: 100,
            instruction_payout: 100,
            signed_deadline: 1_750_000_000i64,
            instruction_deadline: 1_750_000_000i64,
            signed_public_config_seed: public_config_seed,
            state_public_config_seed: public_config_seed,
            current_time: 1_650_000_000i64,
            vault_balance: 1_000_000_000u64,
            global_state_max_payout_bps: 10u8,
        };

        Self::builder(svm, params)
    }

    pub fn with_mismatched_signed_payout(svm: &mut LiteSVM) -> Result<([Instruction;2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let vault = Pubkey::new_unique();

        let public_config_seed = Pubkey::new_unique().to_bytes();

        let params = FinalizeWonTestParams {
            instruction_player,
            state_player,
            instruction_vault: vault,
            state_vault: vault,
            signed_payout: 200, // different from instruction
            instruction_payout: 100, // different from signed(commited to by the message signer)
            signed_deadline: 1_750_000_000i64,
            instruction_deadline: 1_750_000_000i64,
            state_public_config_seed: public_config_seed,
            signed_public_config_seed: public_config_seed,
            current_time: 1_650_000_000i64,
            vault_balance: 1_000_000_000u64,
            global_state_max_payout_bps: 10u8,
        };

        Self::builder(svm, params)
    }

    pub fn with_mismatched_signed_deadline(svm: &mut LiteSVM) -> Result<([Instruction;2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();
        let vault = Pubkey::new_unique();

        let public_config_seed = Pubkey::new_unique().to_bytes();

        let params = FinalizeWonTestParams {
            instruction_player,
            state_player,
            instruction_vault: vault,
            state_vault: vault,
            signed_payout: 100,
            instruction_payout: 100,
            signed_deadline: 1_750_000_100i64, // different from instruction
            instruction_deadline: 1_750_000_000i64, // different from signed(commited to by the message signer)
            state_public_config_seed: public_config_seed,
            signed_public_config_seed: public_config_seed,
            current_time: 1_650_000_000i64,
            vault_balance: 1_000_000_000u64,
            global_state_max_payout_bps: 10u8,
        };

        Self::builder(svm, params)
    }

    pub fn with_invalid_public_config(svm: &mut LiteSVM) -> Result<([Instruction;2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();
        let vault = Pubkey::new_unique();

        let signed_public_config = Pubkey::new_unique().to_bytes();

        let state_public_config = Pubkey::new_unique().to_bytes();

        let params = FinalizeWonTestParams {
            state_player,
            instruction_player,
            state_vault: vault,
            instruction_vault: vault,
            signed_payout: 100,
            instruction_payout: 100,
            signed_deadline: 1_750_000_000i64,
            instruction_deadline: 1_750_000_000i64,
            signed_public_config_seed: signed_public_config, // different from state(stored in the game session account)
            state_public_config_seed: state_public_config, // different from signed(commited to by the message signer)
            current_time: 1_650_000_000i64,
            vault_balance: 1_000_000_000u64,
            global_state_max_payout_bps: 10u8,
        };

        Self::builder(svm, params)
    }

    pub fn with_deadline_passed(svm: &mut LiteSVM) -> Result<([Instruction;2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();
        let vault = Pubkey::new_unique();

        let public_config_seed = Pubkey::new_unique().to_bytes();

        let instruction_deadline = 1_650_000_000i64;

        let current_time = instruction_deadline + 100; // after deadline

        let params = FinalizeWonTestParams {
            state_player,
            instruction_player,
            state_vault: vault,
            instruction_vault: vault,
            signed_payout: 100,
            instruction_payout: 100,
            instruction_deadline,
            signed_deadline: instruction_deadline,
            state_public_config_seed: public_config_seed,
            signed_public_config_seed: public_config_seed,
            current_time,
            vault_balance: 1_000_000_000u64,
            global_state_max_payout_bps: 10u8,
        };

        Self::builder(svm, params)
    }

    pub fn with_payout_exceeds_max(svm: &mut LiteSVM) -> Result<([Instruction;2], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();
        let vault = Pubkey::new_unique();

        let public_config_seed = Pubkey::new_unique().to_bytes();

        let vault_balance = 1_000_000_000u64;

        let max_bps = 10u8; // Max payout = 1_000_000_000

        let params = FinalizeWonTestParams {
            state_player,
            instruction_player,
            state_vault: vault,
            instruction_vault: vault,
            signed_payout: 10_000_000u64, // > Max payout
            instruction_payout: 10_000_000u64, // > Max payout
            signed_deadline: 1_750_000_000i64,
            instruction_deadline: 1_750_000_000i64,
            state_public_config_seed: public_config_seed,
            signed_public_config_seed: public_config_seed,
            current_time: 1_650_000_000i64,
            vault_balance,
            global_state_max_payout_bps: max_bps,
        };

        Self::builder(svm, params)
    }
}


#[test]
fn test_finalize_game_as_won_success() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_default(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    println!("Data for ED25519 program: {:?}", instructions[0].data);    

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            panic!("Expected success but transaction failed");
        }
    }
}

#[test]
fn test_finalize_game_as_won_fails_with_invalid_player() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_player(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Invalid player");
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_fails_with_invalid_vault() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_vault(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Invalid vault");
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_fails_with_mismatched_signed_payout() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_mismatched_signed_payout(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Mismatched signed payout");
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_fails_with_mismatched_signed_deadline() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_mismatched_signed_deadline(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Mismatched signed deadline");
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_fails_with_invalid_public_config() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_public_config(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Invalid public config");
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_fails_when_deadline_passed() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_deadline_passed(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Deadline passed");
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_finalize_game_as_won_fails_when_payout_exceeds_max() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_payout_exceeds_max(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();
    
    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(res) => {
            println!("Program succeeded (compute units: {:?})", res.compute_units_consumed);
            panic!("This transaction should have failed - Payout exceeds maximum");
        }
        Err(err) => {
            println!("Program failed: {:?}", err);
            println!("Transaction failed successfully");
        }
    }
}
