#![cfg(feature = "core")]

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod base_types;
pub mod model_types;

pub use base_types::{Vector2, Vector4};
pub use base_types::{MocError, CubismVersion, MocVersion};
pub use base_types::{TextureIndex, DrawableIndex};

pub use model_types::CanvasInfo;
pub use model_types::{ParameterType, Parameter};
pub use model_types::Part;
pub use model_types::{ConstantDrawableFlags, ConstantDrawableFlagSet, DynamicDrawableFlags, DynamicDrawableFlagSet, Drawable};

mod internal;

use internal::platform_impl::{PlatformCubismCore, PlatformMoc, PlatformModelStatic, PlatformModelDynamic};

if_native! {
  use static_assertions::assert_impl_all;

  assert_impl_all!(CubismCore: Send, Sync);
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
  pub fn get_drawable(&self, index: DrawableIndex) -> Option<&Drawable> { self.inner.get_drawable(index) }
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
