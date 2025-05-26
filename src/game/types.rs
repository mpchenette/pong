// Game constants
pub const BLOCK_SIZE: f64 = 40.0;
pub const GRID_SIZE: usize = 10;
pub const CANVAS_WIDTH: f64 = GRID_SIZE as f64 * BLOCK_SIZE;  // Exactly 400.0
pub const CANVAS_HEIGHT: f64 = GRID_SIZE as f64 * BLOCK_SIZE; // Exactly 400.0
pub const BALL_SIZE: f64 = 10.0;

// Colors
pub const NAVY_GREY: (u8, u8, u8) = (70, 80, 90);    // Navy grey for left half
pub const NAVY_BLUE: (u8, u8, u8) = (30, 50, 120);   // Navy blue for right half
pub const WHITE: (u8, u8, u8) = (255, 255, 255);     // White ball (left)
pub const BLACK: (u8, u8, u8) = (0, 0, 0);           // Black ball (right)

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlockColor {
    NavyGrey,
    NavyBlue,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BallType {
    White,  // Can hit navy blue blocks
    Black,  // Can hit navy grey blocks
}

#[derive(Clone)]
pub struct Ball {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub ball_type: BallType,
}

#[derive(Clone)]
pub struct Block {
    pub x: f64,
    pub y: f64,
    pub color: BlockColor,
}

#[derive(Clone)]
pub struct Game {
    pub balls: Vec<Ball>,
    pub blocks: Vec<Vec<Block>>,
    pub navy_grey_count: usize,
    pub navy_blue_count: usize,
    pub background_color: (u8, u8, u8),
}
