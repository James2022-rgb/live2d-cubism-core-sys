
pub use crate::core::base_types::{Vector2, Vector4};
pub use crate::core::base_types::{MocError, CubismVersion, MocVersion};
pub use crate::core::model_types::CanvasInfo;
pub use crate::core::model_types::{ParameterType, Parameter};
pub use crate::core::model_types::Part;
pub use crate::core::model_types::{ConstantDrawableFlagSet, DynamicDrawableFlagSet, Drawable};

pub trait PlatformCubismCoreInterface {
  type PlatformMoc;

  #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
  unsafe fn set_log_function<F>(f: F)
  where
    F: FnMut(&str) + Send + 'static;

  fn version(&self) -> CubismVersion;
  fn latest_supported_moc_version(&self) -> MocVersion;

  fn platform_moc_from_bytes(&self, bytes: &[u8]) -> Result<(MocVersion, Self::PlatformMoc), MocError>;
}

pub trait PlatformMocInterface {
  type PlatformModelStatic;
  type PlatformModelDynamic;

  fn new_platform_model(&self) -> (Self::PlatformModelStatic, Self::PlatformModelDynamic);
}

pub trait PlatformModelStaticInterface {
  fn canvas_info(&self) -> CanvasInfo;
  fn parameters(&self) -> &[Parameter];
  fn parts(&self) -> &[Part];
  fn drawables(&self) -> &[Drawable];
}

pub trait PlatformModelDynamicInterface {
  fn parameter_values(&self) -> &[f32];
  fn parameter_values_mut(&mut self) -> &mut [f32];
  fn part_opacities(&self) -> &[f32];
  fn part_opacities_mut(&mut self) -> &mut [f32];
  fn drawable_dynamic_flagsets(&self) -> &[DynamicDrawableFlagSet];
  fn drawable_dynamic_flagsets_mut(&mut self) -> &mut [DynamicDrawableFlagSet];

  fn drawable_draw_orders(&self) -> &[i32];
  fn drawable_render_orders(&self) -> &[i32];
  fn drawable_opacities(&self) -> &[f32];
  fn drawable_vertex_position_containers(&self) -> &[&[Vector2]];
  fn drawable_multiply_colors(&self) -> &[Vector4];
  fn drawable_screen_colors(&self) -> &[Vector4];

  fn update(&mut self);
  fn reset_drawable_dynamic_flags(&mut self);
}


