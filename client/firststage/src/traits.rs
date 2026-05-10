use crate::{errors::UpdateFileErro, structs::AdditionalMetadata};

pub trait UpdateFileProvider {
    fn seek(&mut self, offset: u64);
    fn tell(&mut self) -> u64;
    fn get_file_length(&self) -> u64;
    fn read_exact(&mut self, destination: &mut [u8]) -> Result<(), UpdateFileErro>;
}

pub trait UpdateEffector {
    fn check_if_compatible(&self, metadata: &AdditionalMetadata) -> Result<(), UpdateFileErro>;
    // This function shall not return. It shall take over the execution environment until the end.
    // After this has been triggered, the control shall be ceded to the second stage permanently.
    fn export(&self, source: &mut dyn UpdateFileProvider) -> Result<(), UpdateFileErro>;
}

pub trait KeyProvider {
    fn get_mldsa87_pubkey(&self, id: u64) -> Option<[u8; 2592]>;
}
