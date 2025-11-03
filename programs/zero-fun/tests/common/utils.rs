use std::path::{Path, PathBuf};
use litesvm::LiteSVM;
use solana_sdk::{
        account::Account as SolanaAccount, clock::Clock, ed25519_instruction::new_ed25519_instruction_with_signature, hash::hashv, instruction::Instruction, pubkey::Pubkey, signer::{
            EncodableKey, Signer, keypair::Keypair
        }
};
use zero_fun::{
    GameSession, GlobalState, ID as ZERO_FUN_PROGRAM_ID
};
use anchor_lang::{
    AccountSerialize,
    Space,
};


pub fn create_global_state_account(
    svm: &mut LiteSVM,
    global_state_pubkey: Pubkey,
    global_state: GlobalState,
) {
    let mut data = Vec::with_capacity(8 + GlobalState::INIT_SPACE);

    // Serialize account
    global_state
        .try_serialize(&mut data)
        .expect("Could not serialize GlobalState");

    let rent = svm.minimum_balance_for_rent_exemption(data.len());

    let account = SolanaAccount {
        lamports: rent,
        data,
        owner: ZERO_FUN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    };

    let result = svm.set_account(global_state_pubkey, account);

    match result {
        Ok(()) => {},

        Err(error) =>{
            panic!("Could not insert account into SVM:- {:?}", error);
        }
    }
}

pub fn get_initializer_keypair()->Keypair{
    Keypair::
        from_base58_string("3JzA5QAwszDUeHVoK8jZwNNmCvKKDQjz6u47TuW1cVBvsCmvt9Fhpb1WvxyHi8xkrv66NGw8GSsKfiko7NnYbuCW")
}

pub fn create_vault_account(svm: &mut LiteSVM, vault_pubkey: Pubkey, lamports: u64) {
    let account = SolanaAccount {
        lamports,
        data: Vec::new(),
        owner: zero_fun::ID,
        executable: false,
        rent_epoch: 0,
    };

    svm.set_account(vault_pubkey, account).expect("Could not add in vault account");
}

pub fn add_zero_fun_program(litesvm:&mut LiteSVM){
    let binary_path = include_bytes!("../../../../target/deploy/zero_fun.so");

    litesvm.add_program(ZERO_FUN_PROGRAM_ID, binary_path);
}

pub fn disable_signer(instruction:&mut Instruction, key:Pubkey){
    let account_meta = instruction.accounts.iter_mut().find(
        |account| account.pubkey.eq(&key)
    );

    if account_meta.is_none(){
        panic!("Account not in instruction")
    }

    account_meta.map(
        |account_meta| account_meta.is_signer = false
    );
}

pub fn create_game_session_account(
    svm: &mut LiteSVM,
    game_session_pubkey: Pubkey,
    game_session: &GameSession,
) {
    let mut data = Vec::with_capacity(8 + GameSession::INIT_SPACE);

    // Anchor writes the discriminator + fields
    game_session
        .try_serialize(&mut data)
        .expect("Could not serialize GameSession");

    let rent = svm.minimum_balance_for_rent_exemption(data.len());

    let account = SolanaAccount {
        lamports: rent,
        data,
        owner: ZERO_FUN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    };

    svm.set_account(game_session_pubkey, account)
        .expect("Could not insert GameSession account into SVM");
}

pub fn set_current_time(svm: &mut LiteSVM, time:i64){
        let mut initial_clock = svm.get_sysvar::<Clock>();
        initial_clock.unix_timestamp = time;
        svm.set_sysvar::<Clock>(&initial_clock);
}

pub fn ed25519_instruction_for_parts(
    signer: &Keypair,
    parts: &[&[u8]],
) -> Instruction {
    let message_hash = hashv(parts).to_bytes();

    let sig_bytes = signer.sign_message(&message_hash).into();

    let pubkey_bytes: [u8; 32] = signer.pubkey().to_bytes();

    new_ed25519_instruction_with_signature(&message_hash, &sig_bytes, &pubkey_bytes)
}
