struct CameraUniform {
    transform: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> camera: CameraUniform;

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
};

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
    );
    var out: VertexOutput;
    out.position = camera.transform * transform * vec4<f32>(vertex.position, 1.);
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.color.w == 0.) {
        discard;
    }
    return in.color;
}
