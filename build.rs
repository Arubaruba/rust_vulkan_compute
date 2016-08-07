extern crate vulkano_shaders;

fn main() {
    // building the shaders used in the examples
    vulkano_shaders::build_glsl_shaders([
        ("src/compute.glsl", vulkano_shaders::ShaderType::Compute),
    ].iter().cloned());
}
