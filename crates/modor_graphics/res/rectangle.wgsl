struct CameraUniform {
    transform: mat4x4<f32>,
};

struct VertexInput {
    @location(0)
    position: vec3<f32>,
};

struct InstanceInput {
    @location(1)
    transform_0: vec4<f32>,
    @location(2)
    transform_1: vec4<f32>,
    @location(3)
    transform_2: vec4<f32>,
    @location(4)
    transform_3: vec4<f32>,
    @location(5)
    color: vec4<f32>,
    @location(6)
    has_texture: u32,
};

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    color: vec4<f32>,
    @location(1)
    has_texture: u32,
    @location(2)
    texture_coords: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: CameraUniform;

@group(1)
@binding(0)
var texture: texture_2d<f32>;

@group(1)
@binding(1)
var texture_sampler: sampler;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    return VertexOutput(
        camera.transform * transform * vec4<f32>(vertex.position, 1.),
        instance.color,
        instance.has_texture,
        (vertex.position.xy + vec2<f32>(0.5, 0.5)) / vec2<f32>(1., -1.)
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, in.texture_coords) * in.color;
    if (color.w == 0.) {
        discard;
    }
    return color;
}
