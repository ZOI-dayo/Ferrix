use std::process::Command;
use tun_tap::Iface;

mod byte_stream;
mod byte_object;
mod ether_type;
mod mac_address;
mod ipv4_address;
mod ipv6_address;
mod ethernet_header;
mod arp_header;
mod ipv4_header;
mod ipv6_header;

use byte_stream::ByteStream;
use byte_object::ByteObject;
use ether_type::EtherType;
use ipv4_address::IPv4Address;
use ethernet_header::EthernetHeader;
use arp_header::ArpHeader;
use ipv4_header::IPv4Header;
use ipv6_header::IPv6Header;

use crate::mac_address::MacAddress;

fn parse_mac_address_string(mac_str: &str) -> Option<[u8; 6]> {
    let parts: Vec<&str> = mac_str.split(':').collect();
    if parts.len() != 6 {
        return None;
    }
    let mut address = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        address[i] = u8::from_str_radix(part, 16).ok()?;
    }
    Some(address)
}

fn get_mac_address_from_ip_link_output(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("link/ether") {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }
    }
    None
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

    let ip_link_output = String::from_utf8(
        Command::new("sh")
            .arg("-c")
            .arg(format!("ip link show {}", name))
            .output()
            .expect("failed to execute process")
            .stdout
    )
    .unwrap();
    println!("{}", ip_link_output);

    let mac_address_str = get_mac_address_from_ip_link_output(&ip_link_output)
        .expect("Failed to get MAC address from ip link output");
    println!("Detected MAC Address: {}", mac_address_str);

    let my_mac_address = MacAddress {
        address: parse_mac_address_string(&mac_address_str)
            .expect("Failed to parse MAC address string")
    };

    loop {
        let mut data = vec![0; 1504];
        let len = iface.recv(&mut data).expect("failed to receive data");
        println!("--- Received {} bytes ---", len);
        println!("Raw data: {:x?}", &data[..len]);
        let mut stream = ByteStream { data: data[..len].to_vec(), pos: 0 };
        println!("begin: {:x?}", stream.pop(4));
        let ethernet_header = EthernetHeader::from_bytes(&mut stream);
        println!("Ethernet Header: {}", ethernet_header);
        println!("  Destination: {}", ethernet_header.destination);
        println!("  Source: {}", ethernet_header.source);
        println!("  EtherType: {:?}", ethernet_header.ether_type);

        // println!("Payload: {:x?}", &data[18..len]);
        match ethernet_header.ether_type {
            EtherType::ARP => {
                println!("Detected EtherType: ARP");
                let arp_header = ArpHeader::from_bytes(&mut stream);
                // loc += arp_header.length();
                println!("ARP Header: {}", &arp_header);
                println!("  Received ARP Request: Who has {}? Tell {}", arp_header.tpa, arp_header.spa);
                println!("  Source MAC: {}, Source IP: {}", arp_header.sha, arp_header.spa);
                println!("  Target MAC: {}, Target IP: {}", arp_header.tha, arp_header.tpa);
                let mut send_stream = ByteStream{ data: vec![], pos: 0 };

                // Ethernetヘッダの構築と追加
                let send_ethernet_header = EthernetHeader {
                    destination: ethernet_header.source, // 受信したパケットの送信元MACを宛先MACに設定
                    source: my_mac_address.clone(), // 受信したパケットの宛先MACを送信元MACに設定 (自身のMACアドレス)
                    ether_type: EtherType::ARP, // EtherTypeをARPに設定
                };
                send_ethernet_header.append_to(&mut send_stream);

                // ARP応答ヘッダの構築と追加
                let send_arp_header = ArpHeader {
                    htype: 1, // ハードウェアタイプ (Ethernet)
                    ptype: EtherType::IPv4, // プロトコルタイプ (IPv4)
                    hlen: 6, // ハードウェアアドレス長 (MACアドレスは6バイト)
                    plen: 4, // プロトコルアドレス長 (IPv4アドレスは4バイト)
                    oper: 2, // オペレーションコード (ARP応答)
                    sha: my_mac_address.clone(), // 自身のMACアドレス
                    spa: IPv4Address { address: [192, 168, 178, 2] }, // 自身のIPアドレス
                    tha: arp_header.sha, // 応答先のMACアドレス (受信したARPリクエストの送信元MAC)
                    tpa: arp_header.spa, // 応答先のIPアドレス (受信したARPリクエストの送信元IP)
                };
                send_arp_header.append_to(&mut send_stream);
                println!("  Sending ARP Reply: {} is at {}", send_arp_header.spa, send_arp_header.sha);
                println!("  Reply Destination MAC: {}, Reply Destination IP: {}", send_arp_header.tha, send_arp_header.tpa);

                // パケットの送信
                let bytes_sent: usize = iface
                    .send(send_stream.data.as_slice()) // send_stream.data.as_slice() を使用して完全なバイト列を送信
                    .expect("Failed to send ARP reply");
                println!("Sent {} bytes ARP reply", bytes_sent);
            }
            EtherType::IPv4 => {
                println!("Detected EtherType: IPv4");
                let ipv4_header = IPv4Header::from_bytes(&mut stream);
                println!("IPv4 Header: {}", &ipv4_header);
            }
            EtherType::IPv6 => {
                println!("Detected EtherType: IPv6");
                let ipv6_header = IPv6Header::from_bytes(&mut stream);
                println!("IPv6 Header: {}", &ipv6_header);
            }
        }
    }
}
