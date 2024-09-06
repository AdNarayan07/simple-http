use std::{
    io::{ Read, Result, Write}, net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream}, thread
};
use simple_http::http::request;

fn create_socket() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5500)
}

fn handle_client(stream: &mut TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    let buf_str = String::from_utf8_lossy(&buffer);
    let request = request::HttpRequest::new(&buf_str)?;
    let response = request.response()?;

    stream.write(&response.to_buf())?;
    stream.flush()?;
    Ok(())
}

fn serve(socket: SocketAddr) -> Result<()> {
    let listener = TcpListener::bind(socket)?;
    let mut counter = 0;
    for stream in listener.incoming() {
        match thread::spawn(|| handle_client(&mut stream?)).join(){
            Ok(_) => {
                counter += 1;
                println!("Connected stream... {}", counter);
            }
            Err(_) => continue
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let socket = create_socket();
    serve(socket)
}
