//! バイトストリームを扱うユーティリティ。
//!
//! バイト配列からのデータの読み書きを効率的に行う `ByteStream` 構造体を提供する。

/// バイト配列からの読み書きを管理するストリーム。
///
/// `data` フィールドはバイトデータを保持し、`pos` フィールドは
/// 現在の読み取り位置を示す。
pub struct ByteStream {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl ByteStream {
    /// ストリームから指定された長さのバイトを読み取り、スライスとして返す。
    ///
    /// 読み取り位置 (`pos`) は読み取られたバイト数だけ進む。
    ///
    /// # 引数
    /// * `len` - 読み取るバイト数。
    ///
    /// # 戻り値
    /// 読み取られたバイトのスライス。
    ///
    /// # パニック
    /// ストリームの終端を超えて読み取ろうとした場合、パニックする。
    pub fn pop(&mut self, len: usize) -> &[u8] {
        let res = &self.data[self.pos..(self.pos + len)];
        self.pos += len;
        res
    }

    /// ストリームの末尾にバイトデータを追加する。
    ///
    /// # 引数
    /// * `data` - 追加するバイトデータのスライス。
    ///
    /// # 戻り値
    /// データ追加後のストリームの全長。
    pub fn append(&mut self, data: &[u8]) -> usize {
        self.data.extend_from_slice(data);
        self.data.len()
    }
}