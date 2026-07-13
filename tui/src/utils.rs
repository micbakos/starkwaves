use crossterm::terminal::window_size;
use starknet_rust_core::types::Felt;

pub fn window_ratio() -> f32 {
    if let Ok(size) = window_size() {
        if size.width > 0 && size.height > 0 && size.width > 0 && size.height > 0 {
            let cell_px_w = size.width as f32 / size.columns as f32;
            let cell_px_h = size.height as f32 / size.rows as f32;
            cell_px_h / cell_px_w
        } else {
            2.0
        }
    } else {
        2.0
    }
}

pub fn format_address_felt(address: Felt) -> String {
    format_address_string(address.to_fixed_hex_string())
}

pub fn format_address_string(address: String) -> String {
    let count = address.chars().count();
    if count <= 10 {
        return address.to_string();
    }
    let start: String = address.chars().take(6).collect();
    let end: String = address.chars().skip(count - 4).collect();
    format!("{start}...{end}")
}