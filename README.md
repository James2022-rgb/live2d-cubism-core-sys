# Rust FFI bindings for Live2D® Cubism SDK Core 🦀

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust bindings to the Live2D Cubism SDK Core library.
Provides:
- direct, unsafe Rust bindings to the C interface for Native
- a higher-level interface for Native and Web (optional but enabled by default)

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
Only these versions have been tested:
|        | Version |
| ------ | ------- |
| Native | 4-r.5.1 |
| Web    | 4-r.5   |

Build target support
----------------------------
|                            | Windows            | Linux              |
| -------------------------- | ------------------ | ------------------ |
| `x86_64-pc-windows-msvc`   | :white_check_mark: |                    |
| `aarch64-linux-android`    | :white_check_mark: |                    |
| `x86_64-unknown-linux-gnu` |                    | :white_check_mark: |
| `wasm32-unknown-unknown`   | :white_check_mark: | :white_check_mark: |

`aarch64-unknown-linux-gnu` support is unfortunately not possible, due to Live2D Inc. not providing a binary for this target in the SDK.

Building
----------------------------
An enviroment variable *MUST* be set that that points to an existing _Live2D Cubism SDK_ directory, _for Native_ and _Web_, respectively:
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

Feature gate
----------------------------
The `core` feature, enabled by default, provides a high-level interface on top of the direct unsafe bindings.

Usage
----------------------------

`Cargo.toml`:
```toml
[dependencies]
live2d-cubism-core-sys = { git = "https://github.com/James2022-rgb/live2d-cubism-core-sys" }
```

Rust code (C interface):
```rust
unsafe {
  live2d_cubism_core_sys::csmReviveMocInPlace(...);
}
```

Rust code (high-level interface):
```rust
use live2d_cubism_core_sys::core as live2d_core;

let cubism_core = live2d_core::CubismCore::default();
let moc = cubism_core.moc_from_bytes(moc_bytes).unwrap();

let model = live2d_core::Model::from_moc(&moc);

{
  let mut dynamic = model.dynamic.write();

  dynamic.reset_drawable_dynamic_flags();
  dynamic.update();
}

```

Running tests
----------------------------

Native:
```shell
cargo test
```

Web:
```shell
wasm-pack test --chrome
```
