#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cubism_core_version() {
    unsafe {
      let core_version = csmGetVersion();
      let latest_moc_version = csmGetLatestMocVersion();

      println!("core_version: {}", core_version);
      println!("latest_moc_version: {}", latest_moc_version);
    }
  }
}
