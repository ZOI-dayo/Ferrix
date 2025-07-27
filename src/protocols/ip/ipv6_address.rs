//! IPv6アドレスを扱うモジュール。
//!
//! IPv6アドレスを表す `IPv6Address` 構造体と、
//! そのバイトストリームとの変換、表示機能を提供する。

use std::fmt::{Display, Formatter};
use crate::types::bit_stream::BitStream;
use crate::types::byte_object::ByteObject;

/// IPv6アドレスを表す構造体。
///
/// 16バイトの配列でIPv6アドレスを保持する。
pub struct IPv6Address {
    pub address: [u8; 16],
}

impl ByteObject for IPv6Address {
    /// バイトストリームから `IPv6Address` を生成する。
    ///
    /// # 引数
    /// * `stream` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `IPv6Address`。
    fn from_bytes(stream: &mut BitStream) -> Self {
        IPv6Address {
            address: stream.pop(16).try_into().unwrap(),
        }
    }

    /// `IPv6Address` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数 (常に16)。
    fn append_to(&self, dst: &mut BitStream) -> usize {
        dst.append(&self.address);
        16
    }
}

impl Display for IPv6Address {
    /// `IPv6Address` を人間が読める形式でフォーマットする。
    /// 例: `ipv6(fe80::1)`
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ipv6({:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x})",
            self.address[0], self.address[1],
            self.address[2], self.address[3],
            self.address[4], self.address[5],
            self.address[6], self.address[7],
            self.address[8], self.address[9],
            self.address[10], self.address[11],
            self.address[12], self.address[13],
            self.address[14], self.address[15]
        )
    }
}