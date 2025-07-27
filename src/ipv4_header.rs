use crate::bit_stream::{BitStream, Bits, BitsCompatible};
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
    fn from_stream(src: &mut BitStream) -> Self {
        let version = src.pop(4).to_u8();
        let ihl = src.pop(4).to_u8();
        let dscp = src.pop(6).to_u8();
        let ecn = src.pop(2).to_u8();
        let total_length = src.pop(16).to_u16();
        let identification = src.pop(16).to_u16();
        let flags = src.pop(3).to_u8();
        let fragment_offset = src.pop(13).to_u16();
        let ttl = src.pop(8).to_u8();
        let protocol = src.pop(8).to_u8();
        let header_checksum = src.pop(16).to_u16();
        let source_address = IPv4Address::from_stream(src);
        let destination_address = IPv4Address::from_stream(src);

        // オプションフィールドのスキップ
        let header_len_bytes = ihl as usize * 4;
        if header_len_bytes > 20 * 8 {
            src.pop(header_len_bytes - 20 * 8);
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
    fn to_bits(&self) -> Bits {
        let mut bits = Bits::new();
        bits.append(&self.version.to_bits()[4..].to_bits());
        bits.append(&self.ihl.to_bits()[4..].to_bits());
        bits.append(&self.dscp.to_bits()[2..].to_bits());
        bits.append(&self.ecn.to_bits()[6..].to_bits());
        bits.append(&self.total_length.to_bits());
        bits.append(&self.identification.to_bits());
        bits.append(&self.flags.to_bits()[5..].to_bits());
        bits.append(&self.fragment_offset.to_bits()[3..].to_bits());
        bits.append(&self.ttl.to_bits());
        bits.append(&self.protocol.to_bits());
        bits.append(&self.header_checksum.to_bits());
        bits.append(&self.source_address.to_bits());
        bits.append(&self.destination_address.to_bits());
        bits
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
        sum += src_bytes.to_u16s()[0] as u32;
        sum += src_bytes.to_u16s()[1] as u32;

        // Destination Address (32 bits = 2 x 16 bits)
        let dst_bytes = &self.destination_address.address;
        sum += dst_bytes.to_u16s()[0] as u32;
        sum += dst_bytes.to_u16s()[1] as u32;

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

    pub fn new_with_checksum(
        version: u8,
        ihl: u8,
        dscp: u8,
        ecn: u8,
        total_length: u16,
        identification: u16,
        flags: u8,
        fragment_offset: u16,
        ttl: u8,
        protocol: u8,
        source_address: IPv4Address,
        destination_address: IPv4Address,
    ) -> Self {
        let mut header = IPv4Header {
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
            header_checksum: 0, // 初期値は0
            source_address,
            destination_address,
        };
        header.update_checksum(); // チェックサムを計算して更新
        header
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
            self.flags,
            self.fragment_offset,
            self.ttl,
            self.protocol,
            self.header_checksum,
            self.source_address,
            self.destination_address
        )
    }
}
