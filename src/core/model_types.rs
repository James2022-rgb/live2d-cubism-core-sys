//! Types for rendering a _Live2D®_ model.

use static_assertions::{assert_eq_align, assert_eq_size};
use num_enum::TryFromPrimitive;
use flagset::{FlagSet, flags};

//
// Canvas
//

/// Model canvas.
#[derive(Debug, Clone, Copy)]
pub struct CanvasInfo {
  /// Canvas dimensions.
  pub size_in_pixels: (f32, f32),
  /// Origin of model on canvas.
  pub origin_in_pixels: (f32, f32),
  /// Aspect used for scaling pixels to units.
  pub pixels_per_unit: f32,
}

//
// Parameter
//

/// Parameter type.
///
/// ## Informative
/// Seems to be purely informative (not required for rendering).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
#[repr(i32)]
pub enum ParameterType {
  Normal = 0,
  BlendShape = 1,
}

/// Properties for a single parameter of a _Live2D®_ model.
#[derive(Debug, Clone)]
pub struct Parameter {
  pub(crate) id: String,
  pub(crate) ty: ParameterType,
  pub(crate) value_range: (f32, f32),
  pub(crate) default_value: f32,
  pub(crate) keys: Box<[f32]>,
}
impl Parameter {
  pub fn id(&self) -> &str {
    &self.id
  }
  pub fn ty(&self) -> ParameterType {
    self.ty
  }
  pub fn value_range(&self) -> (f32, f32) {
    self.value_range
  }
  pub fn default_value(&self) -> f32 {
    self.default_value
  }
  pub fn keys(&self) -> &[f32] {
    &self.keys
  }
}

//
// Part
//

#[derive(Debug, Clone)]
pub struct Part {
  pub(crate) id: String,
  pub(crate) parent_part_index: Option<usize>,
}
impl Part {
  pub fn id(&self) -> &str {
    self.id.as_str()
  }
  pub fn parent_part_index(&self) -> Option<usize> {
    self.parent_part_index
  }
}

//
// Drawable
//

use super::base_types::Vector2;

flags! {
  /// Constant Drawable flag values.
  pub enum ConstantDrawableFlags: u8 {
    /// Mutually exclusive with `BlendMultiplicative`.
    BlendAdditive,
    /// Mutually exclusive with `BlendAdditive`.
    BlendMultiplicative,
    IsDoubleSided,
    IsInvertedMask,
  }
}

pub type ConstantDrawableFlagSet = FlagSet<ConstantDrawableFlags>;
assert_eq_align!(ConstantDrawableFlagSet, u8);
assert_eq_size!(ConstantDrawableFlagSet, u8);

flags! {
  /// Dynamic drawable flag values (updated with changes to parameter values).
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

pub type DynamicDrawableFlagSet = FlagSet<DynamicDrawableFlags>;
assert_eq_align!(DynamicDrawableFlagSet, u8);
assert_eq_size!(DynamicDrawableFlagSet, u8);

#[derive(Debug, Clone)]
pub struct Drawable {
  pub(crate) id: String,
  pub(crate) constant_flagset: ConstantDrawableFlagSet,
  pub(crate) texture_index: usize,
  pub(crate) masks: Box<[usize]>,
  pub(crate) vertex_uvs: Box<[Vector2]>,
  pub(crate) triangle_indices: Box<[u16]>,
  pub(crate) parent_part_index: Option<usize>,
}
impl Drawable {
  pub fn id(&self) -> &str {
    self.id.as_str()
  }
  pub fn constant_flagset(&self) -> ConstantDrawableFlagSet {
    self.constant_flagset
  }
  pub fn texture_index(&self) -> usize {
    self.texture_index
  }
  pub fn masks(&self) -> &[usize] {
    &self.masks
  }
  pub fn vertex_uvs(&self) -> &[Vector2] {
    &self.vertex_uvs
  }
  pub fn triangle_indices(&self) -> &[u16] {
    &self.triangle_indices
  }
  pub fn parent_part_index(&self) -> Option<usize> {
    self.parent_part_index
  }
}
