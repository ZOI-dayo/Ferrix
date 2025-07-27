use crate::bit_stream::{BitStream, BitUtils};
use crate::byte_object::ByteObject;
use crate::ipv4_address::IPv4Address;
use std::fmt::{Display, Formatter};

pub struct IPv4Header {
    pub version: u8,
    pub ihl: u8,
    pub dscp: u8,
    pub ecn: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags: u8,
    pub fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub header_checksum: u16,
    pub source_address: IPv4Address,
    pub destination_address: IPv4Address,
}

impl ByteObject for IPv4Header {
    fn from_bytes(stream: &mut BitStream) -> Self {
        let version = BitUtils::bits_to_u8(stream.pop(4));
        let ihl = BitUtils::bits_to_u8(stream.pop(4));
        let dscp = BitUtils::bits_to_u8(stream.pop(6));
        let ecn = BitUtils::bits_to_u8(stream.pop(2));
        let total_length = BitUtils::bits_to_u16(stream.pop(16));
        let identification = BitUtils::bits_to_u16(stream.pop(16));
        let flags = BitUtils::bits_to_u8(stream.pop(3));
        let fragment_offset = BitUtils::bits_to_u16(stream.pop(13));
        let ttl = BitUtils::bits_to_u8(stream.pop(8));
        let protocol = BitUtils::bits_to_u8(stream.pop(8));
        let header_checksum = BitUtils::bits_to_u16(stream.pop(16));
        let source_address = IPv4Address::from_bytes(stream);
        let destination_address = IPv4Address::from_bytes(stream);

        // オプションフィールドのスキップ
        let header_len_bytes = ihl as usize * 4;
        if header_len_bytes > 20 * 8 {
            stream.pop(header_len_bytes - 20 * 8);
        }

        IPv4Header {
            version,
            ihl,
            dscp,
            ecn,
            total_length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            header_checksum,
            source_address,
            destination_address,
        }
    }
    fn append_to(&self, dst: &mut BitStream) -> usize {
        let mut total_len = 0;
        total_len += dst.append(&BitUtils::u8_to_bits(self.version)[4..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.ihl)[4..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.dscp)[2..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.ecn)[6..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.total_length)[..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.identification)[..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.flags)[5..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.fragment_offset)[3..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.ttl)[..]);
        total_len += dst.append(&BitUtils::u8_to_bits(self.protocol)[..]);
        total_len += dst.append(&BitUtils::u16_to_bits(self.header_checksum)[..]);
        total_len += self.source_address.append_to(dst);
        total_len += self.destination_address.append_to(dst);
        total_len
    }
}

impl IPv4Header {
    /// IPv4ヘッダのチェックサムを計算する
    /// RFC 791に従って、ヘッダの16ビット単位の1の補数の和を計算する
    pub fn calculate_checksum(&self) -> u16 {
        let mut sum: u32 = 0;
        
        // Version (4 bits) + IHL (4 bits) + DSCP (6 bits) + ECN (2 bits)
        let version_ihl = ((self.version as u16) << 4) | (self.ihl as u16);
        let dscp_ecn = ((self.dscp as u16) << 2) | (self.ecn as u16);
        let first_word = (version_ihl << 8) | dscp_ecn;
        sum += first_word as u32;
        
        // Total Length
        sum += self.total_length as u32;
        
        // Identification
        sum += self.identification as u32;
        
        // Flags (3 bits) + Fragment Offset (13 bits)
        let flags_fragment = ((self.flags as u16) << 13) | (self.fragment_offset & 0x1FFF);
        sum += flags_fragment as u32;
        
        // TTL + Protocol
        let ttl_protocol = ((self.ttl as u16) << 8) | (self.protocol as u16);
        sum += ttl_protocol as u32;
        
        // Header Checksum フィールドは0として計算
        // sum += 0;
        
        // Source Address (32 bits = 2 x 16 bits)
        let src_bytes = &self.source_address.address;
        let src_word1 = ((src_bytes[0] as u16) << 8) | (src_bytes[1] as u16);
        let src_word2 = ((src_bytes[2] as u16) << 8) | (src_bytes[3] as u16);
        sum += src_word1 as u32;
        sum += src_word2 as u32;
        
        // Destination Address (32 bits = 2 x 16 bits)
        let dst_bytes = &self.destination_address.address;
        let dst_word1 = ((dst_bytes[0] as u16) << 8) | (dst_bytes[1] as u16);
        let dst_word2 = ((dst_bytes[2] as u16) << 8) | (dst_bytes[3] as u16);
        sum += dst_word1 as u32;
        sum += dst_word2 as u32;
        
        // キャリーを加算
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        
        // 1の補数を取る
        !sum as u16
    }
    
    /// チェックサムが正しいかどうかを検証する
    pub fn verify_checksum(&self) -> bool {
        self.calculate_checksum() == self.header_checksum
    }
    
    /// チェックサムを再計算して更新する
    pub fn update_checksum(&mut self) {
        self.header_checksum = self.calculate_checksum();
    }
}

impl Display for IPv4Header {
    /// `IPv4Header` を人間が読める形式でフォーマットする。
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "IPv4 {{ Version: {}, IHL: {}, Total Length: {}, ID: {}, Flags: {:b}, Fragment Offset: {}, TTL: {}, Protocol: {}, Checksum: {}, Src: {}, Dst: {} }}",
            self.version,
            self.ihl,
            self.total_length,
            self.identification,
            (self.flags as u16 & 0xE000) >> 13, //  // 3 bits
            self.fragment_offset & 0x1FFF,      // 13 bits
            self.ttl,
            self.protocol,
            self.header_checksum,
            self.source_address,
            self.destination_address
        )
    }
}
