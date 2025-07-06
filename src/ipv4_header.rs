//! IPv4ヘッダーを扱うモジュール。
//!
//! IPv4パケットのヘッダーを表す `IPv4Header` 構造体と、
//! バイトストリームとの変換、表示機能を提供する。

use std::fmt::{Display, Formatter};
use crate::byte_stream::ByteStream;
use crate::byte_object::ByteObject;
use crate::ipv4_address::IPv4Address;

/// IPv4 (Internet Protocol version 4) ヘッダーを表す構造体。
///
/// IPv4パケットの様々なフィールドを含む。
pub struct IPv4Header {
    pub version_ihl: u8,
    pub dscp_ecn: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags_fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub header_checksum: u16,
    pub source_address: IPv4Address,
    pub destination_address: IPv4Address,
    // オプションフィールドは今回は考慮しない
}

impl IPv4Header {
    /// IHL (Internet Header Length) を取得する。
    /// IHLはヘッダ長を32ビットワード単位で示す。
    pub fn ihl(&self) -> usize {
        (self.version_ihl & 0x0F) as usize
    }

    /// ヘッダ長をバイト単位で取得する。
    pub fn header_length(&self) -> usize {
        self.ihl() * 4
    }

    /// バージョンを取得する。
    pub fn version(&self) -> u8 {
        (self.version_ihl & 0xF0) >> 4
    }
}

impl ByteObject for IPv4Header {
    /// バイトストリームから `IPv4Header` を生成する。
    ///
    /// # 引数
    /// * `stream` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `IPv4Header`。
    fn from_bytes(stream: &mut ByteStream) -> Self {
        let version_ihl = stream.pop(1)[0];
        let dscp_ecn = stream.pop(1)[0];
        let total_length = u16::from_be_bytes(stream.pop(2).try_into().unwrap());
        let identification = u16::from_be_bytes(stream.pop(2).try_into().unwrap());
        let flags_fragment_offset = u16::from_be_bytes(stream.pop(2).try_into().unwrap());
        let ttl = stream.pop(1)[0];
        let protocol = stream.pop(1)[0];
        let header_checksum = u16::from_be_bytes(stream.pop(2).try_into().unwrap());
        let source_address = IPv4Address::from_bytes(stream);
        let destination_address = IPv4Address::from_bytes(stream);

        // オプションフィールドのスキップ
        let header_len_bytes = (version_ihl & 0x0F) as usize * 4;
        if header_len_bytes > 20 {
            stream.pop(header_len_bytes - 20);
        }

        IPv4Header {
            version_ihl,
            dscp_ecn,
            total_length,
            identification,
            flags_fragment_offset,
            ttl,
            protocol,
            header_checksum,
            source_address,
            destination_address,
        }
    }

    /// `IPv4Header` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数。
    fn append_to(&self, dst: &mut ByteStream) -> usize {
        let mut total_len = 0;
        total_len += dst.append(&[self.version_ihl]);
        total_len += dst.append(&[self.dscp_ecn]);
        total_len += dst.append(&self.total_length.to_be_bytes());
        total_len += dst.append(&self.identification.to_be_bytes());
        total_len += dst.append(&self.flags_fragment_offset.to_be_bytes());
        total_len += dst.append(&[self.ttl]);
        total_len += dst.append(&[self.protocol]);
        total_len += dst.append(&self.header_checksum.to_be_bytes());
        total_len += self.source_address.append_to(dst);
        total_len += self.destination_address.append_to(dst);
        total_len
    }
}

impl Display for IPv4Header {
    /// `IPv4Header` を人間が読める形式でフォーマットする。
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "IPv4 {{ Version: {}, IHL: {}, Total Length: {}, ID: {}, Flags: {:b}, Fragment Offset: {}, TTL: {}, Protocol: {}, Checksum: {}, Src: {}, Dst: {} }}",
            self.version(),
            self.ihl(),
            self.total_length,
            self.identification,
            (self.flags_fragment_offset & 0xE000) >> 13, // 3 bits
            self.flags_fragment_offset & 0x1FFF, // 13 bits
            self.ttl,
            self.protocol,
            self.header_checksum,
            self.source_address,
            self.destination_address
        )
    }
}