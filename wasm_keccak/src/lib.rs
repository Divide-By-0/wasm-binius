#![feature(step_trait)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use console_error_panic_hook;
mod keccakf;

#[wasm_bindgen]
pub fn run_keccak_example() {
		keccakf::main();
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
		#[cfg(feature = "console_error_panic_hook")]
		console_error_panic_hook::set_once();

		run_keccak_example();
		Ok(())
}