use crate::game::types::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

impl Game {
    pub fn new() -> Self {
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

    pub fn randomize_background(&mut self) {
        // Generate random RGB values
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let seed = hasher.finish();
        
        let r = ((seed >> 32) % 256) as u8;
        let g = ((seed >> 16) % 256) as u8; 
        let b = (seed % 256) as u8;
        
        self.background_color = (r, g, b);
    }

    pub fn to_json(&self) -> String {
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
}
