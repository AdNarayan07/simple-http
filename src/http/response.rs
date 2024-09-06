use super::request::{HttpRequest, Method, Version};
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
        let content_length: usize;
        let mut content_type: String = String::from("text/html");
        let mut accept_ranges: AcceptRanges = AcceptRanges::None;
        let mut response_body: Vec<u8> = Vec::new();

        let server_root_path = current_dir()?.join("serve-dir");
        let mut resource = String::new();
        url_escape::decode_to_string(request.resource.path.clone(), &mut resource);
        resource = resource
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();
        let new_path = server_root_path.join(&resource);
        let method = request.method;
        let four_o_four = b"
        <html>
        <body>
        <h1>404 NOT FOUND</h1>
        </body>
        </html>"
            .to_vec();

        match method {
            Method::Get => {
                if new_path.exists() {
                    let server_root = server_root_path.canonicalize()?;
                    let new_path = new_path.canonicalize()?;
                    let allowed = new_path.starts_with(server_root);

                    if allowed
                    {
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
                            let mut content = format!(
                                "
                        <html>
                        <head><title>Directory Listing</title></head>
                        <body>
                        <h1><a href=\"../{}\"><button>Back</button></a>Directory Listing</h1>
                        <ul>
                        ",
                                prev_path
                            )
                            .into_bytes();

                            if let Ok(entries) = fs::read_dir(new_path) {
                                for entry in entries {
                                    if let Ok(entry) = entry {
                                        let path = entry.path();
                                        let file_name =
                                            path.file_name().unwrap_or_default().to_string_lossy();

                                        let resource_path = if resource.is_empty() {
                                            file_name.to_string()
                                        } else {
                                            format!("/{}/{}", resource, file_name)
                                        };

                                        content.extend_from_slice(
                                            &format!(
                                                r#"<li><a href="{}">{}</a></li>"#,
                                                &resource_path, &file_name
                                            )
                                            .as_bytes(),
                                        );
                                    }
                                }
                            }
                            content.extend_from_slice(
                                b"
                        </ul>
                        </body>
                        </html>
                        ",
                            );
                            content_length = content.len();
                            status = ResponseStatus::OK;
                            accept_ranges = AcceptRanges::Bytes;
                            response_body.extend_from_slice(&content);
                        } else {
                            println!("Not a file or directory");
                            content_length = four_o_four.len();
                            response_body.extend_from_slice(&four_o_four);
                        }
                    } else {
                        status = ResponseStatus::Forbidden;
                        let four_o_one = b"
                            <html>
                            <body>
                            <h1>403 Forbidden</h1>
                            </body>
                            </html>".to_vec();
                        content_length = four_o_one.len();
                        response_body.extend_from_slice(&four_o_one)
                    }
                } else {
                    println!("Path \"{:?}\" does not exist", new_path);
                    content_length = four_o_four.len();
                    response_body.extend_from_slice(&four_o_four);
                }
            }
            _ => {
                println!("Invalid request type");
                content_length = four_o_four.len();
                response_body.extend_from_slice(&four_o_four)
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
