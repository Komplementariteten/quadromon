use std::fs::File;
use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use crate::consts::DEFAULT_SOCKET;
use crate::shared::file;

pub fn local_socket() -> PathBuf {
    let path = file::base_path();
    let fs = path.join(DEFAULT_SOCKET);
    fs
}

pub fn bind_socket_sream() -> UnixStream {
    let p = local_socket();

    match UnixStream::connect(&p) {
        Ok(s) => s,
        Err(_) => {
            let listener = match UnixListener::bind(&p) {
                Ok(listener) => listener,
                Err(e) => panic!("failed to create socket: {}", e),
            };
            let addr = listener.local_addr().unwrap();
            UnixStream::connect_addr(&addr)
                .unwrap_or_else(|e| panic!("Could not connect to socket with error: {}", e))
        }
    }
}

pub fn bind_socket() -> UnixDatagram {
    let p = local_socket();

    match UnixDatagram::bind(&p) {
        Ok(s) => {
            // s.set_nonblocking(true).expect("Failed to set nonblocking");
            s
        }
        Err(e) => panic!("failed to create socket: {}", e),
    }
}


pub fn connect_socket(wait: Option<Duration>) -> UnixDatagram {
    let p = local_socket();
    if !p.exists() && let Some(timeout) = wait {
        thread::sleep(timeout);
    }
    if !p.exists() {
        panic!("Socket does not exist to connect to");
    }
    
    let client = UnixDatagram::unbound().expect("failed to create socket");
    client.connect(&p).expect("failed to connect socket");
    client.set_nonblocking(true).expect("Failed to set nonblocking");
    client.set_read_timeout(Some(Duration::from_millis(100))).expect("Failed to set read timeout");
    client
}
