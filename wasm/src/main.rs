#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
extern crate wasm_bindgen;

extern crate gloo_events;
extern crate gloo_net;
extern crate gloo_storage;
extern crate gloo_utils;
extern crate slint;

use alloc::string::String;

use gloo_utils::{document, window};
use slint::{ModelRc, SharedString};
use wasm_bindgen::prelude::*;

use gloo_net::http::Request;

// Include Slint's generated code
slint::include_modules!();

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsError> {
    document().get_element_by_id("loading").unwrap().remove();
    // match Request::get("/api/status").send().await {
    //     Ok(res) => {
    //         // res.json().await?;
    //         window()
    //             .document()
    //             .unwrap()
    //             .append_with_str_1(&res.text().await?);
    //     }
    //     Err(e) => {}
    // }

    let status = if let Ok(res) = Request::get("/api/status").send().await {
        res.json().await.unwrap()
    } else {
        scouting_v3::Status {
            version: String::from(env!("CARGO_PKG_VERSION")),
            enabled: true,
            managed: false,
            event: String::from("[offline]"),
        }
    };

    let sc = ScoutingApp::new().unwrap();

    sc.set_enabled(status.enabled);
    sc.set_event(SharedString::from(status.event));

    // XXX: In the Slint code, this is defined as `{ width, height }`; here it's the reverse.
    // Why tf is Slint doing this?
    sc.set_dimensions((
        unsafe {
            window()
                .inner_height()
                .unwrap()
                .as_f64()
                .unwrap()
                .to_int_unchecked::<i32>()
        },
        unsafe {
            window()
                .inner_width()
                .unwrap()
                .as_f64()
                .unwrap()
                .to_int_unchecked::<i32>()
        },
    ));

    sc.run().unwrap();

    Ok(())
}
