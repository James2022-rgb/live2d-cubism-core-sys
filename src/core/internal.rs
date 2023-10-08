
pub mod platform_iface;

#[cfg(not(target_arch = "wasm32"))]
#[path = "internal/platform_impl_native.rs"]
pub mod platform_impl;

#[cfg(target_arch = "wasm32")]
#[path = "internal/platform_impl_web.rs"]
pub mod platform_impl;
