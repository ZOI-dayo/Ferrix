//! イーサネットヘッダーを扱うモジュール。
//!
//! イーサネットフレームのヘッダーを表す
//! `EthernetHeader` 構造体と、そのバイトストリームとの変換、表示機能を提供する。

use std::fmt::{Display, Formatter};
use crate::bit_stream::BitStream;
use crate::byte_object::ByteObject;
use crate::mac_address::MacAddress;
use crate::ether_type::EtherType;

/// イーサネットフレームのヘッダーを表す構造体。
///
/// 宛先MACアドレス、送信元MACアドレス、およびイーサネットタイプを含む。
pub struct EthernetHeader {
    pub destination: MacAddress,
    pub source: MacAddress,
    pub ether_type: EtherType,
}

impl ByteObject for EthernetHeader {
    /// バイトストリームから `EthernetHeader` を生成する。
    ///
    /// # 引数
    /// * `stream` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `EthernetHeader`。
    fn from_bytes(stream: &mut BitStream) -> EthernetHeader {
        EthernetHeader {
            destination: MacAddress::from_bytes(stream),
            source: MacAddress::from_bytes(stream),
            ether_type: EtherType::from_bytes(stream),
        }
    }

    /// `EthernetHeader` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数。
    fn append_to(&self, dst: &mut BitStream) -> usize {
        self.destination.append_to(dst)
            + self.source.append_to(dst)
            + self.ether_type.append_to(dst)
    }
}

impl Display for EthernetHeader {
    /// `EthernetHeader` を人間が読める形式でフォーマットする。
    /// 例: `Ethernet { dest: mac(...), src: mac(...), type: IPv4 }`
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Ethernet {{ dest: {}, src: {}, type: {} }}",
            self.destination, self.source, self.ether_type
        )
    }
}