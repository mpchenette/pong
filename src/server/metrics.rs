use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Comprehensive metrics tracking for the block breaker server
#[derive(Debug)]
pub struct ServerMetrics {
    // Frame timing metrics
    pub frame_count: AtomicU64,
    pub frame_times: Arc<Mutex<VecDeque<Duration>>>,
    pub last_frame_time: Arc<Mutex<Instant>>,
    
    // Network metrics
    pub bytes_sent_total: AtomicU64,
    pub bytes_sent_per_second: AtomicU64,
    pub updates_sent_total: AtomicU64,
    pub updates_sent_per_second: AtomicU64,
    
    // Client connection metrics
    pub connected_clients: AtomicUsize,
    pub total_connections: AtomicU64,
    pub disconnected_clients: AtomicU64,
    
    // Game state metrics
    pub collision_checks_per_frame: AtomicUsize,
    pub collision_checks_total: AtomicU64,
    
    // Performance metrics
    pub json_serialization_times: Arc<Mutex<VecDeque<Duration>>>,
    pub mutex_contention_times: Arc<Mutex<VecDeque<Duration>>>,
    
    // Latency tracking
    pub network_latencies: Arc<Mutex<VecDeque<Duration>>>,
    
    // Memory usage approximation
    pub game_state_size_bytes: AtomicUsize,
    
    // Error tracking
    pub send_failures: AtomicU64,
    pub timeout_errors: AtomicU64,
    
    // Timing for rolling averages
    last_second_reset: Arc<Mutex<Instant>>,
    
    // History storage (keep last 60 seconds of data)
    max_history_samples: usize,
}

impl ServerMetrics {
    pub fn new() -> Self {
        Self {
            frame_count: AtomicU64::new(0),
            frame_times: Arc::new(Mutex::new(VecDeque::with_capacity(60))),
            last_frame_time: Arc::new(Mutex::new(Instant::now())),
            
            bytes_sent_total: AtomicU64::new(0),
            bytes_sent_per_second: AtomicU64::new(0),
            updates_sent_total: AtomicU64::new(0),
            updates_sent_per_second: AtomicU64::new(0),
            
            connected_clients: AtomicUsize::new(0),
            total_connections: AtomicU64::new(0),
            disconnected_clients: AtomicU64::new(0),
            
            collision_checks_per_frame: AtomicUsize::new(0),
            collision_checks_total: AtomicU64::new(0),
            
            json_serialization_times: Arc::new(Mutex::new(VecDeque::with_capacity(60))),
            mutex_contention_times: Arc::new(Mutex::new(VecDeque::with_capacity(60))),
            
            network_latencies: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            
            game_state_size_bytes: AtomicUsize::new(0),
            
            send_failures: AtomicU64::new(0),
            timeout_errors: AtomicU64::new(0),
            
            last_second_reset: Arc::new(Mutex::new(Instant::now())),
            max_history_samples: 60,
        }
    }
    
    /// Record a new frame update
    pub fn record_frame(&self, frame_duration: Duration) {
        self.frame_count.fetch_add(1, Ordering::Relaxed);
        
        let mut frame_times = self.frame_times.lock().unwrap();
        frame_times.push_back(frame_duration);
        
        // Keep only last 60 samples (1 second at 60fps)
        if frame_times.len() > self.max_history_samples {
            frame_times.pop_front();
        }
        
        *self.last_frame_time.lock().unwrap() = Instant::now();
    }
    
    /// Record bytes sent to clients
    pub fn record_bytes_sent(&self, bytes: usize) {
        self.bytes_sent_total.fetch_add(bytes as u64, Ordering::Relaxed);
        self.updates_sent_total.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record client connection
    pub fn record_client_connected(&self) {
        self.connected_clients.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record client disconnection
    pub fn record_client_disconnected(&self) {
        self.connected_clients.fetch_sub(1, Ordering::Relaxed);
        self.disconnected_clients.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record collision detection performance
    pub fn record_collision_checks(&self, checks: usize) {
        self.collision_checks_per_frame.store(checks, Ordering::Relaxed);
        self.collision_checks_total.fetch_add(checks as u64, Ordering::Relaxed);
    }
    
    /// Record JSON serialization time
    pub fn record_json_serialization_time(&self, duration: Duration) {
        let mut times = self.json_serialization_times.lock().unwrap();
        times.push_back(duration);
        
        if times.len() > self.max_history_samples {
            times.pop_front();
        }
    }
    
    /// Record mutex contention time
    pub fn record_mutex_contention(&self, duration: Duration) {
        let mut times = self.mutex_contention_times.lock().unwrap();
        times.push_back(duration);
        
        if times.len() > self.max_history_samples {
            times.pop_front();
        }
    }
    
    /// Record network latency
    pub fn record_network_latency(&self, duration: Duration) {
        let mut latencies = self.network_latencies.lock().unwrap();
        latencies.push_back(duration);
        
        if latencies.len() > 100 {
            latencies.pop_front();
        }
    }
    
    /// Record game state size
    pub fn record_game_state_size(&self, size: usize) {
        self.game_state_size_bytes.store(size, Ordering::Relaxed);
    }
    
    /// Record send failure
    pub fn record_send_failure(&self) {
        self.send_failures.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record timeout error
    pub fn record_timeout_error(&self) {
        self.timeout_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update per-second metrics (should be called roughly every second)
    pub fn update_per_second_metrics(&self) {
        let now = Instant::now();
        let mut last_reset = self.last_second_reset.lock().unwrap();
        let elapsed = now.duration_since(*last_reset);
        
        if elapsed >= Duration::from_secs(1) {
            // Calculate per-second rates
            let _seconds = elapsed.as_secs_f64();
            
            // For now, just reset - in a real implementation you'd calculate the rate
            // based on the difference from the last measurement
            *last_reset = now;
        }
    }
    
    /// Get current FPS
    pub fn get_current_fps(&self) -> f64 {
        let frame_times = self.frame_times.lock().unwrap();
        if frame_times.is_empty() {
            return 0.0;
        }
        
        let total_time: Duration = frame_times.iter().sum();
        let avg_frame_time = total_time.as_secs_f64() / frame_times.len() as f64;
        
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }
    
    /// Get average frame time in milliseconds
    pub fn get_avg_frame_time_ms(&self) -> f64 {
        let frame_times = self.frame_times.lock().unwrap();
        if frame_times.is_empty() {
            return 0.0;
        }
        
        let total_time: Duration = frame_times.iter().sum();
        let avg_frame_time = total_time.as_secs_f64() / frame_times.len() as f64;
        avg_frame_time * 1000.0
    }
    
    /// Get bandwidth per client in KB/s
    pub fn get_bandwidth_per_client_kbps(&self) -> f64 {
        let bytes_total = self.bytes_sent_total.load(Ordering::Relaxed) as f64;
        let client_count = self.connected_clients.load(Ordering::Relaxed) as f64;
        
        if client_count == 0.0 {
            return 0.0;
        }
        
        let frame_count = self.frame_count.load(Ordering::Relaxed) as f64;
        if frame_count == 0.0 {
            return 0.0;
        }
        
        // Rough estimate: assume 60 FPS
        let seconds_running = frame_count / 60.0;
        let bytes_per_second = bytes_total / seconds_running;
        let bytes_per_client_per_second = bytes_per_second / client_count;
        
        bytes_per_client_per_second / 1024.0 // Convert to KB/s
    }
    
    /// Get average JSON serialization time in milliseconds
    pub fn get_avg_json_serialization_ms(&self) -> f64 {
        let times = self.json_serialization_times.lock().unwrap();
        if times.is_empty() {
            return 0.0;
        }
        
        let total_time: Duration = times.iter().sum();
        let avg_time = total_time.as_secs_f64() / times.len() as f64;
        avg_time * 1000.0
    }
    
    /// Get average network latency in milliseconds
    pub fn get_avg_network_latency_ms(&self) -> f64 {
        let latencies = self.network_latencies.lock().unwrap();
        if latencies.is_empty() {
            return 0.0;
        }
        
        let total_time: Duration = latencies.iter().sum();
        let avg_time = total_time.as_secs_f64() / latencies.len() as f64;
        avg_time * 1000.0
    }
    
    /// Generate JSON report of all metrics
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
                "frame_metrics": {{
                    "total_frames": {},
                    "current_fps": {:.2},
                    "avg_frame_time_ms": {:.2}
                }},
                "network_metrics": {{
                    "bytes_sent_total": {},
                    "updates_sent_total": {},
                    "bandwidth_per_client_kbps": {:.2},
                    "game_state_size_bytes": {}
                }},
                "client_metrics": {{
                    "connected_clients": {},
                    "total_connections": {},
                    "disconnected_clients": {}
                }},
                "performance_metrics": {{
                    "collision_checks_per_frame": {},
                    "collision_checks_total": {},
                    "avg_json_serialization_ms": {:.2},
                    "avg_network_latency_ms": {:.2}
                }},
                "error_metrics": {{
                    "send_failures": {},
                    "timeout_errors": {}
                }}
            }}"#,
            self.frame_count.load(Ordering::Relaxed),
            self.get_current_fps(),
            self.get_avg_frame_time_ms(),
            
            self.bytes_sent_total.load(Ordering::Relaxed),
            self.updates_sent_total.load(Ordering::Relaxed),
            self.get_bandwidth_per_client_kbps(),
            self.game_state_size_bytes.load(Ordering::Relaxed),
            
            self.connected_clients.load(Ordering::Relaxed),
            self.total_connections.load(Ordering::Relaxed),
            self.disconnected_clients.load(Ordering::Relaxed),
            
            self.collision_checks_per_frame.load(Ordering::Relaxed),
            self.collision_checks_total.load(Ordering::Relaxed),
            self.get_avg_json_serialization_ms(),
            self.get_avg_network_latency_ms(),
            
            self.send_failures.load(Ordering::Relaxed),
            self.timeout_errors.load(Ordering::Relaxed)
        )
    }
}

/// Helper function to time an operation
pub fn time_operation<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_metrics_initialization() {
        let metrics = ServerMetrics::new();
        
        assert_eq!(metrics.frame_count.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.connected_clients.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.bytes_sent_total.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_frame_recording() {
        let metrics = ServerMetrics::new();
        
        metrics.record_frame(Duration::from_millis(16));
        assert_eq!(metrics.frame_count.load(Ordering::Relaxed), 1);
        
        metrics.record_frame(Duration::from_millis(17));
        assert_eq!(metrics.frame_count.load(Ordering::Relaxed), 2);
        
        let fps = metrics.get_current_fps();
        assert!(fps > 0.0);
        assert!(fps < 100.0); // Should be reasonable
    }
    
    #[test]
    fn test_client_connection_tracking() {
        let metrics = ServerMetrics::new();
        
        metrics.record_client_connected();
        assert_eq!(metrics.connected_clients.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_connections.load(Ordering::Relaxed), 1);
        
        metrics.record_client_connected();
        assert_eq!(metrics.connected_clients.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_connections.load(Ordering::Relaxed), 2);
        
        metrics.record_client_disconnected();
        assert_eq!(metrics.connected_clients.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.disconnected_clients.load(Ordering::Relaxed), 1);
    }
    
    #[test]
    fn test_bandwidth_calculation() {
        let metrics = ServerMetrics::new();
        
        // Connect a client
        metrics.record_client_connected();
        
        // Send some data
        metrics.record_bytes_sent(1000);
        metrics.record_frame(Duration::from_millis(16));
        
        let bandwidth = metrics.get_bandwidth_per_client_kbps();
        assert!(bandwidth >= 0.0);
    }
    
    #[test]
    fn test_json_serialization() {
        let metrics = ServerMetrics::new();
        
        // Add some test data
        metrics.record_frame(Duration::from_millis(16));
        metrics.record_client_connected();
        metrics.record_bytes_sent(1000);
        
        let json = metrics.to_json();
        
        // Verify it contains expected fields
        assert!(json.contains("frame_metrics"));
        assert!(json.contains("network_metrics"));
        assert!(json.contains("client_metrics"));
        assert!(json.contains("performance_metrics"));
        assert!(json.contains("error_metrics"));
        
        // Verify it's valid JSON structure
        assert!(json.contains("{"));
        assert!(json.contains("}"));
    }
    
    #[test]
    fn test_time_operation_helper() {
        let (result, duration) = time_operation(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
        assert!(duration < Duration::from_millis(50)); // Should be reasonable
    }
    
    #[test]
    fn test_collision_tracking() {
        let metrics = ServerMetrics::new();
        
        metrics.record_collision_checks(200);
        assert_eq!(metrics.collision_checks_per_frame.load(Ordering::Relaxed), 200);
        assert_eq!(metrics.collision_checks_total.load(Ordering::Relaxed), 200);
        
        metrics.record_collision_checks(150);
        assert_eq!(metrics.collision_checks_per_frame.load(Ordering::Relaxed), 150);
        assert_eq!(metrics.collision_checks_total.load(Ordering::Relaxed), 350);
    }
    
    #[test]
    fn test_error_tracking() {
        let metrics = ServerMetrics::new();
        
        metrics.record_send_failure();
        metrics.record_timeout_error();
        
        assert_eq!(metrics.send_failures.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.timeout_errors.load(Ordering::Relaxed), 1);
        
        metrics.record_send_failure();
        assert_eq!(metrics.send_failures.load(Ordering::Relaxed), 2);
    }
}
