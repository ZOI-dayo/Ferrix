//! IPv4アドレスを扱うモジュール。
//!
//! IPv4アドレスを表す `IPv4Address` 構造体と、
//! そのバイトストリームとの変換、表示機能を提供する。

use std::fmt::{Display, Formatter};
use crate::byte_stream::ByteStream;
use crate::byte_object::ByteObject;

/// IPv4アドレスを表す構造体。
///
/// 4バイトの配列でIPv4アドレスを保持する。
pub struct IPv4Address {
    pub address: [u8; 4],
}

impl ByteObject for IPv4Address {
    /// バイトストリームから `IPv4Address` を生成する。
    ///
    /// # 引数
    /// * `stream` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `IPv4Address`。
    fn from_bytes(stream: &mut ByteStream) -> Self {
        IPv4Address {
            address: stream.pop(4).try_into().unwrap(),
        }
    }

    /// `IPv4Address` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数 (常に4)。
    fn append_to(&self, dst: &mut ByteStream) -> usize {
        dst.append(&self.address);
        4
    }
}

impl Display for IPv4Address {
    /// `IPv4Address` を人間が読める形式でフォーマットする。
    /// 例: `ipv4(192.168.1.1)`
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ipv4({}.{}.{}.{})",
            self.address[0], self.address[1], self.address[2], self.address[3]
        )
    }
}