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
    @location(0)
    texture_position: vec2<f32>,
    @location(1)
    inner_position: vec2<f32>,
    @location(2)
    front_texture_position: vec2<f32>,
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
        vec2<f32>(vertex.position.x, vertex.position.y),
        vertex.texture_position * front_ratio + (vec2(1., 1.) - front_ratio) / 2.,
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    let back_color = textureSample(texture, texture_sampler, fragment.texture_position)
        * material.color;
    let front_color = textureSample(front_texture, front_texture_sampler, fragment.front_texture_position)
        * material.front_color;
    let back_alpha = (1. - front_color.a) * back_color.a;
    let alpha = front_color.a + back_alpha;
    let rgb = (front_color.a * front_color.rgb + back_alpha * back_color.rgb) / alpha;
    if (alpha == 0.) {
        discard;
    }
    let distance = sqrt(pow(fragment.inner_position.x, 2.) + pow(fragment.inner_position.y, 2.));
    if (distance > 0.5) {
        discard;
    }
    return vec4(rgb, alpha);
}
