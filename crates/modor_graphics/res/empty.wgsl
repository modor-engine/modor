struct Fragment {
    @builtin(position)
    position: vec4<f32>,
};

@vertex
fn vs_main() -> Fragment {
    return Fragment(vec4(0., 0., 0., 0.));
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    discard;
}
