use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;

// Game constants
const BLOCK_SIZE: f64 = 40.0;
const GRID_SIZE: usize = 10;
const CANVAS_WIDTH: f64 = GRID_SIZE as f64 * BLOCK_SIZE;  // Exactly 400.0
const CANVAS_HEIGHT: f64 = GRID_SIZE as f64 * BLOCK_SIZE; // Exactly 400.0
const BALL_SIZE: f64 = 10.0;

// Colors
const NAVY_GREY: (u8, u8, u8) = (70, 80, 90);    // Navy grey for left half
const NAVY_BLUE: (u8, u8, u8) = (30, 50, 120);   // Navy blue for right half
const WHITE: (u8, u8, u8) = (255, 255, 255);     // White ball (left)
const BLACK: (u8, u8, u8) = (0, 0, 0);           // Black ball (right)

#[derive(Clone, Copy, PartialEq, Debug)]
enum BlockColor {
    NavyGrey,
    NavyBlue,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum BallType {
    White,  // Can hit navy blue blocks
    Black,  // Can hit navy grey blocks
}

#[derive(Clone)]
struct Ball {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    ball_type: BallType,
}

#[derive(Clone)]
struct Block {
    x: f64,
    y: f64,
    color: BlockColor,
}

#[derive(Clone)]
struct Game {
    balls: Vec<Ball>,
    blocks: Vec<Vec<Block>>,
    navy_grey_count: usize,
    navy_blue_count: usize,
    background_color: (u8, u8, u8),
}

impl Game {
    fn new() -> Self {
        // Create 2 balls
        let white_ball = Ball {
            x: CANVAS_WIDTH / 4.0, // Start on left side
            y: CANVAS_HEIGHT / 2.0,
            dx: 2.0,
            dy: 1.5,
            ball_type: BallType::White,
        };

        let black_ball = Ball {
            x: 3.0 * CANVAS_WIDTH / 4.0, // Start on right side
            y: CANVAS_HEIGHT / 2.0,
            dx: -2.0,
            dy: -1.5,
            ball_type: BallType::Black,
        };

        // Create 10x10 grid of blocks filling the entire canvas
        let mut blocks = Vec::new();
        let mut navy_grey_count = 0;
        let mut navy_blue_count = 0;

        for row in 0..GRID_SIZE {
            let mut block_row = Vec::new();
            for col in 0..GRID_SIZE {
                // Left half (cols 0-4) are navy grey, right half (cols 5-9) are navy blue
                let color = if col < GRID_SIZE / 2 {
                    navy_grey_count += 1;
                    BlockColor::NavyGrey
                } else {
                    navy_blue_count += 1;
                    BlockColor::NavyBlue
                };

                block_row.push(Block {
                    x: col as f64 * BLOCK_SIZE,
                    y: row as f64 * BLOCK_SIZE,
                    color,
                });
            }
            blocks.push(block_row);
        }

        Game {
            balls: vec![white_ball, black_ball],
            blocks,
            navy_grey_count,
            navy_blue_count,
            background_color: (34, 34, 34), // Default dark grey background
        }
    }

    fn randomize_background(&mut self) {
        // Generate random RGB values
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Simple pseudo-random number generation using system time
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let seed = hasher.finish();
        
        let r = ((seed >> 32) % 256) as u8;
        let g = ((seed >> 16) % 256) as u8; 
        let b = (seed % 256) as u8;
        
        self.background_color = (r, g, b);
    }

    fn update(&mut self) {
        // Update each ball
        for ball in &mut self.balls {
            // Store previous position for collision response
            let prev_x = ball.x;
            let prev_y = ball.y;

            // Calculate new position
            let new_x = ball.x + ball.dx;
            let new_y = ball.y + ball.dy;

            // Ball collision with walls
            let mut final_x = new_x;
            let mut final_y = new_y;
            let mut final_dx = ball.dx;
            let mut final_dy = ball.dy;

            // Wall collision with proper positioning
            if new_x <= 0.0 {
                final_x = 0.0;
                final_dx = -ball.dx;
            } else if new_x >= CANVAS_WIDTH - BALL_SIZE {
                final_x = CANVAS_WIDTH - BALL_SIZE;
                final_dx = -ball.dx;
            }

            if new_y <= 0.0 {
                final_y = 0.0;
                final_dy = -ball.dy;
            } else if new_y >= CANVAS_HEIGHT - BALL_SIZE {
                final_y = CANVAS_HEIGHT - BALL_SIZE;
                final_dy = -ball.dy;
            }

            // Check collision with blocks using swept collision detection
            let mut hit_block = false;
            for row in &mut self.blocks {
                for block in row {
                    if hit_block {
                        break;
                    }

                    // Check if ball can interact with this block
                    let can_interact = match (ball.ball_type, block.color) {
                        (BallType::White, BlockColor::NavyBlue) => true, // White ball hits navy blue blocks
                        (BallType::Black, BlockColor::NavyGrey) => true, // Black ball hits navy grey blocks
                        _ => false,
                    };

                    if can_interact {
                        // Check if ball will collide with block
                        let ball_left = final_x;
                        let ball_right = final_x + BALL_SIZE;
                        let ball_top = final_y;
                        let ball_bottom = final_y + BALL_SIZE;

                        let block_left = block.x;
                        let block_right = block.x + BLOCK_SIZE;
                        let block_top = block.y;
                        let block_bottom = block.y + BLOCK_SIZE;

                        // AABB collision detection
                        if ball_right >= block_left
                            && ball_left <= block_right
                            && ball_bottom >= block_top
                            && ball_top <= block_bottom
                        {
                            // Convert block to the other color
                            match block.color {
                                BlockColor::NavyGrey => {
                                    block.color = BlockColor::NavyBlue;
                                    self.navy_grey_count -= 1;
                                    self.navy_blue_count += 1;
                                }
                                BlockColor::NavyBlue => {
                                    block.color = BlockColor::NavyGrey;
                                    self.navy_blue_count -= 1;
                                    self.navy_grey_count += 1;
                                }
                            }

                            // Determine collision side and bounce accordingly
                            let prev_ball_left = prev_x;
                            let prev_ball_right = prev_x + BALL_SIZE;
                            let prev_ball_top = prev_y;
                            let prev_ball_bottom = prev_y + BALL_SIZE;

                            // Check which side was hit by comparing previous and current positions
                            let hit_from_left =
                                prev_ball_right <= block_left && ball_right >= block_left;
                            let hit_from_right =
                                prev_ball_left >= block_right && ball_left <= block_right;
                            let hit_from_top =
                                prev_ball_bottom <= block_top && ball_bottom >= block_top;
                            let hit_from_bottom =
                                prev_ball_top >= block_bottom && ball_top <= block_bottom;

                            if hit_from_left || hit_from_right {
                                // Horizontal collision
                                final_dx = -ball.dx;
                                if hit_from_left {
                                    final_x = block_left - BALL_SIZE;
                                } else {
                                    final_x = block_right;
                                }
                            } else if hit_from_top || hit_from_bottom {
                                // Vertical collision
                                final_dy = -ball.dy;
                                if hit_from_top {
                                    final_y = block_top - BALL_SIZE;
                                } else {
                                    final_y = block_bottom;
                                }
                            } else {
                                // Corner collision - bounce both directions
                                final_dx = -ball.dx;
                                final_dy = -ball.dy;
                            }

                            hit_block = true;
                            break;
                        }
                    }
                }
                if hit_block {
                    break;
                }
            }

            // Update ball position and velocity
            ball.x = final_x;
            ball.y = final_y;
            ball.dx = final_dx;
            ball.dy = final_dy;
        }
    }

    fn to_json(&self) -> String {
        // Manual JSON serialization to avoid serde dependency
        let mut balls_json = String::from("[");
        for (idx, ball) in self.balls.iter().enumerate() {
            if idx > 0 {
                balls_json.push(',');
            }
            let (r, g, b) = match ball.ball_type {
                BallType::White => WHITE,
                BallType::Black => BLACK,
            };
            balls_json.push_str(&format!(
                "{{\"x\":{},\"y\":{},\"color\":[{},{},{}]}}",
                ball.x, ball.y, r, g, b
            ));
        }
        balls_json.push(']');

        let mut blocks_json = String::from("[");
        for (row_idx, row) in self.blocks.iter().enumerate() {
            if row_idx > 0 {
                blocks_json.push(',');
            }
            blocks_json.push('[');
            for (col_idx, block) in row.iter().enumerate() {
                if col_idx > 0 {
                    blocks_json.push(',');
                }
                let (r, g, b) = match block.color {
                    BlockColor::NavyGrey => NAVY_GREY,
                    BlockColor::NavyBlue => NAVY_BLUE,
                };
                blocks_json.push_str(&format!(
                    "{{\"x\":{},\"y\":{},\"color\":[{},{},{}]}}",
                    block.x, block.y, r, g, b
                ));
            }
            blocks_json.push(']');
        }
        blocks_json.push(']');

        format!(
            "{{\"balls\":{},\"blocks\":{},\"navy_grey_count\":{},\"navy_blue_count\":{},\"background_color\":[{},{},{}]}}",
            balls_json, blocks_json, self.navy_grey_count, self.navy_blue_count, 
            self.background_color.0, self.background_color.1, self.background_color.2
        )
    }
}

// WebSocket implementation using only std library
mod websocket {
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
}

fn main() {
    let game = Arc::new(Mutex::new(Game::new()));
    let clients: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let client_counter = Arc::new(Mutex::new(0usize));
    
    let game_clone = Arc::clone(&game);
    let clients_clone = Arc::clone(&clients);

    // Start game loop in separate thread
    thread::spawn(move || {
        let mut last_update = Instant::now();
        loop {
            let now = Instant::now();
            if now.duration_since(last_update) >= Duration::from_millis(16) {
                // ~60 FPS
                game_clone.lock().unwrap().update();
                
                // Send game state to all connected clients
                let game_state = game_clone.lock().unwrap().to_json();
                let mut clients_guard = clients_clone.lock().unwrap();
                let mut disconnected_clients = Vec::new();
                
                for (client_id, stream) in clients_guard.iter_mut() {
                    // Set a short timeout for writing
                    if let Ok(_) = stream.set_write_timeout(Some(Duration::from_millis(10))) {
                        if let Err(_) = websocket::send_text_frame(stream, &game_state) {
                            disconnected_clients.push(*client_id);
                        }
                    } else {
                        disconnected_clients.push(*client_id);
                    }
                }
                
                // Remove disconnected clients
                for client_id in disconnected_clients {
                    clients_guard.remove(&client_id);
                    println!("Client {} disconnected", client_id);
                }
                
                last_update = now;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Start WebSocket server
    // let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    // println!("WebSocket server running on ws://127.0.0.1:8000");
    println!("WebSocket server running on ws://0.0.0.0:8000");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let clients_clone = Arc::clone(&clients);
        let client_counter_clone = Arc::clone(&client_counter);
        let game_clone = Arc::clone(&game);

        thread::spawn(move || {
            handle_websocket_connection(stream, clients_clone, client_counter_clone, game_clone);
        });
    }
}

fn handle_websocket_connection(
    mut stream: TcpStream, 
    clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
    client_counter: Arc<Mutex<usize>>,
    game: Arc<Mutex<Game>>
) {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Received request: {}", request.lines().next().unwrap_or(""));
    
    // Check if it's a WebSocket upgrade request
    if request.contains("Upgrade: websocket") {
        println!("WebSocket upgrade request detected");
        // Extract the Sec-WebSocket-Key
        let key = request
            .lines()
            .find(|line| line.starts_with("Sec-WebSocket-Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|key| key.trim())
            .unwrap_or("");

        let accept_key = websocket::generate_accept_key(key);
        
        // Send WebSocket handshake response
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );
        
        if stream.write_all(response.as_bytes()).is_err() {
            println!("Failed to send WebSocket handshake response");
            return;
        }
        
        if stream.flush().is_err() {
            println!("Failed to flush WebSocket handshake response");
            return;
        }
        
        println!("WebSocket handshake completed successfully");
        
        // Add client to the list
        let client_id = {
            let mut counter = client_counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        
        println!("Client {} connected via WebSocket", client_id);
        
        // Clone the stream for the clients map
        let stream_clone = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to clone stream: {}", e);
                return;
            }
        };
        
        clients.lock().unwrap().insert(client_id, stream_clone);
        
        // Send initial game state
        let game_state = "{}"; // Empty initial state
        if let Err(_) = websocket::send_text_frame(&mut stream, game_state) {
            println!("Failed to send initial game state to client {}", client_id);
        }
        
        // The game loop will handle ongoing communication
        // Keep the connection handler minimal
        thread::sleep(Duration::from_millis(100)); // Brief delay then exit handler
        
    } else if request.contains("GET /randomize-background") {
        // Handle background randomization request
        println!("Background randomization requested via HTTP");
        game.lock().unwrap().randomize_background();
        let response = "HTTP/1.1 200 OK\r\n\r\nBackground randomized";
        let _ = stream.write_all(response.as_bytes());
        
    } else if request.contains("GET /") && (request.contains("GET / ") || request.contains("GET /?")) {
        // Serve HTML page for regular HTTP requests
        println!("Serving HTML page");
        let html_content = include_str!("../static/index.html");
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             Content-Length: {}\r\n\r\n{}",
            html_content.len(),
            html_content
        );
        let _ = stream.write_all(response.as_bytes());
    } else {
        // 404 for other requests
        println!("404 for request: {}", request.lines().next().unwrap_or(""));
        let response = "HTTP/1.1 404 Not Found\r\n\r\n404 Not Found";
        let _ = stream.write_all(response.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_initialization() {
        let game = Game::new();

        // Should have 2 balls
        assert_eq!(game.balls.len(), 2);

        // Check initial ball positions
        let white_ball = &game.balls[0];
        let black_ball = &game.balls[1];

        assert_eq!(white_ball.ball_type, BallType::White);
        assert_eq!(black_ball.ball_type, BallType::Black);

        // White ball should start on left side
        assert!(white_ball.x < CANVAS_WIDTH / 2.0);
        // Black ball should start on right side
        assert!(black_ball.x > CANVAS_WIDTH / 2.0);

        // Should have 10x10 grid
        assert_eq!(game.blocks.len(), 10);
        assert_eq!(game.blocks[0].len(), 10);

        // Initial counts should be 50 each
        assert_eq!(game.navy_grey_count, 50);
        assert_eq!(game.navy_blue_count, 50);

        // Check initial block colors
        // Left half should be navy grey
        for row in &game.blocks {
            for (col, block) in row.iter().enumerate() {
                if col < 5 {
                    assert_eq!(block.color, BlockColor::NavyGrey);
                } else {
                    assert_eq!(block.color, BlockColor::NavyBlue);
                }
            }
        }
    }

    #[test]
    fn test_ball_wall_collision() {
        let mut game = Game::new();

        // Set up ball to hit left wall
        game.balls[0].x = 0.0;
        game.balls[0].y = 100.0;
        game.balls[0].dx = -2.0;
        game.balls[0].dy = 1.0;

        game.update();

        // Ball should bounce off left wall
        assert_eq!(game.balls[0].x, 0.0);
        assert_eq!(game.balls[0].dx, 2.0); // Should reverse direction
        assert_eq!(game.balls[0].dy, 1.0); // Y velocity unchanged
    }

    #[test]
    fn test_ball_top_wall_collision() {
        let mut game = Game::new();

        // Set up ball to hit top wall
        game.balls[0].x = 100.0;
        game.balls[0].y = 0.0;
        game.balls[0].dx = 1.0;
        game.balls[0].dy = -2.0;

        game.update();

        // Ball should bounce off top wall
        assert_eq!(game.balls[0].y, 0.0);
        assert_eq!(game.balls[0].dx, 1.0); // X velocity unchanged
        assert_eq!(game.balls[0].dy, 2.0); // Should reverse direction
    }

    #[test]
    fn test_white_ball_hits_navy_blue_block() {
        let mut game = Game::new();

        // Position white ball to hit a navy blue block (right side)
        game.balls[0].x = 200.0 - BALL_SIZE; // Just left of block at (200, 0)
        game.balls[0].y = 0.0;
        game.balls[0].dx = 2.0;
        game.balls[0].dy = 0.0;
        game.balls[0].ball_type = BallType::White;

        // Ensure there's a navy blue block where we expect
        assert_eq!(game.blocks[0][5].color, BlockColor::NavyBlue);
        let initial_navy_blue = game.navy_blue_count;
        let initial_navy_grey = game.navy_grey_count;

        game.update();

        // Block should convert to navy grey
        assert_eq!(game.blocks[0][5].color, BlockColor::NavyGrey);
        assert_eq!(game.navy_blue_count, initial_navy_blue - 1);
        assert_eq!(game.navy_grey_count, initial_navy_grey + 1);

        // Ball should bounce back
        assert!(game.balls[0].dx < 0.0);
    }

    #[test]
    fn test_black_ball_hits_navy_grey_block() {
        let mut game = Game::new();

        // Position black ball to hit a navy grey block (left side)
        game.balls[1].x = BLOCK_SIZE; // Just right of block at (0, 0)
        game.balls[1].y = 0.0;
        game.balls[1].dx = -2.0;
        game.balls[1].dy = 0.0;
        game.balls[1].ball_type = BallType::Black;

        // Ensure there's a navy grey block where we expect
        assert_eq!(game.blocks[0][0].color, BlockColor::NavyGrey);
        let initial_navy_blue = game.navy_blue_count;
        let initial_navy_grey = game.navy_grey_count;

        game.update();

        // Block should convert to navy blue
        assert_eq!(game.blocks[0][0].color, BlockColor::NavyBlue);
        assert_eq!(game.navy_blue_count, initial_navy_blue + 1);
        assert_eq!(game.navy_grey_count, initial_navy_grey - 1);

        // Ball should bounce back
        assert!(game.balls[1].dx > 0.0);
    }

    #[test]
    fn test_white_ball_ignores_navy_grey_block() {
        let mut game = Game::new();

        // Position white ball to pass through a navy grey block (left side)
        game.balls[0].x = 0.0;
        game.balls[0].y = 0.0;
        game.balls[0].dx = 2.0;
        game.balls[0].dy = 0.0;
        game.balls[0].ball_type = BallType::White;

        let initial_navy_grey = game.navy_grey_count;
        let initial_navy_blue = game.navy_blue_count;

        game.update();

        // Block should remain unchanged
        assert_eq!(game.blocks[0][0].color, BlockColor::NavyGrey);
        assert_eq!(game.navy_grey_count, initial_navy_grey);
        assert_eq!(game.navy_blue_count, initial_navy_blue);

        // Ball should continue moving (not bounce)
        assert!(game.balls[0].dx > 0.0);
    }

    #[test]
    fn test_json_serialization_structure() {
        let game = Game::new();
        let json = game.to_json();

        // Should contain required fields
        assert!(json.contains("\"balls\":"));
        assert!(json.contains("\"blocks\":"));
        assert!(json.contains("\"navy_grey_count\":"));
        assert!(json.contains("\"navy_blue_count\":"));
        assert!(json.contains("\"background_color\":"));

        // Should contain initial counts
        assert!(json.contains("\"navy_grey_count\":50"));
        assert!(json.contains("\"navy_blue_count\":50"));
    }

    #[test]
    fn test_ball_positions_stay_in_bounds() {
        let mut game = Game::new();

        // Run many updates to ensure balls stay in bounds
        for _ in 0..1000 {
            game.update();

            for ball in &game.balls {
                assert!(ball.x >= 0.0);
                assert!(ball.y >= 0.0);
                assert!(ball.x <= CANVAS_WIDTH - BALL_SIZE);
                assert!(ball.y <= CANVAS_HEIGHT - BALL_SIZE);
            }
        }
    }
}
