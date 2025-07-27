use std::net::Ipv4Addr;
use tokio_tun::Tun;

mod protocols;
mod types;

use crate::protocols::ip::ipv4_header::{IPv4Header};
use crate::protocols::tcp::tcp_flags::{TCP_ACK, TCP_FIN, TCP_SYN};
use crate::protocols::tcp::tcp_header::TcpHeader;
use crate::types::bit_stream::{BitStream, Bits, BitsCompatible};
use crate::protocols::ip::ipv4_address::IPv4Address;
use crate::types::byte_object::ByteObject;

#[tokio::main]
async fn main() {
    // --- サーバーの起動 ---
    println!("Hello, world!");

    // TUNデバイスの設定
    let tun = &Tun::builder()
        .name("") // 名称はOSに委ねる
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

    println!("Please execute `curl 10.1.0.2` from another terminal to test.");

    let mut buf = vec![0; 1504];

    // メインループ
    loop {
        let buf = tokio::select! {
            Ok(n) = tun.recv(&mut buf) => &buf[..n],
        };
        println!("reading {} bytes from tun: {:?}", buf.len(), buf);

        if let Err(e) = handle_packet(buf, tun).await {
            eprintln!("Error handling packet: {}", e);
        }
    }
}

// パケット受信時、処理を行う
async fn handle_packet(buf: &[u8], tun: &Tun) -> Result<(), Box<dyn std::error::Error>> {
    // 前から読んでいくため、Streamに変換
    let mut stream = BitStream::new(buf.to_vec().to_bits());
    // 先頭4bitがプロコトルを表す
    match stream.view(4).to_u8() {
        4 => {
            // IPv4パケットの処理
            println!("IPv4 Packet Detected");
            let ipv4_header = IPv4Header::from_stream(&mut stream);
            println!("IPv4 Header: {}", ipv4_header);
            match ipv4_header.protocol {
                6 => {
                    // TCPパケットの処理
                    println!("TCP Packet Detected");
                    let tcp_header = TcpHeader::from_stream(&mut stream);
                    println!("TCP Header: {}", tcp_header);
                    if (tcp_header.flags & TCP_SYN) != 0 {
                        // SYNフラグが立っている場合、SYN-ACKを送信
                        println!("TCP SYN Packet Detected");
                        send_syn_ack(
                            ipv4_header.source_address,
                            ipv4_header.destination_address,
                            tcp_header.source_port,
                            tcp_header.destination_port,
                            tcp_header.sequence_number,
                            tun,
                        )
                        .await?;
                    } else if tcp_header.destination_port == 80 {
                        // HTTPリクエストを受信した場合、HTTPレスポンスを送信
                        println!("HTTP Packet Detected");
                        send_http_response(
                            ipv4_header.source_address,
                            ipv4_header.destination_address,
                            tcp_header.source_port,
                            tcp_header.destination_port,
                            tcp_header.sequence_number,
                            tcp_header.acknowledgment_number,
                            tun,
                        )
                        .await?;
                    }
                }
                17 => {
                    // UDPパケットの処理
                    println!("UDP Packet Detected");
                    todo!("Handle UDP packet");
                }
                _ => {
                    // その他のプロトコルは無視
                    println!("Unknown Protocol: {}", ipv4_header.protocol);
                    return Ok(());
                }
            }
        }
        6 => {
            // IPv6パケットの処理
            println!("IPv6 Packet Detected");
            // todo!("Handle IPv6 packet");
        }
        _ => {
            println!("Unknown Packet Type: {}", stream.view(4).to_u8());
            return Ok(());
        }
    }
    return Ok(());
}

async fn send_http_response(
    src_ip: IPv4Address,
    dest_ip: IPv4Address,
    src_port: u16,
    dest_port: u16,
    seq_num: u32,
    ack_num: u32,
    tun: &Tun,
) -> Result<(), Box<dyn std::error::Error>> {
    // 送信内容
    // TODO: ファイルから読み込むなど、実際のHTTPレスポンスを生成する
    let http_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 53\r\nConnection: close\r\n\r\n<html><body><h1>Hello from Ferrix!</h1></body></html>";

    let mut response = BitStream::new(Bits::new());

    // IPv4ヘッダーの作成
    let ipv4_header = IPv4Header::new_with_checksum(
        4,
        5,
        0,
        0,
        (20 + 20 + http_response.len() as u16) as u16, // IPv4ヘッダー(20 bytes) + TCPヘッダー(20 bytes) + HTTPデータ
        (ack_num + 1) as u16,
        2, // Don't Fragment
        0,
        64,
        6, // TCP
        dest_ip.clone(),
        src_ip.clone(),
    );
    response.append(ipv4_header.to_bits());

    // TCPヘッダーの作成
    let tcp_header = TcpHeader::new_with_checksum(
        dest_port,
        src_port,
        ack_num,
        seq_num + 1,
        5, // 5 * 4 = 20 bytes
        0,
        TCP_ACK | TCP_FIN,    // ACK + FINフラ
        (seq_num + 1) as u16, // 最大ウィンドウサイズ
        0,
        &src_ip,
        &dest_ip,
        http_response.as_bytes(),
    );
    response.append(tcp_header.to_bits());

    response.append(http_response.as_bytes().to_bits());

    tun.send(&response.bits.to_u8s()).await?;
    println!("Sent HTTP response with FIN");
    Ok(())
}

async fn send_syn_ack(
    src_ip: IPv4Address,
    dest_ip: IPv4Address,
    src_port: u16,
    dest_port: u16,
    seq_num: u32,
    tun: &Tun,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut response = BitStream::new(Bits::new());

    let mut tcp_header = TcpHeader {
        source_port: dest_port,
        destination_port: src_port,
        sequence_number: 12345,
        acknowledgment_number: seq_num + 1,
        data_offset: 5, // 5 * 4 = 20 bytes
        reserved: 0,
        flags: 0b00010010,                 // SYN flag
        window_size: (seq_num + 1) as u16, // Maximum window size
        checksum: 0,                       // Placeholder for checksum
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

    response.append(ipv4_header.to_bits());
    response.append(tcp_header.to_bits());

    println!(
        "{:?}",
        &response
            .bits
            .to_u8s()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
    );
    // exit(0);
    tun.send(&response.bits.to_u8s()).await?;
    println!("Sent SYN-ACK response");

    Ok(())
}
