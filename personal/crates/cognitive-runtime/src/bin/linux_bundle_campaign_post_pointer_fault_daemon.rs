//! Campaign-only payload for validating post-pointer compensation on Linux.
//!
//! A signed non-production campaign release packages this binary as
//! `bin/kernel-server`. It binds only the fixed Personal loopback health
//! address, returns one contract-valid liveness response, and exits cleanly.
//! The first confirmation therefore succeeds, while the final confirmation
//! after pointer publication deterministically fails without any installer
//! runtime override or production controller change.

use std::io::{Read, Write};
use std::net::TcpListener;

const HEALTH_RESPONSE: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 155\r\nConnection: close\r\n\r\n{\"authority_side_effects\":false,\"profile_claim\":\"not-claimed\",\"readiness_claim\":\"not-claimed\",\"schema_version\":1,\"status\":\"ok\",\"surface\":\"personal-health\"}";

fn main() {
    let bind_address = match parse_fixed_bind_address() {
        Some(address) => address,
        None => std::process::exit(2),
    };
    let listener = match TcpListener::bind(bind_address) {
        Ok(listener) => listener,
        Err(_) => std::process::exit(1),
    };
    let (mut stream, _) = match listener.accept() {
        Ok(connection) => connection,
        Err(_) => std::process::exit(1),
    };
    let mut request_buffer = [0_u8; 1024];
    if stream.read(&mut request_buffer).is_err() || stream.write_all(HEALTH_RESPONSE).is_err() {
        std::process::exit(1);
    }
}

fn parse_fixed_bind_address() -> Option<String> {
    let arguments: Vec<String> = std::env::args().collect();
    let bind_index = arguments.iter().position(|argument| argument == "--bind")?;
    let bind_address = arguments.get(bind_index + 1)?;
    (bind_address == "127.0.0.1:48181").then(|| bind_address.to_owned())
}
