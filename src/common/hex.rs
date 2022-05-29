//! For hex

const fn val(c: u8) -> u8 {
    match c {
        b'A'..=b'F' => c - b'A' + 10,
        b'a'..=b'f' => c - b'a' + 10,
        b'0'..=b'9' => c - b'0',
        _ => 0,
    }
}

const HEX_CHARS_UPPER: &[u8; 16] = b"0123456789ABCDEF";

/// Encode bytes into a hex string
pub fn encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);

    for c in bytes.iter() {
        hex.push(HEX_CHARS_UPPER[(c >> 4) as usize] as char);
        hex.push(HEX_CHARS_UPPER[(c & 0x0F) as usize] as char);
    }
    hex
}

/// Decode a hex string to `Vec<u8>`
pub fn decode(hex_str: &str) -> Result<Vec<u8>, std::io::Error> {
    if hex_str.len() % 2 != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid hex string",
        ));
    }
    let array: Vec<u8> = hex_str
        .as_bytes()
        .chunks(2)
        .map(|pair| val(pair[0]) << 4 | val(pair[1]))
        .collect();
    Ok(array)
}
