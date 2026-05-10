use core::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum FirmwareFileError {
    IllegalDataFoundInSectionDeclaration,
    IncorrectMagic,
    ChecksumMismatch,
    SignatureError,
    KeyNotFound,
    LengthMismatch,
    GarbageDataFound,
    DowngradeAttempted,

    ReadError,
    WriteError,
}

impl Display for FirmwareFileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Firmware update error: {:?}", self)
    }
}

impl Error for FirmwareFileError {}
