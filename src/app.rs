use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[function_component(App)]
pub fn app() -> Html {
    let greet_input_ref = use_node_ref();
    let replays_directory_ref = use_node_ref();

    let name = use_state(|| String::new());
    let replays_directory = use_state(|| String::new());

    let greet_msg = use_state(|| String::new());
    {
        let greet_msg = greet_msg.clone();
        let name = name.clone();
        let name2 = name.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if name.is_empty() {
                        return;
                    }

                    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
                    let new_msg =
                        invoke("greet", to_value(&GreetArgs { name: &*name }).unwrap()).await;
                    log(&new_msg.as_string().unwrap());
                    greet_msg.set(new_msg.as_string().unwrap());
                });

                || {}
            },
            name2,
        );
    }

    let greet = {
        let name = name.clone();
        let greet_input_ref = greet_input_ref.clone();
        Callback::from(move |_| {
            name.set(
                greet_input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };

    let open_replays_directory = {
        // let replays_directory_ref = replays_directory_ref.clone();
        let replays_directory = replays_directory.clone();
        Callback::from(move |_| {
            let replays_directory = replays_directory.clone();
            spawn_local(async move {
                let directory = invoke("open_replays_directory", JsValue::UNDEFINED).await;
                if directory.is_null() {
                    return;
                }

                if let Some(directory) = directory.as_string() {
                    replays_directory.set(directory);
                }
            });
        })
    };

    html! {
        <main class="container">
            <div class="row">
                <input type="text" placeholder="WoWs Replay Directory" ref={replays_directory_ref} readonly=true value={(*replays_directory).clone()}/>
                <button type="button" onclick={open_replays_directory}>{"..."}</button>
            </div>

            <p>{"Click on the Tauri and Yew logos to learn more."}</p>

            <div class="row">
                <input id="greet-input" ref={greet_input_ref} placeholder="Enter a name..." />
                <button type="button" onclick={greet}>{"Greet"}</button>
            </div>

            <p><b>{ &*replays_directory}</b></p>
        </main>
    }
}
