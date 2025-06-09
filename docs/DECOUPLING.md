# Should I decouple display rate and game logic?

## Context

This document captures the decision-making process around whether to decouple game logic framerate from display framerate in our block breaker simulation server.

**Updated Context**: This is an **interactive simulation** where users can control global parameters (like speed) that affect all viewers simultaneously, not just a passive "digital aquarium."

## The Question

Should we implement a Minecraft-style architecture where:
- Game logic runs at fixed 60 FPS (consistent physics)
- Display updates run as fast as possible (variable framerate)

Or should we use a **Modified Coupled Approach** where:
- Game logic and display remain coupled
- But the framerate is globally adjustable via user interaction

Instead of our current approach where both are coupled at fixed 60 FPS.

## Analysis

### Current Implementation
```rust
loop {
    game.update();           // Game logic update
    serialize_and_send();    // Display update  
    thread::sleep(16ms);     // Both locked to 60 FPS
}
```

### Proposed Decoupled Implementation
```rust
// Game logic thread - FIXED timestep
thread::spawn(|| {
    loop {
        game.update();       // Always 60 FPS for consistent physics
        thread::sleep(16ms);
    }
});

// Display thread - VARIABLE framerate
thread::spawn(|| {
    loop {
        let game_state = game.to_json();  // Get current state
        send_to_clients(game_state);      // Send as fast as possible
        // No sleep - run as fast as CPU allows!
    }
});
```

### Proposed Modified Coupled Implementation
```rust
// Single thread with globally adjustable timing
let global_speed = Arc::new(Mutex::new(Duration::from_millis(16))); // Default 60 FPS

loop {
    let current_speed = *global_speed.lock().unwrap();
    if now.duration_since(last_update) >= current_speed {
        game.update();           // Physics update
        send_to_all_clients();   // Display update - everyone gets same state
        last_update = now;
    }
}

// When any user clicks speed button via WebSocket:
// *global_speed.lock().unwrap() = new_speed; // Global impact!
```

## Decision: USE MODIFIED COUPLED APPROACH

### Reasons For Modified Coupled Approach

#### 1. **Perfect for Interactive Global Control**
- Users want **global impact** - when one user changes speed, everyone sees it
- Coupled approach ensures **perfect synchronization** across all clients
- **Immediate response** - speed changes apply instantly to everyone
- **One physics calculation** serves all clients efficiently

#### 2. **Maintains Shared Experience**
- Everyone watches the **exact same simulation state**
- No risk of clients seeing different states at different times
- Acts like a **collaborative remote control** for the simulation
- Natural for "everyone tunes into the same show" concept

#### 3. **Simple and Reliable**
- **Single point of control** - one global speed setting
- **No complex threading** or synchronization issues
- **Easy to implement** - just make timer duration configurable
- **Predictable behavior** - easy to debug and maintain

#### 4. **Efficient Resource Usage**
- **One game update** per frame regardless of client count
- **Bandwidth scales predictably** with frame rate, not client count
- **No redundant calculations** or state management overhead

### Why Decoupling Would Be Wrong Here

#### 1. **Synchronization Nightmare**
- Physics at 60 FPS, display at variable rates = potential state inconsistencies
- Risk of users seeing **slightly different simulation states**
- Complex race conditions between physics and display threads

#### 2. **Defeats the Purpose**
- We want **global impact** - decoupling makes this much harder
- Would need complex synchronization to ensure all clients see same thing
- Added complexity with zero benefit for our use case

#### 3. **Overengineering**
- Decoupling solves problems we don't have (local rendering, input lag)
- Our bottleneck is network (WebSocket JSON), not CPU
- The "fast physics" benefit is meaningless when limited by network speed

### When Decoupling WOULD Make Sense

Decoupling is beneficial for:
- **Interactive games** with per-user input responsiveness requirements
- **Local rendering** where network isn't involved
- **Competitive gaming** where individual input lag matters
- **VR/AR applications** where high framerate prevents motion sickness
- **Client-side games** where each user has independent rendering needs

**Note**: Our use case has **global interactivity** (not per-user), which makes coupled approach ideal.

### Our Sweet Spot

Our **Modified Coupled** approach is **ideal** for a **collaborative interactive simulation** because it provides:
- ✅ **Perfect synchronization** across all clients
- ✅ **Immediate global responsiveness** to user interactions
- ✅ **Bandwidth efficiency** with predictable resource usage
- ✅ **Simple and reliable** architecture
- ✅ **Natural collaboration** - users affect shared experience
- ✅ **Easy to extend** with additional global controls (pause, reset, etc.)

## Implementation Strategy

### Phase 1: Add Global Speed Control
```rust
// Add to main.rs
let global_frame_duration = Arc::new(Mutex::new(Duration::from_millis(16)));

// Add WebSocket message handling for speed changes
match message {
    "speed_up" => {
        let mut duration = global_frame_duration.lock().unwrap();
        *duration = (*duration).saturating_sub(Duration::from_millis(2)); // Faster
    },
    "speed_down" => {
        let mut duration = global_frame_duration.lock().unwrap();
        *duration = (*duration).saturating_add(Duration::from_millis(2)); // Slower
    },
    // ... existing game state requests
}
```

### Phase 2: Add UI Controls
- Add speed up/down buttons to `static/index.html`
- Send WebSocket messages on button clicks
- Display current speed setting to users

### Phase 3: Enhanced Controls (Future)
- Global pause/resume
- Reset simulation
- Different simulation modes
- User voting on speed changes

## Alternative Optimizations to Consider

Instead of decoupling, complementary optimizations for our interactive use case:

1. **Adaptive Quality**: Send fewer updates to clients with slow connections
2. **Payload Compression**: Optimize JSON payload size and structure  
3. **Smart Caching**: Avoid re-serializing identical game states
4. **Connection Management**: Handle client disconnects and reconnects gracefully
5. **Delta Updates**: Send only changes instead of full game state
6. **User Feedback**: Show who initiated speed changes, current FPS, etc.
7. **Rate Limiting**: Prevent spam clicking of speed controls

## Conclusion

**Decision: Use Modified Coupled Approach with Global Speed Control.**

This architecture is **perfectly suited** for our **interactive collaborative simulation** and provides the optimal balance of:
- **User experience** (responsive global controls, perfect sync)
- **Resource efficiency** (predictable bandwidth, single physics calculation)
- **Reliability** (simple architecture, easy to debug)
- **Maintainability** (clean separation of concerns, extensible)
- **Collaboration** (shared experience, global impact)

The traditional decoupling pattern is powerful for individual interactive experiences, but our **global collaborative** use case benefits much more from the modified coupled approach.

**Next Steps:**
1. Implement global speed control WebSocket messages
2. Add speed control buttons to the UI
3. Test with multiple concurrent users
4. Consider additional global controls (pause, reset, etc.)

---

*Decision updated: June 8, 2025*
*Context: Block Breaker simulation server - **interactive collaborative experience***
