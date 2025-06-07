use crate::game::types::*;
use std::collections::HashMap;

// Spatial partitioning to reduce collision detection from O(n²) to O(n)
// Currently: Each ball checks ALL 100 blocks = 200 collision checks per frame
// Optimized: Each ball checks only nearby blocks = ~4-9 collision checks per frame

pub struct SpatialGrid {
    cell_size: f64,
    cells: HashMap<(i32, i32), Vec<(usize, usize)>>, // (row, col) of blocks in each cell
}

impl SpatialGrid {
    pub fn new() -> Self {
        SpatialGrid {
            cell_size: BLOCK_SIZE, // Each cell = one block size
            cells: HashMap::new(),
        }
    }

    pub fn rebuild(&mut self, blocks: &Vec<Vec<Block>>) {
        self.cells.clear();
        
        for (row, block_row) in blocks.iter().enumerate() {
            for (col, block) in block_row.iter().enumerate() {
                let cell_x = (block.x / self.cell_size) as i32;
                let cell_y = (block.y / self.cell_size) as i32;
                
                self.cells.entry((cell_x, cell_y))
                    .or_insert_with(Vec::new)
                    .push((row, col));
            }
        }
    }

    pub fn get_nearby_blocks(&self, ball: &Ball) -> Vec<(usize, usize)> {
        let mut nearby = Vec::new();
        
        // Get cells that the ball overlaps (including movement prediction)
        let min_x = ((ball.x - BALL_SIZE) / self.cell_size) as i32;
        let max_x = ((ball.x + BALL_SIZE + ball.dx.abs()) / self.cell_size) as i32;
        let min_y = ((ball.y - BALL_SIZE) / self.cell_size) as i32;
        let max_y = ((ball.y + BALL_SIZE + ball.dy.abs()) / self.cell_size) as i32;
        
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                if let Some(blocks_in_cell) = self.cells.get(&(x, y)) {
                    nearby.extend_from_slice(blocks_in_cell);
                }
            }
        }
        
        nearby
    }
}

impl Game {
    // For this example, we'll create the grid each time for simplicity
    // In a real implementation, you'd add spatial_grid as a field to Game struct
    pub fn update_optimized(&mut self) {
        let mut grid = SpatialGrid::new();
        grid.rebuild(&self.blocks);
        self.update_with_spatial_grid(&grid);
    }

    fn update_with_spatial_grid(&mut self, grid: &SpatialGrid) {
        for ball in &mut self.balls {
            let prev_x = ball.x;
            let prev_y = ball.y;

            // Calculate new position (wall collision logic same as before)
            let new_x = ball.x + ball.dx;
            let new_y = ball.y + ball.dy;

            let mut final_x = new_x;
            let mut final_y = new_y;
            let mut final_dx = ball.dx;
            let mut final_dy = ball.dy;

            // Wall collision (unchanged)
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

            // OPTIMIZED: Only check nearby blocks instead of all blocks
            let nearby_blocks = grid.get_nearby_blocks(ball);
            
            for &(row, col) in &nearby_blocks {
                
                if let Some(block_row) = self.blocks.get_mut(row) {
                    if let Some(block) = block_row.get_mut(col) {
                        // Same collision logic as before
                        let can_interact = match (ball.ball_type, block.color) {
                            (BallType::White, BlockColor::NavyBlue) => true,
                            (BallType::Black, BlockColor::NavyGrey) => true,
                            _ => false,
                        };

                        if can_interact {
                            // AABB collision detection (same as before)
                            let ball_left = final_x;
                            let ball_right = final_x + BALL_SIZE;
                            let ball_top = final_y;
                            let ball_bottom = final_y + BALL_SIZE;

                            let block_left = block.x;
                            let block_right = block.x + BLOCK_SIZE;
                            let block_top = block.y;
                            let block_bottom = block.y + BLOCK_SIZE;

                            if ball_right >= block_left
                                && ball_left <= block_right
                                && ball_bottom >= block_top
                                && ball_top <= block_bottom
                            {
                                // Same block conversion and collision response logic
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

                                // Same bounce logic
                                let prev_ball_left = prev_x;
                                let prev_ball_right = prev_x + BALL_SIZE;
                                let prev_ball_top = prev_y;
                                let prev_ball_bottom = prev_y + BALL_SIZE;

                                let hit_from_left =
                                    prev_ball_right <= block_left && ball_right >= block_left;
                                let hit_from_right =
                                    prev_ball_left >= block_right && ball_left <= block_right;
                                let hit_from_top =
                                    prev_ball_bottom <= block_top && ball_bottom >= block_top;
                                let hit_from_bottom =
                                    prev_ball_top >= block_bottom && ball_top <= block_bottom;

                                if hit_from_left || hit_from_right {
                                    final_dx = -ball.dx;
                                    if hit_from_left {
                                        final_x = block_left - BALL_SIZE;
                                    } else {
                                        final_x = block_right;
                                    }
                                } else if hit_from_top || hit_from_bottom {
                                    final_dy = -ball.dy;
                                    if hit_from_top {
                                        final_y = block_top - BALL_SIZE;
                                    } else {
                                        final_y = block_bottom;
                                    }
                                } else {
                                    final_dx = -ball.dx;
                                    final_dy = -ball.dy;
                                }

                                break;
                            }
                        }
                    }
                }
            }

            ball.x = final_x;
            ball.y = final_y;
            ball.dx = final_dx;
            ball.dy = final_dy;
        }
    }
}

// Performance improvement:
// Before: 2 balls × 100 blocks = 200 collision checks per frame
// After: 2 balls × ~6 nearby blocks = ~12 collision checks per frame
// ~94% reduction in collision detection overhead
