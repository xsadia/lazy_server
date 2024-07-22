use core::panic;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::Command,
};

use serde::{Deserialize, Serialize};
use serde_json;

use sysinfo;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
enum PayloadType {
    Open,
    ShutDown,
    Unsupported,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
enum ContentType {
    OperaGx,
    OS,
    Unsupported,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
enum InfoType {
    Delayed,
    Instant,
    Unsupported,
}

/*
    Payload format:
    - 3 bytes of headers: content type (OperaGx and OS to start with)
*/
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct Request {
    payload: String,
    info: InfoType,
    content_type: ContentType,
    content_length: u8,
}

impl From<&Request> for PayloadType {
    fn from(value: &Request) -> Self {
        let split_payload: Vec<&str> = value.payload.split(";").collect();
        match split_payload.get(0).unwrap().to_lowercase().as_str() {
            "open" => Self::Open,
            "off" => Self::ShutDown,
            _ => Self::Unsupported,
        }
    }
}

impl From<u8> for InfoType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Instant,
            0x02 => Self::Delayed,
            _ => Self::Unsupported,
        }
    }
}

impl From<u8> for ContentType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::OperaGx,
            0x02 => Self::OS,
            _ => Self::Unsupported,
        }
    }
}

fn shut_down_computer(delay: Option<i32>) {
    let shut_down_delay = match delay {
        Some(delay) => delay,
        None => 1,
    };
    Command::new("shutdown")
        .arg("/s") // /s option for shutdown
        .arg("/t") // /t option to specify the time delay (in seconds)
        .arg(shut_down_delay.to_string())
        .status()
        .unwrap();
}

fn spawn_process() {
    Command::new("").spawn().unwrap();
}

fn kill_process() {
    let mut system = sysinfo::System::new();
    system.refresh_all();

    for process in system.processes_by_name("opera.exe") {
        process.kill();
    }
}

fn read_request(mut stream: &TcpStream) -> Result<Request, &'static str> {
    let buf = &mut vec![0; 1024];
    stream.read(buf).unwrap();
    let headers = &buf[..3];
    let content_length = headers[2];
    let payload_bytes: Vec<u8> =
        Vec::from(&buf[headers.len()..headers.len() + content_length as usize]);

    let payload = String::from_utf8(payload_bytes).unwrap();

    Ok(Request {
        payload,
        content_length,
        content_type: ContentType::from(headers[0]),
        info: InfoType::from(headers[1]),
    })
}

fn execute_request(request: &Request) {
    match (&request.content_type, PayloadType::from(request)) {
        (ContentType::OperaGx, PayloadType::ShutDown) => kill_process(),
        (ContentType::OperaGx, PayloadType::Open) => spawn_process(),
        (ContentType::OS, PayloadType::ShutDown) => shut_down_computer(None),
        _ => panic!(),
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();
    println!("Server running");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => match read_request(&stream) {
                Ok(request) => {
                    println!("{:?}", serde_json::to_string(&request).unwrap());

                    execute_request(&request);

                    stream
                        .write_all(serde_json::to_string(&request).unwrap().as_bytes())
                        .unwrap();
                }
                Err(e) => {
                    println!("{e}")
                }
            },
            Err(e) => {
                println!("Error: {e}")
            }
        }
    }
}
