use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    /// HTTPリクエストの生の文字列から HttpRequest を解析する
    pub fn parse(raw_request: &str) -> Result<Self, &'static str> {
        let lines: Vec<&str> = raw_request.lines().collect();
        
        if lines.is_empty() {
            return Err("Empty request");
        }

        // リクエストラインの解析 (例: "GET /path HTTP/1.1")
        let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line_parts.len() != 3 {
            return Err("Invalid request line");
        }

        let method = request_line_parts[0].to_string();
        let path = request_line_parts[1].to_string();
        let version = request_line_parts[2].to_string();

        // ヘッダーの解析
        let mut headers = HashMap::new();
        let mut body_start = 1;
        
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.is_empty() {
                body_start = i + 1;
                break;
            }
            
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        // ボディの解析
        let body = if body_start < lines.len() {
            lines[body_start..].join("\n")
        } else {
            String::new()
        };

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    /// パス情報を取得（クエリパラメータを除く）
    pub fn get_path(&self) -> String {
        if let Some(question_pos) = self.path.find('?') {
            self.path[..question_pos].to_string()
        } else {
            self.path.clone()
        }
    }
}
