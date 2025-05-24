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
        }
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
            if now.duration_since(last_update) >= Duration::from_millis(16) {
                // ~60 FPS
                game_clone.lock().unwrap().update();
                last_update = now;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Start HTTP server using std only
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    println!("Server running on http://127.0.0.1:8000");

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
    let _ = stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let request_line = request.lines().next().unwrap_or("");

    let (status_line, content_type, body) = if request_line.starts_with("GET /")
        && (request_line.starts_with("GET / ") || request_line == "GET /")
    {
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
        (
            "HTTP/1.1 404 NOT FOUND",
            "text/plain",
            "404 Not Found".to_string(),
        )
    };

    let response = format!(
        "{}\r\nContent-Type: {}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        content_type,
        body.len(),
        body
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
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
