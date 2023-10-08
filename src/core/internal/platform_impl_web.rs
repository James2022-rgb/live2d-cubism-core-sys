
use std::sync::Arc;

use super::platform_iface::{Vector2, Vector4};
use super::platform_iface::{MocError, CubismVersion, MocVersion};
use super::platform_iface::{CanvasInfo, Parameter, Part, Drawable};
use super::platform_iface::{ConstantDrawableFlagSet, DynamicDrawableFlagSet};
use super::platform_iface::{PlatformCubismCoreInterface, PlatformMocInterface, PlatformModelStaticInterface, PlatformModelDynamicInterface};

#[derive(Debug, Default)]
pub struct PlatformCubismCore {
  js_cubism_core: Arc<JsLive2DCubismCore>,
}

impl PlatformCubismCoreInterface for PlatformCubismCore {
  type PlatformMoc = PlatformMoc;

  fn version(&self) -> CubismVersion {
    self.js_cubism_core.cubism_version
  }
  fn latest_supported_moc_version(&self) -> MocVersion {
    self.js_cubism_core.latest_supported_moc_version
  }

  fn platform_moc_from_bytes(&self, bytes: &[u8]) -> Result<(MocVersion, self::PlatformMoc), MocError> {
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
      .ok_or(MocError::InvalidMoc)
  }
}

#[derive(Debug)]
pub struct PlatformMoc {
  js_moc: JsMoc,
  js_cubism_core: Arc<JsLive2DCubismCore>,
}

impl PlatformMocInterface for PlatformMoc {
  type PlatformModelStatic  = PlatformModelStatic;
  type PlatformModelDynamic = PlatformModelDynamic;

  fn new_platform_model(&self) -> (Self::PlatformModelStatic, Self::PlatformModelDynamic) {
    let js_model = self.js_cubism_core.js_model_from_moc(&self.js_moc);

    let canvas_info = js_model.canvas_info;
    let parameters = js_model.parameters.to_aos().into_boxed_slice();
    let parts = js_model.parts.to_aos().into_boxed_slice();
    let drawables = js_model.drawables.to_aos().into_boxed_slice();

    let platform_model_static = PlatformModelStatic {
      canvas_info,
      parameters,
      parts,
      drawables,
    };

    let platform_model_dynamic = PlatformModelDynamic {
      js_model,
    };

    (platform_model_static, platform_model_dynamic)
  }
}

#[derive(Debug)]
pub struct PlatformModelStatic {
  canvas_info: CanvasInfo,
  parameters: Box<[Parameter]>,
  parts: Box<[Part]>,
  drawables: Box<[Drawable]>,
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
}

#[derive(Debug)]
pub struct PlatformModelDynamic {
  js_model: JsModel,
}

impl PlatformModelDynamicInterface for PlatformModelDynamic {
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

  fn drawable_dynamic_flagsets(&self) -> &[DynamicDrawableFlagSet] {
    self.js_model.scratch.drawable_dynamic_flagsets()
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
  fn drawable_vertex_position_containers(&self) -> &[&[Vector2]] {
    self.js_model.scratch.drawable_vertex_position_containers()
  }
  fn drawable_multiply_colors(&self) -> &[Vector4] {
    self.js_model.scratch.drawable_multiply_colors()
  }
  fn drawable_screen_colors(&self) -> &[Vector4] {
    self.js_model.scratch.drawable_screen_colors()
  }

  fn update(&mut self) {
    self.js_model.update()
  }
  fn reset_drawable_dynamic_flags(&mut self) {
    self.js_model.reset_drawable_dynamic_flags()
  }
}

use js::*;

/// Not-so-direct bindings to the JavaScript interface of _Live2D® Cubism SDK Core_ for Web.
mod js {
  const LIVE2DCUBISMCORE_JS_STR: &str = include_str!(concat!(env!("LIVE2D_CUBISM_SDK_WEB_DIR"), "/Core/live2dcubismcore.min.js"));

  use wasm_bindgen::JsCast as _;

  use crate::core;

  #[allow(non_snake_case)]
  #[derive(Debug)]
  pub struct JsLive2DCubismCore {
    pub cubism_version: core::CubismVersion,
    pub latest_supported_moc_version: core::MocVersion,

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
    pub version: core::MocVersion,
    /// An `Live2DCubismCore.Moc` instance object, acquired through the `Live2DCubismCore.Moc.fromArrayBuffer` static method.
    moc_instance: wasm_bindgen::JsValue,
  }

  #[derive(Debug)]
  pub struct JsModel {
    pub canvas_info: core::CanvasInfo,
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
    pub types: Box<[core::ParameterType]>,
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
    pub constant_flagsets: Box<[core::ConstantDrawableFlagSet]>,
    pub texture_indices: Box<[usize]>,
    pub mask_containers: Box<[Box<[usize]>]>,
    pub vertex_uv_containers: Box<[Box<[core::Vector2]>]>,
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
        core::CubismVersion(csmGetVersion.call0(&version_class).unwrap().as_f64().unwrap() as u32)
      };
      let latest_supported_moc_version = {
        let csmGetLatestMocVersion = get_member_function(&version_class, "csmGetLatestMocVersion");
        core::MocVersion::try_from(csmGetLatestMocVersion.call0(&version_class).unwrap().as_f64().unwrap() as u32).unwrap()
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
    #[allow(dead_code)]
    pub fn moc_from_bytes(&self, bytes: &[u8]) -> Option<JsMoc> {
      let array = js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());
      array.copy_from(bytes);

      self.moc_from_js_array_buffer(array.buffer())
    }

    /// Equivalent to `csmGetMocVersion`.
    pub fn get_moc_version(&self, js_moc_instance: &wasm_bindgen::JsValue, array_buffer: &js_sys::ArrayBuffer) -> core::MocVersion {
      let moc_version = self.csmGetMocVersion.call2(
        &self.version_class, js_moc_instance, array_buffer.as_ref()
      )
      .unwrap().as_f64().unwrap() as u32;
      core::MocVersion::try_from(moc_version).unwrap()
    }

    pub fn js_model_from_moc(&self, moc: &JsMoc) -> JsModel {
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

        core::CanvasInfo {
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
        .map(|value| core::ParameterType::try_from(value.as_f64().unwrap() as i32).unwrap())
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

    pub fn to_aos(&self) -> Vec<core::Parameter> {
      itertools::izip!(self.ids.iter(), self.types.iter(), self.minimum_values.iter(), self.maximum_values.iter(), self.default_values.iter(), self.key_value_containers.iter())
        .map(|(id, ty, minimum_value, maximum_value, default_value, key_value_container)| {
          core::Parameter {
            id: id.clone(),
            ty: *ty,
            value_range: (*minimum_value, *maximum_value),
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

    pub fn to_aos(&self) -> Vec<core::Part> {
      itertools::izip!(self.ids.iter(), self.parent_part_indices.iter())
        .map(|(id, parent_part_index)| {
          core::Part {
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
          core::ConstantDrawableFlagSet::new(value.as_f64().unwrap() as u8).unwrap()
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

    pub fn to_aos(&self) -> Vec<core::Drawable> {
      itertools::izip!(self.ids.iter(), self.constant_flagsets.iter(), self.texture_indices.iter(), self.mask_containers.iter(), self.vertex_uv_containers.iter(), self.triangle_index_containers.iter(), self.parent_part_indices.iter())
        .map(|(id, constant_flagset, texture_index, mask_container, vertex_uv_container, triangle_index_container, parent_part_index)| {
          core::Drawable {
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
    drawable_dynamic_flagsets: Box<[core::DynamicDrawableFlagSet]>,
    drawable_draw_orders: Box<[i32]>,
    drawable_render_orders: Box<[i32]>,
    drawable_opacities: Box<[f32]>,
    drawable_vertex_position_containers: Box<[Box<[core::Vector2]>]>,
    drawable_vertex_position_container_refs: Box<[&'static [core::Vector2]]>,
    drawable_multiply_colors: Box<[core::Vector4]>,
    drawable_screen_colors: Box<[core::Vector4]>,
  }
  impl Scratch {
    pub fn parameter_values(&self) -> &[f32] { &self.parameter_values }
    pub fn parameter_values_mut(&mut self) -> &mut [f32] { &mut self.parameter_values }
    pub fn part_opacities(&self) -> &[f32] { &self.part_opacities }
    pub fn part_opacities_mut(&mut self) -> &mut [f32] { &mut self.part_opacities }

    pub fn drawable_dynamic_flagsets(&self) -> &[core::DynamicDrawableFlagSet] { &self.drawable_dynamic_flagsets }
    pub fn drawable_draw_orders(&self) -> &[i32] { &self.drawable_draw_orders }
    pub fn drawable_render_orders(&self) -> & [i32] { &self.drawable_render_orders }
    pub fn drawable_opacities(&self) -> &[f32] { &self.drawable_opacities }
    pub fn drawable_vertex_position_containers(&self) -> &[&[core::Vector2]] { &self.drawable_vertex_position_container_refs }
    pub fn drawable_multiply_colors(&self) -> &[core::Vector4] { &self.drawable_multiply_colors }
    pub fn drawable_screen_colors(&self) -> &[core::Vector4] { &self.drawable_screen_colors }

    fn new(parameters: &JsParameters, parts: &JsParts, drawables: &JsDrawables) -> Self {
      let parameter_values = float32_array_to_new_vec(&parameters.values).into_boxed_slice();
      let part_opacities = float32_array_to_new_vec(&parts.opacities).into_boxed_slice();
      let drawable_dynamic_flagsets = uint8_array_to_new_vec::<core::DynamicDrawableFlagSet>(&drawables.dynamic_flags).into_boxed_slice();
      let drawable_draw_orders = int32_array_to_new_vec(&drawables.draw_orders).into_boxed_slice();
      let drawable_render_orders = int32_array_to_new_vec(&drawables.render_orders).into_boxed_slice();
      let drawable_opacities = float32_array_to_new_vec(&drawables.opacities).into_boxed_slice();

      let drawable_vertex_position_containers: Box<[_]> = drawables.vertex_positions.iter()
        .map(|f32_array| {
          let f32_array = f32_array.dyn_into::<js_sys::Float32Array>().unwrap();
          float32_array_to_new_vec::<core::Vector2>(&f32_array).into_boxed_slice()
        })
        .collect();
      let drawable_vertex_position_container_refs: Box<[_]> = drawable_vertex_position_containers.iter()
        .map(|v| {
          // SAFETY: A boxed slice is pointer-stable.
          unsafe { std::slice::from_raw_parts(v.as_ptr(), v.len()) }}
        )
        .collect();

      let drawable_multiply_colors = float32_array_to_new_vec::<core::Vector4>(&drawables.multiply_colors).into_boxed_slice();
      let drawable_screen_colors = float32_array_to_new_vec::<core::Vector4>(&drawables.screen_colors).into_boxed_slice();

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
          std::slice::from_raw_parts(self.drawable_dynamic_flagsets.as_ptr().cast::<u8>(), self.drawable_dynamic_flagsets.len())
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

    writer(dst.as_mut_ptr().cast::<E>())
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
    writer(dst.as_mut_ptr().cast::<E>());

    // SAFETY:
    // 1. Constructed with `with_capacity`.
    // 2. `writer` must have initialized the elements.
    unsafe {
      dst.set_len(dst_len);
    }
    dst
  }
}
