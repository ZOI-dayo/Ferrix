//! ARPヘッダーを扱うモジュール。
//!
//! ARPパケットのヘッダーを表す `ArpHeader` 構造体と、
//! バイトストリームとの変換、表示機能を提供する。
//! バイト配列を整数に変換するユーティリティ関数も含む。

use std::fmt::{Display, Formatter};
use crate::byte_stream::ByteStream;
use crate::byte_object::ByteObject;
use crate::ether_type::EtherType;
use crate::mac_address::MacAddress;
use crate::ipv4_address::IPv4Address;

/// バイト配列を `usize` に変換する。
///
/// バイト配列をビッグエンディアンの整数として解釈する。
///
/// # 引数
/// * `raw` - 変換するバイトデータのスライス。
///
/// # 戻り値
/// 変換された `usize` 値。
pub fn bytes_to_int(raw: &[u8]) -> usize {
    let mut res: usize = 0;
    for &x in raw {
        res = (res << 8) | (x as usize);
    }
    res
}

/// ARP (Address Resolution Protocol) ヘッダーを表す構造体。
///
/// ARPパケットの様々なフィールドを含む。
pub struct ArpHeader {
    pub htype: usize,
    pub ptype: EtherType,
    pub hlen: usize,
    pub plen: usize,
    pub oper: usize,
    pub sha: MacAddress,
    pub spa: IPv4Address,
    pub tha: MacAddress,
    pub tpa: IPv4Address,
}

impl ByteObject for ArpHeader {
    /// バイトストリームから `ArpHeader` を生成する。
    ///
    /// # 引数
    /// * `stream` - 読み取り元の `ByteStream`。
    ///
    /// # 戻り値
    /// 生成された `ArpHeader`。
    fn from_bytes(stream: &mut ByteStream) -> Self {
        ArpHeader {
            htype: bytes_to_int(stream.pop(2)),
            ptype: EtherType::from_bytes(stream),
            hlen: bytes_to_int(stream.pop(1)),
            plen: bytes_to_int(stream.pop(1)),
            oper: bytes_to_int(stream.pop(2)),
            sha: MacAddress::from_bytes(stream),
            spa: IPv4Address::from_bytes(stream),
            tha: MacAddress::from_bytes(stream),
            tpa: IPv4Address::from_bytes(stream),
        }
    }

    /// `ArpHeader` をバイトストリームに追加する。
    ///
    /// # 引数
    /// * `dst` - 書き込み先の `ByteStream`。
    ///
    /// # 戻り値
    /// 追加されたバイト数。
    fn append_to(&self, dst: &mut ByteStream) -> usize {
        let mut total_len = 0;
        total_len += dst.append(&self.htype.to_be_bytes()[6..]); // 2バイト
        total_len += self.ptype.append_to(dst);
        total_len += dst.append(&self.hlen.to_be_bytes()[7..]); // 1バイト
        total_len += dst.append(&self.plen.to_be_bytes()[7..]); // 1バイト
        total_len += dst.append(&self.oper.to_be_bytes()[6..]); // 2バイト
        total_len += self.sha.append_to(dst);
        total_len += self.spa.append_to(dst);
        total_len += self.tha.append_to(dst);
        total_len += self.tpa.append_to(dst);
        total_len
    }
}

impl Display for ArpHeader {
    /// `ArpHeader` を人間が読める形式でフォーマットする。
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ARP {{ HTYPE: {}, PTYPE: {}, HLEN: {}, PLEN: {}, OPER: {}, SHA: {}, SPA: {}, THA: {}, TPA: {} }}",
            self.htype,
            self.ptype,
            self.hlen,
            self.plen,
            self.oper,
            self.sha,
            self.spa,
            self.tha,
            self.tpa
        )
    }
}