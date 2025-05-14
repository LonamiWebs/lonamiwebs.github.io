use super::{EOH, EOL, Status, VERSION};
use std::io::Write as _;

pub struct Response {
    pub status: Status,
    pub body: Vec<u8>,
}

impl Response {
    pub fn from_status(status: Status) -> Self {
        Self {
            status,
            body: Vec::new(),
        }
    }

    pub fn closes(&self) -> bool {
        !self.status.is_success() && !matches!(self.status, Status::NotFound)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(128 + self.body.len());

        result.extend_from_slice(VERSION);
        write!(result, " {} ", self.status.code()).unwrap();
        result.extend_from_slice(self.status.phrase().as_bytes());

        result.extend_from_slice(EOL);
        write!(result, "Content-Length: {}", self.body.len()).unwrap();

        if self.closes() {
            result.extend_from_slice(EOL);
            result.extend_from_slice(b"Connection: close");
        }

        result.extend_from_slice(EOH);
        result.extend_from_slice(&self.body);

        result
    }
}
