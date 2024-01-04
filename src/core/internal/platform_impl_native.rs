
use std::sync::Arc;

use static_assertions::{assert_eq_align, assert_eq_size};
use parking_lot::Mutex;

use crate::memory::AlignedStorage;
use crate::sys::*;

use super::platform_iface::{Vector2, Vector4};
use super::platform_iface::{MocError, CubismVersion, MocVersion};
use super::platform_iface::{CanvasInfo, Parameter, Part, Drawable};
use super::platform_iface::{ConstantDrawableFlagSet, DynamicDrawableFlagSet};
use super::platform_iface::{PlatformCubismCoreInterface, PlatformMocInterface, PlatformModelStaticInterface, PlatformModelDynamicInterface};

use super::super::base_types::{TextureIndex, DrawableIndex};
use super::super::model_types::ParameterType;

assert_eq_align!(Vector2, csmVector2);
assert_eq_size!(Vector2, csmVector2);
assert_eq_align!(Vector4, csmVector4);
assert_eq_size!(Vector4, csmVector4);

type Trampoline = extern "C" fn(*const core::ffi::c_char);
type BoxedTrampoline = Box<Trampoline>;

/// This idea is from how [glow](https://github.com/grovesNL/glow/blob/e94890f17ed29e4d01b8e9c6173a735933bc3b6c/src/native.rs#L12-L17) handles debug callbacks.
///
/// We store a boxed trampoline (i.e., `Box<Box<extern "C" fn(...)>>`) as a raw pointer, so that it can be
/// referenced by the C API and later converted back into a `Box` and dropped.
///
/// We use a raw pointer here because `Box` aliasing rules are not fully defined, so we can't
/// guarantee that it's not undefined behavior to keep a `Box` here while it's used as a raw
/// pointer in the C API.
///
struct LogFunctionTrampolineRaw {
  raw_boxed_boxed_trampoline: *mut Box<Trampoline>,
  boxed_user_func: Box<dyn FnMut(&str) + Send + 'static>,
}

unsafe impl Send for LogFunctionTrampolineRaw {}
unsafe impl Sync for LogFunctionTrampolineRaw {}

impl Drop for LogFunctionTrampolineRaw {
  fn drop(&mut self) {
    unsafe {
      // Convert the `*mut Box<Trampoline>` back into a `Box<Box<Trampoline>>` and drop it.
      let boxed_boxed_trampoline = Box::from_raw(self.raw_boxed_boxed_trampoline as *mut BoxedTrampoline);
      let boxed_trampoline = *boxed_boxed_trampoline;
      drop(boxed_trampoline);
    }
  }
}

static mut S_LOG_FUNCTION_TRAMPOLINE_RAW: Mutex<Option<LogFunctionTrampolineRaw>> = Mutex::new(None);

#[derive(Debug, Default)]
pub struct PlatformCubismCore {
  _private: (),
}

impl PlatformCubismCoreInterface for PlatformCubismCore {
  type PlatformMoc = PlatformMoc;

  unsafe fn set_log_function<F>(f: F)
  where
    F: FnMut(&str) + Send + 'static,
  {
    extern "C" fn trampoline(message_c_str: *const core::ffi::c_char) {
      let message_str = unsafe { core::ffi::CStr::from_ptr(message_c_str).to_str().unwrap() };

      // SAFETY: We use a `Mutex`.
      let mut log_function_trampoline_raw = unsafe { S_LOG_FUNCTION_TRAMPOLINE_RAW.lock() };
      if let Some(log_function_trampoline_raw) = log_function_trampoline_raw.as_mut() {
        (log_function_trampoline_raw.boxed_user_func)(message_str);
      }
    }

    let mut log_function_trampoline_raw = S_LOG_FUNCTION_TRAMPOLINE_RAW.lock();

    let boxed_trampoline = BoxedTrampoline::new(trampoline);
    let boxed_boxed_trampoline = Box::new(boxed_trampoline);
    let raw_boxed_boxed_trampoline: *mut Box<Trampoline> = Box::into_raw(boxed_boxed_trampoline).cast();

    let boxed_user_func: Box<dyn FnMut(&str) + Send + 'static> = Box::new(f);

    *log_function_trampoline_raw = Some(LogFunctionTrampolineRaw {
      raw_boxed_boxed_trampoline,
      boxed_user_func,
    });

    unsafe {
      csmSetLogFunction(Some(**raw_boxed_boxed_trampoline));
    }
  }

  fn version(&self) -> CubismVersion {
    CubismVersion(unsafe { csmGetVersion() })
  }
  fn latest_supported_moc_version(&self) -> MocVersion {
    unsafe { csmGetLatestMocVersion() }.try_into().unwrap()
  }

  fn platform_moc_from_bytes(&self, bytes: &[u8]) -> Result<(MocVersion, Self::PlatformMoc), MocError> {
    const MOC_ALIGNMENT: usize = csmAlignofMoc as usize;

    let mut aligned_storage = AlignedStorage::new(bytes.len(), MOC_ALIGNMENT).unwrap();
    aligned_storage.copy_from_slice(bytes);

    let size_in_u32: u32 = bytes.len().try_into().expect("Size should fit in a u32");

    let moc_version = unsafe {
      csmGetMocVersion(aligned_storage.as_mut_ptr().cast(), size_in_u32)
    };
    let moc_version = MocVersion::try_from(moc_version).map_err(|_| MocError::InvalidMoc)?;

    if self.latest_supported_moc_version() < moc_version {
      return Err(MocError::UnsupportedMocVersion {
        given: moc_version,
        latest_supported: self.latest_supported_moc_version(),
      });
    }

    let csm_moc = unsafe {
      csmReviveMocInPlace(aligned_storage.as_mut_ptr().cast(), size_in_u32)
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

impl PlatformMocInterface for PlatformMoc {
  type PlatformModelStatic  = PlatformModelStatic;
  type PlatformModelDynamic = PlatformModelDynamic;

  fn new_platform_model(&self) -> (Self::PlatformModelStatic, Self::PlatformModelDynamic) {
    const MODEL_ALIGNMENT: usize = csmAlignofModel as usize;

    let storage_size = unsafe {
      csmGetSizeofModel(self.csm_moc)
    };

    let mut csm_model_storage = AlignedStorage::new(storage_size as _, MODEL_ALIGNMENT).unwrap();

    let csm_model = unsafe {
      csmInitializeModelInPlace(self.csm_moc, csm_model_storage.as_mut_ptr().cast(), storage_size)
    };

    let canvas_info = unsafe {
      let mut size_in_pixels = csmVector2 { X: 0.0, Y: 0.0 };
      let mut origin_in_pixels = csmVector2 { X: 0.0, Y: 0.0 };
      let mut pixels_per_unit: f32 = 0.0;

      csmReadCanvasInfo(csm_model, &mut size_in_pixels, &mut origin_in_pixels, &mut pixels_per_unit);

      CanvasInfo {
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
        .map(|value| ParameterType::try_from(*value).unwrap())
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
          Parameter {
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
      let count: usize = csmGetPartCount(csm_model).try_into().unwrap();

      let ids: Vec<_> = std::slice::from_raw_parts(csmGetPartIds(csm_model), count).iter()
        .map(|&c_str_ptr| to_string(c_str_ptr))
        .collect();

      let parent_part_indices: Vec<_> = std::slice::from_raw_parts(csmGetPartParentPartIndices(csm_model), count).iter()
        .map(|&value| (value > 0).then_some(value as usize)).collect();

      itertools::izip!(ids, parent_part_indices)
        .map(|(id, parent_part_index)| {
          Part {
            id,
            parent_part_index,
          }
        })
        .collect()
    };

    let drawables: Box<[_]> = unsafe {
      let count: usize = csmGetDrawableCount(csm_model).try_into().unwrap();

      let ids: Vec<_> = std::slice::from_raw_parts(csmGetDrawableIds(csm_model), count).iter()
        .map(|&c_str_ptr| to_string(c_str_ptr))
        .collect();

      let constant_flagsets: Vec<_> = std::slice::from_raw_parts(csmGetDrawableConstantFlags(csm_model), count).iter()
        .map(|value| ConstantDrawableFlagSet::new(*value).unwrap())
        .collect();

      let texture_indices: Vec<_> = std::slice::from_raw_parts(csmGetDrawableTextureIndices(csm_model), count).iter()
        .map(|value| *value as usize)
        .collect();

      let mask_containers: Box<[_]> = {
        let mask_counts = std::slice::from_raw_parts(csmGetDrawableMaskCounts(csm_model), count);
        let mask_container_ptrs = std::slice::from_raw_parts(csmGetDrawableMasks(csm_model), count);

        itertools::izip!(mask_counts, mask_container_ptrs)
          .map(|(&mask_count, &mask_container_ptr)| {
            let mask_count: usize = mask_count.try_into().unwrap();
            std::slice::from_raw_parts(mask_container_ptr, mask_count).iter().map(|mask| *mask as usize).collect::<Box<[_]>>()
          })
          .collect()
      };

      let vertex_uv_containers: Box<[_]> = {
        let vertex_counts = std::slice::from_raw_parts(csmGetDrawableVertexCounts(csm_model), count);
        let vertex_uv_ptrs = std::slice::from_raw_parts(csmGetDrawableVertexUvs(csm_model), count);

        itertools::izip!(vertex_counts, vertex_uv_ptrs)
          .map(|(&vertex_count, &vertex_uv_ptr)| {
            let vertex_count: usize = vertex_count.try_into().unwrap();
            std::slice::from_raw_parts(vertex_uv_ptr.cast::<Vector2>(), vertex_count).to_vec().into_boxed_slice()
          })
          .collect()
      };

      let triangle_index_containers: Box<[_]> = {
        let triangle_index_counts = std::slice::from_raw_parts(csmGetDrawableIndexCounts(csm_model), count);
        let triangle_index_ptrs = std::slice::from_raw_parts(csmGetDrawableIndices(csm_model), count);

        itertools::izip!(triangle_index_counts, triangle_index_ptrs)
          .map(|(&triangle_index_count, &triangle_index_ptr)| {
            let triangle_index_count: usize = triangle_index_count.try_into().unwrap();
            if triangle_index_count > 0 {
              std::slice::from_raw_parts(triangle_index_ptr, triangle_index_count).to_vec().into_boxed_slice()
            } else {
              [].into()
            }
          })
          .collect()
      };

      let parent_part_indices: Vec<_> = std::slice::from_raw_parts(csmGetDrawableParentPartIndices(csm_model), count).iter()
        .map(|&value| (value > 0).then_some(value as usize)).collect();

      itertools::izip!(ids, constant_flagsets, texture_indices, mask_containers.iter(), vertex_uv_containers.iter(), triangle_index_containers.iter(), parent_part_indices)
        .enumerate()
        .map(|(index, (id, constant_flagset, texture_index, mask_container, vertex_uv_container, triangle_index_container, parent_part_index))| {
          Drawable {
            id,
            index: DrawableIndex(index as u64),
            constant_flagset,
            texture_index: TextureIndex(texture_index as u64),
            masks: mask_container.clone(),
            vertex_count: vertex_uv_container.len() as u32,
            vertex_uvs: vertex_uv_container.clone(),
            triangle_indices: triangle_index_container.clone(),
            parent_part_index,
          }
        })
        .collect()
    };

    let parameter_count = parameters.len();
    let part_count = parts.len();
    let drawable_count = drawables.len();

    let model_storage = Arc::new(ModelStorage {
      _csm_model_storage: csm_model_storage,
      csm_model,
      _moc_storage: Arc::clone(&self.moc_storage),
    });

    let platform_model_static = PlatformModelStatic {
      canvas_info,
      parameters,
      parts,
      drawables,

      _model_storage: Arc::clone(&model_storage),
    };

    let platform_model_dynamic = PlatformModelDynamic {
       // SAFETY: `csm_model` is behind an `Arc` we own.
      parameter_values: unsafe { std::slice::from_raw_parts_mut(csmGetParameterValues(csm_model), parameter_count) },
      part_opactities: unsafe { std::slice::from_raw_parts_mut(csmGetPartOpacities(csm_model), part_count) },
      drawable_dynamic_flagsets: unsafe { std::slice::from_raw_parts(csmGetDrawableDynamicFlags(csm_model).cast(), drawable_count) },
      drawable_draw_orders: unsafe { std::slice::from_raw_parts(csmGetDrawableDrawOrders(csm_model), drawable_count) },
      drawable_render_orders: unsafe { std::slice::from_raw_parts(csmGetDrawableRenderOrders(csm_model), drawable_count) },
      drawable_opacities: unsafe { std::slice::from_raw_parts(csmGetDrawableOpacities(csm_model), drawable_count) },
      vertex_position_containers: unsafe {
        VertexPositionContainers::new(csm_model)
      },
      drawable_multiply_colors: unsafe { std::slice::from_raw_parts(csmGetDrawableMultiplyColors(csm_model).cast(), drawable_count) },
      drawable_screen_colors: unsafe { std::slice::from_raw_parts(csmGetDrawableScreenColors(csm_model).cast(), drawable_count) },

      platform_model: Arc::clone(&model_storage),
    };

    (platform_model_static, platform_model_dynamic)
  }
}

#[derive(Debug)]
struct ModelStorage {
  /// Where `csm_model` is instantiated. Needs to outlive any reference obtained through `csm_model`.
  _csm_model_storage: AlignedStorage,
  /// Points inside `csm_model_storage`.
  csm_model: *mut csmModel,

  /// The memory block for the `csmMoc` used to generate this `csmModel`, which needs to outlive this `ModelStorage`.
  _moc_storage: Arc<AlignedStorage>,
}

// SAFETY: The underlying `csmModel` is never mutated except through methods taking a mutable reference.
unsafe impl Send for ModelStorage {}
unsafe impl Sync for ModelStorage {}

#[derive(Debug)]
pub struct PlatformModelStatic {
  canvas_info: CanvasInfo,
  parameters: Box<[Parameter]>,
  parts: Box<[Part]>,
  drawables: Box<[Drawable]>,

  /// Above members all reference the memory block inside this, which needs to outlive them.
  _model_storage: Arc<ModelStorage>,
}

impl PlatformModelStaticInterface for PlatformModelStatic {
  fn canvas_info(&self) -> CanvasInfo {
    self.canvas_info
  }
  fn parameters(&self) -> &[Parameter] {
    &self.parameters
  }
  fn parts(&self) -> &[Part] {
    &self.parts
  }
  fn drawables(&self) -> &[Drawable] {
    &self.drawables
  }
  fn get_drawable(&self, index: DrawableIndex) -> Option<&Drawable> {
    self.drawables.get(index.as_usize())
  }
}

#[derive(Debug)]
pub struct PlatformModelDynamic {
  parameter_values: &'static mut [f32],
  part_opactities: &'static mut [f32],
  drawable_dynamic_flagsets: &'static [DynamicDrawableFlagSet],
  drawable_draw_orders: &'static [i32],
  drawable_render_orders: &'static [i32],
  drawable_opacities: &'static [f32],
  vertex_position_containers: VertexPositionContainers<'static>,
  drawable_multiply_colors: &'static [Vector4],
  drawable_screen_colors: &'static [Vector4],

  /// Above members all reference the memory block inside this, which needs to outlive them.
  platform_model: Arc<ModelStorage>,
}

// SAFETY: The underlying `csmModel` is never mutated except through methods taking a mutable reference.
unsafe impl Send for PlatformModelDynamic {}
unsafe impl Sync for PlatformModelDynamic {}

impl PlatformModelDynamicInterface for PlatformModelDynamic {
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

  fn drawable_dynamic_flagsets(&self) -> &[DynamicDrawableFlagSet] {
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
  fn drawable_vertex_position_containers(&self) -> &[&[Vector2]] {
    &self.vertex_position_containers.inner
  }
  fn drawable_multiply_colors(&self) -> &[Vector4] {
    self.drawable_multiply_colors
  }
  fn drawable_screen_colors(&self) -> &[Vector4] {
    self.drawable_screen_colors
  }

  fn update(&mut self) {
    unsafe {
      csmUpdateModel(self.platform_model.csm_model);
    }

    // SAFETY: `csm_model` is behind an `Arc` we own.
    unsafe {
      self.vertex_position_containers = VertexPositionContainers::new(self.platform_model.csm_model);
    }
  }
  fn reset_drawable_dynamic_flags(&mut self) {
    unsafe {
      csmResetDrawableDynamicFlags(self.platform_model.csm_model);
    }
  }
}

#[derive(Debug)]
struct VertexPositionContainers<'a> {
  inner: Box<[&'a [Vector2]]>,
}
impl<'a> VertexPositionContainers<'a> {
  /// ## Safety
  /// - `csm_model` MUST be valid for lifetime `'a`.
  unsafe fn new(csm_model: *mut csmModel) -> Self {
    Self {
      inner: unsafe {
        let drawable_count: usize = csmGetDrawableCount(csm_model).try_into().unwrap();

        let vertex_counts = std::slice::from_raw_parts(csmGetDrawableVertexCounts(csm_model), drawable_count);
        let vertex_position_ptrs = std::slice::from_raw_parts(csmGetDrawableVertexPositions(csm_model), drawable_count);

        itertools::izip!(vertex_counts, vertex_position_ptrs)
          .map(|(&vertex_count, &vertex_position_ptr)| {
            let vertex_count: usize = vertex_count.try_into().unwrap();
            std::slice::from_raw_parts(vertex_position_ptr.cast::<Vector2>(), vertex_count)
          })
          .collect()
      }
    }
  }
}

unsafe fn to_string(c_str_ptr: *const std::os::raw::c_char) -> String {
  std::ffi::CStr::from_ptr(c_str_ptr).to_str().unwrap().to_string()
}
