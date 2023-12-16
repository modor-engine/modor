struct Camera {
    transform: mat4x4<f32>,
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

struct Fragment {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    texture_position: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: Camera;

@group(1)
@binding(1)
var texture1: texture_2d<f32>;

@group(1)
@binding(2)
var texture1_sampler: sampler;

@group(1)
@binding(3)
var texture2: texture_2d<f32>;

@group(1)
@binding(4)
var texture2_sampler: sampler;

@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    return Fragment(
        camera.transform * transform * vec4<f32>(vertex.position, 1.),
        vertex.texture_position,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    let color1 = textureSample(texture1, texture1_sampler, fragment.texture_position);
    let color2 = textureSample(texture2, texture2_sampler, fragment.texture_position);
    return color1 * color2;
}
