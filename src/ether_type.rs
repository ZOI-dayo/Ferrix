//! イーサネットタイプを定義するモジュール。
//!
//! イーサネットフレームのペイロードタイプを識別する
//! `EtherType` 列挙型と、そのバイトストリームとの変換、表示機能を提供する。

use std::fmt::{Display, Formatter};
use crate::bit_stream::BitStream;
use crate::byte_object::ByteObject;

/// イーサネットフレームのペイロードタイプを表す列挙型。
///
/// サポートされるタイプは IPv4, ARP, IPv6 である。
#[derive(Debug)]
pub enum EtherType {
    IPv4,
    ARP,
    IPv6,
}

impl ByteObject for EtherType {
    /// バイトストリームから `EtherType` を生成する。
    ///
    /// # 引数
    /// * `stream` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `EtherType`。
    ///
    /// # パニック
    /// 未知のイーサネットタイプが検出された場合、パニックします。
    fn from_bytes(stream: &mut BitStream) -> EtherType {
        let raw = stream.pop(2);
        match raw {
            [0x08, 0x00] => EtherType::IPv4,
            [0x08, 0x06] => EtherType::ARP,
            [0x86, 0xdd] => EtherType::IPv6,
            _ => panic!("raw.len = {}, raw 1 = {}, raw 2 = {}", raw.len(), raw[0], raw[1]),
        }
    }

    /// `EtherType` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数。
    fn append_to(&self, dst: &mut BitStream) -> usize {
        let content: &[u8] = match self {
            EtherType::IPv4 => &[0x08, 0x00],
            EtherType::ARP => &[0x08, 0x06],
            EtherType::IPv6 => &[0x86, 0xdd],
        };
        dst.append(content);
        content.len()
    }
}

impl Display for EtherType {
    /// `EtherType` を人間が読める形式でフォーマットする。
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EtherType::IPv4 => "IPv4",
                EtherType::ARP => "ARP",
                EtherType::IPv6 => "IPv6",
            }
        )
    }
}