use std::fmt::{Display, Formatter};
use std::process::Command;
use tun_tap::Iface;

enum EtherType {
    IPv4,
    ARP,
    IPv6,
}

impl EtherType {
    fn from(raw: [u8; 2]) -> EtherType {
        match raw {
            [0x08, 0x00] => EtherType::IPv4,
            [0x08, 0x06] => EtherType::ARP,
            [0x86, 0xdd] => EtherType::IPv6,
            _ => todo!(),
        }
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

impl Display for IPv4Address {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ipv4({}.{}.{}.{})",
            self.address[0], self.address[1], self.address[2], self.address[3]
        )
    }
}

trait ProtocolHeader<T> {
    fn decode(raw: &Vec<u8>) -> T;
    fn length(&self) -> usize;
}

struct EthernetHeader {
    destination: MacAddress,
    source: MacAddress,
    ether_type: EtherType,
}

impl ProtocolHeader<EthernetHeader> for EthernetHeader {
    fn decode(raw: &Vec<u8>) -> EthernetHeader {
        EthernetHeader {
            destination: MacAddress {
                address: raw[0..6].try_into().unwrap(),
            },
            source: MacAddress {
                address: raw[6..12].try_into().unwrap(),
            },
            ether_type: EtherType::from(raw[12..14].try_into().unwrap()),
        }
    }
    fn length(&self) -> usize {
        14
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

impl ProtocolHeader<ArpHeader> for ArpHeader {
    fn decode(raw: &Vec<u8>) -> ArpHeader {
        ArpHeader {
            htype: usize::from(raw[0]) * 16 + usize::from(raw[1]),
            ptype: EtherType::from(raw[2..4].try_into().unwrap()),
            hlen: usize::from(raw[4]),
            plen: usize::from(raw[5]),
            oper: usize::from(raw[6]) * 16 + usize::from(raw[7]),
            sha: MacAddress {
                address: raw[8..14].try_into().unwrap(),
            },
            spa: IPv4Address {
                address: raw[14..18].try_into().unwrap(),
            },
            tha: MacAddress {
                address: raw[18..24].try_into().unwrap(),
            },
            tpa: IPv4Address {
                address: raw[24..28].try_into().unwrap(),
            },
        }
    }

    fn length(&self) -> usize {
        28
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
        println!("---");
        // println!("{}, {:x?}", len, &data[0..len]);
        let mut loc: usize = 0;
        println!("begin: {:x?}", &data[0..4]);
        loc += 4;
        let ethernet_header = EthernetHeader::decode(&data[loc..].to_owned());
        loc += ethernet_header.length();
        println!("Ethernet Header: {}", ethernet_header);

        println!("Payload: {:x?}", &data[18..len]);
        match ethernet_header.ether_type {
            EtherType::ARP => {
                let arp_header = ArpHeader::decode(&data[loc..].to_owned());
                loc += arp_header.length();
                println!("ARP Header: {}", arp_header);
            }
            EtherType::IPv4 => {
                todo!()
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
                todo!()
            }
        }
    }
}
