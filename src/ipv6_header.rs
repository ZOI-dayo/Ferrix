//! IPv6ヘッダーを扱うモジュール。
//!
//! IPv6パケットのヘッダーを表す `IPv6Header` 構造体と、
//! バイトストリームとの変換、表示機能を提供する。

use std::fmt::{Display, Formatter};
use crate::bit_stream::BitStream;
use crate::byte_object::ByteObject;
use crate::ipv6_address::IPv6Address;

/// IPv6 (Internet Protocol version 6) ヘッダーを表す構造体。
///
/// IPv6パケットの様々なフィールドを含む。
pub struct IPv6Header {
    pub version_traffic_class_flow_label: u32,
    pub payload_length: u16,
    pub next_header: u8,
    pub hop_limit: u8,
    pub source_address: IPv6Address,
    pub destination_address: IPv6Address,
}

impl IPv6Header {
    /// バージョンを取得する。
    pub fn version(&self) -> u8 {
        ((self.version_traffic_class_flow_label >> 28) & 0xF) as u8
    }

    /// トラフィッククラスを取得する。
    pub fn traffic_class(&self) -> u8 {
        ((self.version_traffic_class_flow_label >> 20) & 0xFF) as u8
    }

    /// フローラベルを取得する。
    pub fn flow_label(&self) -> u32 {
        self.version_traffic_class_flow_label & 0xFFFFF
    }
}

impl ByteObject for IPv6Header {
    /// バイトストリームから `IPv6Header` を生成する。
    ///
    /// # 引数
    /// * `stream` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `IPv6Header`。
    fn from_bytes(stream: &mut BitStream) -> Self {
        let version_traffic_class_flow_label = u32::from_be_bytes(stream.pop(4).try_into().unwrap());
        let payload_length = u16::from_be_bytes(stream.pop(2).try_into().unwrap());
        let next_header = stream.pop(1)[0];
        let hop_limit = stream.pop(1)[0];
        let source_address = IPv6Address::from_bytes(stream);
        let destination_address = IPv6Address::from_bytes(stream);

        IPv6Header {
            version_traffic_class_flow_label,
            payload_length,
            next_header,
            hop_limit,
            source_address,
            destination_address,
        }
    }

    /// `IPv6Header` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数。
    fn append_to(&self, dst: &mut BitStream) -> usize {
        let mut total_len = 0;
        total_len += dst.append(&self.version_traffic_class_flow_label.to_be_bytes());
        total_len += dst.append(&self.payload_length.to_be_bytes());
        total_len += dst.append(&[self.next_header]);
        total_len += dst.append(&[self.hop_limit]);
        total_len += self.source_address.append_to(dst);
        total_len += self.destination_address.append_to(dst);
        total_len
    }
}

impl Display for IPv6Header {
    /// `IPv6Header` を人間が読める形式でフォーマットする。
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "IPv6 {{ Version: {}, Traffic Class: {}, Flow Label: {}, Payload Length: {}, Next Header: {}, Hop Limit: {}, Src: {}, Dst: {} }}",
            self.version(),
            self.traffic_class(),
            self.flow_label(),
            self.payload_length,
            self.next_header,
            self.hop_limit,
            self.source_address,
            self.destination_address
        )
    }
}