
pub use platform_impl::*;

#[cfg(not(target_arch = "wasm32"))]
mod platform_impl {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[cfg(target_arch = "wasm32")]
mod platform_impl {
  const LIVE2DCUBISMCORE_JS_STR: &str = include_str!(concat!(env!("LIVE2D_CUBISM_SDK_WEB_DIR"), "/Core/live2dcubismcore.js"));

  use wasm_bindgen::JsCast as _;

  pub type csmVersion = u32;
  pub type csmMocVersion = u32;
  pub type csmParameterType = u32;

  pub struct Live2DCubismCore {
    /// The `Live2DCubismCore` namespace object.
    live2d_cubism_core_namespace: wasm_bindgen::JsValue,

    /// The `Live2DCubismCore.Version` class object.
    version_class: wasm_bindgen::JsValue,
    csmGetVersion: js_sys::Function,
    csmGetLatestMocVersion: js_sys::Function,

    /// The `Live2DCubismCore.Moc` class object.
    moc_class: wasm_bindgen::JsValue,
    fromArrayBuffer: js_sys::Function,

    /// The `Live2DCubismCore.Model` class object.
    model_class: wasm_bindgen::JsValue,
    fromMoc: js_sys::Function,
  }

  pub struct Moc {
    /// The `Live2DCubismCore.Moc` class object.
    moc_class: wasm_bindgen::JsValue,
    /// An `Live2DCubismCore.Moc` instance object, acquired through the `Live2DCubismCore.Moc.fromArrayBuffer` static method.
    moc_instance: wasm_bindgen::JsValue,
  }
  pub struct Model {
    /// An `Live2DCubismCore.Model` instance object, acquired through the `Live2DCubismCore.Model.fromMoc` static method.
    model_instance: wasm_bindgen::JsValue,
    pub parameters: Parameters,
  }
  pub struct Parameters {
    /// The `parameters` member variable, which is a `Live2DCubismCore.Parameters` instance object, in a `Live2DCubismCore.Model` instance object, 
    parameters_instance: wasm_bindgen::JsValue,
  }

  impl Default for Live2DCubismCore {
    fn default() -> Self {
      let code = format!("{LIVE2DCUBISMCORE_JS_STR}\n Live2DCubismCore");
      let live2d_cubism_core_namespace = js_sys::eval(&code).expect("Failed to evaluate synthesized JavaScript code!");

      let version_class = js_sys::Reflect::get(&live2d_cubism_core_namespace, &"Version".into()).unwrap();

      let csmGetVersion = js_sys::Reflect::get(&version_class, &"csmGetVersion".into()).unwrap()
        .dyn_into::<js_sys::Function>().unwrap();

      let csmGetLatestMocVersion = js_sys::Reflect::get(&version_class, &"csmGetLatestMocVersion".into()).unwrap()
        .dyn_into::<js_sys::Function>().unwrap();

      let moc_class = js_sys::Reflect::get(&live2d_cubism_core_namespace, &"Moc".into()).unwrap();

      let fromArrayBuffer = js_sys::Reflect::get(&moc_class, &"fromArrayBuffer".into()).unwrap()
        .dyn_into::<js_sys::Function>().unwrap();

      let model_class = js_sys::Reflect::get(&live2d_cubism_core_namespace, &"Model".into()).unwrap();

      let fromMoc = js_sys::Reflect::get(&model_class, &"fromMoc".into()).unwrap()
        .dyn_into::<js_sys::Function>().unwrap();

      Self {
        live2d_cubism_core_namespace,

        version_class,

        csmGetVersion,
        csmGetLatestMocVersion,

        moc_class,
        fromArrayBuffer,

        model_class,
        fromMoc,
      }
    }
  }

  impl Live2DCubismCore {
    pub fn csmGetVersion(&self) -> csmVersion {
      self.csmGetVersion.call0(&self.version_class).unwrap().as_f64().unwrap() as _
    }
    pub fn csmGetLatestMocVersion(&self) -> csmMocVersion {
      self.csmGetLatestMocVersion.call0(&self.version_class).unwrap().as_f64().unwrap() as _
    }

    pub fn moc_fromArrayBuffer(&self, array_buffer: js_sys::ArrayBuffer) -> Moc {
      let moc_instance = self.fromArrayBuffer.call1(&self.moc_class, array_buffer.as_ref()).unwrap();
      Moc {
        moc_class: self.moc_class.clone(),
        moc_instance,
     }
    }
    pub fn moc_from_bytes(&self, bytes: &[u8]) -> Moc {
      let array = js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());
      array.copy_from(bytes);

      self.moc_fromArrayBuffer(array.buffer())
    }

    pub fn model_fromMoc(&self, moc: &Moc) -> Model {
      let model_instance = self.fromMoc.call1(&self.moc_class, moc.moc_instance.as_ref()).unwrap();

      let parameters = Parameters {
        parameters_instance: js_sys::Reflect::get(&model_instance, &"parameters".into()).unwrap()
      };

      Model {
        model_instance,
        parameters,
      }
    }
  }

  impl Parameters {
    pub fn count(&self) -> u32 {
      js_sys::Reflect::get(&self.parameters_instance, &"count".into()).unwrap().as_f64().unwrap() as u32
    }

    pub fn ids(&self) -> Vec<String> {
      let value = js_sys::Reflect::get(&self.parameters_instance, &"ids".into()).unwrap();
      let array = js_sys::Array::from(&value);

      array.iter().map(|value| value.as_string().unwrap()).collect()
    }

    pub fn types(&self) -> Vec<i32> {
      let value = js_sys::Reflect::get(&self.parameters_instance, &"types".into()).unwrap();
      let array = js_sys::Array::from(&value);

      array.iter().map(|value| value.as_f64().unwrap() as i32).collect()
    }
  }
}

#[cfg(target_arch = "wasm32")]
#[cfg(test)]
pub mod tests {
  wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

  use wasm_bindgen_test::*;

  #[wasm_bindgen_test]
  fn cubism_core_basic_use() {
    console_log::init_with_level(log::Level::Trace).unwrap();

    let live2d_cubism_core = crate::platform_impl::Live2DCubismCore::default();
    
    let core_version = live2d_cubism_core.csmGetVersion();
    let latest_moc_version = live2d_cubism_core.csmGetLatestMocVersion();

    log::info!("core_version: {core_version}");
    log::info!("latest_moc_version: {latest_moc_version}");

    let moc_bytes = include_bytes!(concat!(env!("LIVE2D_CUBISM_SDK_WEB_DIR"), "/Samples/Resources/Haru/Haru.moc3"));
    let moc = live2d_cubism_core.moc_from_bytes(moc_bytes);
    
    let model = live2d_cubism_core.model_fromMoc(&moc);

    log::info!("Parameter count: {}", model.parameters.count());
    log::info!("Parameter IDs: {:?}", model.parameters.ids());
    log::info!("Parameter types: {:?}", model.parameters.types());
  }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cubism_core_basic_use() {
    unsafe {
      let core_version = csmGetVersion();
      let latest_moc_version = csmGetLatestMocVersion();

      println!("core_version: {}", core_version);
      println!("latest_moc_version: {}", latest_moc_version);

      let moc_bytes = include_bytes!(concat!(env!("LIVE2D_CUBISM_SDK_NATIVE_DIR"), "/Samples/Resources/Haru/Haru.moc3"));

      {
        // Refer to: https://docs.live2d.com/cubism-sdk-manual/cubism-core-api-reference/#

        const MOC_ALIGNMENT: usize = csmAlignofMoc as usize;
        const MODEL_ALIGNMENT: usize = csmAlignofModel as usize;

        let mut aligned_moc_storage = aligned_storage::AlignedStorage::new(moc_bytes.len(), MOC_ALIGNMENT).unwrap();
        aligned_moc_storage.copy_from_slice(moc_bytes);

        let csm_moc = csmReviveMocInPlace(aligned_moc_storage.as_mut_ptr() as *mut _, aligned_moc_storage.len() as _);

        let moc_model_storage_size = csmGetSizeofModel(csm_moc);

        let mut aligned_model_storage = aligned_storage::AlignedStorage::new(moc_model_storage_size as _, MODEL_ALIGNMENT).unwrap();

        let csm_model = csmInitializeModelInPlace(csm_moc, aligned_model_storage.as_mut_ptr() as *mut _, moc_model_storage_size);

        let parameter_count = csmGetParameterCount(csm_model);

        

        let paramter_ids = csmGetParameterIds(csm_model);
        let paramter_ids = std::slice::from_raw_parts(paramter_ids, parameter_count as _);
        let parameter_ids: Vec<String> = paramter_ids.iter().map(|&c_str_ptr| std::ffi::CStr::from_ptr(c_str_ptr).to_str().unwrap().to_string()).collect();

        let parameter_types = csmGetParameterTypes(csm_model);
        let parameter_types = std::slice::from_raw_parts(parameter_types, parameter_count as _);

        println!("Parameter count: {}", parameter_count);
        println!("Parameter IDs: {:?}", parameter_ids);
        println!("Parameter types: {:?}", parameter_types);
      }
    }
  }
}

#[cfg(test)]
mod aligned_storage {
  use std::{
    alloc::{Layout, alloc, dealloc, LayoutError},
    ops,
  };

  #[derive(Debug)]
  pub struct AlignedStorage {
    ptr: *mut u8,
    layout: Layout,
  }

  impl AlignedStorage {
    pub fn new(size: usize, alignment: usize) -> Result<Self, LayoutError>  {
      let layout = Layout::from_size_align(size, alignment)?;

      unsafe {
        let ptr = alloc(layout);
        assert!(!ptr.is_null());
        Ok(AlignedStorage { ptr, layout })
      }
    }
  }
  impl Drop for AlignedStorage {
    fn drop(&mut self) {
      unsafe {
        dealloc(self.ptr, self.layout);
      }
    }
  }

  impl ops::Deref for AlignedStorage {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
      unsafe {
        std::slice::from_raw_parts(self.ptr, self.layout.size())
      }
    }
  }
  impl ops::DerefMut for AlignedStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
      unsafe {
        std::slice::from_raw_parts_mut(self.ptr, self.layout.size())
      }
    }
  }
}
