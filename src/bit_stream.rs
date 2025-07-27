pub struct BitStream {
    pub bits: Vec<bool>,
    pub pos: usize,
}

impl BitStream {
    pub fn new(data: &[bool]) -> Self {
        BitStream {
            bits: data.to_vec(),
            pos: 0,
        }
    }

    pub fn pop(&mut self, len: usize) -> &[bool] {
        let res = &self.bits[self.pos..(self.pos + len)];
        self.pos += len;
        res
    }

    pub fn view(&self, len: usize) -> &[bool] {
        &self.bits[self.pos..(self.pos + len)]
    }

    pub fn append(&mut self, data: &[bool]) -> usize {
        self.bits.extend_from_slice(data);
        self.bits.len()
    }
}

pub struct BitUtils;
impl BitUtils {
    pub fn u8_to_bits(value: u8) -> Vec<bool> {
        (0..8).map(|i| (value & (1 << (7 - i))) != 0).collect()
    }

    pub fn bits_to_u8(bits: &[bool]) -> u8 {
        bits.iter()
            .enumerate()
            .fold(0, |acc, (i, &bit)| acc | ((bit as u8) << (bits.len() - 1 - i)))
    }

    pub fn u8s_to_bits(values: &[u8]) -> Vec<bool> {
        values.iter().flat_map(|&v| Self::u8_to_bits(v)).collect()
    }
    pub fn bits_to_u8s(bits: &[bool]) -> Vec<u8> {
        bits.chunks(8).map(|chunk| Self::bits_to_u8(chunk)).collect()
    }

    pub fn bits_to_u16(bits: &[bool]) -> u16 {
        bits.iter()
            .enumerate()
            .fold(0, |acc, (i, &bit)| acc | ((bit as u16) << (15 - i)))
    }
    pub fn u16_to_bits(value: u16) -> Vec<bool> {
        (0..16).map(|i| (value & (1 << (15 - i))) != 0).collect()
    }
}
