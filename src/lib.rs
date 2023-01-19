
if_native! {
  mod memory;
  mod sys;

  pub use sys::*;
}

#[cfg(feature = "core")]
pub mod core;

#[cfg(all(test, feature = "core"))]
pub mod core_api_tests {
  // Use:
  // wasm-pack test --chrome
  if_wasm! {
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    use wasm_bindgen_test::*;
  }

  use super::*;

  #[cfg(not(target_arch = "wasm32"))]
  macro_rules! ENV_CUBISM_SDK_DIR {
    () => { env!("LIVE2D_CUBISM_SDK_NATIVE_DIR") };
  }
  #[cfg(target_arch = "wasm32")]
  macro_rules! ENV_CUBISM_SDK_DIR {
    () => { env!("LIVE2D_CUBISM_SDK_WEB_DIR") };
  }

  fn impl_public_api_use() {
    #[cfg(not(target_arch = "wasm32"))]
    {
      struct MyLogger;
      impl log::Log for MyLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
          metadata.level() <= log::Level::Trace
        }
        fn log(&self, record: &log::Record) {
          if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
          }
        }
        fn flush(&self) {}
      }

      static MY_LOGGER: MyLogger = MyLogger;
      log::set_logger(&MY_LOGGER).unwrap();
      log::set_max_level(log::LevelFilter::Info);
    }
    #[cfg(target_arch = "wasm32")]
    {
      console_log::init_with_level(log::Level::Trace).unwrap();
    }

    let cubism_core = core::CubismCore::default();
    log::info!("Live2D Cubism Core Version: {}", cubism_core.version());
    log::info!("Latest supported moc version: {}", cubism_core.latest_supported_moc_version());

    {
      let invalid_moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/Samples/Resources/Hiyori/Hiyori.model3.json"));
      cubism_core.moc_from_bytes(invalid_moc_bytes).expect_err("moc_from_bytes should fail");
    }

    let moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/Samples/Resources/Hiyori/Hiyori.moc3"));
    // let moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/AdditionalSamples/simple/runtime/simple.moc3"));

    let moc = cubism_core.moc_from_bytes(moc_bytes).expect("moc_from_bytes should succeed");
    log::info!("Moc version: {}", moc.version);

    let mut model = moc.to_model();

    log::info!("{:?}", model.canvas_info);
    log::info!("{:?}", model.parameters);
    log::info!("{:?}", model.parts);
    log::info!("{:?}", model.drawables);

    log::info!("Parameter values: {:?}", model.dynamic.parameter_values());
    log::info!("Part opacities: {:?}", model.dynamic.part_opacities());
    log::info!("Drawables[0] dynamic flagset: {:?}", model.dynamic.drawable_dynamic_flagsets()[0]);
    log::info!("Drawable draw orders: {:?}", model.dynamic.drawable_draw_orders());
    log::info!("Drawable render orders: {:?}", model.dynamic.drawable_render_orders());
    log::info!("Drawable opacities: {:?}", model.dynamic.drawable_opacities());
    log::info!("Drawables[0] vertex position container: {:?}", model.dynamic.drawable_vertex_position_containers()[0]);
    log::info!("Drawable multiply colors: {:?}", model.dynamic.drawable_multiply_colors());
    log::info!("Drawable screen colors: {:?}", model.dynamic.drawable_screen_colors());

    model.dynamic.reset_drawable_dynamic_flags();
    model.dynamic.update();

    log::info!("Drawable dynamic flags: {:?}", model.dynamic.drawable_dynamic_flagsets()[0]);
  }

  #[cfg(not(target_arch = "wasm32"))]
  #[test]
  fn public_api_use() {
    impl_public_api_use();
  }

  #[cfg(target_arch = "wasm32")]
  #[wasm_bindgen_test]
  fn public_api_use() {
    impl_public_api_use();
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! if_native {
  ($($code:tt)*) => {
    $($code)*
  }
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! if_native {
  ($($code:tt)*) => {};
}

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! if_wasm {
  ($($code:tt)*) => {
    $($code)*
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! if_wasm {
  ($($code:tt)*) => {};
}
