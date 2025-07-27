//*
use etherparse::{PacketBuilder, err::ip};
use std::{net::Ipv4Addr, process::exit};
use tokio_tun::Tun;

use crate::{
    bit_stream::{BitStream, BitUtils},
    byte_object::ByteObject,
    ipv4_address::IPv4Address,
    ipv4_header::IPv4Header,
    tcp_header::TcpHeader,
};

// mod arp_header;
mod bit_stream;
mod byte_object;
// mod ether_type;
// mod ethernet_header;
mod ipv4_address;
mod ipv4_header;
// mod ipv6_address;
// mod ipv6_header;
// mod mac_address;
mod tcp_header;

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

        if let Err(e) = handle_packet(buf, tun).await {
            eprintln!("Error handling packet: {}", e);
        }
    }
}

async fn handle_packet(buf: &[u8], tun: &Tun) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{:?}",
        buf.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>()
    );
    let mut stream = BitStream::new(&BitUtils::u8s_to_bits(buf));
    match BitUtils::bits_to_u8(stream.view(4)) {
        4 => {
            println!("IPv4 Packet Detected");
            let ipv4_header = IPv4Header::from_bytes(&mut stream);
            println!("IPv4 Header: {}", ipv4_header);
            match ipv4_header.protocol {
                6 => {
                    println!("TCP Packet Detected");
                    let tcp_header = TcpHeader::from_bytes(&mut stream);
                    println!("TCP Header: {}", tcp_header);
                    let payload_bytes = BitUtils::bits_to_u8s(&stream.bits);
                    println!("{:?}, {:?}, {:?}", BitUtils::bits_to_u8s(&stream.bits), payload_bytes, String::from_utf8_lossy(&payload_bytes[..]));
                    if (tcp_header.flags & 0b00000010 != 0) {
                        println!("TCP SYN Packet Detected");
                        send_syn_ack(
                            ipv4_header.source_address.address,
                            ipv4_header.destination_address.address,
                            tcp_header.source_port,
                            tcp_header.destination_port,
                            tcp_header.sequence_number,
                            tun,
                        )
                        .await?;
                        send_syn_ack2(
                            ipv4_header.source_address,
                            ipv4_header.destination_address,
                            tcp_header.source_port,
                            tcp_header.destination_port,
                            tcp_header.sequence_number,
                            tun,
                        )
                        .await?;
                    } else if tcp_header.destination_port == 80 {
                        println!("HTTP Packet Detected");
                        // send_http_response(
                        //     ipv4_header.source_address,
                        //     ipv4_header.destination_address,
                        //     tcp_header.source_port,
                        //     tcp_header.destination_port,
                        //     tcp_header.acknowledgment_number,
                        //     tcp_header.sequence_number,
                        //     tun,
                        // )
                        // .await?;
                        send_http_response(
                            ipv4_header.source_address.address,
                            ipv4_header.destination_address.address,
                            tcp_header.source_port,
                            tcp_header.destination_port,
                            tcp_header.sequence_number,
                            tcp_header.acknowledgment_number,
                            tun,
                        ).await?;
                    }
                }
                17 => {
                    println!("UDP Packet Detected");
                    todo!("Handle UDP packet");
                }
                _ => {
                    println!("Unknown Protocol: {}", ipv4_header.protocol);
                    return Ok(());
                }
            }
        }
        6 => {
            println!("IPv6 Packet Detected");
            // todo!("Handle IPv6 packet");
        }
        _ => {
            println!(
                "Unknown Packet Type: {}",
                BitUtils::bits_to_u8(stream.view(4))
            );
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

async fn send_http_response(
    src_ip: [u8; 4],
    dest_ip: [u8; 4],
    src_port: u16, 
    dest_port: u16,
    seq_num: u32,
    ack_num: u32,
    tun: &Tun
) -> Result<(), Box<dyn std::error::Error>> {
    let http_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 53\r\nConnection: close\r\n\r\n<html><body><h1>Hello from Ferrix!</h1></body></html>";

    let builder = PacketBuilder::ipv4(dest_ip, src_ip, 64)
        .tcp(dest_port, src_port, ack_num, (seq_num + 1) as u16)
        .ack(seq_num + 1)
        .fin(); // FINフラグを追加して接続を終了

    let mut response = Vec::new();
    builder.write(&mut response, http_response.as_bytes())?;

    tun.send(&response).await?;
    println!("Sent HTTP response with FIN");
    Ok(())
}

/*
async fn send_http_response(
    dest_ip: IPv4Address,
    src_ip: IPv4Address,
    dest_port: u16,
    src_port: u16,
    ack_num: u32,
    seq_num: u32,
    tun: &Tun,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = BitStream::new(&mut Vec::new());

    let http_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 52\r\nConnection: close\r\n\r\n<html><body><h1>Hello from Ferrix!</h1></body></html>";

    let mut tcp_header = TcpHeader {
        source_port: src_port,
        destination_port: dest_port,
        sequence_number: ack_num,
        acknowledgment_number: seq_num + 1,
        data_offset: 5, // 5 * 4 = 20 bytes
        reserved: 0,
        flags: 0b00010001,  // ACK + FIN flag
        window_size: 65535, // Maximum window size
        checksum: 0,        // Placeholder for checksum
        urgent_pointer: 0,
    };
    tcp_header.update_checksum(&src_ip, &dest_ip, http_response.as_bytes());

    // Calculate total length: IPv4 header (20 bytes) + TCP header (20 bytes) + HTTP data
    let ipv4_header_size = 20u16; // IHL = 5, so 5 * 4 = 20 bytes
    let tcp_header_size = 20u16; // Data offset = 5, so 5 * 4 = 20 bytes
    let http_data_size = http_response.len() as u16;
    let total_length = ipv4_header_size + tcp_header_size + http_data_size;

    let mut ipv4_header = IPv4Header {
        version: 4,
        ihl: 5,
        dscp: 0,
        ecn: 0,
        total_length,
        identification: 0,
        flags: 2,
        fragment_offset: 0,
        ttl: 64,
        protocol: 6,
        header_checksum: 0,
        source_address: src_ip.clone(),
        destination_address: dest_ip.clone(),
    };
    ipv4_header.update_checksum();

    ipv4_header.append_to(&mut response);
    tcp_header.append_to(&mut response);
    response.append(&BitUtils::u8s_to_bits(http_response.as_bytes()));

    // tun.send(&BitUtils::bits_to_u8s(&response.bits)).await?;
    println!(
        "{:?}",
        &BitUtils::bits_to_u8s(&response.bits)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
    );
    println!("Sent HTTP response with FIN");

    //

    let builder = PacketBuilder::ipv4(src_ip.address, dest_ip.address, 64)
        .tcp(src_port, dest_port, ack_num, (seq_num + 1) as u16)
        .ack(seq_num + 1)
        .fin(); // FINフラグを追加して接続を終了

    let mut response = Vec::new();
    builder.write(&mut response, http_response.as_bytes())?;
    println!(
        "{:?}",
        &response
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
    );
    tun.send(&response).await?;

    Ok(())
}
    */

async fn send_syn_ack(
    src_ip: [u8; 4],
    dest_ip: [u8; 4],
    src_port: u16,
    dest_port: u16,
    seq_num: u32,
    tun: &Tun,
) -> Result<(), Box<dyn std::error::Error>> {
    let builder = PacketBuilder::ipv4(dest_ip, src_ip, 64)
        .tcp(dest_port, src_port, 12345, (seq_num + 1) as u16)
        .syn()
        .ack(seq_num + 1);

    let mut response = Vec::new();
    builder.write(&mut response, &[])?;

    // tun.send(&response).await?;
    println!(
        "{:?}",
        &response
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
    );
    println!("Sent SYN-ACK response");
    Ok(())
}

async fn send_syn_ack2(
    src_ip: IPv4Address,
    dest_ip: IPv4Address,
    src_port: u16,
    dest_port: u16,
    seq_num: u32,
    tun: &Tun,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = BitStream::new(&mut Vec::new());

    let mut tcp_header = TcpHeader {
        source_port: dest_port,
        destination_port: src_port,
        sequence_number: 12345,
        acknowledgment_number: seq_num + 1,
        data_offset: 5, // 5 * 4 = 20 bytes
        reserved: 0,
        flags: 0b00010010, // SYN flag
        window_size: (seq_num + 1) as u16, // Maximum window size
        checksum: 0,        // Placeholder for checksum
        urgent_pointer: 0,
    };
    tcp_header.update_checksum(&src_ip, &dest_ip, &[]);

    // Calculate total length: IPv4 header (20 bytes) + TCP header (20 bytes)
    let ipv4_header_size = 20u16; // IHL = 5, so 5 * 4 = 20 bytes
    let tcp_header_size = 20u16; // Data offset = 5, so 5 * 4 = 20 bytes
    let total_length = ipv4_header_size + tcp_header_size;

    let mut ipv4_header = IPv4Header {
        version: 4,
        ihl: 5,
        dscp: 0,
        ecn: 0,
        total_length,
        identification: 0,
        flags: 2,
        fragment_offset: 0,
        ttl: 64,
        protocol: 6,
        header_checksum: 0,
        source_address: dest_ip.clone(),
        destination_address: src_ip.clone(),
    };
    ipv4_header.update_checksum();

    ipv4_header.append_to(&mut response);
    tcp_header.append_to(&mut response);

    println!(
        "{:?}",
        &BitUtils::bits_to_u8s(&response.bits)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
    );
    // exit(0);
    tun.send(&BitUtils::bits_to_u8s(&response.bits)).await?;
    println!("Sent SYN-ACK response");

    Ok(())
}


//     */
/*
use std::net::Ipv4Addr;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use tokio_tun::Tun;
use etherparse::{PacketBuilder, TransportSlice, NetSlice};

#[tokio::main]
async fn main() {
    let queues = 1;

    let tuns = Tun::builder()
        .name("")
        .mtu(1350)
        .up()
        .address(Ipv4Addr::new(10, 0, 0, 1))
        .destination(Ipv4Addr::new(10, 1, 0, 1))
        .broadcast(Ipv4Addr::BROADCAST)
        .netmask(Ipv4Addr::new(255, 255, 255, 0))
        .queues(queues)
        .build()
        .unwrap();

    println!("--------------");
    println!("{} tuns created", queues);
    println!("--------------");

    println!(
        "┌ name: {}\n├ fd: {}\n├ mtu: {}\n├ flags: {}\n├ address: {}\n├ destination: {}\n├ broadcast: {}\n└ netmask: {}",
        tuns[0].name(),
        tuns[0].as_raw_fd(),
        tuns[0].mtu().unwrap(),
        tuns[0].flags().unwrap(),
        tuns[0].address().unwrap(),
        tuns[0].destination().unwrap(),
        tuns[0].broadcast().unwrap(),
        tuns[0].netmask().unwrap(),
    );

    println!("---------------------");
    println!("ping 10.1.0.2 to test");
    println!("---------------------");

    let mut tuns = tuns.into_iter();
    let tun0 = Arc::new(tuns.next().unwrap());

    let mut buf0 = [0u8; 1024];

    loop {
        let (buf, id) = tokio::select! {
            Ok(n) = tun0.recv(&mut buf0) => (&buf0[..n], 0),
        };
        println!("reading {} bytes from tuns[{}]: {:?}", buf.len(), id, buf);
        
        // パケットを解析してHTTP応答を送信
        if let Err(e) = handle_packet(buf, &tun0).await {
            eprintln!("Error handling packet: {}", e);
        }
    }
}

async fn handle_packet(
    data: &[u8], 
    tun0: &Arc<Tun>
) -> Result<(), Box<dyn std::error::Error>> {
    // パケットを解析
    let packet = match etherparse::SlicedPacket::from_ip(data) {
        Ok(packet) => packet,
        Err(_) => return Ok(()), // 解析失敗は無視
    };

    // TCP HTTPリクエストの場合のみ処理
    if let (Some(net_header), Some(transport_header)) = (packet.net, packet.transport) {
        if let (NetSlice::Ipv4(ipv4_header), TransportSlice::Tcp(tcp_header)) = (net_header, transport_header) {
            // HTTPポート(80)への接続をチェック
            if tcp_header.destination_port() == 80 {
                let ipv4 = ipv4_header.header();
                let src_ip = ipv4.source();
                let dest_ip = ipv4.destination();
                let src_port = tcp_header.source_port();
                let dest_port = tcp_header.destination_port();

                println!("HTTP packet detected: {}:{} -> {}:{}, SYN: {}, ACK: {}, FIN: {}", 
                    format_ip(src_ip), src_port, format_ip(dest_ip), dest_port,
                    tcp_header.syn(), tcp_header.ack(), tcp_header.fin());

                // SYNパケットの場合、SYN-ACKで応答
                if tcp_header.syn() && !tcp_header.ack() {
                    send_syn_ack(src_ip, dest_ip, src_port, dest_port, 
                                tcp_header.sequence_number(), tun0).await?;
                }
                // HTTPリクエストデータが含まれている場合のみHTTPレスポンスを送信
                else if tcp_header.ack() && !tcp_header.syn() && !tcp_header.fin() {
                    // 簡単なペイロード検出: 一定サイズ以上でHTTPメソッドを含む場合
                    if data.len() > 60 { // IP(20) + TCP(20) + HTTPヘッダ最小値
                        let payload_str = String::from_utf8_lossy(data);
                        if payload_str.contains("GET ") || payload_str.contains("POST ") {
                            println!("HTTP request detected in packet");
                            send_http_response(src_ip, dest_ip, src_port, dest_port, 
                                              tcp_header.sequence_number(), tcp_header.acknowledgment_number(),
                                              tun0).await?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn send_syn_ack(
    src_ip: [u8; 4],
    dest_ip: [u8; 4], 
    src_port: u16,
    dest_port: u16,
    seq_num: u32,
    tun0: &Arc<Tun>
) -> Result<(), Box<dyn std::error::Error>> {
    let builder = PacketBuilder::ipv4(dest_ip, src_ip, 64)
        .tcp(dest_port, src_port, 12345, (seq_num + 1) as u16)
        .syn()
        .ack(seq_num + 1);

    let mut response = Vec::new();
    builder.write(&mut response, &[])?;

    let tun = tun0;

    tun.send(&response).await?;
    println!("Sent SYN-ACK response");
    Ok(())
}

async fn send_http_response(
    src_ip: [u8; 4],
    dest_ip: [u8; 4],
    src_port: u16, 
    dest_port: u16,
    seq_num: u32,
    ack_num: u32,
    tun0: &Arc<Tun>
) -> Result<(), Box<dyn std::error::Error>> {
    let http_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 52\r\nConnection: close\r\n\r\n<html><body><h1>Hello from Ferrix!</h1></body></html>";

    let builder = PacketBuilder::ipv4(dest_ip, src_ip, 64)
        .tcp(dest_port, src_port, ack_num, (seq_num + 1) as u16)
        .ack(seq_num + 1)
        .fin(); // FINフラグを追加して接続を終了

    let mut response = Vec::new();
    builder.write(&mut response, http_response.as_bytes())?;

    let tun = tun0;

    tun.send(&response).await?;
    println!("Sent HTTP response with FIN");
    Ok(())
}

fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}
    */