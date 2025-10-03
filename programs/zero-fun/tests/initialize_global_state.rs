use anchor_lang::InstructionData;
use litesvm::LiteSVM;
use anyhow::Result;
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
    system_program::ID as SYSTEM_PROGRAM_ID,
    transaction::Transaction,
};

mod common;
use common::utils::{
    add_zero_fun_program,
    get_initializer_keypair
};

use zero_fun::{
    instruction::InitializeGlobalState,
    InitializeGlobalStateArgs,
    GameState,
    ID as ZERO_FUN_PROGRAM_ID
};

use crate::common::disable_signer;

// Again the idea is that relevant things are tested, stuff that has no relevance 
// to the test are filled with defaults, here what is important is that the global state
// can only be created once, any attempts to create another instance should fail, another
// is that the initializer should match what is in the program and lastly that the initializer
// should sign

struct TestSetup {

}

impl TestSetup {

    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;
    const SYSTEM_PROGRAM_ID: Pubkey = SYSTEM_PROGRAM_ID;

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction;1], Vec<Keypair>)> {

        let initializer = get_initializer_keypair();

        Self::builder(svm, initializer)
    }

    pub fn with_invalid_initializer(svm: &mut LiteSVM) -> Result<([Instruction;1], Vec<Keypair>)> {

        let initializer = Keypair::new();

        Self::builder(svm, initializer)
    }

    fn builder(svm: &mut LiteSVM, initializer:Keypair) -> Result<([Instruction;1], Vec<Keypair>)> {

        let message_signer = Keypair::new();

        let admin = Keypair::new();

        // Create the initializer account if it doesn't exist
        svm.airdrop(&initializer.pubkey(), 1_000_000_000).unwrap();

        let (global_state_pda, _global_bump) = Pubkey::find_program_address(
                &[b"global-state"], &Self::ZERO_FUN_PROGRAM_ID);

        let (vault_pda, _vault_bump) = Pubkey::find_program_address(
            &[b"vault"], &Self::ZERO_FUN_PROGRAM_ID);

        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(global_state_pda, false),
            AccountMeta::new_readonly(initializer.pubkey(), true),
            AccountMeta::new(vault_pda, false),
            AccountMeta::new(message_signer.pubkey(), true),
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new_readonly(Self::SYSTEM_PROGRAM_ID, false),
        ];

        let args = InitializeGlobalStateArgs {
            max_deposit: 10u8,
            max_payout: 100u8,
            initial_state: GameState::Active
        };

        let initialize_ix = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts: accounts,
            data: InitializeGlobalState { args }.data(),
        };

        Ok(([initialize_ix], vec![initializer, message_signer, admin]))
    }

}


#[test]
fn test_initialize_global_state_success() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);


    let result = TestSetup::with_default(&mut svm);

    let (instructions, signers) = 
        match result {
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
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
        }
        Err(error) => {
            println!("Program failed: {:?}", error);

            panic!("Expected success but transaction failed");
        }
    }
}

#[test]
fn test_initialize_global_state_fails_with_invalid_initializer() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_initializer(&mut svm);

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
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);

            panic!("This transaction should have failed - The initializer is not valid")
        }
        Err(error) => {
            println!("Program failed: {:?}", error);

            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_initialize_global_state_fails_when_accounts_already_exist() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_default(&mut svm);

    let (instructions, signers) = 
        match result {
            Ok(result) => result,

            Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    // Run the first transaction
    let result = svm.send_transaction(transaction);    

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
        }
        Err(error) => {
            println!("Program failed: {:?}", error);

            panic!("Expected success but transaction failed");
        }
    }

    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    svm.expire_blockhash();

    // Run it again
    let result = svm.send_transaction(transaction);    

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);

            panic!("This transaction should have failed - The global state accounts already exists")
        }
        Err(error) => {
            println!("Program failed: {:?}", error);

            println!("Transaction failed successfully");
        }
    }

}

#[test]
fn test_initialize_global_state_fails_when_initializer_does_not_sign() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);


    let result = TestSetup::with_default(&mut svm);

    let (mut instructions, mut signers) = 
        match result {
            Ok(result) => result,

            Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();

    disable_signer(&mut instructions[0], signers[0].pubkey()); // Set the initializer as a non signer

    let payer = Keypair::new();

    let payer_key = payer.pubkey();

    svm.airdrop(&payer_key, 1_000_000_000).expect("Could not airdrop to payer");

    signers.push(payer);
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer_key), &signers[1..], recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);

            panic!("This transaction should have failed - The initializer did not sign")
        }
        Err(error) => {
            println!("Program failed: {:?}", error);

            println!("Transaction failed successfully");
        }
    }
}
