use std::io::{Read, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::{fs, io, thread};

use super::{ParsedRequest, Request, Response, Status};
use crate::conf;

const REQUEST_HEADERS_MAX_SIZE: usize = 1024;

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.expect("incomming connection to be alive");
        thread::spawn(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut read = 0;
    let mut buffer = vec![0; REQUEST_HEADERS_MAX_SIZE];

    loop {
        let response = match Request::from_buffer(&buffer[..read]) {
            ParsedRequest::Ok { request, consumed } => {
                buffer.copy_within(consumed..read, 0);
                read -= consumed;
                handle_request(request)
            }
            ParsedRequest::Err(status) => Response::from_status(status),
            ParsedRequest::TooShort if read == REQUEST_HEADERS_MAX_SIZE => {
                Response::from_status(Status::RequestHeaderFieldsTooLarge)
            }
            ParsedRequest::TooShort => match stream.read(&mut buffer[read..]) {
                Ok(0) => return,
                Ok(n) => {
                    read += n;
                    continue;
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                    Response::from_status(Status::RequestTimeout)
                }
                Err(_) => Response::from_status(Status::BadRequest),
            },
        };

        match stream.write_all(&response.serialize()) {
            Ok(_) => {
                if response.closes() {
                    return;
                }
            }
            Err(e) => {
                if !matches!(
                    e.kind(),
                    io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted
                ) {
                    eprintln!(
                        "unexpected failure while writing response: {e} {}",
                        e.kind()
                    );
                }
                return;
            }
        }
    }
}

fn handle_request(request: Request) -> Response {
    let root = match PathBuf::from(conf::OUTPUT_FOLDER).canonicalize() {
        Ok(root) => root,
        Err(_) => return Response::from_status(Status::NotFound),
    };

    let path = match root.join(&request.target[1..]).canonicalize() {
        Ok(x) if x.starts_with(&root) => x,
        _ => return Response::from_status(Status::NotFound),
    };

    let contents = match fs::read(&path) {
        Ok(x) => x,
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            // Probably tried to read a directory as a file
            match fs::read(path.join("index.html")) {
                Ok(x) => x,
                Err(e) => return response_from_error_kind(e.kind()),
            }
        }
        Err(e) => return response_from_error_kind(e.kind()),
    };

    Response {
        status: Status::Ok,
        content_type: match path.extension().and_then(|e| e.to_str()) {
            Some("css") => "text/css",
            Some("html") => "text/html",
            Some("ico") => "image/x-icon",
            Some("js") => "text/javascript",
            Some("png") => "image/png",
            Some("svg") => "image/svg+xml",
            _ => "",
        },
        body: contents,
    }
}

fn response_from_error_kind(error_kind: io::ErrorKind) -> Response {
    match error_kind {
        io::ErrorKind::NotFound => Response::from_status(Status::NotFound),
        _ => Response::from_status(Status::ServiceUnavailable),
    }
}
