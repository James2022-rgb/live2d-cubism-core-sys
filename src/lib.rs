
if_native! {
  mod memory;
  mod sys;

  pub use sys::*;
}

mod public_api {
  use static_assertions::{assert_eq_align, assert_eq_size};
  use thiserror::Error;
  use shrinkwraprs::Shrinkwrap;
  use derive_more::Display;
  use num_enum::TryFromPrimitive;
  use flagset::{FlagSet, flags};

  use super::platform_impl;

  pub type Vector2 = mint::Vector2<f32>;
  pub type Vector4 = mint::Vector4<f32>;

  #[derive(Debug, Clone, Error)]
  pub enum MocError {
    #[error("Not a valid moc file.")]
    InvalidMoc,
    /// ## Platform-specific
    /// - **Web:** Unsupported.
    #[error("Unsupported moc version. given: \"{given}\" latest supported:\"{latest_supported}\"")]
    UnsupportedMocVersion { given: MocVersion, latest_supported: MocVersion },
  }

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
    pub(super) inner: platform_impl::PlatformCubismCore,
  }

  #[derive(Debug)]
  /// Cubism moc.
  pub struct Moc {
    pub version: MocVersion,
    pub(super) inner: platform_impl::PlatformMoc,
  }

  /// Cubism model.
  pub struct Model {
    pub canvas_info: CanvasInfo,
    pub parameters: Box<[Parameter]>,
    pub parts: Box<[Part]>,
    pub drawables: Box<[Drawable]>,
    pub dynamic: ModelDynamic,
    pub(super) inner: platform_impl::PlatformModel,
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
    pub keys: Box<[f32]>,
  }

  #[derive(Debug)]
  pub struct Part {
    pub id: String,
    pub parent_part_index: Option<usize>,
  }

  #[derive(Debug)]
  pub struct Drawable {
    pub id: String,
    pub constant_flagset: ConstantDrawableFlagSet,
    pub texture_index: usize,
    pub masks: Box<[usize]>,
    pub vertex_uvs: Box<[Vector2]>,
    pub triangle_indices: Box<[u16]>,
    pub parent_part_index: Option<usize>,
  }

  /// Dynamic states of a model.
  #[derive(Debug)]
  pub struct ModelDynamic {
    pub(super) inner: platform_impl::PlatformModelDynamic,
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
  assert_eq_align!(ConstantDrawableFlagSet, u8);
  assert_eq_size!(ConstantDrawableFlagSet, u8);

  pub type DynamicDrawableFlagSet = FlagSet<DynamicDrawableFlags>;
  flags! {
    pub enum DynamicDrawableFlags: u8 {
      IsVisible,
      VisibilityDidChange,
      OpacityDidChange,
      DrawOrderDidChange,
      RenderOrderDidChange,
      VertexPositionsDidChange,
      BlendColorDidChange,
    }
  }
  assert_eq_align!(DynamicDrawableFlagSet, u8);
  assert_eq_size!(DynamicDrawableFlagSet, u8);
}

#[cfg(not(target_arch = "wasm32"))]
mod platform_impl {
  use std::sync::Arc;

  use static_assertions::{assert_eq_align, assert_eq_size};

  use crate::memory::AlignedStorage;

  use super::public_api;
  use super::sys::*;

  assert_eq_align!(public_api::Vector2, csmVector2);
  assert_eq_size!(public_api::Vector2, csmVector2);
  assert_eq_align!(public_api::Vector4, csmVector4);
  assert_eq_size!(public_api::Vector4, csmVector4);

  #[derive(Debug, Default)]
  pub struct PlatformCubismCore;
  impl public_api::CubismCore {
    pub fn version(&self) -> public_api::CubismVersion {
      public_api::CubismVersion(unsafe { csmGetVersion() })
    }
    pub fn latest_supported_moc_version(&self) -> public_api::MocVersion {
      unsafe { csmGetLatestMocVersion() }.try_into().unwrap()
    }

    pub fn moc_from_bytes(&self, bytes: &[u8]) -> Result<public_api::Moc, public_api::MocError> {
      const MOC_ALIGNMENT: usize = csmAlignofMoc as usize;

      let mut aligned_storage = AlignedStorage::new(bytes.len(), MOC_ALIGNMENT).unwrap();
      aligned_storage.copy_from_slice(bytes);

      let size_in_u32: u32 = bytes.len().try_into().expect("Size should fit in a u32");

      let moc_version = unsafe {
        csmGetMocVersion(aligned_storage.as_mut_ptr() as *mut _, size_in_u32)
      };
      let moc_version = public_api::MocVersion::try_from(moc_version).map_err(|_| public_api::MocError::InvalidMoc)?;

      if self.latest_supported_moc_version() < moc_version {
        return Err(public_api::MocError::UnsupportedMocVersion { given: moc_version, latest_supported: self.latest_supported_moc_version() });
      }

      let csm_moc = unsafe {
        csmReviveMocInPlace(aligned_storage.as_mut_ptr() as *mut _, size_in_u32)
      };

      Ok(public_api::Moc {
        version: moc_version,
        inner: PlatformMoc {
          csm_moc,
          moc_storage: Arc::new(aligned_storage),
        },
      })
    }
  }

  #[derive(Debug)]
  pub struct PlatformMoc {
    csm_moc: *mut csmMoc,
    /// This is an [`Arc`] because the memory block for a `csmMoc` needs to outlive
    /// the memory blocks for all `csmModel`s generated from it.
    moc_storage: Arc<AlignedStorage>,
  }
  impl public_api::Moc {
    pub fn to_model(&self) -> public_api::Model {
      const MODEL_ALIGNMENT: usize = csmAlignofModel as usize;

      let storage_size = unsafe {
        csmGetSizeofModel(self.inner.csm_moc)
      };

      let mut aligned_storage = AlignedStorage::new(storage_size as _, MODEL_ALIGNMENT).unwrap();

      let csm_model = unsafe {
        csmInitializeModelInPlace(self.inner.csm_moc, aligned_storage.as_mut_ptr() as *mut _, storage_size)
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

      let parameters: Box<[_]> = unsafe {
        let count = csmGetParameterCount(csm_model);

        let ids: Vec<_> = std::slice::from_raw_parts(csmGetParameterIds(csm_model), count as _).iter()
          .map(|&c_str_ptr| to_string(c_str_ptr))
          .collect();

        let types: Vec<_> = std::slice::from_raw_parts(csmGetParameterTypes(csm_model), count as _).iter()
          .map(|value| public_api::ParameterType::try_from(*value).unwrap())
          .collect();

        let minimum_values = std::slice::from_raw_parts(csmGetParameterMinimumValues(csm_model), count as _);
        let maximum_values = std::slice::from_raw_parts(csmGetParameterMaximumValues(csm_model), count as _);
        let default_values = std::slice::from_raw_parts(csmGetParameterDefaultValues(csm_model), count as _);

        let key_value_containers: Box<[_]> = {
          let key_counts = std::slice::from_raw_parts(csmGetParameterKeyCounts(csm_model), count as _);
          let key_value_ptrs = std::slice::from_raw_parts(csmGetParameterKeyValues(csm_model), count as _);

          itertools::izip!(key_counts, key_value_ptrs)
            .map(|(&key_count, &key_value_ptr)| {
              std::slice::from_raw_parts(key_value_ptr, key_count.try_into().unwrap()).to_vec().into_boxed_slice()
            })
            .collect()
        };

        itertools::izip!(ids, types, minimum_values, maximum_values, default_values, key_value_containers.iter())
          .map(|(id, ty, &minimum_value, &maximum_value, &default_value, key_value_container)| {
            public_api::Parameter {
              id,
              ty,
              value_range: minimum_value..maximum_value,
              default_value,
              keys: key_value_container.clone(),
            }
          })
          .collect()
      };

      let parts: Box<[_]> = unsafe {
        let count = csmGetPartCount(csm_model);

        let ids: Vec<_> = std::slice::from_raw_parts(csmGetPartIds(csm_model), count as _).iter()
          .map(|&c_str_ptr| to_string(c_str_ptr))
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

      let drawables: Box<[_]> = unsafe {
        let count = csmGetDrawableCount(csm_model);

        let ids: Vec<_> = std::slice::from_raw_parts(csmGetDrawableIds(csm_model), count as _).iter()
          .map(|&c_str_ptr| to_string(c_str_ptr))
          .collect();

        let constant_flagsets: Vec<_> = std::slice::from_raw_parts(csmGetDrawableConstantFlags(csm_model), count as _).iter()
          .map(|value| public_api::ConstantDrawableFlagSet::new(*value).unwrap())
          .collect();

        let texture_indices: Vec<_> = std::slice::from_raw_parts(csmGetDrawableTextureIndices(csm_model), count as _).iter()
          .map(|value| *value as usize)
          .collect();

        let mask_containers: Box<[_]> = {
          let mask_counts = std::slice::from_raw_parts(csmGetDrawableMaskCounts(csm_model), count as _);
          let mask_container_ptrs = std::slice::from_raw_parts(csmGetDrawableMasks(csm_model), count as _);

          itertools::izip!(mask_counts, mask_container_ptrs)
            .map(|(&mask_count, &mask_container_ptr)| {
              std::slice::from_raw_parts(mask_container_ptr, mask_count as _).iter().map(|mask| *mask as usize).collect::<Box<[_]>>()
            })
            .collect()
        };

        let vertex_uv_containers: Box<[_]> = {
          let vertex_counts = std::slice::from_raw_parts(csmGetDrawableVertexCounts(csm_model), count as _);
          let vertex_uv_ptrs = std::slice::from_raw_parts(csmGetDrawableVertexUvs(csm_model), count as _);

          itertools::izip!(vertex_counts, vertex_uv_ptrs)
            .map(|(&vertex_count, &vertex_uv_ptr)| {
              std::slice::from_raw_parts(vertex_uv_ptr as *const public_api::Vector2, vertex_count as _).to_vec().into_boxed_slice()
            })
            .collect()
        };

        let triangle_index_containers: Box<[_]> = {
          let triangle_index_counts = std::slice::from_raw_parts(csmGetDrawableIndexCounts(csm_model), count as _);
          let triangle_index_ptrs = std::slice::from_raw_parts(csmGetDrawableIndices(csm_model), count as _);

          itertools::izip!(triangle_index_counts, triangle_index_ptrs)
            .map(|(&triangle_index_count, &triangle_index_ptr)| {
              std::slice::from_raw_parts(triangle_index_ptr, triangle_index_count as _).to_vec().into_boxed_slice()
            })
            .collect()
        };

        let parent_part_indices: Vec<_> = std::slice::from_raw_parts(csmGetDrawableParentPartIndices(csm_model), count as _).iter()
          .map(|&value| (value > 0).then_some(value as usize)).collect();

        itertools::izip!(ids, constant_flagsets, texture_indices, mask_containers.iter(), vertex_uv_containers.iter(), triangle_index_containers.iter(), parent_part_indices)
          .map(|(id, constant_flagset, texture_index, mask_container, vertex_uv_container, triangle_index_container, parent_part_index),| {
            public_api::Drawable {
              id,
              constant_flagset,
              texture_index,
              masks: mask_container.clone(),
              vertex_uvs: vertex_uv_container.clone(),
              triangle_indices: triangle_index_container.clone(),
              parent_part_index,
            }
          })
          .collect()
      };

      let dynamic = public_api::ModelDynamic {
        inner: PlatformModelDynamic {
          csm_model,
          parameter_values: unsafe { std::slice::from_raw_parts_mut(csmGetParameterValues(csm_model), parameters.len()) },
          part_opactities: unsafe { std::slice::from_raw_parts_mut(csmGetPartOpacities(csm_model), parts.len()) },
          drawable_dynamic_flagsets: unsafe { std::slice::from_raw_parts_mut(csmGetDrawableDynamicFlags(csm_model) as *mut _, drawables.len()) },
          drawable_draw_orders: unsafe { std::slice::from_raw_parts(csmGetDrawableDrawOrders(csm_model), drawables.len()) },
          drawable_render_orders: unsafe { std::slice::from_raw_parts(csmGetDrawableRenderOrders(csm_model), drawables.len()) },
          drawable_opacities: unsafe { std::slice::from_raw_parts(csmGetDrawableOpacities(csm_model), drawables.len()) },
          drawable_multiply_colors: unsafe { std::slice::from_raw_parts(csmGetDrawableMultiplyColors(csm_model) as *const _, drawables.len()) },
          drawable_screen_colors: unsafe { std::slice::from_raw_parts(csmGetDrawableScreenColors(csm_model) as *const _, drawables.len()) },

          vertex_position_containers: VertexPositionContainers::new(csm_model),
        }
      };

      let inner = PlatformModel {
        csm_model,
        model_storage: aligned_storage,
        moc_storage: Arc::clone(&self.inner.moc_storage),
      };

      public_api::Model {
        canvas_info,
        parameters,
        parts,
        drawables,
        dynamic,
        inner,
      }
    }
  }

  pub struct PlatformModel {
    csm_model: *mut csmModel,
    model_storage: AlignedStorage,
    /// The memory block for the `csmMoc` used to generate this `csmModel`, which need to outlive this `csm_model`.
    moc_storage: Arc<AlignedStorage>,
  }

  #[derive(Debug)]
  pub struct PlatformModelDynamic {
    csm_model: *mut csmModel,
    parameter_values: &'static mut [f32],
    part_opactities: &'static mut [f32],
    drawable_dynamic_flagsets: &'static mut [public_api::DynamicDrawableFlagSet],
    drawable_draw_orders: &'static [i32],
    drawable_render_orders: &'static [i32],
    drawable_opacities: &'static [f32],
    vertex_position_containers: VertexPositionContainers<'static>,
    drawable_multiply_colors: &'static [public_api::Vector4],
    drawable_screen_colors: &'static [public_api::Vector4],
  }

  impl public_api::ModelDynamic {
    pub fn parameter_values(&self) -> &[f32] { self.inner.parameter_values }
    pub fn parameter_values_mut(&mut self) -> &mut [f32] { self.inner.parameter_values }
    pub fn part_opacities(&self) -> &[f32] { self.inner.part_opactities }
    pub fn part_opacities_mut(&mut self) -> &mut [f32] { self.inner.part_opactities }
    pub fn drawable_dynamic_flagsets(&self) -> &[public_api::DynamicDrawableFlagSet] { self.inner.drawable_dynamic_flagsets }
    pub fn drawable_dynamic_flagsets_mut(&mut self) -> &mut [public_api::DynamicDrawableFlagSet] { self.inner.drawable_dynamic_flagsets }
    pub fn drawable_draw_orders(&self) -> &[i32] { &self.inner.drawable_draw_orders }
    pub fn drawable_render_orders(&self) -> &[i32] { self.inner.drawable_render_orders }
    pub fn drawable_opacities(&self) -> &[f32] { self.inner.drawable_opacities }
    pub fn drawable_vertex_position_containers(&self) -> &[&[public_api::Vector2]] {
      &self.inner.vertex_position_containers.inner
    }
    pub fn drawable_multiply_colors(&self) -> &[public_api::Vector4] { &self.inner.drawable_multiply_colors }
    pub fn drawable_screen_colors(&self) -> &[public_api::Vector4] { self.inner.drawable_screen_colors }

    pub fn update(&mut self) {
      unsafe {
        csmUpdateModel(self.inner.csm_model);
      }

      self.inner.vertex_position_containers = VertexPositionContainers::new(self.inner.csm_model);
    }
    pub fn reset_drawable_dynamic_flags(&mut self) {
      unsafe {
        csmResetDrawableDynamicFlags(self.inner.csm_model);
      }
    }
  }

  #[derive(Debug)]
  struct VertexPositionContainers<'a> {
    inner: Box<[&'a [public_api::Vector2]]>,
  }
  impl<'a> VertexPositionContainers<'a> {
    fn new(csm_model: *mut csmModel) -> Self {
      Self {
        inner: unsafe {
          let drawable_count: usize = csmGetDrawableCount(csm_model) as _;

          let vertex_counts = std::slice::from_raw_parts(csmGetDrawableVertexCounts(csm_model), drawable_count);
          let vertex_position_ptrs = std::slice::from_raw_parts(csmGetDrawableVertexPositions(csm_model), drawable_count);

          itertools::izip!(vertex_counts, vertex_position_ptrs)
            .map(|(&vertex_count, &vertex_position_ptr)| {
              std::slice::from_raw_parts(vertex_position_ptr as *const public_api::Vector2, vertex_count as _)
            })
            .collect()
        }
      }
    }
  }

  unsafe fn to_string(c_str_ptr: *const std::os::raw::c_char) -> String {
    std::ffi::CStr::from_ptr(c_str_ptr).to_str().unwrap().to_string()
  }
}

#[cfg(target_arch = "wasm32")]
mod platform_impl {
  use std::sync::Arc;

  use super::public_api;
  use js::*;

  #[derive(Debug, Default)]
  pub struct PlatformCubismCore {
    js_cubism_core: Arc<JsLive2DCubismCore>,
  }
  impl public_api::CubismCore {
    pub fn version(&self) -> public_api::CubismVersion { self.inner.js_cubism_core.cubism_version }
    pub fn latest_supported_moc_version(&self) -> public_api::MocVersion { self.inner.js_cubism_core.latest_supported_moc_version }

    pub fn moc_from_bytes(&self, bytes: &[u8]) -> Result<public_api::Moc, public_api::MocError> {
      let array = js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());
      array.copy_from(bytes);

      let js_moc = self.inner.js_cubism_core.moc_from_js_array_buffer(array.buffer());
      js_moc
        .map(|js_moc| {
          public_api::Moc {
            version: js_moc.version,
            inner: PlatformMoc {
              js_moc,
              js_cubism_core: Arc::clone(&self.inner.js_cubism_core),
            },
          }
        })
        .ok_or(public_api::MocError::InvalidMoc)
    }
  }

  #[derive(Debug)]
  pub struct PlatformMoc {
    js_moc: JsMoc,
    js_cubism_core: Arc<JsLive2DCubismCore>,
  }
  impl public_api::Moc {
    pub fn to_model(&self) -> public_api::Model {
      let js_model = self.inner.js_cubism_core.model_from_moc(&self.inner.js_moc);

      let canvas_info = js_model.canvas_info;
      let parameters = js_model.parameters.to_aos().into_boxed_slice();
      let parts = js_model.parts.to_aos().into_boxed_slice();
      let drawables = js_model.drawables.to_aos().into_boxed_slice();

      let dynamic = public_api::ModelDynamic {
        inner: PlatformModelDynamic {
          js_model,
        }
      };

      public_api::Model {
        canvas_info,
        parameters,
        parts,
        drawables,
        dynamic,
        inner: PlatformModel,
      }
    }
  }

  pub struct PlatformModel;

  #[derive(Debug)]
  pub struct PlatformModelDynamic {
    js_model: JsModel,
  }

  impl public_api::ModelDynamic {
    pub fn parameter_values(&self) -> &[f32] { self.inner.js_model.scratch.parameter_values() }
    pub fn parameter_values_mut(&mut self) -> &mut [f32] { self.inner.js_model.scratch.parameter_values_mut() }
    pub fn part_opacities(&self) -> &[f32] { self.inner.js_model.scratch.part_opacities() }
    pub fn part_opacities_mut(&mut self) -> &mut [f32] { self.inner.js_model.scratch.part_opacities_mut() }
    pub fn drawable_dynamic_flagsets(&self) -> &[public_api::DynamicDrawableFlagSet] { self.inner.js_model.scratch.drawable_dynamic_flagsets() }
    pub fn drawable_dynamic_flagsets_mut(&mut self) -> &mut [public_api::DynamicDrawableFlagSet] { self.inner.js_model.scratch.drawable_dynamic_flagsets_mut() }
    pub fn drawable_draw_orders(&self) -> &[i32] { &self.inner.js_model.scratch.drawable_draw_orders() }
    pub fn drawable_render_orders(&self) -> &[i32] { self.inner.js_model.scratch.drawable_render_orders() }
    pub fn drawable_opacities(&self) -> &[f32] { self.inner.js_model.scratch.drawable_opacities() }
    pub fn drawable_vertex_position_containers(&self) -> &[&[public_api::Vector2]] { &self.inner.js_model.scratch.drawable_vertex_position_containers() }
    pub fn drawable_multiply_colors(&self) -> &[public_api::Vector4] { &self.inner.js_model.scratch.drawable_multiply_colors() }
    pub fn drawable_screen_colors(&self) -> &[public_api::Vector4] { self.inner.js_model.scratch.drawable_screen_colors()}

    pub fn update(&mut self) {
      self.inner.js_model.update();
    }
    pub fn reset_drawable_dynamic_flags(&mut self) {
      self.inner.js_model.reset_drawable_dynamic_flags();
    }
  }

  /// Not-so-direct bindings to the JavaScript interface of Live2D Cubism SDK Core for Web.
  mod js {
    const LIVE2DCUBISMCORE_JS_STR: &str = include_str!(concat!(env!("LIVE2D_CUBISM_SDK_WEB_DIR"), "/Core/live2dcubismcore.min.js"));

    use wasm_bindgen::JsCast as _;

    use crate::public_api;

    #[allow(non_snake_case)]
    #[derive(Debug)]
    pub struct JsLive2DCubismCore {
      pub cubism_version: public_api::CubismVersion,
      pub latest_supported_moc_version: public_api::MocVersion,

      /// The `Live2DCubismCore.Version` class object.
      version_class: wasm_bindgen::JsValue,
      /// The `Live2DCubismCore.Version.csmGetMocVersion` static method.
      csmGetMocVersion: js_sys::Function,

      /// The `Live2DCubismCore.Moc` class object.
      moc_class: wasm_bindgen::JsValue,
      /// The `Live2DCubismCore.Moc.fromArrayBuffer` static method.
      from_array_buffer_method: js_sys::Function,

      /// The `Live2DCubismCore.Model` class object.
      model_class: wasm_bindgen::JsValue,
      /// The `Live2DCubismCore.Model.fromMoc` static method.
      from_moc_method: js_sys::Function,

      /// `Live2DCubismCore.Drawables.resetDynamicFlags` method.
      reset_dynamic_flags_method: js_sys::Function,
    }

    #[derive(Debug)]
    pub struct JsMoc {
      pub version: public_api::MocVersion,
      /// An `Live2DCubismCore.Moc` instance object, acquired through the `Live2DCubismCore.Moc.fromArrayBuffer` static method.
      moc_instance: wasm_bindgen::JsValue,
    }

    #[derive(Debug)]
    pub struct JsModel {
      pub canvas_info: public_api::CanvasInfo,
      pub parameters: JsParameters,
      pub parts: JsParts,
      pub drawables: JsDrawables,

      pub scratch: Scratch,

      /// An `Live2DCubismCore.Model` instance object, acquired through the `Live2DCubismCore.Model.fromMoc` static method.
      model_instance: wasm_bindgen::JsValue,
      /// `Live2DCubismCore.Model.update` method.
      update_method: js_sys::Function,
      /// `Live2DCubismCore.Model.release` method.
      release_method: js_sys::Function,
    }

    #[derive(Debug)]
    pub struct JsParameters {
      pub ids: Box<[String]>,
      pub types: Box<[public_api::ParameterType]>,
      pub minimum_values: Box<[f32]>,
      pub maximum_values: Box<[f32]>,
      pub default_values: Box<[f32]>,
      pub key_value_containers: Box<[Box<[f32]>]>,

      /// `Live2DCubismCore.Parameters.values` member.
      values: js_sys::Float32Array,
    }

    #[derive(Debug)]
    pub struct JsParts {
      pub ids: Box<[String]>,
      pub parent_part_indices: Box<[Option<usize>]>,

      /// `Live2DCubismCore.Parts.opacities` member.
      opacities: js_sys::Float32Array,
    }

    #[derive(Debug)]
    pub struct JsDrawables {
      pub ids: Box<[String]>,
      pub constant_flagsets: Box<[public_api::ConstantDrawableFlagSet]>,
      pub texture_indices: Box<[usize]>,
      pub mask_containers: Box<[Box<[usize]>]>,
      pub vertex_uv_containers: Box<[Box<[public_api::Vector2]>]>,
      pub triangle_index_containers: Box<[Box<[u16]>]>,
      pub parent_part_indices: Box<[Option<usize>]>,

      /// The `drawables` member variable of `Live2DCubismCore.Model` instance object.
      /// An instance of `Live2DCubismCore.Drawables` class object.
      drawables_instance: wasm_bindgen::JsValue,
      /// `Live2DCubismCore.Drawables.dynamicFlags` member.
      dynamic_flags: js_sys::Uint8Array,
      /// `Live2DCubismCore.Drawables.drawOrders` member.
      draw_orders: js_sys::Int32Array,
      /// `Live2DCubismCore.Drawables.renderOrders` member.
      render_orders: js_sys::Int32Array,
      /// `Live2DCubismCore.Drawables.opacities` member.
      opacities: js_sys::Float32Array,
      /// `Live2DCubismCore.Drawables.vertexPositions` member.
      vertex_positions: js_sys::Array,
      /// `Live2DCubismCore.Drawables.multiplyColors` member.
      multiply_colors: js_sys::Float32Array,
      /// `Live2DCubismCore.Drawables.screenColors` member.
      screen_colors: js_sys::Float32Array,
      /// `Live2DCubismCore.Drawables.resetDynamicFlags` method.
      reset_dynamic_flags_method: js_sys::Function,
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
        let from_array_buffer_method = get_member_function(&moc_class, "fromArrayBuffer");

        let model_class = get_member_value(&live2d_cubism_core_namespace, "Model");
        let from_moc_method = get_member_function(&model_class, "fromMoc");

        let drawables_class = get_member_value(&live2d_cubism_core_namespace, "Drawables");
        let prototype = get_member_value(&drawables_class, "prototype");
        let reset_dynamic_flags_method = get_member_function(&prototype, "resetDynamicFlags");

        Self {
          cubism_version,
          latest_supported_moc_version,

          version_class,
          csmGetMocVersion,

          moc_class,
          from_array_buffer_method,

          model_class,
          from_moc_method,

          reset_dynamic_flags_method,
        }
      }
    }

    impl JsLive2DCubismCore {
      pub fn moc_from_js_array_buffer(&self, array_buffer: js_sys::ArrayBuffer) -> Option<JsMoc> {
        // `Version.csmGetMocVersion` requires a `Moc`, unlike the `csmGetMocVersion` in the Native SDK.
        let moc_instance = self.from_array_buffer_method.call1(&self.moc_class, array_buffer.as_ref()).unwrap();
        if moc_instance.is_null() {
          log::error!("Live2DCubismCore.Moc.fromArrayBuffer failed!");
          return None;
        }

        let version = self.get_moc_version(&moc_instance, &array_buffer);

        Some(JsMoc {
          version,
          moc_instance,
        })
      }
      pub fn moc_from_bytes(&self, bytes: &[u8]) -> Option<JsMoc> {
        let array = js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());
        array.copy_from(bytes);

        self.moc_from_js_array_buffer(array.buffer())
      }

      /// Equivalent to `csmGetMocVersion`.
      pub fn get_moc_version(&self, js_moc_instance: &wasm_bindgen::JsValue, array_buffer: &js_sys::ArrayBuffer) -> public_api::MocVersion {
        let moc_version = self.csmGetMocVersion.call2(
          &self.version_class, js_moc_instance, array_buffer.as_ref()
        )
        .unwrap().as_f64().unwrap() as u32;
        public_api::MocVersion::try_from(moc_version).unwrap()
      }

      pub fn model_from_moc(&self, moc: &JsMoc) -> JsModel {
        let model_instance = self.from_moc_method.call1(&self.moc_class, moc.moc_instance.as_ref()).unwrap();

        let prototype = get_member_value(&self.model_class, "prototype");
        let update_method = get_member_function(&prototype, "update");
        let release_method = get_member_function(&prototype, "release");

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
        let drawables = JsDrawables::from_drawables_instance(
          self.reset_dynamic_flags_method.clone(),
          get_member_value(&model_instance, "drawables")
        );

        let scratch = Scratch::new(&parameters, &parts, &drawables);

        JsModel {
          canvas_info,
          parameters,
          parts,
          drawables,

          scratch,

          model_instance,
          update_method,
          release_method,
        }
      }
    }

    impl JsModel {
      pub fn update(&mut self) {
        self.scratch.store_into(&self.parameters, &self.parts, &self.drawables);
        self.update_method.call0(&self.model_instance).unwrap();
        self.scratch.load_from(&self.drawables);
      }
      pub fn reset_drawable_dynamic_flags(&mut self) {
        self.drawables.reset_dynamic_flags_method.call0(&self.drawables.drawables_instance).unwrap();
        self.scratch.load_from(&self.drawables);
      }
    }
    impl Drop for JsModel {
      fn drop(&mut self) {
        self.release_method.call0(&self.model_instance).unwrap();
      }
    }

    impl JsParameters {
      /// * `parameters_instance` - The `parameters` member variable of a `Live2DCubismCore.Model` instance object, i.e an instance of `Live2DCubismCore.Parameters` class object.
      fn from_parameters_instance(parameters_instance: wasm_bindgen::JsValue) -> Self {
        let ids: Box<[_]> = get_member_array(&parameters_instance, "ids").iter()
          .map(|value| value.as_string().unwrap())
          .collect();

        let types: Box<[_]> = get_member_array(&parameters_instance, "types").iter()
          .map(|value| public_api::ParameterType::try_from(value.as_f64().unwrap() as i32).unwrap())
          .collect();

        let minimum_values: Box<[_]> = get_member_array(&parameters_instance, "minimumValues").iter()
          .map(|value| value.as_f64().unwrap() as f32)
          .collect();

        let maximum_values: Box<[_]> = get_member_array(&parameters_instance, "maximumValues").iter()
          .map(|value| value.as_f64().unwrap() as f32)
          .collect();

        let default_values: Box<[_]> = get_member_array(&parameters_instance, "defaultValues").iter()
          .map(|value| value.as_f64().unwrap() as f32)
          .collect();

        let key_value_containers: Box<[Box<[f32]>]> = get_member_array(&parameters_instance, "keyValues").iter()
          .map(|value| {
            js_sys::Array::from(&value).iter()
              .map(|value| value.as_f64().unwrap() as f32)
              .collect()
          })
          .collect();

        let values = get_member_value(&parameters_instance, "values").dyn_into::<js_sys::Float32Array>().unwrap();

        Self {
          ids,
          types,
          minimum_values,
          maximum_values,
          default_values,
          key_value_containers,

          values,
        }
      }

      pub fn to_aos(&self) -> Vec<public_api::Parameter> {
        itertools::izip!(self.ids.iter(), self.types.iter(), self.minimum_values.iter(), self.maximum_values.iter(), self.default_values.iter(), self.key_value_containers.iter())
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
      /// * `parts_instance` - The `parts` member variable of a `Live2DCubismCore.Model` instance object, i.e an instance of `Live2DCubismCore.Parts` class object.
      fn from_parts_instance(parts_instance: wasm_bindgen::JsValue) -> Self {
        let ids: Box<[_]> = get_member_array(&parts_instance, "ids").iter()
          .map(|value| value.as_string().unwrap())
          .collect();

        let parent_part_indices: Box<[_]> = get_member_array(&parts_instance, "parentIndices").iter()
          .map(|value| {
            let number = value.as_f64().unwrap();
            (number > 0.0).then_some(number as usize)
          })
          .collect();

        let opacities = get_member_value(&parts_instance, "opacities").dyn_into::<js_sys::Float32Array>().unwrap();

        Self {
          ids,
          parent_part_indices,

          opacities,
        }
      }

      pub fn to_aos(&self) -> Vec<public_api::Part> {
        itertools::izip!(self.ids.iter(), self.parent_part_indices.iter())
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
      fn from_drawables_instance(reset_dynamic_flags_method: js_sys::Function, drawables_instance: wasm_bindgen::JsValue) -> Self {
        let ids: Box<[_]> = get_member_array(&drawables_instance, "ids").iter()
          .map(|value| value.as_string().unwrap())
          .collect();

        let constant_flagsets: Box<[_]> = get_member_array(&drawables_instance, "constantFlags").iter()
          .map(|value| {
            public_api::ConstantDrawableFlagSet::new(value.as_f64().unwrap() as u8).unwrap()
          })
          .collect();

        let texture_indices: Box<[_]> = get_member_array(&drawables_instance, "textureIndices").iter()
          .map(|value| value.as_f64().unwrap() as usize)
          .collect();

        let mask_containers: Box<[_]> = get_member_array(&drawables_instance, "masks").iter()
          .map(|mask_container| {
            js_sys::Array::from(&mask_container).iter()
              .map(|mask| mask.as_f64().unwrap() as usize)
              .collect::<Box<[_]>>()
          })
          .collect();

        let vertex_uv_containers: Box<[_]> = get_member_array(&drawables_instance, "vertexUvs").iter()
          .map(|v| {
            let typed_array = v.dyn_into::<js_sys::Float32Array>().unwrap();
            float32_array_to_new_vec(&typed_array).into_boxed_slice()
          })
          .collect();

        let triangle_index_containers: Box<[_]> = get_member_array(&drawables_instance, "indices").iter()
          .map(|v| {
            let typed_array = v.dyn_into::<js_sys::Uint16Array>().unwrap();
            uint16_array_to_new_vec(&typed_array).into_boxed_slice()
          })
          .collect();

        let parent_part_indices: Box<[_]> = get_member_array(&drawables_instance, "parentPartIndices").iter()
          .map(|value| {
            let number = value.as_f64().unwrap();
            (number > 0.0).then_some(number as usize)
            })
          .collect();

        let dynamic_flags = get_member_value(&drawables_instance, "dynamicFlags").dyn_into::<js_sys::Uint8Array>().unwrap();
        let draw_orders = get_member_value(&drawables_instance, "drawOrders").dyn_into::<js_sys::Int32Array>().unwrap();
        let render_orders = get_member_value(&drawables_instance, "renderOrders").dyn_into::<js_sys::Int32Array>().unwrap();
        let opacities = get_member_value(&drawables_instance, "opacities").dyn_into::<js_sys::Float32Array>().unwrap();
        let vertex_positions = get_member_array(&drawables_instance, "vertexPositions");
        let multiply_colors = get_member_value(&drawables_instance, "multiplyColors").dyn_into::<js_sys::Float32Array>().unwrap();
        let screen_colors = get_member_value(&drawables_instance, "screenColors").dyn_into::<js_sys::Float32Array>().unwrap();

        Self {
          ids,
          constant_flagsets,
          texture_indices,
          mask_containers,
          vertex_uv_containers,
          triangle_index_containers,
          parent_part_indices,

          drawables_instance,
          dynamic_flags,
          draw_orders,
          render_orders,
          opacities,
          vertex_positions,
          multiply_colors,
          screen_colors,
          reset_dynamic_flags_method,
        }
      }

      pub fn to_aos(&self) -> Vec<public_api::Drawable> {
        itertools::izip!(self.ids.iter(), self.constant_flagsets.iter(), self.texture_indices.iter(), self.mask_containers.iter(), self.vertex_uv_containers.iter(), self.triangle_index_containers.iter(), self.parent_part_indices.iter())
          .map(|(id, constant_flagset, texture_index, mask_container, vertex_uv_container, triangle_index_container, parent_part_index)| {
            public_api::Drawable {
              id: id.clone(),
              constant_flagset: *constant_flagset,
              texture_index: *texture_index,
              masks: mask_container.clone(),
              vertex_uvs: vertex_uv_container.clone(),
              triangle_indices: triangle_index_container.clone(),
              parent_part_index: *parent_part_index,
            }
          })
          .collect()
      }
    }

    /// Scratch buffer for dynamic values.
    #[derive(Debug)]
    pub struct Scratch {
      parameter_values: Box<[f32]>,
      part_opacities: Box<[f32]>,
      drawable_dynamic_flagsets: Box<[public_api::DynamicDrawableFlagSet]>,
      drawable_draw_orders: Box<[i32]>,
      drawable_render_orders: Box<[i32]>,
      drawable_opacities: Box<[f32]>,
      drawable_vertex_position_containers: Box<[Box<[public_api::Vector2]>]>,
      drawable_vertex_position_container_refs: Box<[&'static [public_api::Vector2]]>,
      drawable_multiply_colors: Box<[public_api::Vector4]>,
      drawable_screen_colors: Box<[public_api::Vector4]>,
    }
    impl Scratch {
      pub fn parameter_values(&self) -> &[f32] { &self.parameter_values }
      pub fn parameter_values_mut(&mut self) -> &mut [f32] { &mut self.parameter_values }
      pub fn part_opacities(&self) -> &[f32] { &self.part_opacities }
      pub fn part_opacities_mut(&mut self) -> &mut [f32] { &mut self.part_opacities }
      pub fn drawable_dynamic_flagsets(&self) -> &[public_api::DynamicDrawableFlagSet] { &self.drawable_dynamic_flagsets }
      pub fn drawable_dynamic_flagsets_mut(&mut self) -> &mut [public_api::DynamicDrawableFlagSet] { &mut self.drawable_dynamic_flagsets }
      pub fn drawable_draw_orders(&self) -> &[i32] { &self.drawable_draw_orders }
      pub fn drawable_render_orders(&self) -> & [i32] { &self.drawable_render_orders }
      pub fn drawable_opacities(&self) -> &[f32] { &self.drawable_opacities }
      pub fn drawable_vertex_position_containers(&self) -> &[&[public_api::Vector2]] { &self.drawable_vertex_position_container_refs }
      pub fn drawable_multiply_colors(&self) -> &[public_api::Vector4] { &self.drawable_multiply_colors }
      pub fn drawable_screen_colors(&self) -> &[public_api::Vector4] { &self.drawable_screen_colors }

      fn new(parameters: &JsParameters, parts: &JsParts, drawables: &JsDrawables) -> Self {
        let parameter_values = float32_array_to_new_vec(&parameters.values).into_boxed_slice();
        let part_opacities = float32_array_to_new_vec(&parts.opacities).into_boxed_slice();
        let drawable_dynamic_flagsets = uint8_array_to_new_vec::<public_api::DynamicDrawableFlagSet>(&drawables.dynamic_flags).into_boxed_slice();
        let drawable_draw_orders = int32_array_to_new_vec(&drawables.draw_orders).into_boxed_slice();
        let drawable_render_orders = int32_array_to_new_vec(&drawables.render_orders).into_boxed_slice();
        let drawable_opacities = float32_array_to_new_vec(&drawables.opacities).into_boxed_slice();

        let drawable_vertex_position_containers: Box<[_]> = drawables.vertex_positions.iter()
          .map(|f32_array| {
            let f32_array = f32_array.dyn_into::<js_sys::Float32Array>().unwrap();
            float32_array_to_new_vec::<public_api::Vector2>(&f32_array).into_boxed_slice()
          })
          .collect();
        let drawable_vertex_position_container_refs: Box<[_]> = drawable_vertex_position_containers.iter()
          .map(|v| {
            // SAFETY: A boxed slice is pointer-stable.
            unsafe { std::slice::from_raw_parts(v.as_ptr(), v.len()) }}
          )
          .collect();

        let drawable_multiply_colors = float32_array_to_new_vec::<public_api::Vector4>(&drawables.multiply_colors).into_boxed_slice();
        let drawable_screen_colors = float32_array_to_new_vec::<public_api::Vector4>(&drawables.screen_colors).into_boxed_slice();

        Self {
          parameter_values,
          part_opacities,
          drawable_dynamic_flagsets,
          drawable_draw_orders,
          drawable_render_orders,
          drawable_opacities,
          drawable_vertex_position_containers,
          drawable_vertex_position_container_refs,
          drawable_multiply_colors,
          drawable_screen_colors,
        }
      }

      fn store_into(&mut self, parameters: &JsParameters, parts: &JsParts, drawables: &JsDrawables) {
        parameters.values.copy_from(&self.parameter_values);
        parts.opacities.copy_from(&self.part_opacities);
        {
          // SAFETY: Size and alignment asserted to match.
          let src = unsafe {
            std::slice::from_raw_parts(self.drawable_dynamic_flagsets.as_ptr() as *const u8, self.drawable_dynamic_flagsets.len())
          };
          drawables.dynamic_flags.copy_from(src);
        }
      }
      fn load_dynamic_flags_from(&mut self, drawables: &JsDrawables) {
        uint8_array_overwrite_slice(&mut self.drawable_dynamic_flagsets, &drawables.dynamic_flags);
      }
      fn load_from(&mut self, drawables: &JsDrawables) {
        self.load_dynamic_flags_from(drawables);

        int32_array_overwrite_slice(&mut self.drawable_draw_orders, &drawables.draw_orders);
        int32_array_overwrite_slice(&mut self.drawable_render_orders, &drawables.render_orders);
        f32_array_overwrite_slice(&mut self.drawable_opacities, &drawables.opacities);

        for (vertex_position_container, f32_array) in itertools::izip!(self.drawable_vertex_position_containers.iter_mut(), drawables.vertex_positions.iter()) {
          let f32_array = f32_array.dyn_into::<js_sys::Float32Array>().unwrap();
          f32_array_overwrite_slice(vertex_position_container, &f32_array);
        }

        f32_array_overwrite_slice(&mut self.drawable_multiply_colors, &drawables.multiply_colors);
        f32_array_overwrite_slice(&mut self.drawable_screen_colors, &drawables.screen_colors);
      }
    }

    fn get_member_value<N: AsRef<str> + std::fmt::Debug>(value: &wasm_bindgen::JsValue, name: N) -> wasm_bindgen::JsValue {
      js_sys::Reflect::get(value, &name.as_ref().into()).unwrap_or_else(|e| panic!("No member {name:?}! {e:?}"))
    }
    /// Requires `N` to be [`Clone`] to allow error reporting when panicking.
    fn get_member_function<N: AsRef<str> + Clone + std::fmt::Debug>(value: &wasm_bindgen::JsValue, name: N) -> js_sys::Function {
      get_member_value(value, name.clone()).dyn_into().unwrap_or_else(|e| panic!("member {name:?} not a Function! {e:?}"))
    }
    fn get_member_array<N: AsRef<str> + std::fmt::Debug>(value: &wasm_bindgen::JsValue, name: N) -> js_sys::Array {
      js_sys::Array::from(&get_member_value(value, name))
    }

    fn uint8_array_overwrite_slice<O>(dst: &mut [O], typed_array: &js_sys::Uint8Array) {
      typed_array_overwrite_slice(dst, typed_array.length(), |ptr| unsafe { typed_array.raw_copy_to_ptr(ptr) })
    }
    fn int32_array_overwrite_slice<O>(dst: &mut [O], typed_array: &js_sys::Int32Array) {
      typed_array_overwrite_slice(dst, typed_array.length(), |ptr| unsafe { typed_array.raw_copy_to_ptr(ptr) })
    }
    fn f32_array_overwrite_slice<O>(dst: &mut [O], typed_array: &js_sys::Float32Array) {
      typed_array_overwrite_slice(dst, typed_array.length(), |ptr| unsafe { typed_array.raw_copy_to_ptr(ptr) })
    }
    fn typed_array_overwrite_slice<O, E, W: FnOnce(*mut E)>(dst: &mut [O], length: u32, writer: W) {
      let src_element_size = std::mem::size_of::<E>();
      let dst_element_size = std::mem::size_of::<O>();
      let dst_len = length as usize / (dst_element_size / src_element_size);
      assert!(dst_len <= dst.len());

      writer(dst.as_mut_ptr() as *mut E)
    }

    fn uint8_array_to_new_vec<O>(typed_array: &js_sys::Uint8Array) -> Vec<O> {
      typed_array_to_new_vec(typed_array.length(), |ptr| unsafe { typed_array.raw_copy_to_ptr(ptr); })
    }
    fn uint16_array_to_new_vec<O>(typed_array: &js_sys::Uint16Array) -> Vec<O> {
      typed_array_to_new_vec(typed_array.length(), |ptr| unsafe { typed_array.raw_copy_to_ptr(ptr); })
    }
    fn int32_array_to_new_vec<O>(typed_array: &js_sys::Int32Array) -> Vec<O> {
      typed_array_to_new_vec(typed_array.length(), |ptr| unsafe { typed_array.raw_copy_to_ptr(ptr); })
    }
    fn float32_array_to_new_vec<O>(typed_array: &js_sys::Float32Array) -> Vec<O> {
      typed_array_to_new_vec(typed_array.length(), |ptr| unsafe { typed_array.raw_copy_to_ptr(ptr); })
    }
    fn typed_array_to_new_vec<O, E, W: FnOnce(*mut E)>(length: u32, writer: W) -> Vec<O> {
      let src_element_size = std::mem::size_of::<E>();
      let dst_element_size = std::mem::size_of::<O>();

      let dst_len = length as usize / (dst_element_size / src_element_size);
      let mut dst = Vec::<O>::with_capacity(dst_len);
      writer(dst.as_mut_ptr() as *mut E);

      // SAFETY:
      // 1. Constructed with `with_capacity`.
      // 2. `writer` must have initialized the elements.
      unsafe {
        dst.set_len(dst_len);
      }
      dst
    }
  }
}

#[cfg(test)]
pub mod public_api_tests {
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

    let cubism_core = public_api::CubismCore::default();
    log::info!("Live2D Cubism Core Version: {}", cubism_core.version());
    log::info!("Latest supported moc version: {}", cubism_core.latest_supported_moc_version());

    {
      let invalid_moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/Samples/Resources/Hiyori/Hiyori.model3.json"));
      cubism_core.moc_from_bytes(invalid_moc_bytes).expect_err("moc_from_bytes should fail");
    }

    // let moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/Samples/Resources/Hiyori/Hiyori.moc3"));
    let moc_bytes = include_bytes!(concat!(ENV_CUBISM_SDK_DIR!(), "/AdditionalSamples/simple/runtime/simple.moc3"));

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
