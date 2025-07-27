use std::ops::Index;

pub struct BitStream {
    pub bits: Bits,
    pub pos: usize,
    pub remaining: usize,
}

impl BitStream {
    pub fn new(data: Bits) -> Self {
        BitStream {
            bits: data.clone(),
            pos: 0,
            remaining: data.size,
        }
    }

    pub fn pop(&mut self, len: usize) -> Bits {
        let res = Bits {
            size: len,
            data: self.bits.data[self.pos..(self.pos + len)].to_vec(),
        };
        self.pos += len;
        self.remaining -= len;
        res
    }

    pub fn view(&self, len: usize) -> Bits {
        Bits {
            size: len,
            data: self.bits.data[self.pos..(self.pos + len)].to_vec(),
        }
    }

    pub fn append(&mut self, data: Bits) -> usize {
        self.bits.data.extend_from_slice(&data.data);
        self.remaining += data.size;
        self.bits.size - self.pos
    }
}

#[derive(Clone)]
pub struct Bits {
    pub size: usize,
    pub data: Vec<bool>,
}

use std::slice::SliceIndex;

impl<I> Index<I> for Bits
where
    I: SliceIndex<[bool], Output = [bool]>,
{
    type Output = [bool];

    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl Bits {
    pub fn new() -> Self {
        Bits {
            size: 0,
            data: Vec::new(),
        }
    }

    pub fn append(&mut self, bits: &Bits) {
        self.data.extend_from_slice(&bits.data);
        self.size += bits.size;
    }

    pub fn from_vec(data: Vec<bool>) -> Self {
        let size = data.len();
        Bits { size, data }
    }

    pub fn to_bools(&self) -> Vec<bool> {
        self.data.clone()
    }
    pub fn to_u8s(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for chunk in self.data.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            result.push(byte);
        }
        result
    }
    pub fn to_u16s(&self) -> Vec<u16> {
        let mut result = Vec::new();
        for chunk in self.data.chunks(16) {
            let mut value = 0u16;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    value |= 1 << (15 - i);
                }
            }
            result.push(value);
        }
        result
    }
    pub fn to_u8(&self) -> u8 {
        assert!(self.size <= 8, "Cannot convert more than 8 bits to u8");
        let mut byte = 0u8;
        for (i, &bit) in self.data.iter().enumerate().take(8) {
            if bit {
                byte |= 1 << (self.size - 1 - i);
            }
        }
        byte
    }
    pub fn to_u16(&self) -> u16 {
        assert!(self.size <= 16, "Cannot convert more than 16 bits to u16");
        let mut value = 0u16;
        for (i, &bit) in self.data.iter().enumerate().take(16) {
            if bit {
                value |= 1 << (self.size - 1 - i);
            }
        }
        value
    }
    pub fn to_u32(&self) -> u32 {
        assert!(self.size <= 32, "Cannot convert more than 32 bits to u32");
        let mut value = 0u32;
        for (i, &bit) in self.data.iter().enumerate().take(32) {
            if bit {
                value |= 1 << (self.size - 1 - i);
            }
        }
        value
    }
}

pub trait BitsCompatible {
    fn to_bits(&self) -> Bits;
}

impl BitsCompatible for u8 {
    fn to_bits(&self) -> Bits {
        let bits = (0..8).map(|i| (self & (1 << (7 - i))) != 0).collect();
        Bits {
            size: 8,
            data: bits,
        }
    }
}

impl BitsCompatible for u16 {
    fn to_bits(&self) -> Bits {
        let bits = (0..16).map(|i| (self & (1 << (15 - i))) != 0).collect();
        Bits {
            size: 16,
            data: bits,
        }
    }
}

impl BitsCompatible for u32 {
    fn to_bits(&self) -> Bits {
        let bits = (0..32).map(|i| (self & (1 << (31 - i))) != 0).collect();
        Bits {
            size: 32,
            data: bits,
        }
    }
}

impl BitsCompatible for u64 {
    fn to_bits(&self) -> Bits {
        let bits = (0..64).map(|i| (self & (1 << (63 - i))) != 0).collect();
        Bits {
            size: 64,
            data: bits,
        }
    }
}

impl BitsCompatible for usize {
    fn to_bits(&self) -> Bits {
        let bits = (0..std::mem::size_of::<usize>() * 8)
            .map(|i| (self & (1 << (std::mem::size_of::<usize>() * 8 - 1 - i))) != 0)
            .collect();
        Bits {
            size: std::mem::size_of::<usize>() * 8,
            data: bits,
        }
    }
}

impl BitsCompatible for Vec<u8> {
    fn to_bits(&self) -> Bits {
        let bits: Vec<bool> = self.iter().flat_map(|&v| v.to_bits().data).collect();
        Bits {
            size: bits.len(),
            data: bits,
        }
    }
}

impl BitsCompatible for Vec<bool> {
    fn to_bits(&self) -> Bits {
        Bits {
            size: self.len(),
            data: self.clone(),
        }
    }
}

impl BitsCompatible for [u8] {
    fn to_bits(&self) -> Bits {
        let bits: Vec<bool> = self.iter().flat_map(|&v| v.to_bits().data).collect();
        Bits {
            size: bits.len(),
            data: bits,
        }
    }
}

impl BitsCompatible for [bool] {
    fn to_bits(&self) -> Bits {
        Bits {
            size: self.len(),
            data: self.to_vec(),
        }
    }
}
