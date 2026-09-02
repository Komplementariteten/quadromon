use std::fs::{File, remove_file};
use std::net::Shutdown;
use std::os::unix::net::UnixDatagram;

fn main() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let sock1 = temp_dir.join("sock1");
    let sock2 = temp_dir.join("sock2");
    println!("sock1: {:?}", sock1);
    /* if !sock1.exists() {
        let _ = File::create(&sock1)?;
    }
    if !sock2.exists() {
        let _ = File::create(&sock2)?;
    } */

    let socket = UnixDatagram::bind(&sock1)?;
    let client = match UnixDatagram::bind(&sock2) {
        Ok(c) => {
            c.connect(&sock1).expect("Connect failed");
            c
        }
        Err(e) => return Err(e),
    };
    socket.send_to(b"hello world", &sock1)?;
    let mut buf = [0; 100];
    let count = client.recv(&mut buf)?;
    println!("socket sent {:?}", &buf[..count]);
    socket.shutdown(Shutdown::Both)?;
    remove_file(&sock1)?;
    remove_file(&sock2)?;
    Ok(())
}
