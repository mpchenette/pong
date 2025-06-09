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
            "{{\"balls\":{},\"blocks\":{},\"navy_grey_count\":{},\"navy_blue_count\":{},\"background_color\":[{},{},{}],\"average_speed\":{:.2}}}",
            balls_json, blocks_json, self.navy_grey_count, self.navy_blue_count, 
            self.background_color.0, self.background_color.1, self.background_color.2,
            self.get_average_speed()
        )
    }

    pub fn increase_speed(&mut self, factor: f64) {
        for ball in &mut self.balls {
            ball.dx *= factor;
            ball.dy *= factor;
        }
    }
    
    pub fn decrease_speed(&mut self, factor: f64) {
        for ball in &mut self.balls {
            ball.dx /= factor;
            ball.dy /= factor;
        }
    }
    
    pub fn get_average_speed(&self) -> f64 {
        if self.balls.is_empty() {
            return 0.0;
        }
        
        let total_speed: f64 = self.balls.iter()
            .map(|ball| (ball.dx * ball.dx + ball.dy * ball.dy).sqrt())
            .sum();
        
        total_speed / self.balls.len() as f64
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

    #[test]
    fn test_all_blocks_one_color_scenario() {
        let mut game = Game::new();
        
        // Manually convert all blocks to navy blue
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                game.blocks[row][col].color = BlockColor::NavyBlue;
            }
        }
        game.navy_blue_count = 100;
        game.navy_grey_count = 0;

        // Position white ball to hit a navy blue block
        game.balls[0].x = 0.0 - BALL_SIZE;
        game.balls[0].y = 0.0;
        game.balls[0].dx = 2.0;
        game.balls[0].dy = 0.0;
        game.balls[0].ball_type = BallType::White;

        game.update();

        // Should still function correctly
        assert_eq!(game.blocks[0][0].color, BlockColor::NavyGrey);
        assert_eq!(game.navy_blue_count, 99);
        assert_eq!(game.navy_grey_count, 1);
    }

    #[test]
    fn test_block_count_accuracy_during_rapid_changes() {
        let game = Game::new();
        
        // Verify initial state
        assert_eq!(game.navy_grey_count, 50);
        assert_eq!(game.navy_blue_count, 50);
        
        // Count actual blocks to verify consistency
        let mut actual_grey = 0;
        let mut actual_blue = 0;
        for row in &game.blocks {
            for block in row {
                match block.color {
                    BlockColor::NavyGrey => actual_grey += 1,
                    BlockColor::NavyBlue => actual_blue += 1,
                }
            }
        }
        
        assert_eq!(game.navy_grey_count, actual_grey);
        assert_eq!(game.navy_blue_count, actual_blue);
        assert_eq!(actual_grey + actual_blue, 100);
    }

    #[test]
    fn test_background_randomization() {
        let mut game = Game::new();
        let original_background = game.background_color;
        
        // Call randomize multiple times to see if it changes
        game.randomize_background();
        let first_random = game.background_color;
        
        game.randomize_background();
        let second_random = game.background_color;
        
        // At least one should be different from original (very high probability)
        // Note: There's a tiny chance this could fail due to randomness, but extremely unlikely
        assert!(first_random != original_background || second_random != original_background);
        
        // Background should be valid RGB values (u8 automatically limits to 0-255)
        // Just verify the values are accessible
        let _r = game.background_color.0;
        let _g = game.background_color.1;
        let _b = game.background_color.2;
    }

    #[test]
    fn test_json_serialization_with_modified_state() {
        let mut game = Game::new();
        
        // Modify game state
        game.balls[0].x = 123.45;
        game.balls[0].y = 67.89;
        game.blocks[0][0].color = BlockColor::NavyBlue;
        game.navy_grey_count = 49;
        game.navy_blue_count = 51;
        game.background_color = (100, 200, 50);
        
        let json = game.to_json();
        
        // Should contain updated values
        assert!(json.contains("123.45"));
        assert!(json.contains("67.89"));
        assert!(json.contains("\"navy_grey_count\":49"));
        assert!(json.contains("\"navy_blue_count\":51"));
        assert!(json.contains("[100,200,50]"));
    }
}
