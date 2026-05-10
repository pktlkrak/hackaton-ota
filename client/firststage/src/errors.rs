use core::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum UpdateFileError {
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

impl Display for UpdateFileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Update error: {:?}", self)
    }
}

impl Error for UpdateFileError {}
