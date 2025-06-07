use std::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use std::sync::Arc;
use crate::game::types::*;

// Lock-free optimizations to reduce mutex contention
// Current issue: Main game loop and all client threads compete for game state mutex

#[repr(C, align(64))] // Cache line alignment to prevent false sharing
pub struct AtomicGameCounters {
    pub navy_grey_count: AtomicU32,
    pub navy_blue_count: AtomicU32,
    pub frame_number: AtomicU64,
    pub last_update_ns: AtomicU64,
    pub active_clients: AtomicU32,
}

impl AtomicGameCounters {
    pub fn new() -> Self {
        AtomicGameCounters {
            navy_grey_count: AtomicU32::new(50),
            navy_blue_count: AtomicU32::new(50),
            frame_number: AtomicU64::new(0),
            last_update_ns: AtomicU64::new(0),
            active_clients: AtomicU32::new(0),
        }
    }

    pub fn update_counts(&self, grey_delta: i32, blue_delta: i32) {
        // Atomic operations are much faster than mutex locks
        if grey_delta != 0 {
            if grey_delta > 0 {
                self.navy_grey_count.fetch_add(grey_delta as u32, Ordering::Relaxed);
            } else {
                self.navy_grey_count.fetch_sub((-grey_delta) as u32, Ordering::Relaxed);
            }
        }
        
        if blue_delta != 0 {
            if blue_delta > 0 {
                self.navy_blue_count.fetch_add(blue_delta as u32, Ordering::Relaxed);
            } else {
                self.navy_blue_count.fetch_sub((-blue_delta) as u32, Ordering::Relaxed);
            }
        }
    }

    pub fn get_counts(&self) -> (u32, u32) {
        (
            self.navy_grey_count.load(Ordering::Relaxed),
            self.navy_blue_count.load(Ordering::Relaxed)
        )
    }

    pub fn increment_frame(&self) -> u64 {
        self.frame_number.fetch_add(1, Ordering::Relaxed)
    }

    pub fn update_timestamp(&self, timestamp_ns: u64) {
        self.last_update_ns.store(timestamp_ns, Ordering::Relaxed);
    }
}

// Triple-buffered game state to eliminate blocking
pub struct TripleBufferedGame {
    states: [Game; 3],
    current_read: AtomicU32,
    current_write: AtomicU32,
    next_write: AtomicU32,
    write_in_progress: AtomicBool,
}

impl TripleBufferedGame {
    pub fn new() -> Self {
        TripleBufferedGame {
            states: [Game::new(), Game::new(), Game::new()],
            current_read: AtomicU32::new(0),
            current_write: AtomicU32::new(1),
            next_write: AtomicU32::new(2),
            write_in_progress: AtomicBool::new(false),
        }
    }

    // Writer (game loop) gets exclusive access to write buffer
    pub fn start_write(&mut self) -> Option<&mut Game> {
        if self.write_in_progress.compare_exchange_weak(
            false, true, Ordering::Acquire, Ordering::Relaxed
        ).is_ok() {
            let write_idx = self.current_write.load(Ordering::Relaxed) as usize;
            Some(&mut self.states[write_idx])
        } else {
            None // Write already in progress, skip this frame
        }
    }

    // Writer finishes and swaps buffers
    pub fn finish_write(&mut self) {
        // Swap write and read buffers
        let old_read = self.current_read.load(Ordering::Relaxed);
        let old_write = self.current_write.load(Ordering::Relaxed);
        let old_next = self.next_write.load(Ordering::Relaxed);

        self.current_read.store(old_write, Ordering::Release);
        self.current_write.store(old_next, Ordering::Relaxed);
        self.next_write.store(old_read, Ordering::Relaxed);

        self.write_in_progress.store(false, Ordering::Release);
    }

    // Readers (client threads) get lock-free access to read buffer
    pub fn read(&self) -> &Game {
        let read_idx = self.current_read.load(Ordering::Acquire) as usize;
        &self.states[read_idx]
    }
}

// High-performance game loop with adaptive frame timing
pub struct AdaptiveGameLoop {
    target_frame_time: Duration,
    frame_times: [Duration; 60], // Rolling average
    frame_index: usize,
    skip_frame_threshold: Duration,
    counters: Arc<AtomicGameCounters>,
}

impl AdaptiveGameLoop {
    pub fn new(counters: Arc<AtomicGameCounters>) -> Self {
        AdaptiveGameLoop {
            target_frame_time: Duration::from_nanos(16_666_667), // ~60 FPS
            frame_times: [Duration::from_nanos(16_666_667); 60],
            frame_index: 0,
            skip_frame_threshold: Duration::from_nanos(33_333_333), // 30 FPS minimum
            counters,
        }
    }

    pub fn should_update(&mut self, last_update: std::time::Instant) -> bool {
        let elapsed = last_update.elapsed();
        
        // Record frame time for adaptive timing
        self.frame_times[self.frame_index] = elapsed;
        self.frame_index = (self.frame_index + 1) % 60;
        
        // Calculate average frame time
        let avg_frame_time = self.frame_times.iter().sum::<Duration>() / 60;
        
        // Adaptive timing: if we're running slow, skip frames to catch up
        if avg_frame_time > self.skip_frame_threshold {
            // Running too slow - update every other frame
            self.counters.frame_number.load(Ordering::Relaxed) % 2 == 0
        } else if elapsed >= self.target_frame_time {
            // Normal timing
            true
        } else {
            // Running ahead - no update needed
            false
        }
    }

    pub fn get_target_sleep_time(&self, frame_start: std::time::Instant) -> Duration {
        let frame_duration = frame_start.elapsed();
        if frame_duration < self.target_frame_time {
            self.target_frame_time - frame_duration
        } else {
            Duration::from_millis(0)
        }
    }
}

use std::time::Duration;

// Performance improvements:
// 1. Atomic operations are 10-100x faster than mutex locks
// 2. Triple buffering eliminates all blocking between game loop and client threads
// 3. Adaptive frame timing maintains performance under load
// 4. Cache line alignment prevents false sharing on multi-core CPUs
