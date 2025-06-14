use std::fmt::{Display, Formatter};
use std::process::Command;
use tun_tap::Iface;

struct ByteStream {
    data: Vec<u8>,
    pos: usize,
}

impl ByteStream {
    fn pop(&mut self, len: usize) -> &[u8] {
        let res = &self.data[self.pos..(self.pos + len)];
        self.pos += len;
        res
    }
    fn append(&mut self, data: &[u8]) -> usize {
        self.data.extend_from_slice(data);
        self.data.len()
    }
}

// impl From<Vec<u8>> for ByteStream<'_> {
//     fn from(value: Vec<u8>) -> Self {
//         ByteStream {
//             data: value,
//             pos: 0,
//         }
//     }
// }

trait ByteObject {
    fn from_bytes(src: &mut ByteStream) -> Self;
    fn append_to(&self, dst: &mut ByteStream) -> usize;
}

enum EtherType {
    IPv4,
    ARP,
    IPv6,
}

impl ByteObject for EtherType {
    fn from_bytes(stream: &mut ByteStream) -> EtherType {
        let raw = stream.pop(2);
        match raw {
            [0x08, 0x00] => EtherType::IPv4,
            [0x08, 0x06] => EtherType::ARP,
            [0x86, 0xdd] => EtherType::IPv6,
            _ => panic!("raw.len = {}, raw 1 = {}, raw 2 = {}", raw.len(), raw[0], raw[1]),
        }
    }
    fn append_to(&self, dst: &mut ByteStream) -> usize {
        let content: &[u8] = match self {
            EtherType::IPv4 => &[0x08, 0x00],
            EtherType::ARP => &[0x08, 0x06],
            EtherType::IPv6 => &[0x86, 0xdd],
        };
        dst.append(content);
        content.len()
    }
}

impl Display for EtherType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EtherType::IPv4 => "IPv4",
                EtherType::ARP => "ARP",
                EtherType::IPv6 => "IPv6",
            }
        )
    }
}

struct MacAddress {
    address: [u8; 6],
}

impl ByteObject for MacAddress {
    fn from_bytes(src: &mut ByteStream) -> Self {
        MacAddress {
            address: src.pop(6).try_into().unwrap(),
        }
    }

    fn append_to(&self, dst: &mut ByteStream) -> usize {
        dst.append(&self.address);
        6
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "mac({:02x?}:{:02x?}:{:02x?}:{:02x?}:{:02x?}:{:02x?})",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5]
        )
    }
}

struct IPv4Address {
    address: [u8; 4],
}

impl ByteObject for IPv4Address {
    fn from_bytes(stream: &mut ByteStream) -> Self {
        IPv4Address {
            address: stream.pop(4).try_into().unwrap(),
        }
    }

    fn append_to(&self, dst: &mut ByteStream) -> usize {
        dst.append(&self.address);
        4
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

struct EthernetHeader {
    destination: MacAddress,
    source: MacAddress,
    ether_type: EtherType,
}

impl ByteObject for EthernetHeader {
    fn from_bytes(stream: &mut ByteStream) -> EthernetHeader {
        EthernetHeader {
            destination: MacAddress::from_bytes(stream),
            source: MacAddress::from_bytes(stream),
            ether_type: EtherType::from_bytes(stream),
        }
    }

    fn append_to(&self, dst: &mut ByteStream) -> usize {
        self.destination.append_to(dst)
            + self.source.append_to(dst)
            + self.ether_type.append_to(dst)
    }
}

impl Display for EthernetHeader {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Ethernet {{ dest: {}, src: {}, type: {} }}",
            self.destination, self.source, self.ether_type
        )
    }
}

struct ArpHeader {
    htype: usize,
    ptype: EtherType,
    hlen: usize,
    plen: usize,
    oper: usize,
    sha: MacAddress,
    spa: IPv4Address,
    tha: MacAddress,
    tpa: IPv4Address,
}

fn bytes_to_int(raw: &[u8]) -> usize {
    let mut res: usize = 0;
    for x in raw {
        res *= 16;
        res += x.to_owned() as usize;
    }
    res
}

impl ByteObject for ArpHeader {
    fn from_bytes(stream: &mut ByteStream) -> Self {
        ArpHeader {
            htype: bytes_to_int(stream.pop(2)),
            ptype: EtherType::from_bytes(stream),
            hlen: bytes_to_int(stream.pop(1)),
            plen: bytes_to_int(stream.pop(1)),
            oper: bytes_to_int(stream.pop(2)),
            sha: MacAddress::from_bytes(stream),
            spa: IPv4Address::from_bytes(stream),
            tha: MacAddress::from_bytes(stream),
            tpa: IPv4Address::from_bytes(stream),
        }
    }

    fn append_to(&self, dst: &mut ByteStream) -> usize {
        dst.append(&self.htype.to_be_bytes())
            + self.ptype.append_to(dst)
            + dst.append(&self.hlen.to_be_bytes())
            + dst.append(&self.plen.to_be_bytes())
            + dst.append(&self.oper.to_be_bytes())
            + self.sha.append_to(dst)
            + self.spa.append_to(dst)
            + self.tha.append_to(dst)
            + self.tpa.append_to(dst)
    }
}

impl Display for ArpHeader {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ARP {{ HTYPE: {}, PTYPE: {}, HLEN: {}, PLEN: {}, OPER: {}, SHA: {}, SPA: {}, THA: {}, TPA: {} }}",
            self.htype,
            self.ptype,
            self.hlen,
            self.plen,
            self.oper,
            self.sha,
            self.spa,
            self.tha,
            self.tpa
        )
    }
}

fn main() {
    println!("Hello, world!");
    let iface = Iface::new("tap0", tun_tap::Mode::Tap).expect("failed to open tun file");

    let name = iface.name();
    println!(
        "{}",
        String::from_utf8(
            Command::new("sh")
                .arg("-c")
                .arg(format!("ip a add 192.168.178.2/24 dev {}", name))
                .output()
                .expect("failed to execute process")
                .stdout
        )
        .unwrap()
    );
    println!(
        "{}",
        String::from_utf8(
            Command::new("sh")
                .arg("-c")
                .arg(format!("ip link set {} up", name))
                .output()
                .expect("failed to execute process")
                .stdout
        )
        .unwrap()
    );

    println!("{:?}", iface);
    loop {
        let mut data = vec![0; 1504];
        let len = iface.recv(&mut data).expect("failed to receive data");
        let mut stream = ByteStream { data: data[..len].to_vec(), pos: 0 };
        println!("---");
        // println!("{}, {:x?}", len, &data[0..len]);
        // let mut loc: usize = 0;
        println!("begin: {:x?}", stream.pop(4));
        let ethernet_header = EthernetHeader::from_bytes(&mut stream);
        // loc += ethernet_header.length();
        println!("Ethernet Header: {}", ethernet_header);

        // println!("Payload: {:x?}", &data[18..len]);
        match ethernet_header.ether_type {
            EtherType::ARP => {
                let arp_header = ArpHeader::from_bytes(&mut stream);
                // loc += arp_header.length();
                println!("ARP Header: {}", &arp_header);
                let mut send_stream = ByteStream{ data: vec![], pos: 0 };
                let send = ArpHeader {
                    htype: 1,
                    ptype: EtherType::IPv4,
                    hlen: 6,
                    plen: 4,
                    oper: 2,
                    sha: arp_header.tha, // TODO
                    spa: arp_header.spa, // TODO
                    tha: arp_header.sha,
                    tpa: IPv4Address {
                        address: [1, 2, 3, 4],
                    },
                };
                let len = send.append_to(&mut send_stream);
                let send = iface
                    // .send(&[
                    //     0x00,
                    //     0x01, // htype
                    //     0x08,
                    //     0x00, // ptype
                    //     0x06, // hlen
                    //     0x04, // plen
                    //     0x00,
                    //     0x02, // oper
                    //     0xa6,
                    //     0xf6,
                    //     0x6a,
                    //     0xc5,
                    //     0x77,
                    //     0x0d, // sha
                    //     0xc0,
                    //     0xa8,
                    //     0xb2,
                    //     0x02, // spa
                    //     (&arp_header.sha.address[0]).clone(),
                    //     (&arp_header.sha.address[1]).clone(),
                    //     (&arp_header.sha.address[2]).clone(),
                    //     (&arp_header.sha.address[3]).clone(),
                    //     (&arp_header.sha.address[4]).clone(),
                    //     (&arp_header.sha.address[5]).clone(),
                    //     // tha
                    //     0x01,
                    //     0x02,
                    //     0x03,
                    //     0x04, // tpa
                    // ])
                    .send(send_stream.pop(len))
                    .expect("TODO: panic message");
                println!("{}", send);
            }
            EtherType::IPv4 => {
                // todo!()
                println!("ipv4 is not implemented");
            }
            EtherType::IPv6 => {
                println!("  version: {:x?}", &data[18] | 0b11110000);
                // println!(
                //     "  trafic class: {:x?}",
                //     (&data[19] | 0b00001111) as i64 * 256 + (&data[20] | 0b11110000) as i64
                // );
                // println!("  flow label")
                println!("  payload len: {:x?}", &data[22..24]);
                println!("  nxt header: {:x?}", &data[24]);
                println!("  hop limit: {:x?}", &data[25]);
                println!("  src addr: {:x?}", &data[26..42]);
                println!("  dst addr: {:x?}", &data[42..58]);
                if 58 < len {
                    println!("  content: {:x?}", &data[58..len])
                }
                // todo: decode ndp packet
                println!("ipv6 is not implemented")
            }
        }
    }
}
