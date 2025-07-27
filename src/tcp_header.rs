use crate::bit_stream::{BitStream, BitUtils};
use crate::byte_object::ByteObject;
use crate::ipv4_address::IPv4Address;
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
    fn from_bytes(stream: &mut BitStream) -> Self {
        let source_port = BitUtils::bits_to_u16(stream.pop(16));
        let destination_port = BitUtils::bits_to_u16(stream.pop(16));
        let sequence_number = BitUtils::bits_to_u32(stream.pop(32));
        let acknowledgment_number = BitUtils::bits_to_u32(stream.pop(32));
        let data_offset = BitUtils::bits_to_u8(stream.pop(4));
        let reserved = BitUtils::bits_to_u8(stream.pop(4));
        let flags = BitUtils::bits_to_u8(stream.pop(8));
        let window_size = BitUtils::bits_to_u16(stream.pop(16));
        let checksum = BitUtils::bits_to_u16(stream.pop(16));
        let urgent_pointer = BitUtils::bits_to_u16(stream.pop(16));

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
    fn append_to(&self, dst: &mut BitStream) -> usize {
        let mut total_len = 0;
        total_len += dst.append(&BitUtils::u16_to_bits(self.source_port)[..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.destination_port)[..]);
        total_len += dst.append(&BitUtils::u32_to_bits(self.sequence_number)[..]);
        total_len += dst.append(&BitUtils::u32_to_bits(self.acknowledgment_number)[..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.data_offset)[4..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.reserved)[4..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.flags)[..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.window_size)[..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.checksum)[..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.urgent_pointer)[..]);
        total_len
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
        let src_word1 = ((src_bytes[0] as u16) << 8) | (src_bytes[1] as u16);
        let src_word2 = ((src_bytes[2] as u16) << 8) | (src_bytes[3] as u16);
        sum += src_word1 as u32;
        sum += src_word2 as u32;

        // Destination IP Address (32 bits = 2 x 16 bits)
        let dst_bytes = &dst_ip.address;
        let dst_word1 = ((dst_bytes[0] as u16) << 8) | (dst_bytes[1] as u16);
        let dst_word2 = ((dst_bytes[2] as u16) << 8) | (dst_bytes[3] as u16);
        sum += dst_word1 as u32;
        sum += dst_word2 as u32;

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
        let offset_reserved_flags = ((self.data_offset as u16) << 12)
            | ((self.reserved as u16) << 8)
            | (self.flags as u16);
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
    pub fn update_checksum(
        &mut self,
        src_ip: &IPv4Address,
        dst_ip: &IPv4Address,
        tcp_data: &[u8],
    ) {
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