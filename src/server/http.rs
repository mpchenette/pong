use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

use crate::game::Game;
use crate::server::websocket;

pub fn handle_websocket_connection(
    mut stream: TcpStream, 
    clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
    client_counter: Arc<Mutex<usize>>,
    game: Arc<Mutex<Game>>
) {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Received request: {}", request.lines().next().unwrap_or(""));
    
    // Check if it's a WebSocket upgrade request
    if request.contains("Upgrade: websocket") {
        println!("WebSocket upgrade request detected");
        // Extract the Sec-WebSocket-Key
        let key = request
            .lines()
            .find(|line| line.starts_with("Sec-WebSocket-Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|key| key.trim())
            .unwrap_or("");

        let accept_key = websocket::generate_accept_key(key);
        
        // Send WebSocket handshake response
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );
        
        if stream.write_all(response.as_bytes()).is_err() {
            println!("Failed to send WebSocket handshake response");
            return;
        }
        
        if stream.flush().is_err() {
            println!("Failed to flush WebSocket handshake response");
            return;
        }
        
        println!("WebSocket handshake completed successfully");
        
        // Add client to the list
        let client_id = {
            let mut counter = client_counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        
        println!("Client {} connected via WebSocket", client_id);
        
        // Clone the stream for the clients map
        let stream_clone = match stream.try_clone() {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to clone stream: {}", e);
                return;
            }
        };
        
        clients.lock().unwrap().insert(client_id, stream_clone);
        
        // Send initial game state
        let game_state = "{}"; // Empty initial state
        if let Err(_) = websocket::send_text_frame(&mut stream, game_state) {
            println!("Failed to send initial game state to client {}", client_id);
        }
        
        // The game loop will handle ongoing communication
        // Keep the connection handler minimal
        thread::sleep(Duration::from_millis(100)); // Brief delay then exit handler
        
    } else if request.contains("GET /randomize-background") {
        // Handle background randomization request
        println!("Background randomization requested via HTTP");
        game.lock().unwrap().randomize_background();
        let response = "HTTP/1.1 200 OK\r\n\r\nBackground randomized";
        let _ = stream.write_all(response.as_bytes());
        
    } else if request.contains("GET /") && (request.contains("GET / ") || request.contains("GET /?")) {
        // Serve HTML page for regular HTTP requests
        println!("Serving HTML page");
        let html_content = include_str!("../../static/index.html");
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             Content-Length: {}\r\n\r\n{}",
            html_content.len(),
            html_content
        );
        let _ = stream.write_all(response.as_bytes());
    } else {
        // 404 for other requests
        println!("404 for request: {}", request.lines().next().unwrap_or(""));
        let response = "HTTP/1.1 404 Not Found\r\n\r\n404 Not Found";
        let _ = stream.write_all(response.as_bytes());
    }
}
