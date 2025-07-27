use std::fmt::{Display, Formatter};
use crate::bit_stream::{BitStream, BitUtils};
use crate::byte_object::ByteObject;

#[derive(Clone)]
pub struct IPv4Address {
    pub address: [u8; 4],
}

impl ByteObject for IPv4Address {
    fn from_bytes(stream: &mut BitStream) -> Self {
        IPv4Address {
            address: BitUtils::bits_to_u8s(stream.pop(32))
                .try_into()
                .expect("Invalid IPv4 address length"),
        }
    }

    fn append_to(&self, dst: &mut BitStream) -> usize {
        dst.append(&BitUtils::u8s_to_bits(&self.address));
        32
    }
}

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ipv4({}.{}.{}.{})",
            self.address[0], self.address[1], self.address[2], self.address[3]
        )
    }
}