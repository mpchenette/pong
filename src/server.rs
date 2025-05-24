use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

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

#[derive(Clone, Copy, PartialEq)]
enum BlockColor {
    NavyGrey,
    NavyBlue,
}

#[derive(Clone, Copy, PartialEq)]
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
}

impl Game {
    fn new() -> Self {
        // Create 2 balls
        let white_ball = Ball {
            x: CANVAS_WIDTH / 4.0,  // Start on left side
            y: CANVAS_HEIGHT / 2.0,
            dx: 2.0,
            dy: 1.5,
            ball_type: BallType::White,
        };

        let black_ball = Ball {
            x: 3.0 * CANVAS_WIDTH / 4.0,  // Start on right side
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
        }
    }

    fn update(&mut self) {
        // Update each ball
        for ball in &mut self.balls {
            // Update ball position
            ball.x += ball.dx;
            ball.y += ball.dy;

            // Ball collision with walls
            if ball.x <= 0.0 || ball.x >= CANVAS_WIDTH - BALL_SIZE {
                ball.dx = -ball.dx;
            }
            if ball.y <= 0.0 || ball.y >= CANVAS_HEIGHT - BALL_SIZE {
                ball.dy = -ball.dy;
            }

            // Check collision with blocks
            for row in &mut self.blocks {
                for block in row {
                    // Check if ball can interact with this block
                    let can_interact = match (ball.ball_type, block.color) {
                        (BallType::White, BlockColor::NavyBlue) => true,   // White ball hits navy blue blocks
                        (BallType::Black, BlockColor::NavyGrey) => true,   // Black ball hits navy grey blocks
                        _ => false,
                    };

                    if can_interact
                        && ball.x + BALL_SIZE >= block.x
                        && ball.x <= block.x + BLOCK_SIZE
                        && ball.y + BALL_SIZE >= block.y
                        && ball.y <= block.y + BLOCK_SIZE
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

                        // Bounce the ball
                        ball.dy = -ball.dy;
                        break;
                    }
                }
            }
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
            "{{\"balls\":{},\"blocks\":{},\"navy_grey_count\":{},\"navy_blue_count\":{}}}",
            balls_json, blocks_json, self.navy_grey_count, self.navy_blue_count
        )
    }
}

fn main() {
    let game = Arc::new(Mutex::new(Game::new()));
    let game_clone = Arc::clone(&game);

    // Start game loop in separate thread
    thread::spawn(move || {
        let mut last_update = Instant::now();
        loop {
            let now = Instant::now();
            if now.duration_since(last_update) >= Duration::from_millis(16) { // ~60 FPS
                game_clone.lock().unwrap().update();
                last_update = now;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Start HTTP server using std only
    let listener = TcpListener::bind("127.0.0.1:3031").unwrap();
    println!("Server running on http://127.0.0.1:3031");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let game_clone = Arc::clone(&game);
        
        thread::spawn(move || {
            handle_connection(stream, game_clone);
        });
    }
}

fn handle_connection(mut stream: TcpStream, game: Arc<Mutex<Game>>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");
    
    // Debug logging
    println!("Request line: '{}'", request_line);
    
    let (status_line, content_type, body) = if request_line.starts_with("GET /") && (request_line.starts_with("GET / ") || request_line == "GET /") {
        // Serve the HTML page
        let html_content = include_str!("../static/index.html");
        ("HTTP/1.1 200 OK", "text/html", html_content.to_string())
    } else if request_line.starts_with("GET /game-state") {
        // Serve current game state as JSON
        let game_state = game.lock().unwrap().to_json();
        ("HTTP/1.1 200 OK", "application/json", game_state)
    } else if request_line.starts_with("GET /reset") {
        // Reset game
        *game.lock().unwrap() = Game::new();
        ("HTTP/1.1 200 OK", "text/plain", "Game reset".to_string())
    } else {
        ("HTTP/1.1 404 NOT FOUND", "text/plain", "404 Not Found".to_string())
    };

    let response = format!(
        "{}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content_type,
        body.len(),
        body
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
