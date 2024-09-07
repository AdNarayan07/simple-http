use super::{ request::{ HttpRequest, Method, Version }, template::{ generate_html, generate_ul } };
use std::{env::current_dir, fmt::Display, fs, io::Result};
use url_escape;

use mime_guess;

#[derive(Debug)]
pub struct HttpResponse {
    pub version: Version,
    pub status: ResponseStatus,
    pub content_length: usize,
    pub content_type: String,
    pub accept_ranges: AcceptRanges,
    pub response_body: Vec<u8>,
}

impl HttpResponse {
    pub fn to_buf(&self) -> Vec<u8> {
        let status_line = format!("{} {}\r\n", self.version, self.status);
        let headers = format!(
            "content-type: {}\r\ncontent-length: {}\r\n{}\r\n\r\n",
            self.content_type, self.content_length, self.accept_ranges,
        );
        let mut response = Vec::new();
        response.extend_from_slice(status_line.as_bytes());
        response.extend_from_slice(headers.as_bytes());
        response.extend_from_slice(&self.response_body);
        response
    }
    pub fn new(request: &HttpRequest) -> Result<HttpResponse> {
        let version: Version = Version::V1_1;
        let mut status: ResponseStatus = ResponseStatus::NotFound;
        let mut content_length: usize = 0;
        let mut content_type: String = String::from("text/html");
        let mut accept_ranges: AcceptRanges = AcceptRanges::None;
        let mut response_body: Vec<u8> = Vec::new();

        let server_root_path = current_dir()?;
        let mut resource = String::new();
        url_escape::decode_to_string(request.resource.path.clone(), &mut resource);
        resource = resource
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();
        let new_path = server_root_path.join(&resource);
        let method = request.method;
        let mut generate_err_msg = |error| {
            status = error;
            response_body.extend_from_slice(generate_html(status.to_string(), format!("<h1>{status}</h1>")).as_bytes());
            content_length = response_body.len();
        };
        match method {
            Method::Get => {
                if new_path.exists() {
                    let server_root = server_root_path.canonicalize()?;
                    let new_path = new_path.canonicalize()?;
                    let allowed = new_path.starts_with(server_root);

                    if allowed {
                        if new_path.is_file() {
                            let content = fs::read(&new_path)?;
                            content_length = content.len();
                            content_type = mime_guess::from_path(new_path)
                                .first_or_octet_stream()
                                .to_string();
                            status = ResponseStatus::OK;
                            accept_ranges = AcceptRanges::Bytes;
                            response_body.extend_from_slice(&content);
                        } else if new_path.is_dir() {
                            let prev_path = match resource.rfind('/') {
                                // Find the last slash
                                Some(index) => resource[..index].to_string(), // Slice up to the last slash
                                None => "".to_string(), // Return empty if there's no slash
                            };
                            let header = format!(r#"
                            <h2><a href="/{prev_path}"><button>↩📂</button></a><code> ://root/{resource} </code></h2>
                            "#);

                            let mut list_items: Vec<String> = Vec::new();

                            if let Ok(entries) = fs::read_dir(new_path) {
                                let mut number_entries = 0;
                                for entry in entries {
                                    number_entries += 1;
                                    if let Ok(entry) = entry {
                                        let path = entry.path();
                                        let file_name =
                                            path.file_name().unwrap_or_default().to_string_lossy();

                                        let resource_path = if resource.is_empty() {
                                            file_name.to_string()
                                        } else {
                                            format!("/{}/{}", resource, file_name)
                                        };
                                        let mut icon = "";
                                        if path.is_file() {
                                            icon = "📄"
                                        } else if path.is_dir() {
                                            icon = "📁"
                                        }
                                        list_items.push(format!(r#"<a href="{resource_path}"><li><span>{icon}</span><span>{file_name}</span></li></a>"#))
                                    }
                                }
                                if number_entries == 0 {
                                    list_items.push("<h1>Empty Directory</h1>".to_string());
                                };
                                let unordered_list = generate_ul(&list_items);
                                let content = format!("{header}{unordered_list}");
                                status = ResponseStatus::OK;
                                accept_ranges = AcceptRanges::Bytes;
                                let html_file = generate_html(resource, content);
                                content_length = html_file.len();
                                response_body.extend_from_slice(&html_file.as_bytes());
                            } else {
                                println!("Err reading dir");
                                generate_err_msg(ResponseStatus::NotFound);
                            }
                        } else {
                            println!("Not a file or directory");
                            generate_err_msg(ResponseStatus::NotFound);
                        }
                    } else {
                        generate_err_msg(ResponseStatus::Forbidden);
                    }
                } else {
                    generate_err_msg(ResponseStatus::NotFound);
                }
            }
            _ => {
               generate_err_msg(ResponseStatus::NotFound);
            }
        }
        Ok(HttpResponse {
            version,
            status,
            content_length,
            content_type,
            accept_ranges,
            response_body,
        })
    }
}

#[derive(Debug)]
pub enum ResponseStatus {
    OK = 200,
    NotFound = 404,
    Forbidden = 403,
}

impl Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ResponseStatus::OK => "200 OK",
            ResponseStatus::NotFound => "404 NOT FOUND",
            ResponseStatus::Forbidden => "403 FORBIDDEN",
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
pub enum AcceptRanges {
    Bytes,
    None,
}

impl Display for AcceptRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AcceptRanges::Bytes => "accept-ranges: bytes",
            AcceptRanges::None => "accept-ranges: none",
        };
        write!(f, "{}", msg)
    }
}
