use crate::game::types::*;

impl Game {
    pub fn update(&mut self) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
