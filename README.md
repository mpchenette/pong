# Rust Pong Game

A classic Pong game implemented in Rust using the Yew framework, compiled to WebAssembly for the web.

## Features

- **Pure Rust Implementation**: Built with Rust and compiled to WebAssembly
- **Modern Web Framework**: Uses Yew for reactive UI components
- **Smooth Gameplay**: 60 FPS game loop with canvas rendering
- **Two-Player Controls**: 
  - Left Player: W/S keys
  - Right Player: Arrow Up/Down keys
- **Game Features**:
  - Real-time scoring
  - Pause/Resume functionality
  - Game reset
  - Responsive design
  - Beautiful modern UI

## Quick Start

1. **Install Dependencies** (if not already installed):
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install wasm-pack
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. **Run the Game**:
   ```bash
   ./run.sh
   ```
   
   This will:
   - Build the Rust project to WebAssembly
   - Start a local development server
   - Open the game at `http://localhost:8000`

3. **Manual Build** (alternative):
   ```bash
   # Build the project
   wasm-pack build --target web --out-dir pkg
   
   # Serve the files (choose one)
   python3 -m http.server 8000
   # or
   npx serve .
   # or use any static file server
   ```

## How to Play

- **Left Player**: Use `W` and `S` keys to move the paddle up and down
- **Right Player**: Use `↑` and `↓` arrow keys to move the paddle up and down
- **Pause**: Click the "Pause" button or press `Space`
- **Reset**: Click the "Reset" button to start a new game
- **Scoring**: First to score wins the point, game continues indefinitely

## Project Structure

```
├── src/
│   ├── main.rs          # Main application entry point
│   └── game.rs          # Pong game logic and rendering
├── index.html           # HTML template
├── styles.css           # Modern CSS styling
├── Cargo.toml          # Rust dependencies
└── run.sh              # Build and run script
```

## Technology Stack

- **Rust**: Core game logic and WebAssembly compilation
- **Yew**: Modern Rust framework for building web applications
- **WebAssembly**: Compiles Rust to run in the browser
- **Canvas API**: 2D rendering for smooth game graphics
- **CSS3**: Modern styling with gradients and animations

## Development

To modify the game:

1. Edit the Rust source files in `src/`
2. Rebuild with `wasm-pack build --target web --out-dir pkg`
3. Refresh your browser

## Browser Compatibility

Works in all modern browsers that support WebAssembly:
- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

Enjoy playing Pong built with Rust! 🦀🏓