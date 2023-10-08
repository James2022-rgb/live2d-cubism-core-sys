#![cfg(feature = "core")]

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod base_types;
pub mod model_types;

pub use base_types::{Vector2, Vector4};
pub use base_types::{MocError, CubismVersion, MocVersion};

pub use model_types::CanvasInfo;
pub use model_types::{ParameterType, Parameter};
pub use model_types::Part;
pub use model_types::{ConstantDrawableFlags, ConstantDrawableFlagSet, DynamicDrawableFlags, DynamicDrawableFlagSet, Drawable};

mod internal;

use internal::platform_impl::{PlatformCubismCore, PlatformMoc, PlatformModelStatic, PlatformModelDynamic};

if_native! {
  use static_assertions::assert_impl_all;

  // TODO: Assert `Send` and `Sync` for `CubismCore`?
  assert_impl_all!(Moc: Send, Sync);
  assert_impl_all!(Model: Send, Sync);
}

use internal::platform_iface::{
  PlatformCubismCoreInterface as _,
  PlatformMocInterface as _,
  PlatformModelStaticInterface as _,
  PlatformModelDynamicInterface as _,
};

/// Encapsulates the functionality of _Live2D® Cubism SDK Core_.
#[derive(Debug, Default)]
pub struct CubismCore {
  #[allow(dead_code)]
  inner: PlatformCubismCore,
}
impl CubismCore {
  /// Sets a global log handler function to intercept _Live2D® Cubism SDK Core_'s internal log.
  ///
  /// ## Safety
  /// - Causes a slight memory leak (a heap-allocated closure).
  /// - Must be externally synchronized with calls to `csmGetLogFunction` and `csmSetLogFunction`.
  ///   This is a precaution since their threading behavior is not well documented.
  ///
  /// ## Platform-specific
  /// - **Android:** Unsupported; `libffi-sys-rs` fails to build for Android on Windows.
  /// - **Web:** Unsupported.
  #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
  pub unsafe fn set_log_function<F>(f: F)
  where
    F: FnMut(&str) + Send + 'static,
  {
    PlatformCubismCore::set_log_function(f)
  }

  /// Gets the version of _Live2D® Cubism SDK Core_.
  pub fn version(&self) -> CubismVersion {
    self.inner.version()
  }
  /// Gets the latest moc3 version supported by _Live2D® Cubism SDK Core_.
  pub fn latest_supported_moc_version(&self) -> MocVersion {
    self.inner.latest_supported_moc_version()
  }

  /// Deserializes a `Moc` from bytes.
  pub fn moc_from_bytes(&self, bytes: &[u8]) -> Result<Moc, MocError> {
    self.inner
      .platform_moc_from_bytes(bytes)
      .map(|(moc_version, platform_moc)| {
        Moc {
          version: moc_version,
          inner: platform_moc
        }
      })
  }
}

/// Cubism moc.
#[derive(Debug)]
pub struct Moc {
  // TODO: Incorporate `version` into `PlatformMoc`?
  // TODO: Rename `inner`?
  version: MocVersion,
  inner: PlatformMoc,
}
impl Moc {
  pub fn version(&self) -> MocVersion {
    self.version
  }
}

/// Cubism model.
#[derive(Debug)]
pub struct Model {
  model_static: ModelStatic,
  model_dynamic: RwLock<ModelDynamic>,
}
impl Model {
  pub fn from_moc(moc: &Moc) -> Self {
    let (platform_model_static, platform_model_dynamic) = moc.inner.new_platform_model();

    let model_static = ModelStatic {
      inner: platform_model_static,
    };
    let model_dynamic = ModelDynamic {
      inner: platform_model_dynamic,
    };

    Self {
      model_static,
      model_dynamic: RwLock::new(model_dynamic),
    }
  }

  /// Gets [`ModelStatic`].
  pub fn get_static(&self) -> &ModelStatic {
    &self.model_static
  }

  /// Acquires a read (shared) lock for [`ModelDynamic`].
  pub fn read_dynamic(&self) -> ModelDynamicReadLockGuard {
    ModelDynamicReadLockGuard {
      inner: self.model_dynamic.read(),
    }
  }
  /// Acquires a write (mutable) lock for [`ModelDynamic`].
  pub fn write_dynamic(&self) -> ModelDynamicWriteLockGuard {
    ModelDynamicWriteLockGuard {
      inner: self.model_dynamic.write(),
    }
  }
}

/// Static properties of a model.
#[derive(Debug)]
pub struct ModelStatic {
  inner: PlatformModelStatic,
}
impl ModelStatic {
  pub fn canvas_info(&self) -> CanvasInfo { self.inner.canvas_info() }
  pub fn parameters(&self) -> &[Parameter] { self.inner.parameters() }
  pub fn parts(&self) -> &[Part] { self.inner.parts() }
  pub fn drawables(&self) -> &[Drawable] { self.inner.drawables() }
}

/// Dynamic states of a model.
#[derive(Debug)]
pub struct ModelDynamic {
  inner: PlatformModelDynamic,
}
impl ModelDynamic {
  pub fn parameter_values(&self) -> &[f32] { self.inner.parameter_values() }
  pub fn parameter_values_mut(&mut self) -> &mut [f32] { self.inner.parameter_values_mut() }
  pub fn part_opacities(&self) -> &[f32] { self.inner.part_opacities() }
  pub fn part_opacities_mut(&mut self) -> &mut [f32] { self.inner.part_opacities_mut() }
  pub fn drawable_dynamic_flagsets(&self) -> &[DynamicDrawableFlagSet] { self.inner.drawable_dynamic_flagsets() }

  pub fn drawable_draw_orders(&self) -> &[i32] { self.inner.drawable_draw_orders() }
  pub fn drawable_render_orders(&self) -> &[i32] { self.inner.drawable_render_orders() }
  pub fn drawable_opacities(&self) -> &[f32] { self.inner.drawable_opacities() }
  pub fn drawable_vertex_position_containers(&self) -> &[&[Vector2]] { self.inner.drawable_vertex_position_containers() }
  pub fn drawable_multiply_colors(&self) -> &[Vector4] { self.inner.drawable_multiply_colors() }
  pub fn drawable_screen_colors(&self) -> &[Vector4] { self.inner.drawable_screen_colors() }

  pub fn update(&mut self) {
    self.inner.update()
  }
  pub fn reset_drawable_dynamic_flags(&mut self) {
    self.inner.reset_drawable_dynamic_flags()
  }
}

#[derive(Debug)]
pub struct ModelDynamicReadLockGuard<'a> {
  inner: RwLockReadGuard<'a, ModelDynamic>,
}
impl<'a> std::ops::Deref for ModelDynamicReadLockGuard<'a> {
  type Target = ModelDynamic;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

#[derive(Debug)]
pub struct ModelDynamicWriteLockGuard<'a> {
  inner: RwLockWriteGuard<'a, ModelDynamic>,
}
impl<'a> std::ops::Deref for ModelDynamicWriteLockGuard<'a> {
  type Target = ModelDynamic;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}
impl<'a> std::ops::DerefMut for ModelDynamicWriteLockGuard<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}



//
// Internal
//

/*
#[cfg(not(target_arch = "wasm32"))]
mod platform_impl {
  use std::sync::Arc;

  use static_assertions::{assert_eq_align, assert_eq_size};

  use crate::memory::AlignedStorage;

  use crate::sys::*;

  use super::platform_iface;

  assert_eq_align!(super::Vector2, csmVector2);
  assert_eq_size!(super::Vector2, csmVector2);
  assert_eq_align!(super::Vector4, csmVector4);
  assert_eq_size!(super::Vector4, csmVector4);

  #[derive(Debug, Default)]
  pub struct PlatformCubismCore;
  impl platform_iface::PlatformCubismCoreIface for PlatformCubismCore {
    type PlatformMoc = PlatformMoc;

    unsafe fn set_log_function<F>(mut f: F)
    where
      F: FnMut(&str) + Send + 'static,
    {
      let trampoline = Box::new(move |message: *const core::ffi::c_char| {
        let str = unsafe { core::ffi::CStr::from_ptr(message).to_str().unwrap() };
        f(str);
      });
      let trampoline = Box::leak(trampoline);

      let trampoline = libffi::high::ClosureMut1::new(trampoline);
      let &code = trampoline.code_ptr();

      unsafe {
        csmSetLogFunction(Some(std::mem::transmute(code)));
      }

      std::mem::forget(trampoline);
    }

    fn version(&self) -> super::CubismVersion {
      super::CubismVersion(unsafe { csmGetVersion() })
    }
    fn latest_supported_moc_version(&self) -> super::MocVersion {
      unsafe { csmGetLatestMocVersion() }.try_into().unwrap()
    }

    fn platform_moc_from_bytes(&self, bytes: &[u8]) -> Result<(super::MocVersion, self::PlatformMoc), super::MocError> {
      const MOC_ALIGNMENT: usize = csmAlignofMoc as usize;

      let mut aligned_storage = AlignedStorage::new(bytes.len(), MOC_ALIGNMENT).unwrap();
      aligned_storage.copy_from_slice(bytes);

      let size_in_u32: u32 = bytes.len().try_into().expect("Size should fit in a u32");

      let moc_version = unsafe {
        csmGetMocVersion(aligned_storage.as_mut_ptr() as *mut _, size_in_u32)
      };
      let moc_version = super::MocVersion::try_from(moc_version).map_err(|_| super::MocError::InvalidMoc)?;

      if self.latest_supported_moc_version() < moc_version {
        return Err(super::MocError::UnsupportedMocVersion { given: moc_version, latest_supported: self.latest_supported_moc_version() });
      }

      let csm_moc = unsafe {
        csmReviveMocInPlace(aligned_storage.as_mut_ptr() as *mut _, size_in_u32)
      };

      Ok(
        (moc_version,
        PlatformMoc {
          csm_moc,
          moc_storage: Arc::new(aligned_storage),
        })
      )
    }
  }

  #[derive(Debug)]
  pub struct PlatformMoc {
    csm_moc: *mut csmMoc,
    /// This is an [`Arc`] because the memory block for a `csmMoc` needs to outlive
    /// the memory blocks for all `csmModel`s generated from it.
    moc_storage: Arc<AlignedStorage>,
  }

  // SAFETY: The underlying `csmMoc` is never mutated.
  unsafe impl Send for PlatformMoc {}
  unsafe impl Sync for PlatformMoc {}

  #[allow(dead_code)]
  #[derive(Debug)]
  pub struct PlatformModel {
    model_storage: AlignedStorage,
    /// The memory block for the `csmMoc` used to generate this `csmModel`, which needs to outlive this `PlatformModel`.
    moc_storage: Arc<AlignedStorage>,
  }
  impl platform_iface::PlatformModelIface for PlatformModel {
    type PlatformMoc = PlatformMoc;

    fn from_platform_moc(platform_moc: &Self::PlatformMoc) -> (super::ModelStatic, super::ModelDynamic, Self) {
      const MODEL_ALIGNMENT: usize = csmAlignofModel as usize;

      let storage_size = unsafe {
        csmGetSizeofModel(platform_moc.csm_moc)
      };

      let mut aligned_storage = AlignedStorage::new(storage_size as _, MODEL_ALIGNMENT).unwrap();

      let csm_model = unsafe {
        csmInitializeModelInPlace(platform_moc.csm_moc, aligned_storage.as_mut_ptr() as *mut _, storage_size)
      };

      let canvas_info = unsafe {
        let mut size_in_pixels = csmVector2 { X: 0.0, Y: 0.0 };
        let mut origin_in_pixels = csmVector2 { X: 0.0, Y: 0.0 };
        let mut pixels_per_unit: f32 = 0.0;

        csmReadCanvasInfo(csm_model, &mut size_in_pixels, &mut origin_in_pixels, &mut pixels_per_unit);

        super::CanvasInfo {
          size_in_pixels: (size_in_pixels.X, size_in_pixels.Y),
          origin_in_pixels: (origin_in_pixels.X, origin_in_pixels.Y),
          pixels_per_unit,
        }
      };

      let parameters: Box<[_]> = unsafe {
        let count: usize = csmGetParameterCount(csm_model).try_into().unwrap();

        let ids: Vec<_> = std::slice::from_raw_parts(csmGetParameterIds(csm_model), count).iter()
          .map(|&c_str_ptr| to_string(c_str_ptr))
          .collect();

        let types: Vec<_> = std::slice::from_raw_parts(csmGetParameterTypes(csm_model), count).iter()
          .map(|value| super::ParameterType::try_from(*value).unwrap())
          .collect();

        let minimum_values = std::slice::from_raw_parts(csmGetParameterMinimumValues(csm_model), count);
        let maximum_values = std::slice::from_raw_parts(csmGetParameterMaximumValues(csm_model), count);
        let default_values = std::slice::from_raw_parts(csmGetParameterDefaultValues(csm_model), count);

        let key_value_containers: Box<[_]> = {
          let key_counts = std::slice::from_raw_parts(csmGetParameterKeyCounts(csm_model), count);
          let key_value_ptrs = std::slice::from_raw_parts(csmGetParameterKeyValues(csm_model), count);

          itertools::izip!(key_counts, key_value_ptrs)
            .map(|(&key_count, &key_value_ptr)| {
              std::slice::from_raw_parts(key_value_ptr, key_count.try_into().unwrap()).to_vec().into_boxed_slice()
            })
            .collect()
        };

        itertools::izip!(ids, types, minimum_values, maximum_values, default_values, key_value_containers.iter())
          .map(|(id, ty, &minimum_value, &maximum_value, &default_value, key_value_container)| {
            super::Parameter {
              id,
              ty,
              value_range: (minimum_value, maximum_value),
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
            super::Part {
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
          .map(|value| super::ConstantDrawableFlagSet::new(*value).unwrap())
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
              std::slice::from_raw_parts(vertex_uv_ptr as *const super::Vector2, vertex_count as _).to_vec().into_boxed_slice()
            })
            .collect()
        };

        let triangle_index_containers: Box<[_]> = {
          let triangle_index_counts = std::slice::from_raw_parts(csmGetDrawableIndexCounts(csm_model), count as _);
          let triangle_index_ptrs = std::slice::from_raw_parts(csmGetDrawableIndices(csm_model), count as _);

          itertools::izip!(triangle_index_counts, triangle_index_ptrs)
            .map(|(&triangle_index_count, &triangle_index_ptr)| {
              if triangle_index_count > 0 {
                std::slice::from_raw_parts(triangle_index_ptr, triangle_index_count as _).to_vec().into_boxed_slice()
              } else {
                [].into()
              }
            })
            .collect()
        };

        let parent_part_indices: Vec<_> = std::slice::from_raw_parts(csmGetDrawableParentPartIndices(csm_model), count as _).iter()
          .map(|&value| (value > 0).then_some(value as usize)).collect();

        itertools::izip!(ids, constant_flagsets, texture_indices, mask_containers.iter(), vertex_uv_containers.iter(), triangle_index_containers.iter(), parent_part_indices)
          .map(|(id, constant_flagset, texture_index, mask_container, vertex_uv_container, triangle_index_container, parent_part_index),| {
            super::Drawable {
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

      let dynamic = super::ModelDynamic {
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

      let model_static = super::ModelStatic {
        canvas_info,
        parameters,
        parts,
        drawables,
      };

      let inner = PlatformModel {
        model_storage: aligned_storage,
        moc_storage: Arc::clone(&platform_moc.moc_storage),
      };

      (model_static, dynamic, inner)
    }
  }

  // TODO: Do something about these `'static`s?

  #[derive(Debug)]
  pub struct PlatformModelDynamic {
    csm_model: *mut csmModel,
    parameter_values: &'static mut [f32],
    part_opactities: &'static mut [f32],
    drawable_dynamic_flagsets: &'static mut [super::DynamicDrawableFlagSet],
    drawable_draw_orders: &'static [i32],
    drawable_render_orders: &'static [i32],
    drawable_opacities: &'static [f32],
    vertex_position_containers: VertexPositionContainers<'static>,
    drawable_multiply_colors: &'static [super::Vector4],
    drawable_screen_colors: &'static [super::Vector4],
  }

  // SAFETY: The underlying `csmModel` is never mutated except through methods taking a mutable reference.
  unsafe impl Send for PlatformModelDynamic {}
  unsafe impl Sync for PlatformModelDynamic {}

  impl platform_iface::PlatformModelDynamicIface for PlatformModelDynamic {
    fn parameter_values(&self) -> &[f32] {
      self.parameter_values
    }
    fn parameter_values_mut(&mut self) -> &mut [f32] {
      self.parameter_values
    }
    fn part_opacities(&self) -> &[f32] {
      self.part_opactities
    }
    fn part_opacities_mut(&mut self) -> &mut [f32] {
      self.part_opactities
    }
    fn drawable_dynamic_flagsets(&self) -> &[super::DynamicDrawableFlagSet] {
      self.drawable_dynamic_flagsets
    }
    fn drawable_dynamic_flagsets_mut(&mut self) -> &mut [super::DynamicDrawableFlagSet] {
      self.drawable_dynamic_flagsets
    }

    fn drawable_draw_orders(&self) -> &[i32] {
      self.drawable_draw_orders
    }
    fn drawable_render_orders(&self) -> &[i32] {
      self.drawable_render_orders
    }
    fn drawable_opacities(&self) -> &[f32] {
      self.drawable_opacities
    }
    fn drawable_vertex_position_containers(&self) -> &[&[super::Vector2]] {
      &self.vertex_position_containers.inner
    }
    fn drawable_multiply_colors(&self) -> &[super::Vector4] {
      self.drawable_multiply_colors
    }
    fn drawable_screen_colors(&self) -> &[super::Vector4] {
      self.drawable_screen_colors
    }

    fn update(&mut self) {
      unsafe {
        csmUpdateModel(self.csm_model);
      }

      self.vertex_position_containers = VertexPositionContainers::new(self.csm_model);
    }
    fn reset_drawable_dynamic_flags(&mut self) {
      unsafe {
        csmResetDrawableDynamicFlags(self.csm_model);
      }
    }
  }

  #[derive(Debug)]
  struct VertexPositionContainers<'a> {
    inner: Box<[&'a [super::Vector2]]>,
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
              std::slice::from_raw_parts(vertex_position_ptr as *const super::Vector2, vertex_count as _)
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
*/

/*
#[cfg(target_arch = "wasm32")]
mod platform_impl {
  use std::sync::Arc;

  use js::*;

  use super::platform_iface;

  #[derive(Debug, Default)]
  pub struct PlatformCubismCore {
    js_cubism_core: Arc<JsLive2DCubismCore>,
  }
  impl platform_iface::PlatformCubismCoreIface for PlatformCubismCore {
    type PlatformMoc = PlatformMoc;

    fn version(&self) -> super::CubismVersion {
      self.js_cubism_core.cubism_version
    }
    fn latest_supported_moc_version(&self) -> super::MocVersion {
      self.js_cubism_core.latest_supported_moc_version
    }

    fn platform_moc_from_bytes(&self, bytes: &[u8]) -> Result<(super::MocVersion, self::PlatformMoc), super::MocError> {
      let array = js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());
      array.copy_from(bytes);

      let js_moc = self.js_cubism_core.moc_from_js_array_buffer(array.buffer());
      js_moc
        .map(|js_moc| {
          (js_moc.version,
          PlatformMoc {
            js_moc,
            js_cubism_core: Arc::clone(&self.js_cubism_core),
          })
        })
        .ok_or(super::MocError::InvalidMoc)
    }
  }

  #[derive(Debug)]
  pub struct PlatformMoc {
    js_moc: JsMoc,
    js_cubism_core: Arc<JsLive2DCubismCore>,
  }

  #[derive(Debug)]
  pub struct PlatformModel;
  impl platform_iface::PlatformModelIface for PlatformModel {
    type PlatformMoc = PlatformMoc;

    fn from_platform_moc(platform_moc: &Self::PlatformMoc) -> (super::ModelStatic, super::ModelDynamic, Self) {
      let js_model = platform_moc.js_cubism_core.model_from_moc(&platform_moc.js_moc);

      let canvas_info = js_model.canvas_info;
      let parameters = js_model.parameters.to_aos().into_boxed_slice();
      let parts = js_model.parts.to_aos().into_boxed_slice();
      let drawables = js_model.drawables.to_aos().into_boxed_slice();

      let model_static = super::ModelStatic {
        canvas_info,
        parameters,
        parts,
        drawables,
      };

      let dynamic = super::ModelDynamic {
        inner: PlatformModelDynamic {
          js_model,
        }
      };

      (model_static, dynamic, PlatformModel)
    }
  }

  #[derive(Debug)]
  pub struct PlatformModelDynamic {
    js_model: JsModel,
  }
  impl platform_iface::PlatformModelDynamicIface for PlatformModelDynamic {
    fn parameter_values(&self) -> &[f32] {
      self.js_model.scratch.parameter_values()
    }
    fn parameter_values_mut(&mut self) -> &mut [f32] {
      self.js_model.scratch.parameter_values_mut()
    }
    fn part_opacities(&self) -> &[f32] {
      self.js_model.scratch.part_opacities()
    }
    fn part_opacities_mut(&mut self) -> &mut [f32] {
      self.js_model.scratch.part_opacities_mut()
    }
    fn drawable_dynamic_flagsets(&self) -> &[super::DynamicDrawableFlagSet] {
      self.js_model.scratch.drawable_dynamic_flagsets()
    }
    fn drawable_dynamic_flagsets_mut(&mut self) -> &mut [super::DynamicDrawableFlagSet] {
      self.js_model.scratch.drawable_dynamic_flagsets_mut()
    }

    fn drawable_draw_orders(&self) -> &[i32] {
      self.js_model.scratch.drawable_draw_orders()
    }
    fn drawable_render_orders(&self) -> &[i32] {
      self.js_model.scratch.drawable_render_orders()
    }
    fn drawable_opacities(&self) -> &[f32] {
      self.js_model.scratch.drawable_opacities()
    }
    fn drawable_vertex_position_containers(&self) -> &[&[super::Vector2]] {
      self.js_model.scratch.drawable_vertex_position_containers()
    }
    fn drawable_multiply_colors(&self) -> &[super::Vector4] {
      self.js_model.scratch.drawable_multiply_colors()
    }
    fn drawable_screen_colors(&self) -> &[super::Vector4] {
      self.js_model.scratch.drawable_screen_colors()
    }

    fn update(&mut self) {
      self.js_model.update()
    }
    fn reset_drawable_dynamic_flags(&mut self) {
      self.js_model.reset_drawable_dynamic_flags()
    }
  }
}
*/

#[cfg(not(target_arch = "wasm32"))]
macro_rules! if_native {
  ($($code:tt)*) => {
    $($code)*
  };
}
#[cfg(target_arch = "wasm32")]
macro_rules! if_native {
  ($($code:tt)*) => {};
}
use if_native;
