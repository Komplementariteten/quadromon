use crate::consts::{CLIENT_SOCKET, SERVER_SOCKET, SRV_PORT};
use crate::shared::file;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::os::unix::net::{UnixDatagram};
use std::path::PathBuf;
use std::time::Duration;

pub fn srv_socket_file() -> PathBuf {
    let path = file::base_path();
    path.join(SERVER_SOCKET)
}

fn cln_socket_file() -> PathBuf {
    let path = file::base_path();
    path.join(CLIENT_SOCKET)
}

pub fn bind_socket_listener() -> TcpListener {
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", SRV_PORT)) {
        Ok(listener) => listener,
        Err(e) => panic!("failed to create socket: {}", e),
    };
    listener
}

pub fn bind_socket_dgram() -> UnixDatagram {
    let p = srv_socket_file();

    match UnixDatagram::bind(&p) {
        Ok(s) => {
            // s.set_nonblocking(true).expect("Failed to set nonblocking");
            s
        }
        Err(e) => panic!("failed to create socket: {}", e),
    }
}

pub fn connect_socket(wait: Option<Duration>) -> TcpStream {
    if let Some(timeout) = wait {
        let client = match TcpStream::connect_timeout(
            &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), SRV_PORT),
            timeout,
        ) {
            Ok(c) => c,
            Err(e) => panic!("failed to connect socket: {}", e),
        };
        client
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");
        client
    } else {
        let client = match TcpStream::connect(format!("127.0.0.1:{}", SRV_PORT)) {
            Ok(c) => c,
            Err(e) => panic!("failed to connect socket: {}", e),
        };
        client
            .set_nonblocking(true)
            .expect("Failed to set nonblocking");
        client
    }
}
pub fn connect_socket_dgram(wait: Option<Duration>) -> UnixDatagram {
    let p = cln_socket_file();
    let s = srv_socket_file();
    let client = UnixDatagram::bind(p).expect("failed to create socket");
    client.connect(&s).expect("failed to connect socket");
    // client.set_nonblocking(true).expect("Failed to set nonblocking");
    client
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("Failed to set read timeout");
    client
}
