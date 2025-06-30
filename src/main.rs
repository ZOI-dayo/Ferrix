use std::process::Command;
use tun_tap::Iface;

mod byte_stream;
mod byte_object;
mod ether_type;
mod mac_address;
mod ipv4_address;
mod ethernet_header;
mod arp_header;

use byte_stream::ByteStream;
use byte_object::ByteObject;
use ether_type::EtherType;
use ipv4_address::IPv4Address;
use ethernet_header::EthernetHeader;
use arp_header::ArpHeader;

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
                    sha: arp_header.tha,
                    spa: arp_header.spa,
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
                println!("IPv4は未実装である");
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
                // NDPパケットのデコードは未実装
                println!("IPv6は未実装である")
            }
        }
    }
}
