use crate::types::bit_stream::{BitStream, Bits, BitsCompatible};
use crate::types::byte_object::ByteObject;
use crate::protocols::ip::ipv4_address::IPv4Address;
use std::fmt::{Display, Formatter};

pub struct TcpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub sequence_number: u32,
    pub acknowledgment_number: u32,
    pub data_offset: u8,
    pub reserved: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
}

impl ByteObject for TcpHeader {
    fn from_stream(src: &mut BitStream) -> Self {
        let source_port = src.pop(16).to_u16();
        let destination_port = src.pop(16).to_u16();
        let sequence_number = src.pop(32).to_u32();
        let acknowledgment_number = src.pop(32).to_u32();
        let data_offset = src.pop(4).to_u8();
        let reserved = src.pop(4).to_u8();
        let flags = src.pop(8).to_u8();
        let window_size = src.pop(16).to_u16();
        let checksum = src.pop(16).to_u16();
        let urgent_pointer = src.pop(16).to_u16();

        TcpHeader {
            source_port,
            destination_port,
            sequence_number,
            acknowledgment_number,
            data_offset,
            reserved,
            flags,
            window_size,
            checksum,
            urgent_pointer,
        }
    }
    fn to_bits(&self) -> Bits {
        let mut bits = Bits::new();
        bits.append(&self.source_port.to_bits());
        bits.append(&self.destination_port.to_bits());
        bits.append(&self.sequence_number.to_bits());
        bits.append(&self.acknowledgment_number.to_bits());
        bits.append(&self.data_offset.to_bits()[4..].to_bits());
        bits.append(&self.reserved.to_bits()[4..].to_bits());
        bits.append(&self.flags.to_bits());
        bits.append(&self.window_size.to_bits());
        bits.append(&self.checksum.to_bits());
        bits.append(&self.urgent_pointer.to_bits());
        bits
    }
}

impl TcpHeader {
    /// TCPセグメントのチェックサムを計算する
    /// RFC 793に従って、IPv4疑似ヘッダー + TCPヘッダー + データの16ビット単位の1の補数の和を計算する
    pub fn calculate_checksum(
        &self,
        src_ip: &IPv4Address,
        dst_ip: &IPv4Address,
        tcp_data: &[u8],
    ) -> u16 {
        let mut sum: u32 = 0;

        // IPv4疑似ヘッダーの計算
        // Source IP Address (32 bits = 2 x 16 bits)
        let src_bytes = &src_ip.address;
        sum += src_bytes.to_u16s()[0] as u32;
        sum += src_bytes.to_u16s()[1] as u32;

        // Destination IP Address (32 bits = 2 x 16 bits)
        let dst_bytes = &dst_ip.address;
        sum += dst_bytes.to_u16s()[0] as u32;
        sum += dst_bytes.to_u16s()[1] as u32;

        // Protocol (TCP = 6)
        sum += 6u16 as u32;

        // TCP Length (TCP Header + Data)
        let tcp_length = 20 + tcp_data.len(); // 基本的なTCPヘッダーは20バイト
        sum += tcp_length as u32;

        // TCPヘッダーの計算
        sum += self.source_port as u32;
        sum += self.destination_port as u32;
        sum += (self.sequence_number >> 16) as u32;
        sum += (self.sequence_number & 0xFFFF) as u32;
        sum += (self.acknowledgment_number >> 16) as u32;
        sum += (self.acknowledgment_number & 0xFFFF) as u32;

        // Data Offset (4 bits) + Reserved (4 bits) + Flags (8 bits)
        let offset_reserved_flags =
            ((self.data_offset as u16) << 12) | ((self.reserved as u16) << 8) | (self.flags as u16);
        sum += offset_reserved_flags as u32;

        sum += self.window_size as u32;
        // Checksum フィールドは0として計算
        // sum += 0;
        sum += self.urgent_pointer as u32;

        // TCPデータの計算
        for chunk in tcp_data.chunks(2) {
            if chunk.len() == 2 {
                let word = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
                sum += word as u32;
            } else {
                // 奇数バイトの場合、最後のバイトの後に0を追加
                let word = (chunk[0] as u16) << 8;
                sum += word as u32;
            }
        }

        // キャリーを加算
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        // 1の補数を取る
        !sum as u16
    }

    /// チェックサムが正しいかどうかを検証する
    pub fn verify_checksum(
        &self,
        src_ip: &IPv4Address,
        dst_ip: &IPv4Address,
        tcp_data: &[u8],
    ) -> bool {
        self.calculate_checksum(src_ip, dst_ip, tcp_data) == self.checksum
    }

    /// チェックサムを再計算して更新する
    pub fn update_checksum(&mut self, src_ip: &IPv4Address, dst_ip: &IPv4Address, tcp_data: &[u8]) {
        self.checksum = self.calculate_checksum(src_ip, dst_ip, tcp_data);
    }

    /// 新しいTCPヘッダーを作成し、チェックサムを自動計算する
    pub fn new_with_checksum(
        source_port: u16,
        destination_port: u16,
        sequence_number: u32,
        acknowledgment_number: u32,
        data_offset: u8,
        reserved: u8,
        flags: u8,
        window_size: u16,
        urgent_pointer: u16,
        src_ip: &IPv4Address,
        dst_ip: &IPv4Address,
        tcp_data: &[u8],
    ) -> Self {
        let mut header = TcpHeader {
            source_port,
            destination_port,
            sequence_number,
            acknowledgment_number,
            data_offset,
            reserved,
            flags,
            window_size,
            checksum: 0, // 一時的に0に設定
            urgent_pointer,
        };

        // チェックサムを計算して設定
        header.checksum = header.calculate_checksum(src_ip, dst_ip, tcp_data);
        header
    }
}

impl Display for TcpHeader {
    /// `TcpHeader` を人間が読める形式でフォーマットする。
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "TcpHeader {{ Source Port: {}, Destination Port: {}, Sequence Number: {}, Acknowledgment Number: {}, Data Offset: {}, Reserved: {}, Flags: {:b}, Window Size: {}, Checksum: {}, Urgent Pointer: {} }}",
            self.source_port,
            self.destination_port,
            self.sequence_number,
            self.acknowledgment_number,
            self.data_offset,
            self.reserved,
            self.flags,
            self.window_size,
            self.checksum,
            self.urgent_pointer
        )
    }
}
