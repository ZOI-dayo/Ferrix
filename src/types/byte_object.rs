use crate::types::bit_stream::{BitStream, Bits};
pub trait ByteObject {
    fn from_stream(src: &mut BitStream) -> Self;
    fn to_bits(&self) -> Bits;
}