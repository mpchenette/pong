use yew::prelude::*;

mod game;

use game::PongGame;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="app">
            <header class="header">
                <h1>{"Rust Pong Game"}</h1>
                <p>{"Use W/S keys for left paddle, ↑/↓ keys for right paddle"}</p>
            </header>
            <main class="game-container">
                <PongGame />
            </main>
            <footer class="footer">
                <p>{"Built with Rust and Yew"}</p>
            </footer>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
