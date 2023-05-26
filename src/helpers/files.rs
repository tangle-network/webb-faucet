use hex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

fn convert_hex_to_bytes(hex_string: &str) -> Result<[u8; 20], &'static str> {
    // Remove the "0x" prefix
    let stripped = hex_string.strip_prefix("0x").unwrap_or(hex_string);

    // Convert hex to bytes
    let vec = hex::decode(stripped).map_err(|_| "Failed to decode hex string")?;

    // Ensure the byte array is the correct length
    if vec.len() != 20 {
        return Err("Hex string length is not 20 bytes");
    }

    // Convert Vec<u8> to [u8; 20]
    let mut array = [0u8; 20];
    array.copy_from_slice(&vec[..]);
    Ok(array)
}

#[macro_export]
macro_rules! read_json {
    ($file:expr, $type:ty) => {{
        let file = File::open($file).expect("file should open read only");
        let reader = BufReader::new(file);
        let map: $type = serde_json::from_reader(reader).expect("JSON was not well-formatted");
        map
    }};
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct RpcData {
    chainId: u32,
    rpc: Vec<String>,
}

pub fn read_rpc_data() -> HashMap<u32, Vec<String>> {
    read_json!("chainIdRpcs.json", HashMap<u32, Vec<String>>)
}

pub fn read_token_address_data() -> HashMap<u32, String> {
    read_json!("chainIdFaucetToken.json", HashMap<u32, String>)
}

pub fn get_rpc_url(chain_id: u32) -> String {
    let rpc_map = read_rpc_data();
    let rpc_data = rpc_map
        .get(&chain_id)
        .unwrap_or(&vec!["http://localhost:8545".to_string()]);
    rpc_data[0].clone()
}

pub fn get_token_address(chain_id: u32) -> [u8; 20] {
    let token_map = read_token_address_data();
    let token_data = token_map
        .get(&chain_id)
        .unwrap_or(&"0x00000000000000000000000000000000".to_string());
    convert_hex_to_bytes(token_data).unwrap_or([0u8; 20])
}
