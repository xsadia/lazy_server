use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

#[allow(dead_code)]
#[derive(Debug)]
enum PayloadType {
    Open,
    ShutDown,
}

#[allow(dead_code)]
#[derive(Debug)]
enum ContentType {
    OperaGx,
    OS,
}

#[allow(dead_code)]
#[derive(Debug)]
enum InfoType {
    Delayed,
    Instant,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Request {
    payload: String,
    info: u8,
    content_type: u8,
    content_length: u8,
}

fn hex_to_string(buf: Vec<u8>) -> String {
    String::from_utf8(buf).unwrap()
}

fn read_request(mut stream: TcpStream) -> Result<Request, &'static str> {
    let buf = &mut vec![0; 1024];
    stream.read(buf).unwrap();
    let headers = &buf[..3];
    let content_length = headers[2];
    let payload = buf[3..headers.len() + content_length as usize]
        .into_iter()
        .map(|&byte| hex_to_string(vec![byte]))
        .collect::<Vec<String>>()
        .join("");

    Ok(Request {
        payload,
        content_length,
        content_type: headers[0],
        info: headers[1],
    })
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                match read_request(stream) {
                    Ok(request) => {
                        println!("request: {:?}", request)
                    }
                    Err(e) => {
                        println!("{e}")
                    }
                }
                // println!("content: {:?} received", buf);
            }
            Err(e) => {
                println!("Error: {e}")
            }
        }
    }
}
