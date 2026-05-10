use crate::errors::UpdateFileErro;

pub const ADDITIONAL_METADATA_OFFSET: u64 = 8 + 8 + 64 + 4627;
pub const MAGIC_OFFSET: u64 = 0;
pub const SECOND_STAGE_OFFSET: u64 = ADDITIONAL_METADATA_OFFSET + 16;

#[derive(PartialEq, Eq)]
pub struct Semver {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub alpha: u16,
}

#[cfg(feature = "parsing")]
impl Semver {
    pub fn parse(string: &str) -> anyhow::Result<Semver> {
        use alloc::vec::Vec;

        let parts: Vec<_> = string.split('.').collect();
        match parts.len() {
            3 => {
                // x.y.z
                Ok(Semver {
                    alpha: 0,
                    major: parts[0].parse()?,
                    minor: parts[1].parse()?,
                    patch: parts[2].parse()?,
                })
            }
            4 => {
                // x.y.z.a
                Ok(Semver {
                    major: parts[0].parse()?,
                    minor: parts[1].parse()?,
                    patch: parts[2].parse()?,
                    alpha: parts[3].parse()?,
                })
            }

            _ => anyhow::bail!(
                "Invalid format of the version! Expected semver or extended semver (x.y.z.a)"
            ),
        }
    }
}

impl PartialOrd for Semver {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Semver {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then(self.alpha.cmp(&other.alpha))
    }
}

pub struct AdditionalMetadata {
    pub semver: Semver,
    pub length: u64,
}

impl AdditionalMetadata {
    pub fn from(data: &[u8; 16]) -> Result<Self, UpdateFileErro> {
        let major = u16::from_le_bytes((&data[0..2]).try_into().unwrap());
        let minor = u16::from_le_bytes((&data[2..4]).try_into().unwrap());
        let patch = u16::from_le_bytes((&data[4..6]).try_into().unwrap());
        let alpha = u16::from_le_bytes((&data[6..8]).try_into().unwrap());

        let length = u64::from_le_bytes((&data[8..16]).try_into().unwrap());
        Ok(Self {
            semver: Semver {
                major,
                minor,
                patch,
                alpha,
            },
            length,
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
