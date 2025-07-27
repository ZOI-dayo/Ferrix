use std::fs;
use std::path::{Path, PathBuf};
use crate::http::request::HttpRequest;
use crate::http::response::HttpResponse;

pub struct FileServer {
    root_dir: PathBuf,
}

impl FileServer {
    /// 新しいファイルサーバーを作成
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    /// HTTPリクエストを処理してレスポンスを生成
    pub fn handle_request(&self, request: &HttpRequest) -> HttpResponse {
        // GETメソッドのみサポート
        if request.method != "GET" {
            let mut response = HttpResponse::new(405, "Method Not Allowed");
            response.set_body_text("405 Method Not Allowed");
            response.set_header("Content-Type", "text/plain");
            return response;
        }

        let requested_path = request.get_path();
        let file_path = self.resolve_path(&requested_path);

        match file_path {
            Some(path) => self.serve_file(&path),
            None => HttpResponse::not_found(),
        }
    }

    /// リクエストされたパスを実際のファイルパスに解決
    fn resolve_path(&self, request_path: &str) -> Option<PathBuf> {
        // パスを正規化（先頭の/を除去）
        let clean_path = if request_path.starts_with('/') {
            &request_path[1..]
        } else {
            request_path
        };

        // 空のパスまたは"/"の場合はindex.htmlを試す
        let target_path = if clean_path.is_empty() {
            "index.html"
        } else {
            clean_path
        };

        let full_path = self.root_dir.join(target_path);

        // パストラバーサル攻撃を防ぐため、root_dir内に収まっているかチェック
        if let Ok(canonical_path) = full_path.canonicalize() {
            if let Ok(canonical_root) = self.root_dir.canonicalize() {
                if canonical_path.starts_with(canonical_root) {
                    return Some(canonical_path);
                }
            }
        }

        None
    }

    /// ファイルを読み込んでレスポンスを生成
    fn serve_file(&self, file_path: &Path) -> HttpResponse {
        match fs::read(file_path) {
            Ok(contents) => {
                let mut response = HttpResponse::ok();
                response.set_body_bytes(contents);
                
                // ファイル拡張子に基づいてContent-Typeを設定
                let content_type = self.get_content_type(file_path);
                response.set_header("Content-Type", &content_type);
                
                response
            }
            Err(_) => HttpResponse::not_found(),
        }
    }

    /// ファイル拡張子に基づいてContent-Typeを決定
    fn get_content_type(&self, file_path: &Path) -> String {
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("html") | Some("htm") => "text/html; charset=utf-8".to_string(),
            Some("css") => "text/css".to_string(),
            Some("js") => "application/javascript".to_string(),
            Some("json") => "application/json".to_string(),
            Some("png") => "image/png".to_string(),
            Some("jpg") | Some("jpeg") => "image/jpeg".to_string(),
            Some("gif") => "image/gif".to_string(),
            Some("svg") => "image/svg+xml".to_string(),
            Some("txt") => "text/plain; charset=utf-8".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
}
