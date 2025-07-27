use std::{net::Ipv4Addr};
use tokio_tun::Tun;

use crate::{bit_stream::{BitStream, BitUtils}, byte_object::ByteObject, ipv4_header::IPv4Header};

// mod arp_header;
mod byte_object;
mod bit_stream;
// mod ether_type;
// mod ethernet_header;
mod ipv4_address;
mod ipv4_header;
// mod ipv6_address;
// mod ipv6_header;
// mod mac_address;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let tun = &Tun::builder()
        .name("")
        .mtu(1350)
        .up()
        .address(Ipv4Addr::new(10, 0, 0, 1))
        .destination(Ipv4Addr::new(10, 1, 0, 1))
        .broadcast(Ipv4Addr::BROADCAST)
        .netmask(Ipv4Addr::new(255, 255, 255, 0))
        .queues(1)
        .build()
        .unwrap()[0];
    println!("tun is created.");

    println!("TUN INFO:");
    println!("  Name: {}", tun.name());
    println!("  MTU: {}", tun.mtu().unwrap());
    println!("  Address: {}", tun.address().unwrap());
    println!("  Destination: {}", tun.destination().unwrap());
    println!("  Broadcast: {}", tun.broadcast().unwrap());
    println!("  Netmask: {}", tun.netmask().unwrap());

    println!("Please execute `ping 10.1.0.2` from another terminal to test.");

    let mut buf = vec![0; 1504];

    loop {
        let (buf, id) = tokio::select! {
            Ok(n) = tun.recv(&mut buf) => (&buf[..n], 0),
        };
        println!("reading {} bytes from tun: {:?}", buf.len(), buf);

        if let Err(e) = handle_packet(buf).await {
            eprintln!("Error handling packet: {}", e);
        }
    }
}

async fn handle_packet(buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", buf.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>());
    let mut stream = BitStream::new(&BitUtils::u8s_to_bits(buf));
    match BitUtils::bits_to_u8(stream.view(4))  {
        4 => {
            println!("IPv4 Packet Detected");
            let header = IPv4Header::from_bytes(&mut stream);
            println!("IPv4 Header: {}", header);
        }
        6 => {
            println!("IPv6 Packet Detected");
        }
        _ => {
            println!("Unknown Packet Type: {:?} {}", stream.view(4), BitUtils::bits_to_u8(stream.view(4)));
            return Ok(());
        }
    }
    return Ok(());
    // IPv6Header::from_bytes(&mut stream);
    // let packet = match SlicedPacket::from_ip(buf) {
    //     Ok(packet) => packet,
    //     Err(e) => {
    //         eprintln!("Failed to parse packet: {}", e);
    //         return Ok(());
    //     }
    // };
    // if let (Some(net_header), Some(transport_header)) = (packet.net, packet.transport) {
    //     if let (NetSlice::Ipv4(ipv4_header), TransportSlice::Tcp(tcp_header)) = (net_header, transport_header) {
    //         if tcp_header.destination_port() == 80 {
    //             let ipv4 = ipv4_header.header();
    //             let src_ip = ipv4.source();
    //             let dest_ip = ipv4.destination();
    //             let src_port = tcp_header.source_port();
    //             let dest_port = tcp_header.destination_port();

    //             println!(
    //                 "TCP Packet: {}:{} -> {}:{} (SYN: {}, ACK: {}, FIN: {})",
    //                 format_ip(src_ip), src_port, format_ip(dest_ip), dest_port,
    //                 tcp_header.syn(), tcp_header.ack(), tcp_header.fin()
    //             );
    //         }
    //     }
    // }
    // Ok(())
}

fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}