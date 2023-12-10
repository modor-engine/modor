struct CameraUniform {
    transform: mat4x4<f32>,
};

struct MaterialUniform {
    color: vec4<f32>,
    texture_part_position: vec2<f32>,
    texture_part_size: vec2<f32>,
    front_color: vec4<f32>,
}

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
};

@group(0)
@binding(0)
var<uniform> camera: CameraUniform;

@group(1)
@binding(0)
var<uniform> material: MaterialUniform;

@group(1)
@binding(1)
var texture: texture_2d<f32>;

@group(1)
@binding(2)
var texture_sampler: sampler;

@group(1)
@binding(3)
var front_texture: texture_2d<f32>;

@group(1)
@binding(4)
var front_texture_sampler: sampler;

@vertex
fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    )
    return Fragment(camera.transform * transform * vec4<f32>(vertex.position, 1.));
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    return vec4(1., 0., 0., 1.);
}
