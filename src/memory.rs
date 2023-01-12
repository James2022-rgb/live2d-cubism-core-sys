
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
