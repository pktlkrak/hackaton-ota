use ml_dsa::{MlDsa87, Signature, VerifyingKey, signature::Verifier};
use sha2::{Digest, Sha512};

use crate::{errors::FirmwareFileError, structs::{ADDITIONAL_METADATA_OFFSET, AdditionalMetadata}, traits::{FirmwareFileProvider, FirmwareUpdateEffector, KeyProvider}};

// Checks the main header and all signatures.
fn validate_main_header(key_provider: &dyn KeyProvider, data_provider: &mut dyn FirmwareFileProvider) -> Result<(), FirmwareFileError> {
    // Skip to just after the magic:
    data_provider.seek(8);
    let mut key_id_bytes: [u8; 8] = [0; 8];
    data_provider.read_exact(&mut key_id_bytes)?;
    let key_id = u64::from_le_bytes(key_id_bytes);

    let mut provided_sha_sum_of_rest: [u8; 64] = [0; 64];
    data_provider.read_exact(&mut provided_sha_sum_of_rest)?;

    let mut provided_signature: [u8; 4627] = [0; 4627];
    data_provider.read_exact(&mut provided_signature)?;


    let current_pos = data_provider.tell();
    let mut hasher = Sha512::new();

    let total_length = data_provider.get_file_length();
    let mut cursor = current_pos;
    
    let mut buffer: [u8; 512] = [0; 512];
    while cursor < total_length {
        let part_size = 512u64.min(total_length - cursor);
        data_provider.read_exact(&mut buffer[0..part_size as usize])?;
        hasher.update(&buffer[0..part_size as usize]);
        cursor += part_size;
    }

    // Check if the SHAsums match:
    let calculated_digest = hasher.finalize();
    if calculated_digest.as_slice() != provided_sha_sum_of_rest {
        return Err(FirmwareFileError::ChecksumMismatch);
    }

    // And if the signature is correct:
    let verifying_key: VerifyingKey<MlDsa87> = VerifyingKey::decode(
        (&key_provider.get_mldsa87_pubkey(key_id).ok_or(FirmwareFileError::KeyNotFound)?).try_into().unwrap()
    );

    let signature = Signature::decode((&provided_signature).try_into().unwrap()).ok_or(FirmwareFileError::SignatureError)?;

    if verifying_key.verify(&provided_sha_sum_of_rest, &signature).is_err() {
        return Err(FirmwareFileError::SignatureError)
    }

    // If this stage has been reached, the firmware should be valid.

    Ok(())
}

fn validate_magic_and_additional_metadata(data_provider: &mut dyn FirmwareFileProvider, trigger: &dyn FirmwareUpdateEffector) -> Result<(), FirmwareFileError> {
    data_provider.seek(0);
    let mut magic: [u8; 8] = [0; 8];
    data_provider.read_exact(&mut magic)?;
    if magic != "UPXD0001".as_bytes() {
        return Err(FirmwareFileError::IncorrectMagic)
    }

    // Seek to additional metadata section:
    data_provider.seek(ADDITIONAL_METADATA_OFFSET);
    let mut serialized_additional_metadata = [0u8; 16];
    data_provider.read_exact(&mut serialized_additional_metadata)?;
    let metadata = AdditionalMetadata::from(&serialized_additional_metadata)?;

    trigger.check_if_compatible(&metadata)?;
    if metadata.length != data_provider.get_file_length() {
        Err(FirmwareFileError::LengthMismatch)
    } else {
        Ok(())
    }
}

pub fn validate_update(
    key_provider: &dyn KeyProvider,
    data_provider: &mut dyn FirmwareFileProvider,
    trigger: &dyn FirmwareUpdateEffector,
) -> Result<(), FirmwareFileError> {
    // Read whether or not we're even compatible with this firmware update:
    validate_magic_and_additional_metadata(data_provider, trigger)?;
    // Read the main header (and check signatures):
    validate_main_header(key_provider, data_provider)?;

    Ok(())
}

pub fn validate_and_perform_update(
    key_provider: &dyn KeyProvider,
    data_provider: &mut dyn FirmwareFileProvider,
    trigger: &dyn FirmwareUpdateEffector,
) -> Result<(), FirmwareFileError> {
    validate_update(key_provider, data_provider, trigger)?;
    // Export second stage updater
    trigger.export(data_provider)
}
