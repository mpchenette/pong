# Block Breaker Simulation

A server-side block breaker simulation written entirely in Rust using only the standard library.

## Overview

This project implements a unique twist on the classic block breaker game featuring:
- A 10x10 grid of blocks that fills the entire game area
- Two balls with different interaction rules
- Real-time color conversion mechanics
- Pure Rust implementation with no external dependencies

## Game Rules

### Grid Layout
- **10x10 grid** of blocks (400x400 pixels)
- **Left half (5 columns)**: Navy grey blocks
- **Right half (5 columns)**: Navy blue blocks

### Balls
- **White ball**: Starts on the left side, can only hit navy blue blocks
- **Black ball**: Starts on the right side, can only hit navy grey blocks

### Mechanics
- When a ball hits a compatible block, the block converts to the opposite color
- White ball converts navy blue → navy grey
- Black ball converts navy grey → navy blue
- Balls bounce off walls and blocks within the grid boundaries
- Running tally displays the current count of each block color

## Technical Details

### Architecture
- **Backend**: Pure Rust server using only `std` library
- **Frontend**: Vanilla HTML5 Canvas with JavaScript
- **Communication**: HTTP polling at 60 FPS
- **Concurrency**: Thread-safe game state with Arc<Mutex<>>

### Dependencies
- **None!** Uses only Rust's standard library
- No external crates or frameworks required

## Getting Started

### Prerequisites
- Rust (edition 2021 or later)

### Installation & Running

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd pong
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the server:
   ```bash
   cargo run
   ```

4. Open your browser and navigate to:
   ```
   http://127.0.0.1:3031
   ```

### Controls
- The simulation runs automatically
- Use the "Reset Game" button to restart the simulation
- Watch the block count tally update in real-time

## Project Structure

```
├── Cargo.toml              # Project configuration
├── src/
│   └── server.rs           # Main server implementation
├── static/
│   └── index.html          # Web interface
└── README.md               # This file
```

## API Endpoints

- `GET /` - Serves the HTML interface
- `GET /game-state` - Returns current game state as JSON
- `GET /reset` - Resets the game to initial state

## Game State JSON Format

```json
{
  "balls": [
    {
      "x": 100.0,
      "y": 200.0,
      "color": [255, 255, 255]
    }
  ],
  "blocks": [
    [
      {
        "x": 0.0,
        "y": 0.0,
        "color": [70, 80, 90]
      }
    ]
  ],
  "navy_grey_count": 50,
  "navy_blue_count": 50
}
```

## Features

- **Real-time simulation** at 60 FPS
- **Thread-safe** game state management
- **Manual JSON serialization** (no serde dependency)
- **Responsive web interface**
- **Pure standard library** implementation
- **Cross-platform** compatibility

## Building for Release

```bash
cargo build --release
```

The optimized binary will be available at `target/release/server`.

## Performance

- Game loop runs at 60 FPS (16ms intervals)
- HTTP server handles concurrent connections
- Minimal memory footprint with no external dependencies
- Efficient collision detection and state updates

## License

This project is open source. Feel free to modify and distribute as needed.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## Known Limitations

- Uses HTTP polling instead of WebSockets (by design for simplicity)
- No sound effects (keeping it minimal)
- Fixed grid size (10x10)

## Future Enhancements

Potential improvements while maintaining the pure Rust standard library constraint:
- Variable grid sizes
- Adjustable ball speeds
- Different ball physics
- Additional color schemes
- Performance optimizations



## BELOW WRITTEN BY ME< NOT COPILOT. INSTRUCITONS FOR COPILOT: DO NOT DELETE THIS SECTION OR FILE EVER>
- I like the idea of making this a "game" with button peoepl can pay to press. For example:
   - $1 to reset the game
   - $1 to increase the ball speed of one of the balls
   - $1 to change the colors to your choice (or from preapproved color pallete)
   