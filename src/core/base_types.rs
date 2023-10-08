//! Basic types for vector, error and version number.

use static_assertions::const_assert_eq;
use thiserror::Error;
use shrinkwraprs::Shrinkwrap;
use derive_more::Display;
use num_enum::TryFromPrimitive;

/// A 2-component `f32` vector with no padding.
pub type Vector2 = mint::Vector2<f32>;
/// A 4-component `f32` vector with no padding.
pub type Vector4 = mint::Vector4<f32>;

const_assert_eq!(std::mem::size_of::<Vector2>(), std::mem::size_of::<f32>() * 2);
const_assert_eq!(std::mem::size_of::<Vector4>(), std::mem::size_of::<f32>() * 4);

/// Errors generated when deserializing a moc.
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
    // TODO: Hex?
    write!(f, "{:02}.{:02}.{:04} ({})", self.major(), self.minor(), self.patch(), self.0)
  }
}
impl std::fmt::Debug for CubismVersion {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // TODO: Hex?
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
