struct CameraUniform {
    transform: mat4x4<f32>,
};

struct MaterialUniform {
    color: vec4<f32>,
    texture_part_position: vec2<f32>,
    texture_part_size: vec2<f32>,
    front_color: vec4<f32>,
}

struct ModelVertex {
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
    @location(1)
    front_texture_position: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: CameraUniform;

@group(1)
@binding(0)
var<uniform> material: MaterialUniform;

@group(2)
@binding(0)
var texture: texture_2d<f32>;

@group(2)
@binding(1)
var texture_sampler: sampler;

@group(3)
@binding(0)
var front_texture: texture_2d<f32>;

@group(3)
@binding(1)
var front_texture_sampler: sampler;

@vertex
fn vs_main(vertex: ModelVertex, instance: Instance) -> Fragment {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    let front_texture_size = textureDimensions(front_texture);
    let transform_front_texture_ratio = length(instance.transform_0.xyz) / length(instance.transform_1.xyz)
        * f32(front_texture_size.y) / f32(front_texture_size.x);
    let front_ratio = vec2(
        max(transform_front_texture_ratio, 1.),
        max(1. / transform_front_texture_ratio, 1.),
    );
    return Fragment(
        camera.transform * transform * vec4<f32>(vertex.position, 1.),
        vertex.texture_position * material.texture_part_size + material.texture_part_position,
        vertex.texture_position * front_ratio + (vec2(1., 1.) - front_ratio) / 2.,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    let back_color = textureSample(texture, texture_sampler, fragment.texture_position)
        * material.color;
    let front_color = textureSample(front_texture, front_texture_sampler, fragment.front_texture_position)
        * material.front_color;
    let color = front_color.a * vec4(front_color.rgb, 1.) + (1. - front_color.a) * back_color;
    if (color.w == 0.) {
        discard;
    }
    return color;
}
