#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Status {
    Ok = 200,
    BadRequest = 400,
    NotFound = 404,
    MethodNotAllowed = 405,
    RequestTimeout = 408,
    RequestHeaderFieldsTooLarge = 431,
    ServiceUnavailable = 503,
    HttpVersionNotSupported = 505,
}

impl Status {
    pub fn code(&self) -> u16 {
        *self as _
    }

    pub fn phrase(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::BadRequest => "400 Bad Request",
            Self::NotFound => "Not Found",
            Self::MethodNotAllowed => "Method Not Allowed",
            Self::RequestTimeout => "Request Timeout",
            Self::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            Self::ServiceUnavailable => "Service Unavailable",
            Self::HttpVersionNotSupported => "HTTP Version Not Supported",
        }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.code())
    }
}
