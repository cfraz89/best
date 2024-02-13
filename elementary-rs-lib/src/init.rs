use web_sys::wasm_bindgen::JsValue;

pub fn elementary_init() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    Ok(())
}
