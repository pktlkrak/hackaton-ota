use crate::errors::FirmwareFileError;

pub const ADDITIONAL_METADATA_OFFSET: u64 = 8 + 8 + 64 + 4627;
pub const MAGIC_OFFSET: u64 = 0;
pub const SECTIONS_OFFSET: u64 = ADDITIONAL_METADATA_OFFSET + 16;

pub struct Semver {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub alpha: u16,
}

pub struct AdditionalMetadata {
    pub semver: Semver,
    pub length: u64,
}

pub struct FirmwareSection {
    pub id: u32,
    pub size: u64,
    pub sha256sum: [u8; 32],
}

/* Internal layout from markdown doc */
impl FirmwareSection {
    pub fn from(data: &[u8; 48]) -> Result<Self, FirmwareFileError> {
        let id = u32::from_le_bytes((&data[0..4]).try_into().unwrap());
        let padding = u32::from_le_bytes((&data[4..8]).try_into().unwrap());
        let size = u64::from_le_bytes((&data[8..16]).try_into().unwrap());
        let sha256sum: [u8; 32] = TryInto::<&[u8; 32]>::try_into(&data[16..48]).unwrap().clone();

        if padding == 0 {
            Ok(Self { id, size, sha256sum })
        } else {
            Err(FirmwareFileError::IllegalDataFoundInSectionDeclaration)
        }
    }

    pub fn serialize(&self, dest: &mut [u8; 48]) {
        dest[0..4].copy_from_slice(&self.id.to_le_bytes());
        dest[4..8].fill(0);
        dest[8..16].copy_from_slice(&self.size.to_le_bytes());
        dest[16..48].copy_from_slice(&self.sha256sum);
    }
}


impl AdditionalMetadata {
    pub fn from(data: &[u8; 16]) -> Result<Self, FirmwareFileError> {
        let major = u16::from_le_bytes((&data[0..2]).try_into().unwrap());
        let minor = u16::from_le_bytes((&data[2..4]).try_into().unwrap());
        let patch = u16::from_le_bytes((&data[4..6]).try_into().unwrap());
        let alpha = u16::from_le_bytes((&data[6..8]).try_into().unwrap());

        let length = u64::from_le_bytes((&data[8..16]).try_into().unwrap());
        Ok(Self {
            semver: Semver { major, minor, patch, alpha },
            length
        })
    }

    pub fn serialize(&self, dest: &mut [u8; 16]) {
        dest[0..2].copy_from_slice(&self.semver.major.to_le_bytes());
        dest[2..4].copy_from_slice(&self.semver.minor.to_le_bytes());
        dest[4..6].copy_from_slice(&self.semver.patch.to_le_bytes());
        dest[6..8].copy_from_slice(&self.semver.alpha.to_le_bytes());
        dest[8..16].copy_from_slice(&self.length.to_le_bytes());
    }
}