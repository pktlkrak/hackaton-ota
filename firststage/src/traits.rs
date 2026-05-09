use core::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum FirmwareFileError {
    IllegalDataFoundInSectionDeclaration,
}

impl Display for FirmwareFileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Firmware update error: {:?}", self)
    }
}

impl Error for FirmwareFileError {

}

pub trait FirmwareFileProvider {
    fn seek(&mut self, offset: u64);
    fn tell(&self) -> u64;
    fn read_exact(&mut self, length: u64, destination: &mut [u8]) -> Result<(), FirmwareFileError>;
    fn sha512_of_next(&mut self, length: u64, digest: &mut [u8; 64]) -> Result<(), FirmwareFileError>;
}

pub trait FirmwareSectionWriter {
    fn write_part(&mut self, from: &mut dyn FirmwareFileProvider, length: u64) -> Result<(), FirmwareFileError>;
}

pub trait SecondStageExecutor {
    // This function shall not return. It shall take over the execution environment until the end.
    // After this has been triggered, the control shall be ceded to the second stage permanently.
    fn execute(&self, data: &[u8]) -> !;
}
