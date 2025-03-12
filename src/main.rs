#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use secp256k1::{Secp256k1, SecretKey};
use tiny_keccak::{Hasher, Keccak};
use hex;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.on_brute_force_private_key({
        let ui_handle = ui.as_weak();
        move |partial_key, known_address| {
            let ui = ui_handle.unwrap();
            println!("Partial key from UI: {}", partial_key); 
            println!("Known address from UI: {}", known_address); 
            if partial_key.is_empty() {
                ui.set_result("Partial private key is empty.".into());
                return;
            }

            let result = brute_force_private_key(&partial_key, &known_address);
            ui.set_result(result.into());
        }
    });

    ui.run()?;

    Ok(())
}

fn brute_force_private_key(partial_key: &str, known_address: &str) -> String {
    let known_address = known_address.to_lowercase();

    if partial_key.len() != 63 {
        return format!("Invalid partial private key length. Expected 63, got {}.", partial_key.len());
    }

    for i in 0..=partial_key.len() {
        for ch in "0123456789abcdef".chars() {
            let mut candidate = partial_key.to_string();
            candidate.insert(i, ch);

            println!("Testing candidate: {}", candidate);

            if let Some(address) = derive_address_from_private_key(&candidate) {
                println!("Derived address: {}", address);
                if address.to_lowercase() == known_address {
                    return format!("Found matching private key: {}", candidate);
                }
            } else {
                println!("Error deriving address for candidate: {}", candidate);
            }
        }
    }

    "No matching private key found.".to_string()
}

fn derive_address_from_private_key(private_key: &str) -> Option<String> {
    let private_key_bytes = hex::decode(private_key).ok()?;
    
    if private_key_bytes.len() != 32 {
        println!("Invalid private key length. Expected 32 bytes, got {}.", private_key_bytes.len());
        return None;
    }

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&private_key_bytes).ok()?;
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_bytes = public_key.serialize_uncompressed();

    let mut hasher = Keccak::v256();
    hasher.update(&public_key_bytes[1..]); 
    let mut hashed_public_key = [0u8; 32];
    hasher.finalize(&mut hashed_public_key);

    let ethereum_address = &hashed_public_key[12..];
    let ethereum_address_hex = format!("0x{}", hex::encode(ethereum_address));

    Some(ethereum_address_hex)
}
