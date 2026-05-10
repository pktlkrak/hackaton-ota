use std::{
    fs::{self, File},
    io::{Read, Seek, Write},
    path::PathBuf,
};

use anyhow::{Result, bail};
use firststage::{
    errors::FirmwareFileError,
    structs::Semver,
    traits::{FirmwareFileProvider, FirmwareUpdateEffector, KeyProvider},
};

pub const TRIGGER_UPDATE_FILE: i32 = 123;

pub struct FSFirmwareFileProvider {
    file: File,
}

impl FSFirmwareFileProvider {
    // File should have an exclusive lock!
    pub fn new(mut file: File) -> FSFirmwareFileProvider {
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        FSFirmwareFileProvider { file }
    }
}

impl FirmwareFileProvider for FSFirmwareFileProvider {
    fn get_file_length(&self) -> u64 {
        self.file.metadata().unwrap().len()
    }

    fn read_exact(&mut self, destination: &mut [u8]) -> Result<(), FirmwareFileError> {
        if let Err(err) = self.file.read_exact(destination) {
            Err(FirmwareFileError::ReadError)
        } else {
            Ok(())
        }
    }

    fn seek(&mut self, offset: u64) {
        self.file.seek(std::io::SeekFrom::Start(offset)).unwrap();
    }

    fn tell(&mut self) -> u64 {
        self.file.stream_position().unwrap()
    }
}

pub struct FSKeyProvider {
    base_path: PathBuf,
}

impl FSKeyProvider {
    pub fn new(base_path: &str) -> Result<Self> {
        let base: PathBuf = base_path.to_string().try_into().unwrap();
        if fs::metadata(&base)?.is_dir() {
            Ok(Self { base_path: base })
        } else {
            bail!("No base folder for keys provided!");
        }
    }
}

impl KeyProvider for FSKeyProvider {
    fn get_mldsa87_pubkey(&self, id: u64) -> Option<[u8; 2592]> {
        let file = File::open(self.base_path.join(&format!("{id}.key")));
        match file {
            Err(x) => {
                println!("Error while opening key {id}'s file: {x:?}");
                None
            }
            Ok(mut e) => {
                if e.metadata().unwrap().len() != 2592 {
                    println!("Invalid size of key ID {id}");
                    None
                } else {
                    let mut output = [0u8; 2592];
                    if let Err(x) = e.read_exact(&mut output) {
                        println!("Cannot read key ID {id}: {x:?}");
                        None
                    } else {
                        Some(output)
                    }
                }
            }
        }
    }
}

pub struct FSFirmwareUpdateEffector {
    destination_path: Option<String>,
    current_ver: Semver,
}

impl FSFirmwareUpdateEffector {
    pub fn new(current_ver: Semver, destination: &str) -> Self {
        Self {
            current_ver,
            destination_path: Some(destination.to_string()),
        }
    }

    pub fn new_validation_only(current_ver: Semver) -> Self {
        Self {
            current_ver,
            destination_path: None,
        }
    }
}

impl FirmwareUpdateEffector for FSFirmwareUpdateEffector {
    fn check_if_compatible(
        &self,
        metadata: &firststage::structs::AdditionalMetadata,
    ) -> std::prelude::v1::Result<(), FirmwareFileError> {
        if self.current_ver >= metadata.semver {
            println!("Firmware downgrade attempt detected!");
            Err(FirmwareFileError::DowngradeAttempted)
        } else {
            Ok(())
        }
    }

    fn export(&self, source: &mut dyn FirmwareFileProvider) -> Result<(), FirmwareFileError> {
        if let Some(destination_path) = &self.destination_path {
            let mut cursor = source.tell();
            let size = source.get_file_length();
            let mut buffer = [0u8; 512];
            let mut output_file = match File::create(destination_path.clone()) {
                Err(e) => {
                    println!("Failed to create the destination file: {e:?}");
                    return Err(FirmwareFileError::WriteError);
                }
                Ok(e) => e,
            };

            while cursor < size {
                let chunk = buffer.len().min((size - cursor) as usize);
                let subbuff = &mut buffer[0..chunk];
                if source.read_exact(subbuff).is_err() {
                    return Err(FirmwareFileError::ReadError);
                }
                cursor += chunk as u64;
                if output_file.write_all(subbuff).is_err() {
                    return Err(FirmwareFileError::WriteError);
                }
            }
            Ok(())
        } else {
            Err(FirmwareFileError::WriteError)
        }
    }
}
