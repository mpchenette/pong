# Should I decouple display rate and game logic?

## Context

This document captures the decision-making process around whether to decouple game logic framerate from display framerate in our block breaker simulation server.

## The Question

Should we implement a Minecraft-style architecture where:
- Game logic runs at fixed 60 FPS (consistent physics)
- Display updates run as fast as possible (variable framerate)

Instead of our current approach where both are coupled at 60 FPS.

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

## Decision: KEEP CURRENT COUPLED APPROACH

### Reasons Against Decoupling

#### 1. **Use Case Doesn't Benefit**
- This is a **passive viewing experience** (people "tune in" to watch)
- Users don't interact with the simulation
- 60 FPS is already buttery smooth for watching
- Going to 120+ FPS won't meaningfully improve viewing experience

#### 2. **Network is the Bottleneck**
- Sending JSON over WebSockets is much slower than game logic
- Higher framerate = more network traffic = potential lag/packet drops
- Many clients may be on mobile or slower connections
- Bandwidth efficiency is more important than CPU efficiency

#### 3. **Synchronization is Critical**
- Everyone needs to see the **exact same simulation state**
- Decoupling could cause clients to see slightly different states at different times
- Current approach ensures perfect synchronization across all viewers

#### 4. **Resource Concerns**
- **Bandwidth**: Higher framerate = more data consumption
- **Battery**: Mobile users would drain battery faster with high framerate updates
- **CPU**: Some clients might not be able to process updates fast enough

#### 5. **Simplicity = Reliability**
- Current approach is simple and robust
- Adding complexity increases chances of bugs and race conditions
- For a "tune in anytime" service, reliability is more important than raw performance
- Easier to debug and maintain

### When Decoupling WOULD Make Sense

Decoupling is beneficial for:
- **Interactive games** where input responsiveness matters
- **Local rendering** where network isn't involved
- **Competitive gaming** where every millisecond of latency counts
- **VR/AR applications** where high framerate prevents motion sickness
- **Client-side games** where rendering can be independent of game logic

### Our Sweet Spot

Our current 60 FPS coupled approach is **ideal** for a "digital aquarium" simulation because it's:
- ✅ Smooth enough for excellent viewing experience
- ✅ Bandwidth efficient
- ✅ Battery friendly for mobile viewers
- ✅ Perfectly synchronized across all clients
- ✅ Simple and reliable
- ✅ Predictable server resource usage

## Alternative Optimizations to Consider

Instead of decoupling, better optimizations for our use case would be:

1. **Adaptive Quality**: Send fewer updates to clients with slow connections
2. **Payload Compression**: Optimize JSON payload size and structure
3. **Smart Caching**: Avoid re-serializing identical game states
4. **Connection Management**: Handle client disconnects and reconnects gracefully
5. **Delta Updates**: Send only changes instead of full game state

## Conclusion

**Decision: Keep the current coupled 60 FPS approach.**

This architecture is well-suited for our passive viewing simulation and provides the best balance of:
- User experience (smooth viewing)
- Resource efficiency (bandwidth, battery, CPU)
- Reliability (simple, deterministic)
- Maintainability (fewer moving parts)

The Minecraft-style decoupling pattern is powerful for interactive games, but our use case doesn't benefit from its complexity.

---

*Decision made: June 8, 2025*
*Context: Block Breaker simulation server - passive viewing experience*
