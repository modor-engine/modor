struct Camera {
    transform: mat4x4<f32>,
};

struct Material {
    blur_factor: f32,
    padding1: f32,          // some platforms like WebGL require 16 bytes alignment for uniform data
    padding2: vec2<f32>,    // some platforms like WebGL require 16 bytes alignment for uniform data
};

struct Vertex {
    @location(0)
    position: vec3<f32>,
    @location(1)
    texture_position: vec2<f32>,
};

struct Instance {
    @location(2)
    transform_0: vec4<f32>,
    @location(3)
    transform_1: vec4<f32>,
    @location(4)
    transform_2: vec4<f32>,
    @location(5)
    transform_3: vec4<f32>,
};

struct MaterialInstance {
    @location(6)
    sample_count: u32,
};

struct Fragment {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    texture_position: vec2<f32>,
    @location(1)
    sample_count: u32,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(0)
var<uniform> material: Material;

@group(1)
@binding(1)
var texture: texture_2d<f32>;

@group(1)
@binding(2)
var texture_sampler: sampler;

@vertex
fn vs_main(vertex: Vertex, instance: Instance, material_instance: MaterialInstance) -> Fragment {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    return Fragment(
        camera.transform * transform * vec4<f32>(vertex.position, 1.),
        vertex.texture_position,
        material_instance.sample_count,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    var color_sum = vec4<f32>();
    let sample_count_f32 = f32(fragment.sample_count);
    for (var x: f32 = -1. * sample_count_f32; x < 1. * sample_count_f32 + 0.5; x += 1.) {
        for (var y: f32 = -1. * sample_count_f32; y < 1. * sample_count_f32 + 0.5; y += 1.) {
            color_sum += color(fragment, material.blur_factor * x, material.blur_factor * y);
        }
    }
    return color_sum / pow(sample_count_f32 * 2. + 1., 2.);
}

fn color(fragment: Fragment, offset_x: f32, offset_y: f32) -> vec4<f32> {
    let texture_position = fragment.texture_position + vec2<f32>(offset_x, offset_y);
    return textureSample(texture, texture_sampler, texture_position);
}
