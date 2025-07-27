use crate::decoder::Decoder;
use crate::byteobject::ByteObject;

struct IpV6Decoder {}

impl Decoder<IpV6Packet> for IpV6Decoder {
    fn decode(&self, buf: &[u8]) -> Result<IpV6Packet, Box<dyn std::error::Error>> {
        // TODO
        Ok(IpV6Packet::new(buf))
    }
}

struct IpV6Packet {
    version: u8,
    traffic_class: u8,
    flow_label: u32,
    payload_length: u16,
    next_header: u8,
    hop_limit: u8,
    source: IpV6Address,
    destination: IpV6Address,
    payload: ByteObject,
}
impl ByteObject for IpV6Packet {}

struct IpV6Address {
    address: [u8; 16],
}
impl ByteObject for IpV6Address {}