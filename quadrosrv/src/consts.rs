use hex_literal::hex;

pub const CFG_FILE: &str = "quadro.toml";

pub const HIST_FILE: &str = "cache.bin";

pub const QUADRO_DIR: &str = ".quadro";

pub const SERVER_SOCKET: &str = "quadro.sock";

pub const CLIENT_SOCKET: &str = "client.sock";

pub const SRV_PORT: u16 = 12345;

pub const SEPERATOR: [u8; 8] = hex!("8c0ffee48c0ffee4");
