use core::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum UpdateFileErro {
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

impl Display for UpdateFileErro {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Update error: {:?}", self)
    }
}

impl Error for UpdateFileErro {}
