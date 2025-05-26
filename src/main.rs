use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::net::TcpStream;

use block_breaker::game::Game;
use block_breaker::server::{handle_websocket_connection, send_text_frame};

fn main() {
    let game = Arc::new(Mutex::new(Game::new()));
    let clients: Arc<Mutex<HashMap<usize, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
    let client_counter = Arc::new(Mutex::new(0usize));
    
    let game_clone = Arc::clone(&game);
    let clients_clone = Arc::clone(&clients);

    // Start game loop in separate thread
    thread::spawn(move || {
        let mut last_update = Instant::now();
        loop {
            let now = Instant::now();
            if now.duration_since(last_update) >= Duration::from_millis(16) {
                // ~60 FPS
                game_clone.lock().unwrap().update();
                
                // Send game state to all connected clients
                let game_state = game_clone.lock().unwrap().to_json();
                let mut clients_guard = clients_clone.lock().unwrap();
                let mut disconnected_clients = Vec::new();
                
                for (client_id, stream) in clients_guard.iter_mut() {
                    // Set a short timeout for writing
                    if let Ok(_) = stream.set_write_timeout(Some(Duration::from_millis(10))) {
                        if let Err(_) = send_text_frame(stream, &game_state) {
                            disconnected_clients.push(*client_id);
                        }
                    } else {
                        disconnected_clients.push(*client_id);
                    }
                }
                
                // Remove disconnected clients
                for client_id in disconnected_clients {
                    clients_guard.remove(&client_id);
                    println!("Client {} disconnected", client_id);
                }
                
                last_update = now;
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    // Start WebSocket server
    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();
    println!("WebSocket server running on ws://0.0.0.0:8000");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let clients_clone = Arc::clone(&clients);
        let client_counter_clone = Arc::clone(&client_counter);
        let game_clone = Arc::clone(&game);

        thread::spawn(move || {
            handle_websocket_connection(stream, clients_clone, client_counter_clone, game_clone);
        });
    }
}
