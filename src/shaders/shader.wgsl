struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexPayload {
    var position = array<vec2<f32>, 3>(
        vec2<f32>(-0.75, -0.75),
        vec2<f32>(0.75, 0.75),
        vec2<f32>(0.0, 0.75),
    );

    var colors = array<vec3<f32>, 3>(
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0),
    );

    var out: VertexPayload;
    out.position = vec4<f32>(positions[i], 0.0, 1.0);
    out.color = colors[i];
    return out;
}

@fragment
fn fs_main(in: VertexPayload) -> @loaction(0) vec4<f32> {
    return vec4<32f>(in.color, 1.0);
}