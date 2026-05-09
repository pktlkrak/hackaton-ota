use crate::traits::FirmwareFileError;


pub struct FirmwareSection {
    id: u32,
    size: u64,
    shasum: [u8; 64],
}

/* Internal layout from markdown doc */
impl FirmwareSection {
    pub fn from(data: &[u8; 80]) -> Result<FirmwareSection, FirmwareFileError> {
        let id = u32::from_le_bytes((&data[0..4]).try_into().unwrap());
        let padding = u32::from_le_bytes((&data[4..8]).try_into().unwrap());
        let size = u64::from_le_bytes((&data[8..16]).try_into().unwrap());
        let shasum: [u8; 64] = TryInto::<&[u8; 64]>::try_into(&data[16..80]).unwrap().clone();

        if padding == 0 {
            Ok(FirmwareSection { id, size, shasum })
        } else {
            Err(FirmwareFileError::IllegalDataFoundInSectionDeclaration)
        }
    }

    pub fn serialize(&self, dest: &mut [u8; 80]) {
        dest[0..4].copy_from_slice(&self.id.to_le_bytes());
        dest[4..8].fill(0);
        dest[8..16].copy_from_slice(&self.size.to_le_bytes());
        dest[16..80].copy_from_slice(&self.shasum);
    }
}

