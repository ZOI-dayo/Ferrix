//! MACアドレスを扱うモジュール。
//!
//! MACアドレスを表す `MacAddress` 構造体と、
//! そのバイトストリームとの変換、表示機能を提供する。

use std::fmt::{Display, Formatter};
use crate::byte_stream::ByteStream;
use crate::byte_object::ByteObject;

/// MACアドレスを表す構造体。
///
/// 6バイトの配列でMACアドレスを保持する。
#[derive(Clone)]
pub struct MacAddress {
    pub address: [u8; 6],
}

impl ByteObject for MacAddress {
    /// バイトストリームから `MacAddress` を生成する。
    ///
    /// # 引数
    /// * `src` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `MacAddress`。
    fn from_bytes(src: &mut ByteStream) -> Self {
        MacAddress {
            address: src.pop(6).try_into().unwrap(),
        }
    }

    /// `MacAddress` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数 (常に6)。
    fn append_to(&self, dst: &mut ByteStream) -> usize {
        dst.append(&self.address);
        6
    }
}

impl Display for MacAddress {
    /// `MacAddress` を人間が読める形式でフォーマットする。
    /// 例: `mac(00:11:22:33:44:55)`
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "mac({:02x?}:{:02x?}:{:02x?}:{:02x?}:{:02x?}:{:02x?})",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5]
        )
    }
}