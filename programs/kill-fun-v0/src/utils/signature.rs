use anchor_lang::{
    prelude::*, 
    solana_program::{
        ed25519_program::ID as ED25519_ADDRESS, 
        hash::{Hash, hashv},
        sysvar::instructions::load_instruction_at_checked
    }
};

use crate::GameError;

/// Stores the offsets used in the ED25519 instruction data, gotten from here:-
/// https://github.com/anza-xyz/solana-sdk/blob/ae3b4e7bdab8d701f7a928fe2e9194229f36cce3/ed25519-program/src/lib.rs#L20
#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct Ed25519SignatureOffsets {
    signature_offset: u16,             // offset to ed25519 signature of 64 bytes
    signature_instruction_index: u16,  // instruction index to find signature
    public_key_offset: u16,            // offset to public key of 32 bytes
    public_key_instruction_index: u16, // instruction index to find public key
    message_data_offset: u16,          // offset to start of message data
    message_data_size: u16,            // size of message data
    message_instruction_index: u16,    // index of instruction data to get message data
}

/// Checks if an ED25519 signature is valid, the implementation is gotten from here:-
/// https://github.com/anza-xyz/solana-sdk/blob/ae3b4e7bdab8d701f7a928fe2e9194229f36cce3/ed25519-program/src/lib.rs#L59
pub fn is_signature_valid(instruction_sysvar:&AccountInfo, message:&[&[u8]], 
        message_signer:&Pubkey)->Result<()>{
        let message_hash = hashv(message);

        let ed25519_instruction = 
            load_instruction_at_checked(0, instruction_sysvar)?;

        require_keys_eq!(
            ed25519_instruction.program_id,
            ED25519_ADDRESS,
            GameError::InvalidED25519Program
        );

        require_eq!(
            ed25519_instruction.accounts.len(),
            0,
            GameError::InvalidAccountCountForED25519Program
        );

        if ed25519_instruction.data.len().le(&14)
            || *ed25519_instruction.data.get(0).unwrap() != 1
            || *ed25519_instruction.data.get(1).unwrap() != 0
        {
            return Err(GameError::InvalidDataForED25519Program.into());
        }

        let mut data = ed25519_instruction.data.get(2..14)
            .ok_or(GameError::InvalidDataForED25519Program)?;

        let offsets = Ed25519SignatureOffsets::deserialize(&mut data)
            .map_err(|_| GameError::InvalidDataForED25519Program)?;

        let pubkey_offset = usize::from(offsets.public_key_offset);
        let pubkey_end = pubkey_offset + core::mem::size_of::<Pubkey>();
        let pubkey_bytes = ed25519_instruction
            .data
            .get(pubkey_offset..pubkey_end)
            .ok_or(GameError::InvalidDataForED25519Program)?;

        if *message_signer.as_array() != *pubkey_bytes {
            return Err(GameError::InvalidMessageSigner.into());
        }

        require_eq!(
            usize::from(offsets.message_data_size),
            core::mem::size_of::<Hash>(),
            GameError::InvalidCommitment
        );

        let msg_offset = usize::from(offsets.message_data_offset);
        let msg_end = msg_offset + core::mem::size_of::<Hash>();
        let msg_bytes = ed25519_instruction
            .data
            .get(msg_offset..msg_end)
            .ok_or(GameError::InvalidDataForED25519Program)?;

        if *message_hash.as_ref() != *msg_bytes {
            return Err(GameError::InvalidCommitment.into());
        }


        Ok(())
}