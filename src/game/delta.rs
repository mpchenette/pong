use crate::game::types::*;

#[derive(Clone)]
pub struct GameDelta {
    pub ball_updates: Vec<BallUpdate>,
    pub block_changes: Vec<BlockChange>,
    pub count_change: Option<CountChange>,
    pub background_change: Option<(u8, u8, u8)>,
}

#[derive(Clone)]
pub struct BallUpdate {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone)]
pub struct BlockChange {
    pub row: usize,
    pub col: usize,
    pub color: BlockColor,
}

#[derive(Clone)]
pub struct CountChange {
    pub navy_grey_delta: i32,
    pub navy_blue_delta: i32,
}

impl Game {
    pub fn get_delta(&self, previous: &Game) -> GameDelta {
        let mut delta = GameDelta {
            ball_updates: Vec::new(),
            block_changes: Vec::new(),
            count_change: None,
            background_change: None,
        };

        // Check ball updates
        for (i, ball) in self.balls.iter().enumerate() {
            if let Some(prev_ball) = previous.balls.get(i) {
                if ball.x != prev_ball.x || ball.y != prev_ball.y || 
                   ball.dx != prev_ball.dx || ball.dy != prev_ball.dy {
                    delta.ball_updates.push(BallUpdate {
                        id: i,
                        x: ball.x,
                        y: ball.y,
                        dx: ball.dx,
                        dy: ball.dy,
                    });
                }
            }
        }

        // Check block changes
        for (row, block_row) in self.blocks.iter().enumerate() {
            if let Some(prev_row) = previous.blocks.get(row) {
                for (col, block) in block_row.iter().enumerate() {
                    if let Some(prev_block) = prev_row.get(col) {
                        if block.color != prev_block.color {
                            delta.block_changes.push(BlockChange {
                                row,
                                col,
                                color: block.color,
                            });
                        }
                    }
                }
            }
        }

        // Check count changes
        if self.navy_grey_count != previous.navy_grey_count || 
           self.navy_blue_count != previous.navy_blue_count {
            delta.count_change = Some(CountChange {
                navy_grey_delta: self.navy_grey_count as i32 - previous.navy_grey_count as i32,
                navy_blue_delta: self.navy_blue_count as i32 - previous.navy_blue_count as i32,
            });
        }

        // Check background changes
        if self.background_color != previous.background_color {
            delta.background_change = Some(self.background_color);
        }

        delta
    }

    pub fn delta_to_json(&self, delta: &GameDelta) -> String {
        let mut json = String::from("{");
        
        // Ball updates (typically ~60 bytes vs 200+ for full balls)
        if !delta.ball_updates.is_empty() {
            json.push_str("\"ball_updates\":[");
            for (i, update) in delta.ball_updates.iter().enumerate() {
                if i > 0 { json.push(','); }
                json.push_str(&format!(
                    "{{\"id\":{},\"x\":{:.1},\"y\":{:.1},\"dx\":{:.1},\"dy\":{:.1}}}",
                    update.id, update.x, update.y, update.dx, update.dy
                ));
            }
            json.push(']');
        }

        // Block changes (typically ~20-30 bytes vs 2000+ for full blocks)
        if !delta.block_changes.is_empty() {
            if !delta.ball_updates.is_empty() { json.push(','); }
            json.push_str("\"block_changes\":[");
            for (i, change) in delta.block_changes.iter().enumerate() {
                if i > 0 { json.push(','); }
                let color_str = match change.color {
                    BlockColor::NavyGrey => "\"navy_grey\"",
                    BlockColor::NavyBlue => "\"navy_blue\"",
                };
                json.push_str(&format!(
                    "{{\"row\":{},\"col\":{},\"color\":{}}}",
                    change.row, change.col, color_str
                ));
            }
            json.push(']');
        }

        // Count changes
        if let Some(count_change) = &delta.count_change {
            if !delta.ball_updates.is_empty() || !delta.block_changes.is_empty() { 
                json.push(','); 
            }
            json.push_str(&format!(
                "\"count_delta\":{{\"navy_grey\":{},\"navy_blue\":{}}}",
                count_change.navy_grey_delta, count_change.navy_blue_delta
            ));
        }

        // Background changes
        if let Some(bg) = delta.background_change {
            if !json.ends_with('{') { json.push(','); }
            json.push_str(&format!(
                "\"background_color\":[{},{},{}]",
                bg.0, bg.1, bg.2
            ));
        }

        json.push('}');
        json
    }
}
