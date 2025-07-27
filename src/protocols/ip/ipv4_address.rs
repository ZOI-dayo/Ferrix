use crate::types::bit_stream::{BitStream, Bits};
use crate::types::byte_object::ByteObject;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct IPv4Address {
    pub address: Bits,
}

impl ByteObject for IPv4Address {
    fn from_stream(src: &mut BitStream) -> Self {
        let bits = src.pop(32);
        let address = bits.try_into().expect("Invalid IPv4 address length");
        IPv4Address { address }
    }

    fn to_bits(&self) -> Bits {
        self.address.clone()
    }
}

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ipv4({}.{}.{}.{})",
            self.address.to_u8s()[0],
            self.address.to_u8s()[1],
            self.address.to_u8s()[2],
            self.address.to_u8s()[3]
        )
    }
}
