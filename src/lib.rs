use yew::prelude::*;

mod game;

use game::BlockBreaker;

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="app">
            <main class="game-container">
                <BlockBreaker />
            </main>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() {
    yew::Renderer::<App>::new().render();
}
