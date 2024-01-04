struct Camera {
    transform: mat4x4<f32>,
};

struct Material {
    color: vec4<f32>,
    texture_position: vec2<f32>,
    texture_size: vec2<f32>,
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
fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    return Fragment(
        camera.transform * transform * vec4<f32>(vertex.position, 1.),
        vertex.texture_position * material.texture_size + material.texture_position,
        vec2<f32>(vertex.position.x, vertex.position.y),
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    let color = textureSample(texture, texture_sampler, fragment.texture_position) * material.color;
    if (color.a == 0.) {
        discard;
    }
    let distance = sqrt(pow(fragment.inner_position.x, 2.) + pow(fragment.inner_position.y, 2.));
    if (distance > 0.5) {
        discard;
    }
    return color;
}
