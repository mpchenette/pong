use yew::prelude::*;

mod game;

use game::BlockBreaker;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="app">
            <header class="header">
                <h1>{"Block Converter"}</h1>
                <p>{"Two balls compete to convert blocks to their color"}</p>
            </header>
            <main class="game-container">
                <BlockBreaker />
            </main>
            <footer class="footer">
                <p>{"Built with Rust and WebAssembly"}</p>
            </footer>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    yew::Renderer::<App>::new().render();
}
