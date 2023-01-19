//! Direct bindings to the C interface of Live2D Cubism SDK Core for Native.

#![cfg(not(target_arch = "wasm32"))]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
