use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::net::TcpStream;

use block_breaker::game::Game;
use block_breaker::server::{handle_websocket_connection, send_text_frame, ServerMetrics, time_operation};

fn main() {
    let game = Arc::new(Mutex::new(Game::new()));
    let clients: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let client_counter = Arc::new(Mutex::new(0usize));
    let metrics = Arc::new(ServerMetrics::new());
    
    // Global speed control - default 60 FPS, with reasonable limits
    let global_frame_duration = Arc::new(Mutex::new(Duration::from_millis(16))); // Default 60 FPS
    
    let game_clone = Arc::clone(&game);
    let clients_clone = Arc::clone(&clients);
    let metrics_clone = Arc::clone(&metrics);
    let frame_duration_clone = Arc::clone(&global_frame_duration);

    // Start game loop in separate thread
    thread::spawn(move || {
        let mut last_update = Instant::now();
        loop {
            let now = Instant::now();
            let current_frame_duration = *frame_duration_clone.lock().unwrap();
            if now.duration_since(last_update) >= current_frame_duration {
                // Variable FPS based on global setting
                
                // Calculate actual frame interval (time since last frame)
                let frame_interval = now.duration_since(last_update);
                
                // Time the game update
                let mutex_start = Instant::now();
                game_clone.lock().unwrap().update();
                let mutex_duration = mutex_start.elapsed();
                metrics_clone.record_mutex_contention(mutex_duration);
                
                // Time JSON serialization
                let (game_state, json_duration) = time_operation(|| {
                    game_clone.lock().unwrap().to_json()
                });
                metrics_clone.record_json_serialization_time(json_duration);
                metrics_clone.record_game_state_size(game_state.len());
                
                // Send game state to all connected clients
                let mut clients_guard = clients_clone.lock().unwrap();
                let mut disconnected_clients = Vec::new();
                let _client_count = clients_guard.len();
                
                for (client_id, stream) in clients_guard.iter_mut() {
                    // Set a short timeout for writing
                    if let Ok(_) = stream.set_write_timeout(Some(Duration::from_millis(10))) {
                        if let Err(_) = send_text_frame(stream, &game_state) {
                            disconnected_clients.push(*client_id);
                            metrics_clone.record_send_failure();
                        } else {
                            metrics_clone.record_bytes_sent(game_state.len());
                        }
                    } else {
                        disconnected_clients.push(*client_id);
                        metrics_clone.record_timeout_error();
                    }
                }
                
                // Remove disconnected clients
                for client_id in disconnected_clients {
                    clients_guard.remove(&client_id);
                    metrics_clone.record_client_disconnected();
                    println!("Client {} disconnected", client_id);
                }
                
                // Record collision checks (currently 2 balls * 100 blocks = 200 max)
                metrics_clone.record_collision_checks(2 * 100);
                
                // Record frame timing - use actual frame interval, not execution time
                metrics_clone.record_frame(frame_interval);
                
                // Update per-second metrics periodically
                metrics_clone.update_per_second_metrics();
                
                last_update = now;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Start WebSocket server
    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    println!("WebSocket server running on https://0.0.0.0:8000");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let clients_clone = Arc::clone(&clients);
        let client_counter_clone = Arc::clone(&client_counter);
        let game_clone = Arc::clone(&game);
        let metrics_clone = Arc::clone(&metrics);
        let frame_duration_clone = Arc::clone(&global_frame_duration);

        thread::spawn(move || {
            handle_websocket_connection(stream, clients_clone, client_counter_clone, game_clone, metrics_clone, frame_duration_clone);
        });
    }
}
