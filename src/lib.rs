
mod memory;

pub use platform_impl::*;

mod public_api {
  use shrinkwraprs::Shrinkwrap;
  use derive_more::Display;
  use num_enum::TryFromPrimitive;
  use flagset::{FlagSet, flags};

  use super::platform_impl;

  pub type Vector2 = mint::Vector2<f32>;

  /// Cubism version identifier.
  #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shrinkwrap)]
  #[repr(transparent)]
  pub struct CubismVersion(pub u32);
  impl CubismVersion {
    pub fn raw(&self) -> u32 { self.0 }
    pub fn major(&self) -> u32 { (self.0 & 0xFF000000) >> 24 }
    pub fn minor(&self) -> u32 { (self.0 & 0x00FF0000) >> 16 }
    pub fn patch(&self) -> u32 { self.0 & 0x0000FFFF }
  }
  impl std::fmt::Display for CubismVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{:02}.{:02}.{:04} ({})", self.major(), self.minor(), self.patch(), self.0)
    }
  }
  impl std::fmt::Debug for CubismVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "{}", self)
    }
  }

  /// moc3 file format version.
  /// Note that there is no equivalent of `csmMocVersion_Unknown`.
  #[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
  #[repr(u32)]
  pub enum MocVersion {
    /// moc3 file version 3.0.00 - 3.2.07
    #[display(fmt = "30(3.0.00 - 3.2.07)")]
    Moc3_30 = 1,
    /// moc3 file version 3.3.00 - 3.3.03
    #[display(fmt = "33(3.3.00 - 3.3.03)")]
    Moc3_33 = 2,
    /// moc3 file version 4.0.00 - 4.1.05
    #[display(fmt = "33(4.0.00 - 4.1.05)")]
    Moc3_40 = 3,
    /// moc3 file version 4.2.00 -
    #[display(fmt = "33(4.2.00 -)")]
    Moc3_42 = 4,
  }

  /// Parameter type.
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
  #[repr(i32)]
  pub enum ParameterType {
    Normal = 0,
    BlendShape = 1,
  }
  #[derive(Debug, Default)]
  pub struct CubismCore {
    inner: platform_impl::PlatformCubismCore,
  }
  impl std::ops::Deref for CubismCore {
    type Target = platform_impl::PlatformCubismCore;
    fn deref(&self) -> &Self::Target { &self.inner }
  }

  /// Cubism moc.
  pub struct Moc {
    pub(super) inner: platform_impl::PlatformMoc,
    pub(super) version: MocVersion,
  }
  impl std::ops::Deref for Moc {
    type Target = platform_impl::PlatformMoc;
    fn deref(&self) -> &Self::Target { &self.inner }
  }

  /// Cubism model.
  pub struct Model {
    pub(super) inner: platform_impl::PlatformModel,
    pub(super) canvas_info: CanvasInfo,
    pub(super) parameters: Vec<Parameter>,
    pub(super) parts: Vec<Part>,
    pub(super) drawables: Vec<Drawable>,
  }

  #[derive(Debug, Clone, Copy)]
  /// Model canvas.
  pub struct CanvasInfo {
    /// Canvas dimensions.
    pub size_in_pixels: (f32, f32),
    /// Origin of model on canvas.
    pub origin_in_pixels: (f32, f32),
    /// Aspect used for scaling pixels to units.
    pub pixels_per_unit: f32,
  }

  #[derive(Debug)]
  pub struct Parameter {
    pub id: String,
    pub ty: ParameterType,
    pub value_range: std::ops::Range<f32>,
    pub default_value: f32,
    pub keys: Vec<f32>,
  }

  #[derive(Debug)]
  pub struct Part {
    pub id: String,
    pub parent_part_index: Option<usize>,
  }

  #[derive(Debug)]
  pub struct Drawable {
    pub id: String,
    pub constant_flags: ConstantDrawableFlagSet,
    pub texture_index: usize,
    pub masks: Vec<usize>,
    pub vertex_uvs: Vec<Vector2>,
    pub parent_part_index: Option<usize>,
  }

  pub type ConstantDrawableFlagSet = FlagSet<ConstantDrawableFlags>;
  flags! {
    pub enum ConstantDrawableFlags: u8 {
      BlendAdditive,
      BlendMultiplicative,
      IsDoubleSided,
      IsInvertedMask,
    }
  }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform_impl {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]
  include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

  use std::sync::Arc;

  use crate::memory::AlignedStorage;

  use super::public_api;

  #[derive(Debug, Default)]
  pub struct PlatformCubismCore;
  impl PlatformCubismCore {
    pub fn version(&self) -> public_api::CubismVersion {
      public_api::CubismVersion(unsafe { csmGetVersion() })
    }
    pub fn latest_supported_moc_version(&self) -> public_api::MocVersion {
      unsafe { csmGetLatestMocVersion() }.try_into().unwrap()
    }

    // TODO: Return a `Result`.
    pub fn moc_from_bytes(&self, bytes: &[u8]) -> Option<public_api::Moc> {
      const MOC_ALIGNMENT: usize = csmAlignofMoc as usize;

      let mut aligned_storage = AlignedStorage::new(bytes.len(), MOC_ALIGNMENT).unwrap();
      aligned_storage.copy_from_slice(bytes);

      let size_in_u32: u32 = bytes.len().try_into().expect("Size should fit in a u32");

      let moc_version = unsafe {
        csmGetMocVersion(aligned_storage.as_mut_ptr() as *mut _, size_in_u32)
      };
      // TODO: Error
      let moc_version = public_api::MocVersion::try_from(moc_version).ok()?;

      if self.latest_supported_moc_version() < moc_version {
        // TODO: Error
        return None;
      }

      let csm_moc = unsafe {
        csmReviveMocInPlace(aligned_storage.as_mut_ptr() as *mut _, size_in_u32)
      };

      public_api::Moc {
        inner: PlatformMoc {
          csm_moc,
          moc_storage: Arc::new(aligned_storage),
        },
        version: moc_version,
      }.into()
    }
  }

  pub struct PlatformMoc {
    csm_moc: *mut csmMoc,
    /// This is an [`Arc`] because the memory block for a `csmMoc` needs to outlive
    /// the memory blocks for all `csmModel`s generated from it.
    moc_storage: Arc<AlignedStorage>,
  }
  impl PlatformMoc {
    pub fn to_model(&self) -> public_api::Model {
      const MODEL_ALIGNMENT: usize = csmAlignofModel as usize;

      let storage_size = unsafe {
        csmGetSizeofModel(self.csm_moc)
      };

      let mut aligned_storage = AlignedStorage::new(storage_size as _, MODEL_ALIGNMENT).unwrap();

      let csm_model = unsafe {
        csmInitializeModelInPlace(self.csm_moc, aligned_storage.as_mut_ptr() as *mut _, storage_size)
      };

      let canvas_info = unsafe {
        let mut size_in_pixels = csmVector2 { X: 0.0, Y: 0.0 };
        let mut origin_in_pixels = csmVector2 { X: 0.0, Y: 0.0 };
        let mut pixels_per_unit: f32 = 0.0;

        csmReadCanvasInfo(csm_model, &mut size_in_pixels, &mut origin_in_pixels, &mut pixels_per_unit);

        public_api::CanvasInfo {
          size_in_pixels: (size_in_pixels.X, size_in_pixels.Y),
          origin_in_pixels: (origin_in_pixels.X, origin_in_pixels.Y),
          pixels_per_unit,
        }
      };

      let parameters: Vec<_> = unsafe {
        let count = csmGetParameterCount(csm_model);

        let ids: Vec<_> = std::slice::from_raw_parts(csmGetParameterIds(csm_model), count as _).iter()
          .map(|&c_str_ptr| crate::to_string(c_str_ptr))
          .collect();

        let types = std::slice::from_raw_parts(csmGetParameterTypes(csm_model), count as _);
        let types: Vec<public_api::ParameterType> = types.iter()
          .map(|value| public_api::ParameterType::try_from(*value).unwrap())
          .collect();

        let minimum_values = std::slice::from_raw_parts(csmGetParameterMinimumValues(csm_model), count as _);
        let maximum_values = std::slice::from_raw_parts(csmGetParameterMaximumValues(csm_model), count as _);
        let default_values = std::slice::from_raw_parts(csmGetParameterDefaultValues(csm_model), count as _);

        let key_counts = std::slice::from_raw_parts(csmGetParameterKeyCounts(csm_model), count as _);
        let key_value_ptrs = std::slice::from_raw_parts(csmGetParameterKeyValues(csm_model), count as _);

        let key_value_containers: Vec<_> = itertools::izip!(key_counts, key_value_ptrs)
          .map(|(&key_count, &key_value_ptr)| {
            std::slice::from_raw_parts(key_value_ptr, key_count.try_into().unwrap()).to_vec()
          })
          .collect();

        itertools::izip!(ids, types, minimum_values, maximum_values, default_values, key_value_containers)
          .map(|(id, ty, &minimum_value, &maximum_value, &default_value, key_value_container)| {
            public_api::Parameter {
              id,
              ty,
              value_range: minimum_value..maximum_value,
              default_value,
              keys: key_value_container,
            }
          })
          .collect()
      };

      let parts: Vec<_> = unsafe {
        let count = csmGetPartCount(csm_model);

        let ids: Vec<_> = std::slice::from_raw_parts(csmGetPartIds(csm_model), count as _).iter()
          .map(|&c_str_ptr| crate::to_string(c_str_ptr))
          .collect();

        let parent_part_indices: Vec<_> = std::slice::from_raw_parts(csmGetPartParentPartIndices(csm_model), count as _).iter()
          .map(|&value| (value > 0).then_some(value as usize)).collect();

        itertools::izip!(ids, parent_part_indices)
          .map(|(id, parent_part_index)| {
            public_api::Part {
              id,
              parent_part_index,
            }
          })
          .collect()
      };

      let drawables = unsafe {
        let count = csmGetDrawableCount(csm_model);

        let ids: Vec<_> = std::slice::from_raw_parts(csmGetDrawableIds(csm_model), count as _).iter()
          .map(|&c_str_ptr| crate::to_string(c_str_ptr))
          .collect();

        let constant_flagsets: Vec<_> = std::slice::from_raw_parts(csmGetDrawableConstantFlags(csm_model), count as _).iter()
          .map(|value| public_api::ConstantDrawableFlagSet::new(*value).unwrap())
          .collect();

        let texture_indices: Vec<_> = std::slice::from_raw_parts(csmGetDrawableTextureIndices(csm_model), count as _).iter()
          .map(|value| *value as usize)
          .collect();

        let mask_counts = std::slice::from_raw_parts(csmGetDrawableMaskCounts(csm_model), count as _);
        let mask_container_ptrs = std::slice::from_raw_parts(csmGetDrawableMasks(csm_model), count as _);
        let mask_containers: Vec<_> = itertools::izip!(mask_counts, mask_container_ptrs)
          .map(|(&mask_count, &mask_container_ptr)| {
            std::slice::from_raw_parts(mask_container_ptr, mask_count as _).iter().map(|mask| *mask as usize).collect::<Vec<_>>()
          })
          .collect();

        let vertex_counts = std::slice::from_raw_parts(csmGetDrawableVertexCounts(csm_model), count as _);
        let vertex_uv_ptrs = std::slice::from_raw_parts(csmGetDrawableVertexUvs(csm_model), count as _);

        let vertex_uv_containers: Vec<_> = itertools::izip!(vertex_counts, vertex_uv_ptrs)
          .map(|(&vertex_count, &vertex_uv_ptr)| {
            std::slice::from_raw_parts(vertex_uv_ptr as *const public_api::Vector2, vertex_count as _).to_vec()
          })
          .collect();

        let parent_part_indices: Vec<_> = std::slice::from_raw_parts(csmGetDrawableParentPartIndices(csm_model), count as _).iter()
          .map(|&value| (value > 0).then_some(value as usize)).collect();

        itertools::izip!(ids, constant_flagsets, texture_indices, mask_containers, vertex_uv_containers, parent_part_indices)
          .map(|(id, constant_flagset, texture_index, mask_container, vertex_uv_container, parent_part_index),| {
            public_api::Drawable {
              id,
              constant_flags: constant_flagset,
              texture_index,
              masks: mask_container,
              vertex_uvs: vertex_uv_container,
              parent_part_index,
            }
          })
          .collect()
      };

      public_api::Model {
        inner: PlatformModel {
          csm_model,
          model_storage: aligned_storage,
          moc_storage: Arc::clone(&self.moc_storage),
        },
        canvas_info,
        parameters,
        parts,
        drawables,
      }
    }
  }

  pub struct PlatformModel {
    csm_model: *mut csmModel,
    model_storage: AlignedStorage,
    /// The memory block for the `csmMoc` used to generate this `csmModel`, which need to outlive this `csm_model`.
    moc_storage: Arc<AlignedStorage>,
  }
}

#[cfg(target_arch = "wasm32")]
mod platform_impl {
  use std::sync::Arc;

  use super::public_api;

  #[derive(Debug, Default)]
  pub struct PlatformCubismCore {
    js_cubism_core: Arc<JsLive2DCubismCore>,
  }
  impl PlatformCubismCore {
    pub fn version(&self) -> public_api::CubismVersion { self.js_cubism_core.cubism_version }
    pub fn latest_supported_moc_version(&self) -> public_api::MocVersion { self.js_cubism_core.latest_supported_moc_version }

    pub fn moc_from_bytes(&self, bytes: &[u8]) -> Option<public_api::Moc> {
      let array = js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());
      array.copy_from(bytes);

      let js_moc = self.js_cubism_core.moc_from_js_array_buffer(array.buffer());

      let moc_version = self.js_cubism_core.csmGetMocVersion.call2(
        &self.js_cubism_core.version_class, &js_moc.moc_instance, array.buffer().as_ref()
      )
      .unwrap().as_f64().unwrap() as u32;
      let moc_version = public_api::MocVersion::try_from(moc_version).ok()?; // TODO: Error.

      public_api::Moc {
        inner: PlatformMoc {
          js_moc,
          js_cubism_core: Arc::clone(&self.js_cubism_core),
        },
        version: moc_version,
      }.into()
    }
  }

  #[derive(Debug)]
  pub struct PlatformMoc {
    js_moc: JsMoc,
    js_cubism_core: Arc<JsLive2DCubismCore>,
  }
  impl PlatformMoc {
    pub fn to_model(&self) -> public_api::Model {
      let js_model = self.js_cubism_core.model_from_moc(&self.js_moc);

      let canvas_info = js_model.canvas_info;
      let parameters = js_model.parameters.to_aos();
      let parts = js_model.parts.to_aos();
      let drawables = js_model.drawables.to_aos();

      public_api::Model {
        inner: PlatformModel {
          js_model,
        },
        canvas_info,
        parameters,
        parts,
        drawables,
      }
    }
  }

  pub struct PlatformModel {
    js_model: JsModel,
  }

  const LIVE2DCUBISMCORE_JS_STR: &str = include_str!(concat!(env!("LIVE2D_CUBISM_SDK_WEB_DIR"), "/Core/live2dcubismcore.js"));

  use wasm_bindgen::JsCast as _;

  #[derive(Debug)]
  pub struct JsLive2DCubismCore {
    cubism_version: public_api::CubismVersion,
    latest_supported_moc_version: public_api::MocVersion,

    /// The `Live2DCubismCore` namespace object.
    live2d_cubism_core_namespace: wasm_bindgen::JsValue,

    /// The `Live2DCubismCore.Version` class object.
    version_class: wasm_bindgen::JsValue,
    csmGetMocVersion: js_sys::Function,

    /// The `Live2DCubismCore.Moc` class object.
    moc_class: wasm_bindgen::JsValue,
    fromArrayBuffer: js_sys::Function,

    /// The `Live2DCubismCore.Model` class object.
    model_class: wasm_bindgen::JsValue,
    fromMoc: js_sys::Function,
  }

  #[derive(Debug)]
  pub struct JsMoc {
    /// The `Live2DCubismCore.Moc` class object.
    moc_class: wasm_bindgen::JsValue,
    /// An `Live2DCubismCore.Moc` instance object, acquired through the `Live2DCubismCore.Moc.fromArrayBuffer` static method.
    moc_instance: wasm_bindgen::JsValue,
  }
  pub struct JsModel {
    /// An `Live2DCubismCore.Model` instance object, acquired through the `Live2DCubismCore.Model.fromMoc` static method.
    model_instance: wasm_bindgen::JsValue,
    pub canvas_info: public_api::CanvasInfo,
    pub parameters: JsParameters,
    pub parts: JsParts,
    pub drawables: JsDrawables,
  }

  pub struct JsParameters {
    /// The `parameters` member variable of a `Live2DCubismCore.Model` instance object.
    /// An instance of `Live2DCubismCore.Parameters` class object.
    parameters_instance: wasm_bindgen::JsValue,

    ids: Vec<String>,
    types: Vec<public_api::ParameterType>,
    minimum_values: Vec<f32>,
    maximum_values: Vec<f32>,
    default_values: Vec<f32>,
    key_value_containers: Vec<Vec<f32>>,
  }
  pub struct JsParts {
    /// The `parts` member variable of a `Live2DCubismCore.Model` instance object.
    /// An instance of `Live2DCubismCore.Parts` class object.
    parts_instance: wasm_bindgen::JsValue,

    ids: Vec<String>,
    parent_part_indices: Vec<Option<usize>>
  }
  pub struct JsDrawables {
    /// The `drawables` member variable of `Live2DCubismCore.Model` instance object.
    /// An instance of `Live2DCubismCore.Drawables` class object.
    drawables_instance: wasm_bindgen::JsValue,

    ids: Vec<String>,
    constant_flagsets: Vec<public_api::ConstantDrawableFlagSet>,
    texture_indices: Vec<usize>,
    mask_containers: Vec<Vec<usize>>,
    vertex_uv_containers: Vec<Vec<public_api::Vector2>>,
    parent_part_indices: Vec<Option<usize>>,
  }

  impl Default for JsLive2DCubismCore {
    fn default() -> Self {
      #![allow(non_snake_case)]

      let code = format!("{LIVE2DCUBISMCORE_JS_STR}\n Live2DCubismCore");
      let live2d_cubism_core_namespace = js_sys::eval(&code).expect("Failed to evaluate synthesized JavaScript code!");

      let version_class = get_member_value(&live2d_cubism_core_namespace, "Version");

      let cubism_version = {
        let csmGetVersion = get_member_function(&version_class, "csmGetVersion");
        public_api::CubismVersion(csmGetVersion.call0(&version_class).unwrap().as_f64().unwrap() as u32)
      };
      let latest_supported_moc_version = {
        let csmGetLatestMocVersion = get_member_function(&version_class, "csmGetLatestMocVersion");
        public_api::MocVersion::try_from(csmGetLatestMocVersion.call0(&version_class).unwrap().as_f64().unwrap() as u32).unwrap()
      };

      let csmGetMocVersion = get_member_function(&version_class, "csmGetMocVersion");

      let moc_class = get_member_value(&live2d_cubism_core_namespace, "Moc");
      let fromArrayBuffer = get_member_function(&moc_class, "fromArrayBuffer");

      let model_class = get_member_value(&live2d_cubism_core_namespace, "Model");
      let fromMoc = get_member_function(&model_class, "fromMoc");

      Self {
        cubism_version,
        latest_supported_moc_version,

        live2d_cubism_core_namespace,

        version_class,
        csmGetMocVersion,

        moc_class,
        fromArrayBuffer,

        model_class,
        fromMoc,
      }
    }
  }

  impl JsLive2DCubismCore {
    pub fn moc_from_js_array_buffer(&self, array_buffer: js_sys::ArrayBuffer) -> JsMoc {
      // `Version.csmGetMocVersion` requires a `Moc`, unlike Native SDK.
      let moc_instance = self.fromArrayBuffer.call1(&self.moc_class, array_buffer.as_ref()).unwrap();
      assert!(!moc_instance.is_undefined());

      JsMoc {
        moc_class: self.moc_class.clone(),
        moc_instance,
     }
    }
    pub fn moc_from_bytes(&self, bytes: &[u8]) -> JsMoc {
      let array = js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());
      array.copy_from(bytes);

      self.moc_from_js_array_buffer(array.buffer())
    }

    pub fn model_from_moc(&self, moc: &JsMoc) -> JsModel {
      let model_instance = self.fromMoc.call1(&self.moc_class, moc.moc_instance.as_ref()).unwrap();

      let canvas_info = {
        let canvas_info_instance = get_member_value(&model_instance, "canvasinfo");
        let canvas_width = get_member_value(&canvas_info_instance, "CanvasWidth").as_f64().unwrap() as f32;
        let canvas_height = get_member_value(&canvas_info_instance, "CanvasHeight").as_f64().unwrap() as f32;
        let canvas_origin_x = get_member_value(&canvas_info_instance, "CanvasOriginX").as_f64().unwrap() as f32;
        let canvas_origin_y = get_member_value(&canvas_info_instance, "CanvasOriginY").as_f64().unwrap() as f32;
        let pixels_per_unit = get_member_value(&canvas_info_instance, "PixelsPerUnit").as_f64().unwrap() as f32;
        public_api::CanvasInfo {
          size_in_pixels: (canvas_width, canvas_height),
          origin_in_pixels: (canvas_origin_x, canvas_origin_y),
          pixels_per_unit,
        }
      };

      let parameters = JsParameters::from_parameters_instance(get_member_value(&model_instance, "parameters"));
      let parts = JsParts::from_parts_instance(get_member_value(&model_instance, "parts"));
      let drawables = JsDrawables::from_drawables_instance(get_member_value(&model_instance, "drawables"));

      JsModel {
        model_instance,
        canvas_info,
        parameters,
        parts,
        drawables,
      }
    }
  }

  impl JsParameters {
    fn from_parameters_instance(parameters_instance: wasm_bindgen::JsValue) -> Self {
      let ids = get_member_array(&parameters_instance, "ids");
      let ids = ids.iter().map(|value| value.as_string().unwrap()).collect();

      let types = get_member_array(&parameters_instance, "types");
      let types = types.iter().map(|value| public_api::ParameterType::try_from(value.as_f64().unwrap() as i32).unwrap()).collect();

      let minimum_values = get_member_array(&parameters_instance, "minimumValues");
      let minimum_values = minimum_values.iter().map(|value| value.as_f64().unwrap() as f32).collect();

      let maximum_values = get_member_array(&parameters_instance, "maximumValues");
      let maximum_values = maximum_values.iter().map(|value| value.as_f64().unwrap() as f32).collect();

      let default_values = get_member_array(&parameters_instance, "defaultValues");
      let default_values = default_values.iter().map(|value| value.as_f64().unwrap() as f32).collect();

      let key_values = get_member_array(&parameters_instance, "keyValues");
      let key_value_containers: Vec<Vec<f32>> = key_values.iter()
        .map(|value| {
          js_sys::Array::from(&value).iter().map(|value| value.as_f64().unwrap() as f32).collect()
        })
        .collect();

      Self {
        parameters_instance,

        ids,
        types,
        minimum_values,
        maximum_values,
        default_values,
        key_value_containers,
      }
    }

    fn to_aos(&self) -> Vec<public_api::Parameter> {
      itertools::izip!(&self.ids, &self.types, &self.minimum_values, &self.maximum_values, &self.default_values, &self.key_value_containers)
        .map(|(id, ty, minimum_value, maximum_value, default_value, key_value_container)| {
          public_api::Parameter {
            id: id.clone(),
            ty: *ty,
            value_range: *minimum_value..*maximum_value,
            default_value: *default_value,
            keys: key_value_container.clone(),
          }
        })
        .collect()
    }
  }

  impl JsParts {
    fn from_parts_instance(parts_instance: wasm_bindgen::JsValue) -> Self {
      let ids = get_member_array(&parts_instance, "ids");
      let ids = ids.iter().map(|value| value.as_string().unwrap()).collect();

      let parent_part_indices = get_member_array(&parts_instance, "parentIndices");
      let parent_part_indices = parent_part_indices.iter()
        .map(|value| {
          let number = value.as_f64().unwrap();
          (number > 0.0).then_some(number as usize)
          })
        .collect();

      Self {
        parts_instance,

        ids,
        parent_part_indices,
      }
    }

    fn to_aos(&self) -> Vec<public_api::Part> {
      itertools::izip!(&self.ids, &self.parent_part_indices)
        .map(|(id, parent_part_index)| {
          public_api::Part {
            id: id.clone(),
            parent_part_index: *parent_part_index,
          }
        })
        .collect()
    }
  }

  impl JsDrawables {
    fn from_drawables_instance(drawables_instance: wasm_bindgen::JsValue) -> Self {
      let ids = get_member_array(&drawables_instance, "ids");
      let ids = ids.iter().map(|value| value.as_string().unwrap()).collect();

      let constant_flagsets = get_member_array(&drawables_instance, "constantFlags");
      let constant_flagsets: Vec<_> = constant_flagsets.iter()
        .map(|value| public_api::ConstantDrawableFlagSet::new(value.as_f64().unwrap() as u8).unwrap())
        .collect();

      let texture_indices = get_member_array(&drawables_instance, "textureIndices");
      let texture_indices: Vec<_> = texture_indices.iter()
        .map(|value| value.as_f64().unwrap() as usize)
        .collect();

      let mask_containers = get_member_array(&drawables_instance, "masks");
      let mask_containers: Vec<_> = mask_containers.iter()
        .map(|mask_container| {
          js_sys::Array::from(&mask_container).iter()
            .map(|mask| mask.as_f64().unwrap() as usize)
            .collect::<Vec<_>>()
        })
        .collect();

      let vertex_uv_containers = get_member_array(&drawables_instance, "vertexUvs");
      let vertex_uv_containers: Vec<_> = vertex_uv_containers.iter()
        .map(|v| {
          let typed_array = v.dyn_into::<js_sys::Float32Array>().unwrap();

          let dst_len: usize = (typed_array.length() / 2) as _;
          let mut dst = Vec::<public_api::Vector2>::with_capacity(dst_len);
          unsafe {
            typed_array.raw_copy_to_ptr(dst.as_mut_ptr() as *mut f32);
          }

          // SAFETY:
          // 1. Constructed with `with_capacity`.
          // 2. `raw_copy_to_ptr` initialized the elements.
          unsafe {
            dst.set_len(dst_len);
          }
          dst
        })
        .collect();

      let parent_part_indices = get_member_array(&drawables_instance, "parentPartIndices");
      let parent_part_indices: Vec<_> = parent_part_indices.iter()
        .map(|value| {
          let number = value.as_f64().unwrap();
          (number > 0.0).then_some(number as usize)
          })
        .collect();

      Self {
        drawables_instance,

        ids,
        constant_flagsets,
        texture_indices,
        mask_containers,
        vertex_uv_containers,
        parent_part_indices,
      }
    }

    fn to_aos(&self) -> Vec<public_api::Drawable> {
      itertools::izip!(&self.ids, &self.constant_flagsets, &self.texture_indices, &self.mask_containers, &self.vertex_uv_containers, &self.parent_part_indices)
        .map(|(id, constant_flagset, texture_index, mask_container, vertex_uv_container, parent_part_index)| {
          public_api::Drawable {
            id: id.clone(),
            constant_flags: *constant_flagset,
            texture_index: *texture_index,
            masks: mask_container.clone(),
            vertex_uvs: vertex_uv_container.clone(),
            parent_part_index: *parent_part_index,
          }
        })
        .collect()
    }
  }

  fn get_member_value<N: AsRef<str> + std::fmt::Debug>(value: &wasm_bindgen::JsValue, name: N) -> wasm_bindgen::JsValue {
    js_sys::Reflect::get(value, &name.as_ref().into()).unwrap_or_else(|e| panic!("No member {name:?}! {e:?}"))
  }
  /// Requires `N` to be [`Clone`] to allow error reporting.
  fn get_member_function<N: AsRef<str> + Clone + std::fmt::Debug>(value: &wasm_bindgen::JsValue, name: N) -> js_sys::Function {
    get_member_value(value, name.clone()).dyn_into().unwrap_or_else(|e| panic!("member {name:?} not a Function! {e:?}"))
  }
  fn get_member_array<N: AsRef<str> + std::fmt::Debug>(value: &wasm_bindgen::JsValue, name: N) -> js_sys::Array {
    js_sys::Array::from(&get_member_value(value, name))
  }
}

#[cfg(test)]
pub mod public_api_tests {
  // Use:
  // wasm-pack test --chrome
  #[cfg(target_arch = "wasm32")]
  wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

  #[cfg(target_arch = "wasm32")]
  use wasm_bindgen_test::*;

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

    // let moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/Samples/Resources/Hiyori/Hiyori.moc3"));
    let moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/AdditionalSamples/simple/runtime/simple.moc3"));

    let cubism_core = public_api::CubismCore::default();
    log::info!("Live2D Cubism Core Version: {}", cubism_core.version());
    log::info!("Latest supported moc version: {}", cubism_core.latest_supported_moc_version());

    let moc = cubism_core.moc_from_bytes(moc_bytes).unwrap();
    let model = moc.to_model();

    log::info!("{:?}", model.canvas_info);
    log::info!("{:?}", model.parameters);
    log::info!("{:?}", model.parts);
    log::info!("{:?}", model.drawables);
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
unsafe fn to_string(c_str_ptr: *const std::os::raw::c_char) -> String {
  std::ffi::CStr::from_ptr(c_str_ptr).to_str().unwrap().to_string()
}
