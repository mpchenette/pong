use yew::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use wasm_bindgen::{prelude::*, JsCast};
use gloo::timers::callback::Interval;

const CANVAS_WIDTH: f64 = 500.0;
const CANVAS_HEIGHT: f64 = 500.0;
const GRID_SIZE: usize = 10;
const BLOCK_SIZE: f64 = CANVAS_WIDTH / GRID_SIZE as f64;
const BALL_SIZE: f64 = 8.0;
const BALL_SPEED: f64 = 8.0;

#[derive(Clone, Copy, PartialEq)]
pub enum BlockType {
    Grey,
    Blue,
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
    pub left_ball: Ball,
    pub right_ball: Ball,
    pub blocks: Vec<Vec<Block>>,
    pub blue_blocks: usize,
    pub grey_blocks: usize,
}

impl Default for GameState {
    fn default() -> Self {
        let mut blocks = Vec::new();
        let mut blue_blocks = 0;
        let mut grey_blocks = 0;
        
        // Create 10x10 grid: left half grey, right half blue
        for y in 0..GRID_SIZE {
            let mut row = Vec::new();
            for x in 0..GRID_SIZE {
                let block_type = if x < GRID_SIZE / 2 {
                    grey_blocks += 1;
                    BlockType::Grey
                } else {
                    blue_blocks += 1;
                    BlockType::Blue
                };
                
                row.push(Block {
                    x: x as f64 * BLOCK_SIZE,
                    y: y as f64 * BLOCK_SIZE,
                    block_type,
                });
            }
            blocks.push(row);
        }
        
        // Create balls with random angles pointing toward opposite sides
        let left_angle = Self::random_angle(315.0, 45.0);
        let right_angle = Self::random_angle(135.0, 225.0);
        
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

impl GameState {
    fn random_angle(min_degrees: f64, max_degrees: f64) -> f64 {
        let range = if max_degrees < min_degrees {
            max_degrees + 360.0 - min_degrees
        } else {
            max_degrees - min_degrees
        };
        
        let angle_degrees = min_degrees + js_sys::Math::random() * range;
        (angle_degrees % 360.0) * std::f64::consts::PI / 180.0
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
                true
            }
            GameMsg::Reset => {
                self.state = GameState::default();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_reset = ctx.link().callback(|_| GameMsg::Reset);

        html! {
            <div class="game-container">
                <div class="game-info">
                    <div class="score">
                        <span style="color: #4a5568; font-size: 24px; font-weight: bold;">
                            {self.state.grey_blocks}
                        </span>
                        <span style="margin: 0 20px; color: #666;">{"vs"}</span>
                        <span style="color: #2d3748; font-size: 24px; font-weight: bold;">
                            {self.state.blue_blocks}
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
        self.update_ball(&mut self.state.left_ball.clone(), true);
        self.update_ball(&mut self.state.right_ball.clone(), false);
        self.update_block_counts();
    }
    
    fn update_ball(&mut self, ball: &mut Ball, is_left_ball: bool) {
        // Move ball
        ball.x += ball.dx;
        ball.y += ball.dy;
        
        // Bounce off walls
        if ball.y <= 0.0 || ball.y >= CANVAS_HEIGHT - BALL_SIZE {
            ball.dy = -ball.dy;
            ball.y = ball.y.clamp(0.0, CANVAS_HEIGHT - BALL_SIZE);
        }
        
        if ball.x <= 0.0 || ball.x >= CANVAS_WIDTH - BALL_SIZE {
            ball.dx = -ball.dx;
            ball.x = ball.x.clamp(0.0, CANVAS_WIDTH - BALL_SIZE);
        }
        
        // Check block collisions
        self.check_block_collision(ball, is_left_ball);
        
        // Update the ball in state
        if is_left_ball {
            self.state.left_ball = ball.clone();
        } else {
            self.state.right_ball = ball.clone();
        }
    }
    
    fn check_block_collision(&mut self, ball: &mut Ball, is_left_ball: bool) {
        let ball_center_x = ball.x + BALL_SIZE / 2.0;
        let ball_center_y = ball.y + BALL_SIZE / 2.0;
        
        let grid_x = (ball_center_x / BLOCK_SIZE) as usize;
        let grid_y = (ball_center_y / BLOCK_SIZE) as usize;
        
        if grid_x < GRID_SIZE && grid_y < GRID_SIZE {
            let block = &mut self.state.blocks[grid_y][grid_x];
            
            let should_convert = match (is_left_ball, block.block_type) {
                (true, BlockType::Blue) => true,   // Left ball converts blue to grey
                (false, BlockType::Grey) => true,  // Right ball converts grey to blue
                _ => false,
            };
            
            if should_convert {
                // Convert the block
                block.block_type = if is_left_ball {
                    BlockType::Grey
                } else {
                    BlockType::Blue
                };
                
                // Bounce the ball
                let block_center_x = block.x + BLOCK_SIZE / 2.0;
                let block_center_y = block.y + BLOCK_SIZE / 2.0;
                
                let dx = ball_center_x - block_center_x;
                let dy = ball_center_y - block_center_y;
                
                if dx.abs() > dy.abs() {
                    ball.dx = -ball.dx;
                    ball.x = if dx > 0.0 {
                        block.x + BLOCK_SIZE + 1.0
                    } else {
                        block.x - BALL_SIZE - 1.0
                    };
                } else {
                    ball.dy = -ball.dy;
                    ball.y = if dy > 0.0 {
                        block.y + BLOCK_SIZE + 1.0
                    } else {
                        block.y - BALL_SIZE - 1.0
                    };
                }
            }
        }
    }
    
    fn update_block_counts(&mut self) {
        let (mut grey_blocks, mut blue_blocks) = (0, 0);
        
        for row in &self.state.blocks {
            for block in row {
                match block.block_type {
                    BlockType::Grey => grey_blocks += 1,
                    BlockType::Blue => blue_blocks += 1,
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

        // Clear canvas
        context.set_fill_style(&"#000000".into());
        context.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);

        // Draw blocks
        for row in &self.state.blocks {
            for block in row {
                let color = match block.block_type {
                    BlockType::Grey => "#4a5568",
                    BlockType::Blue => "#2d3748",
                };
                
                context.set_fill_style(&color.into());
                context.fill_rect(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                
                // Add border
                context.set_stroke_style(&"#2a2a2a".into());
                context.set_line_width(1.0);
                context.stroke_rect(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
            }
        }

        // Draw balls
        self.draw_ball(&context, &self.state.left_ball, "#ffffff");
        self.draw_ball(&context, &self.state.right_ball, "#000000");
    }
    
    fn draw_ball(&self, context: &CanvasRenderingContext2d, ball: &Ball, color: &str) {
        context.set_fill_style(&color.into());
        context.begin_path();
        let _ = context.arc(
            ball.x + BALL_SIZE / 2.0,
            ball.y + BALL_SIZE / 2.0,
            BALL_SIZE / 2.0,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        context.fill();
    }
}
