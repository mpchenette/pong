use yew::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use wasm_bindgen::{prelude::*, JsCast};
use gloo::timers::callback::Interval;

const CANVAS_WIDTH: f64 = 500.0;
const CANVAS_HEIGHT: f64 = 500.0;
const GRID_SIZE: usize = 10;
const BLOCK_SIZE: f64 = CANVAS_WIDTH / GRID_SIZE as f64; // 50x50 blocks
const BALL_SIZE: f64 = 8.0;
const BALL_SPEED: f64 = 8.0;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockType {
    NavyGrey,  // Passable blocks
    NavyBlue,  // Blocks that get converted when hit
}

#[derive(Clone, PartialEq)]
pub struct Block {
    pub x: f64,
    pub y: f64,
    pub block_type: BlockType,
}

#[derive(Clone, PartialEq)]
pub struct Ball {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, PartialEq)]
pub struct GameState {
    pub left_ball: Ball,   // Converts blue to grey
    pub right_ball: Ball,  // Converts grey to blue
    pub blocks: Vec<Vec<Block>>,
    pub blue_blocks: usize,
    pub grey_blocks: usize,
}

impl Default for GameState {
    fn default() -> Self {
        // Create 10x10 grid of blocks
        let mut blocks = Vec::new();
        let mut blue_blocks = 0;
        let mut grey_blocks = 0;
        
        for y in 0..GRID_SIZE {
            let mut row = Vec::new();
            for x in 0..GRID_SIZE {
                let block_x = x as f64 * BLOCK_SIZE;
                let block_y = y as f64 * BLOCK_SIZE;
                
                // Left half (x < 5) is navy grey, right half (x >= 5) is navy blue
                let block_type = if x < GRID_SIZE / 2 {
                    grey_blocks += 1;
                    BlockType::NavyGrey
                } else {
                    blue_blocks += 1;
                    BlockType::NavyBlue
                };
                
                row.push(Block {
                    x: block_x,
                    y: block_y,
                    block_type,
                });
            }
            blocks.push(row);
        }
        
        // Left ball starts on left side, converts blue to grey
        // Angle between 315° and 45° (pointing generally right)
        let left_angle_degrees = 315.0 + js_sys::Math::random() * 90.0; // 315° to 405° (which wraps to 45°)
        let left_angle = (left_angle_degrees % 360.0) * std::f64::consts::PI / 180.0;
        
        // Right ball starts on right side, converts grey to blue  
        // Angle between 135° and 225° (pointing generally left)
        let right_angle_degrees = 135.0 + js_sys::Math::random() * 90.0; // 135° to 225°
        let right_angle = right_angle_degrees * std::f64::consts::PI / 180.0;
        
        Self {
            left_ball: Ball {
                x: CANVAS_WIDTH * 0.25 - BALL_SIZE / 2.0,
                y: CANVAS_HEIGHT / 2.0 - BALL_SIZE / 2.0,
                dx: left_angle.cos() * BALL_SPEED,
                dy: left_angle.sin() * BALL_SPEED,
            },
            right_ball: Ball {
                x: CANVAS_WIDTH * 0.75 - BALL_SIZE / 2.0,
                y: CANVAS_HEIGHT / 2.0 - BALL_SIZE / 2.0,
                dx: right_angle.cos() * BALL_SPEED,
                dy: right_angle.sin() * BALL_SPEED,
            },
            blocks,
            blue_blocks,
            grey_blocks,
        }
    }
}

pub enum GameMsg {
    Tick,
    Reset,
}

pub struct BlockBreaker {
    canvas_ref: NodeRef,
    state: GameState,
    _interval: Option<Interval>,
}

impl Component for BlockBreaker {
    type Message = GameMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let interval = Interval::new(16, move || {
            link.send_message(GameMsg::Tick);
        });

        Self {
            canvas_ref: NodeRef::default(),
            state: GameState::default(),
            _interval: Some(interval),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GameMsg::Tick => {
                self.update_game();
                self.draw();
                true  // Return true to trigger UI re-render for score updates
            }
            GameMsg::Reset => {
                self.state = GameState::default();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        let on_reset = link.callback(|_| GameMsg::Reset);

        html! {
            <div class="game-container">
                <div class="game-info">
                    <div class="score">
                        <span style="color: #4a5568; font-size: 24px; font-weight: bold;">
                            {self.state.grey_blocks.to_string()}
                        </span>
                        <span style="margin: 0 20px; color: #666;">{"vs"}</span>
                        <span style="color: #2d3748; font-size: 24px; font-weight: bold;">
                            {self.state.blue_blocks.to_string()}
                        </span>
                    </div>
                    <div class="controls">
                        <button onclick={on_reset} class="btn">{"Reset"}</button>
                    </div>
                </div>
                <canvas
                    ref={self.canvas_ref.clone()}
                    width={CANVAS_WIDTH.to_string()}
                    height={CANVAS_HEIGHT.to_string()}
                    class="game-canvas"
                />
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.draw();
        }
    }
}

impl BlockBreaker {
    fn update_game(&mut self) {
        // Update left ball (converts blue to grey)
        self.update_ball(&mut self.state.left_ball.clone(), true);
        
        // Update right ball (converts grey to blue)
        self.update_ball(&mut self.state.right_ball.clone(), false);
        
        // Update block counts
        self.update_block_counts();
    }
    
    fn update_ball(&mut self, ball: &mut Ball, converts_blue_to_grey: bool) {
        // Move ball
        ball.x += ball.dx;
        ball.y += ball.dy;
        
        // Bounce off walls
        if ball.y <= 0.0 || ball.y >= CANVAS_HEIGHT - BALL_SIZE {
            ball.dy = -ball.dy;
            ball.y = ball.y.max(0.0).min(CANVAS_HEIGHT - BALL_SIZE);
        }
        
        if ball.x <= 0.0 || ball.x >= CANVAS_WIDTH - BALL_SIZE {
            ball.dx = -ball.dx;
            ball.x = ball.x.max(0.0).min(CANVAS_WIDTH - BALL_SIZE);
        }
        
        // Check block collisions
        self.check_block_collision(ball, converts_blue_to_grey);
        
        // Update the ball in state
        if converts_blue_to_grey {
            self.state.left_ball = ball.clone();
        } else {
            self.state.right_ball = ball.clone();
        }
    }
    
    fn check_block_collision(&mut self, ball: &mut Ball, converts_blue_to_grey: bool) {
        let ball_center_x = ball.x + BALL_SIZE / 2.0;
        let ball_center_y = ball.y + BALL_SIZE / 2.0;
        
        let grid_x = (ball_center_x / BLOCK_SIZE) as usize;
        let grid_y = (ball_center_y / BLOCK_SIZE) as usize;
        
        // Make sure we're within grid bounds
        if grid_x < GRID_SIZE && grid_y < GRID_SIZE {
            let block = &mut self.state.blocks[grid_y][grid_x];
            
            let should_convert = if converts_blue_to_grey {
                // Left ball converts blue blocks to grey
                block.block_type == BlockType::NavyBlue
            } else {
                // Right ball converts grey blocks to blue
                block.block_type == BlockType::NavyGrey
            };
            
            if should_convert {
                // Convert the block
                if converts_blue_to_grey {
                    block.block_type = BlockType::NavyGrey;
                } else {
                    block.block_type = BlockType::NavyBlue;
                }
                
                // Bounce the ball based on which side it hit the block
                let block_center_x = block.x + BLOCK_SIZE / 2.0;
                let block_center_y = block.y + BLOCK_SIZE / 2.0;
                
                let dx = ball_center_x - block_center_x;
                let dy = ball_center_y - block_center_y;
                
                // Determine bounce direction based on collision angle
                if dx.abs() > dy.abs() {
                    // Hit from left or right side
                    ball.dx = -ball.dx;
                    // Move ball out of block
                    if dx > 0.0 {
                        ball.x = block.x + BLOCK_SIZE + 1.0;
                    } else {
                        ball.x = block.x - BALL_SIZE - 1.0;
                    }
                } else {
                    // Hit from top or bottom
                    ball.dy = -ball.dy;
                    // Move ball out of block
                    if dy > 0.0 {
                        ball.y = block.y + BLOCK_SIZE + 1.0;
                    } else {
                        ball.y = block.y - BALL_SIZE - 1.0;
                    }
                }
            }
            // If no conversion needed, ball passes through
        }
    }
    
    fn update_block_counts(&mut self) {
        let mut grey_blocks = 0;
        let mut blue_blocks = 0;
        
        for row in &self.state.blocks {
            for block in row {
                match block.block_type {
                    BlockType::NavyGrey => grey_blocks += 1,
                    BlockType::NavyBlue => blue_blocks += 1,
                }
            }
        }
        
        self.state.grey_blocks = grey_blocks;
        self.state.blue_blocks = blue_blocks;
    }

    fn draw(&self) {
        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        // Clear canvas with pure black background
        context.set_fill_style(&"#000000".into());
        context.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);

        // Draw blocks
        for row in &self.state.blocks {
            for block in row {
                match block.block_type {
                    BlockType::NavyGrey => {
                        // Navy grey color
                        context.set_fill_style(&"#4a5568".into());
                    }
                    BlockType::NavyBlue => {
                        // Navy blue color
                        context.set_fill_style(&"#2d3748".into());
                    }
                }
                context.fill_rect(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                
                // Add subtle border
                context.set_stroke_style(&"#2a2a2a".into());
                context.set_line_width(1.0);
                context.stroke_rect(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
            }
        }

        // Draw left ball (white)
        context.set_fill_style(&"#ffffff".into());
        context.begin_path();
        let _ = context.arc(
            self.state.left_ball.x + BALL_SIZE / 2.0,
            self.state.left_ball.y + BALL_SIZE / 2.0,
            BALL_SIZE / 2.0,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        context.fill();
        
        // Draw right ball (black)
        context.set_fill_style(&"#000000".into());
        context.begin_path();
        let _ = context.arc(
            self.state.right_ball.x + BALL_SIZE / 2.0,
            self.state.right_ball.y + BALL_SIZE / 2.0,
            BALL_SIZE / 2.0,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        context.fill();
    }
}
