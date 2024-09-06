use std::{ collections::HashMap, fmt::{self, Display, Formatter}, io, result::Result, str::FromStr };
use super::response::HttpResponse;

#[derive(Debug)]
pub struct  HttpRequest {
    pub method: Method,
    pub resource: Resource,
    pub version: Version,
    pub headers: HttpHeader,
    pub request_body: String
}

impl HttpRequest {
    pub fn response(&self) -> io::Result<HttpResponse> {
        HttpResponse::new(self)
    }
    pub fn new(request: &str) -> io::Result<HttpRequest> {
        let method: Method = Method::new(request);
        let resource: Resource = if let Some(resource) = Resource::new(request) {
            resource
        } else {
            Resource { path: "".to_string() }
        };
        let version: Version = Version::new(request).map_err(|err|{
            io::Error::new(io::ErrorKind::InvalidData, err.msg)
        })?;
        let headers: HttpHeader = if let Some(headers) = HttpHeader::new(request) {
            headers
        } else {
            HttpHeader {
                headers: HashMap::new()
            }
        };
        let request_body: String = if let Some((_header, body)) = request.split_once("\r\n\r\n"){
            body.to_string()
        } else {
            String::new()
        };

        Ok(HttpRequest{
            method,
            resource,
            version,
            headers,
            request_body
        })
    }

}
#[derive(Debug)]
pub struct HttpHeader {
    pub headers: HashMap<String, String>
}

impl HttpHeader {
        pub fn new(request: &str) -> Option<HttpHeader> {
            let mut http_header = HttpHeader {
                headers: HashMap::new()
            };
            let (_, header_str) = request.split_once("\r\n")?;

            for split in header_str.split_terminator("\r\n") {
                if split.is_empty() {
                    break;
                }
                let (header, value) = split.split_once(".")?;
                http_header.headers.insert(header.trim().to_string(), value.trim().to_string());
            }
            Some(http_header)
        } 
}

#[derive(Debug)]
pub enum Version {
    V1_1,
    V2_0
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Version::V1_1 => "HTTP/1.1",
            Version::V2_0 => "HTTP/2" 
        };
        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
pub struct VersionError {
    msg: String
}

impl Display for VersionError {
    fn  fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Version {
    pub fn new(request: &str) -> Result<Self, VersionError> {
        Version::from_str(request)
    }
}
impl FromStr for Version {
    type Err = VersionError;
    fn from_str(request: &str) -> Result<Self, Self::Err> {
        let request_split = request.split_once("\r\n");
        if let Some((method_line, _rest)) = request_split {
            let splits = method_line.split_ascii_whitespace();
            for split in splits {
                if split == "HTTP/1.1" {
                    return  Ok(Version::V1_1);
                } else if split == "HTTP/2" || split == "HTTP/2.0" {
                    return Ok(Version::V2_0);
                }
            }
        }
        let invalid = format!("Unknown version is {}", request);
        let vesion_error = VersionError {
            msg: invalid
        };
        Err(vesion_error)
    }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum Method {
    Get,
    Post,
    Unintialised
}

impl Method {
    pub fn new(request: &str) ->  Method {
        let request_split = request.split_once("\r\n");
        if let Some((method_line, _rest)) = request_split {
            let method_line = method_line.split_once(' ');
            if let Some((method, _rest)) = method_line {
                return match method {
                    "GET" => Method::Get,
                    "POST" => Method::Post,
                    _ => Method::Unintialised
                };
            }
        }
        Method::Unintialised
    }

    pub fn identify(s: &str) -> Method {
        match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Unintialised
        }
    }
}

#[derive(Debug)]
pub struct Resource {
    pub path: String
}

impl  Resource {
    pub fn new(request: &str) -> Option<Resource> {
        if let Some((request_method, _)) = request.split_once("\r\n") {
            let (method, rest) = request_method.split_once(' ')?;
            return match Method::identify(method) {
                Method::Get | Method::Post => {
                    let (resource, _protocol_version) = rest.split_once(' ')?;
                    let resource = resource.trim();
                    let resource = resource.trim_start_matches('/');
                    return Some(Resource {
                        path: resource.to_string()
                    })
                }
                Method::Unintialised => None
            };
        }
        None
    }
}