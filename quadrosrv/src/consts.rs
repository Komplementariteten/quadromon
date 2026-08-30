use hex_literal::hex;

pub const MAX_RPM_VALUE: f32 = 1.0;

pub const CFG_FILE: &str = "quadro.toml";

pub const HIST_FILE: &str = "cache.bin";

pub const QUADRO_DIR: &str = ".quadro";

pub const DEFAULT_SOCKET: &str = "quadro.sock";

pub const SEPERATOR: [u8; 4] = hex!("8c0ffee1");
