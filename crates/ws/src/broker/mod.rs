pub mod bridge_message;
pub mod manager;
pub mod init_socket;
pub mod connect_now;
pub mod register_client_msg;

pub const CONNECT_TIMEOUT_SECS: u64 = 10;
pub const JOIN_TIMEOUT_SECS: u64 = 10;