use std::io::{Write, Read};
use std::net::TcpStream;

pub fn generate_accept_key(key: &str) -> String {
    // RFC 6455: Concatenate key with WebSocket GUID and compute SHA-1 hash
    const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let concat = format!("{}{}", key, WEBSOCKET_GUID);
    let hash = sha1(&concat.as_bytes());
    base64_encode(&hash)
}

// Simple SHA-1 implementation using only std library
fn sha1(input: &[u8]) -> [u8; 20] {
    // Initialize hash values (from SHA-1 spec)
    let mut h = [
        0x67452301u32,
        0xEFCDAB89u32,
        0x98BADCFEu32,
        0x10325476u32,
        0xC3D2E1F0u32,
    ];
    
    // Pre-processing: adding padding bits
    let mut message = input.to_vec();
    let original_len = message.len();
    message.push(0x80); // append bit '1' followed by zeros
    
    // Pad to 512 bits (64 bytes) minus 64 bits (8 bytes) for length
    while (message.len() % 64) != 56 {
        message.push(0x00);
    }
    
    // Append original length in bits as 64-bit big-endian
    let bit_len = (original_len as u64) * 8;
    message.extend_from_slice(&bit_len.to_be_bytes());
    
    // Process message in 512-bit chunks
    for chunk in message.chunks(64) {
        let mut w = [0u32; 80];
        
        // Break chunk into sixteen 32-bit words
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        
        // Extend the words
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        
        // Initialize working variables
        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        
        // Main loop
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                60..=79 => (b ^ c ^ d, 0xCA62C1D6),
                _ => unreachable!(),
            };
            
            let temp = a.rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        
        // Update hash values
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    
    // Convert to bytes
    let mut result = [0u8; 20];
    for (i, &val) in h.iter().enumerate() {
        let bytes = val.to_be_bytes();
        result[i * 4..i * 4 + 4].copy_from_slice(&bytes);
    }
    
    result
}

fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in input.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }
        
        let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);
        
        result.push(CHARS[((b >> 18) & 63) as usize] as char);
        result.push(CHARS[((b >> 12) & 63) as usize] as char);
        result.push(if chunk.len() > 1 { CHARS[((b >> 6) & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { CHARS[(b & 63) as usize] as char } else { '=' });
    }
    
    result
}

pub fn send_text_frame(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    let text_bytes = text.as_bytes();
    let text_len = text_bytes.len();
    
    let mut frame = Vec::new();
    
    // FIN (1) + RSV (3) + Opcode (4) = 0x81 for text frame
    frame.push(0x81);
    
    // Payload length
    if text_len < 126 {
        frame.push(text_len as u8);
    } else if text_len < 65536 {
        frame.push(126);
        frame.extend_from_slice(&(text_len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(text_len as u64).to_be_bytes());
    }
    
    // Payload data
    frame.extend_from_slice(text_bytes);
    
    stream.write_all(&frame)?;
    stream.flush()
}

pub fn read_frame(stream: &mut TcpStream) -> std::io::Result<Option<String>> {
    let mut buffer = [0u8; 2];
    if stream.read_exact(&mut buffer).is_err() {
        return Ok(None);
    }
    
    let opcode = buffer[0] & 0x0F;
    let masked = (buffer[1] & 0x80) != 0;
    let mut payload_len = (buffer[1] & 0x7F) as usize;
    
    // Handle extended payload length
    if payload_len == 126 {
        let mut len_bytes = [0u8; 2];
        stream.read_exact(&mut len_bytes)?;
        payload_len = u16::from_be_bytes(len_bytes) as usize;
    } else if payload_len == 127 {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes)?;
        payload_len = u64::from_be_bytes(len_bytes) as usize;
    }
    
    // Read mask if present
    let mask = if masked {
        let mut mask_bytes = [0u8; 4];
        stream.read_exact(&mut mask_bytes)?;
        Some(mask_bytes)
    } else {
        None
    };
    
    // Read payload
    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload)?;
    
    // Unmask payload if needed
    if let Some(mask) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }
    
    // Handle different frame types
    match opcode {
        0x1 => { // Text frame
            Ok(Some(String::from_utf8_lossy(&payload).to_string()))
        },
        0x8 => { // Close frame
            Ok(None)
        },
        _ => Ok(Some(String::new())) // Ignore other frame types
    }
}
