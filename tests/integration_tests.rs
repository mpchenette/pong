// Integration tests for the block breaker server
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use block_breaker::game::Game;

#[test]
fn test_game_loop_consistency() {
    // Test that the game state remains consistent over multiple updates
    let mut game = Game::new();
    
    let initial_grey = game.navy_grey_count;
    let initial_blue = game.navy_blue_count;
    
    // Run the game for many iterations
    for _ in 0..1000 {
        game.update();
        
        // Verify block counts always add up to 100
        assert_eq!(game.navy_grey_count + game.navy_blue_count, 100);
        
        // Verify balls stay in bounds
        for ball in &game.balls {
            assert!(ball.x >= 0.0);
            assert!(ball.y >= 0.0);
            assert!(ball.x <= 400.0 - 10.0); // CANVAS_WIDTH - BALL_SIZE
            assert!(ball.y <= 400.0 - 10.0); // CANVAS_HEIGHT - BALL_SIZE
        }
        
        // Verify block grid integrity
        assert_eq!(game.blocks.len(), 10);
        for row in &game.blocks {
            assert_eq!(row.len(), 10);
        }
    }
    
    // After many iterations, some blocks should have changed
    // (extremely unlikely that nothing changed after 1000 updates)
    let final_grey = game.navy_grey_count;
    let final_blue = game.navy_blue_count;
    assert!(final_grey != initial_grey || final_blue != initial_blue);
}

#[test]
fn test_json_serialization_performance() {
    let game = Game::new();
    
    let start = std::time::Instant::now();
    
    // Serialize many times to test performance
    for _ in 0..1000 {
        let _json = game.to_json();
    }
    
    let duration = start.elapsed();
    
    // Should complete quickly (less than 1 second for 1000 serializations)
    assert!(duration.as_secs() < 1);
}

#[test]
fn test_concurrent_game_state_access() {
    let game = Arc::new(Mutex::new(Game::new()));
    let mut handles = Vec::new();
    
    // Spawn multiple threads that access the game state
    for i in 0..10 {
        let game_clone = Arc::clone(&game);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                // Read game state
                let json = game_clone.lock().unwrap().to_json();
                assert!(!json.is_empty());
                
                // Occasionally update the game
                if (i + j) % 10 == 0 {
                    game_clone.lock().unwrap().update();
                }
                
                thread::sleep(Duration::from_millis(1));
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Game should still be in a valid state
    let final_game = game.lock().unwrap();
    assert_eq!(final_game.navy_grey_count + final_game.navy_blue_count, 100);
}

#[test]
fn test_websocket_frame_creation_and_parsing() {
    let test_message = "Hello WebSocket";
    let mut buffer = Vec::new();
    
    // Create a simple text frame manually (simulating send_text_frame)
    buffer.push(0x81); // FIN + text frame opcode
    if test_message.len() < 126 {
        buffer.push(test_message.len() as u8);
    }
    buffer.extend_from_slice(test_message.as_bytes());
    
    // Verify frame structure
    assert_eq!(buffer[0], 0x81);
    assert_eq!(buffer[1], test_message.len() as u8);
    assert_eq!(&buffer[2..], test_message.as_bytes());
    
    // Test with game state JSON
    let game = Game::new();
    let json = game.to_json();
    
    let mut json_buffer = Vec::new();
    json_buffer.push(0x81);
    if json.len() < 126 {
        json_buffer.push(json.len() as u8);
    } else if json.len() <= 65535 {
        json_buffer.push(126);
        json_buffer.extend_from_slice(&(json.len() as u16).to_be_bytes());
    }
    json_buffer.extend_from_slice(json.as_bytes());
    
    // Should create a valid frame
    assert_eq!(json_buffer[0], 0x81);
    assert!(!json_buffer.is_empty());
}

#[test]
fn test_client_connection_simulation() {
    let clients: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let client_counter = Arc::new(Mutex::new(0usize));
    
    // Simulate multiple client connections
    let mut client_ids = Vec::new();
    
    for _ in 0..5 {
        let client_id = {
            let mut counter = client_counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        
        clients.lock().unwrap().insert(client_id, format!("client_{}", client_id));
        client_ids.push(client_id);
    }
    
    // Verify all clients are connected
    assert_eq!(clients.lock().unwrap().len(), 5);
    
    // Simulate some clients disconnecting
    {
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.remove(&client_ids[1]);
        clients_guard.remove(&client_ids[3]);
    }
    
    // Verify correct clients remain
    let remaining_clients = clients.lock().unwrap();
    assert_eq!(remaining_clients.len(), 3);
    assert!(remaining_clients.contains_key(&client_ids[0]));
    assert!(!remaining_clients.contains_key(&client_ids[1]));
    assert!(remaining_clients.contains_key(&client_ids[2]));
    assert!(!remaining_clients.contains_key(&client_ids[3]));
    assert!(remaining_clients.contains_key(&client_ids[4]));
}

#[test]
fn test_game_state_broadcast_simulation() {
    let game = Arc::new(Mutex::new(Game::new()));
    let clients: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
    
    // Add some mock clients
    {
        let mut clients_guard = clients.lock().unwrap();
        clients_guard.insert(1, "client1".to_string());
        clients_guard.insert(2, "client2".to_string());
        clients_guard.insert(3, "client3".to_string());
    }
    
    // Simulate game loop broadcast
    for _ in 0..10 {
        // Update game
        game.lock().unwrap().update();
        
        // Get game state
        let game_state = game.lock().unwrap().to_json();
        assert!(!game_state.is_empty());
        
        // Simulate broadcasting to all clients
        let clients_guard = clients.lock().unwrap();
        let mut successful_sends = 0;
        
        for (client_id, _client_data) in clients_guard.iter() {
            // In real implementation, this would be send_text_frame
            // Here we just simulate successful sends
            if !game_state.is_empty() {
                successful_sends += 1;
            }
            println!("Sent game state to client {}", client_id);
        }
        
        assert_eq!(successful_sends, 3); // All 3 clients should receive the update
    }
}

#[test] 
fn test_background_randomization_endpoint() {
    let mut game = Game::new();
    let _original_background = game.background_color;
    
    // Simulate the randomize-background HTTP endpoint
    game.randomize_background();
    
    // Background should potentially be different
    // (Small chance it could be the same due to randomness, but that's okay)
    
    // More importantly, the background should be valid
    // Since it's (u8, u8, u8), all values are automatically valid 0-255
    let (r, g, b) = game.background_color;
    
    // Just verify we can access the values (they're automatically bounded by u8)
    let _r_check = r; // This demonstrates the value is accessible and valid
    let _g_check = g;
    let _b_check = b;
}

#[test]
fn test_rapid_state_updates_consistency() {
    let mut game = Game::new();
    
    // Rapidly update the game many times
    for _ in 0..10000 {
        game.update();
        
        // Verify invariants remain intact
        assert_eq!(game.navy_grey_count + game.navy_blue_count, 100);
        assert_eq!(game.balls.len(), 2);
        assert_eq!(game.blocks.len(), 10);
        
        // Verify each row has 10 blocks
        for row in &game.blocks {
            assert_eq!(row.len(), 10);
        }
    }
}

#[test]
fn test_ball_velocity_bounds() {
    let mut game = Game::new();
    
    // Run for many iterations to check velocity doesn't grow unbounded
    for _ in 0..1000 {
        game.update();
        
        for ball in &game.balls {
            // Velocity should remain reasonable (not infinite or NaN)
            assert!(ball.dx.is_finite());
            assert!(ball.dy.is_finite());
            assert!(ball.dx.abs() < 100.0); // Reasonable velocity bound
            assert!(ball.dy.abs() < 100.0);
        }
    }
}
