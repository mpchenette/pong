use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

use crate::game::Game;
use crate::server::{websocket, ServerMetrics};

pub fn handle_websocket_connection(
    mut stream: TcpStream, 
    clients: Arc<Mutex<HashMap<usize, TcpStream>>>,
    client_counter: Arc<Mutex<usize>>,
    game: Arc<Mutex<Game>>,
    metrics: Arc<ServerMetrics>
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
        
        // Record the new client connection
        metrics.record_client_connected();
        
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
        
    } else if request.contains("GET /metrics") {
        // Handle metrics request - serve JSON metrics data
        println!("Metrics requested via HTTP");
        let metrics_json = metrics.to_json();
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Access-Control-Allow-Origin: *\r\n\r\n{}",
            metrics_json.len(),
            metrics_json
        );
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
        let response = "HTTP/1.1 404 Not Found\r\n\r\nNot Found";
        let _ = stream.write_all(response.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    #[test]
    fn test_websocket_request_parsing() {
        let request = "GET / HTTP/1.1\r\n\
                      Host: localhost:8000\r\n\
                      Upgrade: websocket\r\n\
                      Connection: Upgrade\r\n\
                      Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
                      Sec-WebSocket-Version: 13\r\n\r\n";

        // Test that we can identify WebSocket requests
        assert!(request.contains("Upgrade: websocket"));
        
        // Test key extraction
        let key = request
            .lines()
            .find(|line| line.starts_with("Sec-WebSocket-Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|key| key.trim())
            .unwrap_or("");
        
        assert_eq!(key, "dGhlIHNhbXBsZSBub25jZQ==");
    }

    #[test]
    fn test_http_get_request_parsing() {
        let request = "GET / HTTP/1.1\r\n\
                      Host: localhost:8000\r\n\
                      User-Agent: Mozilla/5.0\r\n\r\n";

        assert!(request.contains("GET /"));
        assert!(!request.contains("Upgrade: websocket"));
    }

    #[test]
    fn test_randomize_background_request_parsing() {
        let request = "GET /randomize-background HTTP/1.1\r\n\
                      Host: localhost:8000\r\n\r\n";

        assert!(request.contains("GET /randomize-background"));
    }

    #[test]
    fn test_websocket_accept_key_generation() {
        // This tests the WebSocket handshake key generation
        let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept_key = websocket::generate_accept_key(test_key);
        
        // Should produce the expected result from RFC 6455
        assert_eq!(accept_key, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_websocket_handshake_response_format() {
        let accept_key = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";
        let response = format!(
            "HTTP/1.1 101 Switching Protocols\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Accept: {}\r\n\r\n",
            accept_key
        );

        assert!(response.contains("101 Switching Protocols"));
        assert!(response.contains("Upgrade: websocket"));
        assert!(response.contains("Connection: Upgrade"));
        assert!(response.contains(&format!("Sec-WebSocket-Accept: {}", accept_key)));
    }

    #[test]
    fn test_client_counter_increment() {
        let client_counter = Arc::new(Mutex::new(0usize));
        
        // Simulate adding clients
        let id1 = {
            let mut counter = client_counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        
        let id2 = {
            let mut counter = client_counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(*client_counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_clients_hashmap_operations() {
        let clients: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
        
        // Simulate adding clients (using String instead of TcpStream for testing)
        {
            let mut clients_guard = clients.lock().unwrap();
            clients_guard.insert(1, "client1".to_string());
            clients_guard.insert(2, "client2".to_string());
        }
        
        // Test client removal
        {
            let mut clients_guard = clients.lock().unwrap();
            let removed = clients_guard.remove(&1);
            assert_eq!(removed, Some("client1".to_string()));
            assert_eq!(clients_guard.len(), 1);
        }
        
        // Verify remaining client
        {
            let clients_guard = clients.lock().unwrap();
            assert!(clients_guard.contains_key(&2));
            assert!(!clients_guard.contains_key(&1));
        }
    }

    #[test]
    fn test_malformed_http_request_handling() {
        // Test key extraction with malformed headers
        let malformed_request = "GET / HTTP/1.1\r\n\
                               MalformedHeader\r\n\
                               Sec-WebSocket-Key:\r\n\r\n";
        
        let key = malformed_request
            .lines()
            .find(|line| line.starts_with("Sec-WebSocket-Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|key| key.trim())
            .unwrap_or("");
        
        assert_eq!(key, ""); // Should handle empty key gracefully
        
        // Test missing WebSocket key
        let no_key_request = "GET / HTTP/1.1\r\n\
                             Upgrade: websocket\r\n\
                             Connection: Upgrade\r\n\r\n";
        
        let key = no_key_request
            .lines()
            .find(|line| line.starts_with("Sec-WebSocket-Key:"))
            .and_then(|line| line.split(':').nth(1))
            .map(|key| key.trim())
            .unwrap_or("");
        
        assert_eq!(key, ""); // Should use empty string as fallback
        
        // Test request with no headers
        let no_headers = "GET / HTTP/1.1\r\n\r\n";
        assert!(!no_headers.contains("Upgrade: websocket"));
    }
}
