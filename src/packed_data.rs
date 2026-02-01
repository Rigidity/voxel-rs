use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};

use crate::Material;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PackedData(u64);

impl PackedData {
    pub fn builder() -> PackedDataEncoder {
        PackedDataEncoder {
            data: 0,
            current_bit: 0,
        }
    }

    pub fn decode(self) -> PackedDataDecoder {
        PackedDataDecoder {
            data: self.0,
            current_bit: 0,
        }
    }
}

pub struct PackedDataEncoder {
    data: u64,
    current_bit: u8,
}

impl PackedDataEncoder {
    pub fn with_u8(mut self, value: u8) -> Self {
        assert!(self.current_bit + 8 <= 64);
        self.data |= (value as u64) << self.current_bit;
        self.current_bit += 8;
        self
    }

    pub fn with_u16(mut self, value: u16) -> Self {
        assert!(self.current_bit + 16 <= 64);
        self.data |= (value as u64) << self.current_bit;
        self.current_bit += 16;
        self
    }

    pub fn with_u32(mut self, value: u32) -> Self {
        assert!(self.current_bit + 32 <= 64);
        self.data |= (value as u64) << self.current_bit;
        self.current_bit += 32;
        self
    }

    pub fn with_u64(mut self, value: u64) -> Self {
        assert!(self.current_bit + 64 <= 64);
        self.data |= value << self.current_bit;
        self.current_bit += 64;
        self
    }

    pub fn with_bool(self, value: bool) -> Self {
        self.with_u8(if value { 1 } else { 0 })
    }

    pub fn with_material(self, material: Material) -> Self {
        self.with_u16(material.to_u16().unwrap())
    }

    pub fn build(self) -> PackedData {
        PackedData(self.data)
    }
}

pub struct PackedDataDecoder {
    data: u64,
    current_bit: u8,
}

impl PackedDataDecoder {
    pub fn new(data: PackedData) -> Self {
        Self {
            data: data.0,
            current_bit: 0,
        }
    }

    pub fn take_u8(&mut self) -> u8 {
        assert!(self.current_bit + 8 <= 64);
        let value = (self.data >> self.current_bit) as u8;
        self.current_bit += 8;
        value
    }

    pub fn take_u16(&mut self) -> u16 {
        assert!(self.current_bit + 16 <= 64);
        let value = (self.data >> self.current_bit) as u16;
        self.current_bit += 16;
        value
    }

    pub fn take_u32(&mut self) -> u32 {
        assert!(self.current_bit + 32 <= 64);
        let value = (self.data >> self.current_bit) as u32;
        self.current_bit += 32;
        value
    }

    pub fn take_u64(&mut self) -> u64 {
        assert!(self.current_bit + 64 <= 64);
        let value = self.data >> self.current_bit;
        self.current_bit += 64;
        value
    }

    pub fn take_bool(&mut self) -> bool {
        let value = self.take_u8();
        assert!(value == 0 || value == 1);
        value == 1
    }

    pub fn take_material(&mut self) -> Material {
        Material::from_u16(self.take_u16()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_data() {
        // Empty block data is 0
        assert_eq!(PackedData::builder().build().decode().take_u64(), 0);

        let mut data = PackedData::builder()
            .with_u16(123)
            .with_u16(456)
            .build()
            .decode();
        assert_eq!(data.take_u16(), 123);
        assert_eq!(data.take_u16(), 456);

        let mut data = PackedData::builder().with_u8(255).build().decode();
        assert_eq!(data.take_u8(), 255);
    }
}
