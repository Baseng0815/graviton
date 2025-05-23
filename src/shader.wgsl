struct VertexInput {
    @location(0) v_pos: vec2<f32>,
    @location(1) i_pos: vec2<f32>,
    @location(2) i_col: vec4<f32>,
    @location(3) i_rad: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // [-0.5, 0.5] for xy axes
    out.local_pos = model.v_pos;
    let world_pos = model.i_pos + model.v_pos * model.i_rad;
    out.position = vec4<f32>(world_pos, 0.0, 1.0);
    out.color = model.i_col;
    return out;
}

@fragment
fn fs_main(@location(0) local_pos: vec2<f32>, @location(1) color: vec4<f32>) -> @location(0) vec4<f32> {
    let dist = length(local_pos);
    let alpha = smoothstep(0.5, 0.48, dist);
    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(color.rgb, color.a * alpha);
}
