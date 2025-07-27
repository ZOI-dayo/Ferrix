use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// 新しいHTTPレスポンスを作成
    pub fn new(status_code: u16, status_text: &str) -> Self {
        Self {
            status_code,
            status_text: status_text.to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// 200 OKレスポンスを作成
    pub fn ok() -> Self {
        Self::new(200, "OK")
    }

    /// 404 Not Foundレスポンスを作成
    pub fn not_found() -> Self {
        let mut response = Self::new(404, "Not Found");
        response.set_body_text("404 Not Found");
        response.set_header("Content-Type", "text/plain");
        response
    }

    /// 500 Internal Server Errorレスポンスを作成
    pub fn internal_server_error() -> Self {
        let mut response = Self::new(500, "Internal Server Error");
        response.set_body_text("500 Internal Server Error");
        response.set_header("Content-Type", "text/plain");
        response
    }

    /// ヘッダーを設定
    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    /// テキストボディを設定
    pub fn set_body_text(&mut self, text: &str) {
        self.body = text.as_bytes().to_vec();
        self.set_header("Content-Length", &self.body.len().to_string());
    }

    /// バイナリボディを設定
    pub fn set_body_bytes(&mut self, bytes: Vec<u8>) {
        self.body = bytes;
        self.set_header("Content-Length", &self.body.len().to_string());
    }

    /// HTTPレスポンスを文字列として生成
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text);
        
        // ヘッダーを追加
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        // Connection: close を強制的に追加（簡易実装のため）
        response.push_str("Connection: close\r\n");
        
        // 空行でヘッダー終了
        response.push_str("\r\n");
        
        // レスポンスの文字列部分をバイトに変換
        let mut result = response.into_bytes();
        
        // ボディを追加
        result.extend_from_slice(&self.body);
        
        result
    }
}
