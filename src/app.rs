use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use sycamore::rt::Event;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    let name = create_signal(cx, String::new());
    let greet_msg = create_signal(cx, String::new());

    let greet = move |e: Event| {
        e.prevent_default();
        spawn_local_scoped(cx, async move {
            // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
            let new_msg =
                invoke("greet", to_value(&GreetArgs { name: &name.get() }).unwrap()).await;

            log(&new_msg.as_string().unwrap());

            greet_msg.set(new_msg.as_string().unwrap());
        })
    };

    view! { cx,
        main(class="container") {
            h1() { "Rust Spectra Reader" }
            div(class="row") {
                i(class="fa fa-file-code-o", style="font-size:200px;")
            }

            p {
                "Locate the Spectro file or copy and paste above."
            }

            form(class="row",on:submit=greet) {
                input(id="greet-input",bind:value=name,placeholder="Select one text file...")
                button(type="submit") {
                    "Search"
                }
            }
            p {
                b {
                    (greet_msg.get())
                }
            }
        }
    }
}
