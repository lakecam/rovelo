//! WebAssembly bindings for the Rovelo web app. All diagram logic lives in the Rust crates;
//! the page passes the diagram as JSON and gets JSON (or text) back. See `api.rs`.
use wasm_bindgen::prelude::*;

mod api;
mod dates;
mod templates;

fn js(r: api::Res) -> Result<String, JsError> {
    r.map_err(|e| JsError::new(&e))
}

#[wasm_bindgen]
pub fn parse(text: &str) -> Result<String, JsError> {
    js(api::parse(text))
}

#[wasm_bindgen]
pub fn from_mermaid(text: &str) -> Result<String, JsError> {
    js(api::from_mermaid(text))
}

#[wasm_bindgen]
pub fn serialize(diagram_json: &str) -> Result<String, JsError> {
    js(api::serialize(diagram_json))
}

#[wasm_bindgen]
pub fn repair(diagram_json: &str) -> Result<String, JsError> {
    js(api::repair(diagram_json))
}

#[wasm_bindgen]
pub fn auto_layout(diagram_json: &str) -> Result<String, JsError> {
    js(api::auto_layout(diagram_json))
}

#[wasm_bindgen]
pub fn layout_unplaced(diagram_json: &str) -> Result<String, JsError> {
    js(api::layout_unplaced(diagram_json))
}

#[wasm_bindgen]
pub fn to_svg(diagram_json: &str) -> Result<String, JsError> {
    js(api::to_svg(diagram_json))
}

#[wasm_bindgen]
pub fn scene(diagram_json: &str) -> Result<String, JsError> {
    js(api::scene(diagram_json))
}

#[wasm_bindgen]
pub fn node_rect(diagram_json: &str, id: &str) -> Result<String, JsError> {
    js(api::node_rect(diagram_json, id))
}

#[wasm_bindgen]
pub fn node_size(diagram_json: &str, id: &str) -> Result<String, JsError> {
    js(api::node_size(diagram_json, id))
}

#[wasm_bindgen]
pub fn edge_path(diagram_json: &str, id: &str) -> Result<String, JsError> {
    js(api::edge_path(diagram_json, id))
}

#[wasm_bindgen]
pub fn group_rect(diagram_json: &str, id: &str) -> Result<String, JsError> {
    js(api::group_rect(diagram_json, id))
}

#[wasm_bindgen]
pub fn diagram_bounds(diagram_json: &str, margin: f32) -> Result<String, JsError> {
    js(api::diagram_bounds(diagram_json, margin))
}

#[wasm_bindgen]
pub fn brand_presets() -> Result<String, JsError> {
    js(api::brand_presets())
}

#[wasm_bindgen]
pub fn validate_brand(brand_json: &str) -> Result<String, JsError> {
    js(api::validate_brand(brand_json))
}

#[wasm_bindgen]
pub fn node_colors(brand_json: &str, role: &str) -> Result<String, JsError> {
    js(api::node_colors(brand_json, role))
}

#[wasm_bindgen]
pub fn icon_names() -> Result<String, JsError> {
    js(api::icon_names())
}

#[wasm_bindgen]
pub fn default_diagram() -> Result<String, JsError> {
    js(api::default_diagram())
}

/// `name`: `userflow`, `taskflow` or `blank`. `today`: `YYYY-MM-DD`. `brand_json` may be empty.
#[wasm_bindgen]
pub fn template(name: &str, today: &str, brand_json: &str) -> Result<String, JsError> {
    js(api::template(name, today, brand_json))
}
