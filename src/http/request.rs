use super::{EOH, EOL, Status, VERSION};

static REQUEST_LINE_GET_SPACE: &[u8] = b"GET ";

pub struct Request {
    pub target: String,
    pub body: Vec<u8>,
}

pub enum ParsedRequest {
    Ok { request: Request, consumed: usize },
    Err(Status),
    TooShort,
}

fn split_request_line(buffer: &[u8]) -> (&[u8], &[u8]) {
    match buffer.windows(EOL.len()).position(|window| window == EOL) {
        Some(i) => (&buffer[..i], &buffer[i + 2..]),
        None => (buffer, &buffer[buffer.len()..]),
    }
}

impl Request {
    pub fn from_buffer(mut buffer: &[u8]) -> ParsedRequest {
        let eoh = match buffer.windows(EOH.len()).position(|window| window == EOH) {
            Some(i) => i + EOH.len(),
            None => return ParsedRequest::TooShort,
        };
        buffer = &buffer[..eoh];

        let request_line;
        (request_line, buffer) = split_request_line(buffer);

        if !request_line.ends_with(VERSION)
            || request_line
                .get(request_line.len().saturating_sub(VERSION.len() + 1))
                .is_none_or(|&c| c != b' ')
        {
            return ParsedRequest::Err(Status::HttpVersionNotSupported);
        }
        if !request_line.starts_with(REQUEST_LINE_GET_SPACE) {
            return ParsedRequest::Err(Status::MethodNotAllowed);
        }

        let request_target = &request_line
            [REQUEST_LINE_GET_SPACE.len()..request_line.len() - VERSION.len() - 1]
            .trim_ascii_end();

        if request_target.is_empty() {
            return ParsedRequest::Err(Status::BadRequest);
        }

        let request_body = loop {
            let header;
            (header, buffer) = split_request_line(buffer);
            if header.is_empty() {
                break buffer;
            }
        };

        ParsedRequest::Ok {
            request: Request {
                target: String::from_utf8_lossy(request_target).into_owned(),
                body: request_body.to_vec(),
            },
            consumed: eoh,
        }
    }
}
