# 🦀 Rust FFI bindings for Live2D® Cubism SDK Core

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Low-level Rust bindings to the Live2D Cubism SDK Core library.

:construction: Very much a WIP project - the public API is going through a *huge* redesign in order to add support for `wasm32-unknown-unknown`.

License
----------------------------
This Rust crate is in no way endorsed, certified or otherwise approved by Live2D Inc., nor is its author affiliated with them.

By using this crate, and therefore Live2D Cubism Core, you agree to and accept the _Live2D Proprietary Software License Agreement_.

* [Live2D Proprietary Software License Agreement](https://www.live2d.com/eula/live2d-proprietary-software-license-agreement_en.html)
* [Live2D Proprietary Software 使用許諾契約書](https://www.live2d.com/eula/live2d-proprietary-software-license-agreement_jp.html)
* [Live2D Proprietary Software 使用授权协议](https://www.live2d.com/eula/live2d-proprietary-software-license-agreement_cn.html)

This crate is licensed under the [MIT license](LICENSE-MIT).

SDK version support
----------------------------
|        | Version |
| ------ | ------- |
| Native | 4-r.5.1 |
| Web    | 4-r.5   |

Build target support
----------------------------
|                            | Windows            |
| -------------------------- | ------------------ |
| `x86_64-pc-windows-msvc`   | :construction: WIP |
| `aarch64-linux-android`    | :construction: WIP |
| `x86_64-unknown-linux-gnu` | :construction: WIP |
| `wasm32-unknown-unknown`   | :construction: WIP |

`aarch64-unknown-linux-gnu` support is not possible, due to Live2D Inc. not providing a binary for this target in the SDK.

Building
----------------------------
An enviroment variable *MUST* be set that that points to an existing Live2D Cubism SDK directory, for _Native_ and _Web_, respectively:
|        | Enviroment variable name       |
| ------ | ------------------------------ |
| Native | `LIVE2D_CUBISM_SDK_NATIVE_DIR` |
| Web    | `LIVE2D_CUBISM_SDK_WEB_DIR`    |

e.g.
```
LIVE2D_CUBISM_SDK_NATIVE_DIR=D:/Development/live2d/CubismSdkForNative-4-r.5.1
LIVE2D_CUBISM_SDK_WEB_DIR=D:/Development/live2d/CubismSdkForWeb-4-r.5
```

Live2D Cubism SDK Core is included in _Live2D Cubism SDK for Native_, or _Web_, downloaded from:
https://www.live2d.com/en/download/cubism-sdk/

Usage
----------------------------

`Cargo.toml`:
```toml
[dependencies]
live2d-cubism-core-sys = { git = "https://github.com/James2022-rgb/live2d-cubism-core-sys" }
```

Rust code:
```rust
unsafe {
  live2d_cubism_core_sys::csmReviveMocInPlace(...);
}
```
