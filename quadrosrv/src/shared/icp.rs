use std::fs::File;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use crate::consts::DEFAULT_SOCKET;
use crate::shared::file;

pub fn local_socket() -> PathBuf {
    let path = file::base_path();
    let fs = path.join(DEFAULT_SOCKET);
    if !path.exists() {
        File::create(&fs).unwrap();
    }
    fs
}

pub fn bind_socket() -> UnixStream {
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

pub fn connect_socket() -> UnixStream {
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
