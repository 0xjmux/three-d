//!
//! A collection of lights implementing the [Light] trait.
//!
//! Lights shines onto objects in the scene, note however that some materials are affected by lights, others are not.
//!

macro_rules! impl_light_body {
    ($inner:ident) => {
        fn shader_source(&self, i: u32) -> String {
            self.$inner().shader_source(i)
        }
        fn use_uniforms(&self, program: &Program, i: u32) {
            self.$inner().use_uniforms(program, i)
        }
        fn id(&self) -> LightId {
            self.$inner().id()
        }
    };
}

mod directional_light;
use std::ops::Deref;

#[doc(inline)]
pub use directional_light::*;

mod spot_light;
#[doc(inline)]
pub use spot_light::*;

mod point_light;
#[doc(inline)]
pub use point_light::*;

mod ambient_light;
#[doc(inline)]
pub use ambient_light::*;

mod environment;
#[doc(inline)]
pub use environment::*;

use crate::core::*;
use crate::renderer::viewer::*;
use crate::renderer::LightId;

///
/// Specifies how the intensity of a light fades over distance.
/// The light intensity is scaled by ``` 1 / max(1, constant + distance * linear + distance * distance * quadratic) ```.
///
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Attenuation {
    /// Constant attenuation factor.
    pub constant: f32,
    /// Linear attenuation factor.
    pub linear: f32,
    /// Quadratic attenuation factor.
    pub quadratic: f32,
}

impl Default for Attenuation {
    fn default() -> Self {
        Self {
            constant: 1.0,
            linear: 0.0,
            quadratic: 0.0,
        }
    }
}

/// Represents a light source.
pub trait Light {
    /// The fragment shader source for calculating this lights contribution to the color in a fragment.
    /// It should contain a function with this signature
    /// `vec3 calculate_lighting{}(vec3 surface_color, vec3 position, vec3 normal, vec3 view_direction, float metallic, float roughness, float occlusion)`
    /// Where `{}` is replaced with the number i given as input.
    /// This function should return the color contribution for this light on the surface with the given surface parameters.
    fn shader_source(&self, i: u32) -> String;

    /// Should bind the uniforms that is needed for calculating this lights contribution to the color in [Light::shader_source].
    fn use_uniforms(&self, program: &Program, i: u32);

    ///
    /// Returns a unique ID for each variation of the shader source returned from `Light::shader_source`.
    ///
    /// **Note:** The last bit is reserved to internally implemented materials, so if implementing the `Light` trait
    /// outside of this crate, always return an id in the public use range as defined by [LightId].
    ///
    fn id(&self) -> LightId;
}

impl<T: Light + ?Sized> Light for &T {
    impl_light_body!(deref);
}

impl<T: Light + ?Sized> Light for &mut T {
    impl_light_body!(deref);
}

impl<T: Light> Light for Box<T> {
    impl_light_body!(as_ref);
}

impl<T: Light> Light for std::rc::Rc<T> {
    impl_light_body!(as_ref);
}

impl<T: Light> Light for std::sync::Arc<T> {
    impl_light_body!(as_ref);
}

impl<T: Light> Light for std::cell::RefCell<T> {
    impl_light_body!(borrow);
}

impl<T: Light> Light for std::sync::Arc<std::sync::RwLock<T>> {
    fn shader_source(&self, i: u32) -> String {
        self.read().unwrap().shader_source(i)
    }
    fn use_uniforms(&self, program: &Program, i: u32) {
        self.read().unwrap().use_uniforms(program, i)
    }
    fn id(&self) -> LightId {
        self.read().unwrap().id()
    }
}

///
/// Returns shader source code with the function `calculate_lighting` which calculate the lighting contribution for the given lights and the given [LightingModel].
/// Use this if you want to implement a custom [Material](crate::renderer::Material) but use the default lighting calculations.
///
/// The shader function has the following signature:
/// ```no_rust
/// vec3 calculate_lighting(vec3 camera_position, vec3 surface_color, vec3 position, vec3 normal, float metallic, float roughness, float occlusion)
/// ```
///
pub fn lights_shader_source(lights: &[&dyn Light]) -> String {
    let mut shader_source = include_str!("../core/shared.frag").to_string();
    shader_source.push_str(include_str!("light/shaders/light_shared.frag"));
    let mut dir_fun = String::new();
    for (i, light) in lights.iter().enumerate() {
        shader_source.push_str(&light.shader_source(i as u32));
        dir_fun.push_str(&format!("color += calculate_lighting{}(surface_color, position, normal, view_direction, metallic, roughness, occlusion);\n", i))
    }
    shader_source.push_str(&format!(
        "
            vec3 calculate_lighting(vec3 camera_position, vec3 surface_color, vec3 position, vec3 normal, float metallic, float roughness, float occlusion)
            {{
                vec3 color = vec3(0.0, 0.0, 0.0);
                vec3 view_direction = normalize(camera_position - position);
                {}
                return color;
            }}
            ",
        &dir_fun
    ));
    shader_source
}

fn shadow_matrix(camera: &Camera) -> Mat4 {
    let bias_matrix = crate::Mat4::new(
        0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.5, 0.5, 0.5, 1.0,
    );
    bias_matrix * camera.projection() * camera.view()
}

fn compute_up_direction(direction: Vec3) -> Vec3 {
    if vec3(1.0, 0.0, 0.0).dot(direction).abs() > 0.9 {
        (vec3(0.0, 1.0, 0.0).cross(direction)).normalize()
    } else {
        (vec3(1.0, 0.0, 0.0).cross(direction)).normalize()
    }
}

use crate::renderer::{GeometryFunction, LightingModel, NormalDistributionFunction};
pub(crate) fn lighting_model_to_id(model: LightingModel) -> u32 {
    match model {
        LightingModel::Phong => 1,
        LightingModel::Blinn => 2,
        LightingModel::Cook(
            NormalDistributionFunction::Blinn,
            GeometryFunction::SmithSchlickGGX,
        ) => 3,
        LightingModel::Cook(
            NormalDistributionFunction::Beckmann,
            GeometryFunction::SmithSchlickGGX,
        ) => 4,
        LightingModel::Cook(
            NormalDistributionFunction::TrowbridgeReitzGGX,
            GeometryFunction::SmithSchlickGGX,
        ) => 5,
    }
}
