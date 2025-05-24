# Block Converter Game

A simple block conversion game implemented in Rust using the Yew framework, compiled to WebAssembly for the web.

## What it is

Two balls bounce around a grid, each trying to convert blocks to their color:
- **White ball**: Converts blue blocks to grey
- **Black ball**: Converts grey blocks to blue

The game starts with a 10x10 grid split evenly between grey (left) and blue (right) blocks.

## Features

- **Pure Rust**: Built with Rust and compiled to WebAssembly
- **Modern Framework**: Uses Yew for reactive UI
- **Smooth Animation**: 60 FPS game loop with canvas rendering
- **Real-time Scoring**: Live count of each color's blocks
- **Simple Controls**: Just click Reset to restart

## Quick Start

1. **Install Dependencies**:
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
   
   This will build the project and start a server at `http://localhost:8000`

3. **Manual Build**:
   ```bash
   wasm-pack build --target web --out-dir pkg
   python3 -m http.server 8000
   ```

## Project Structure

```
├── src/
│   ├── lib.rs          # Main app component
│   └── game.rs         # Game logic and rendering
├── index.html          # Web page
├── styles.css          # Styling
├── Cargo.toml          # Rust dependencies
└── run.sh              # Build and run script
```

## Technology

- **Rust** - Systems programming language
- **Yew** - Modern Rust framework for web apps
- **WebAssembly** - High-performance web execution
- **Canvas API** - 2D graphics rendering
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