struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let points_arr = array(
      vec2f(-1, -1),
      vec2f( 1, -1),
      vec2f(-1,  1),
      vec2f(-1,  1),
      vec2f( 1, -1),
      vec2f( 1,  1),
    );
    let tex_coords_arr = array(
      vec2f(0, 0),
      vec2f(1, 0),
      vec2f(0, 1),
      vec2f(0, 1),
      vec2f(1, 0),
      vec2f(1, 1),
    );
    let pos = points_arr[in_vertex_index];

    return VertexOutput(
        vec4<f32>(pos.x, pos.y, 0.0, 1.0),
        tex_coords_arr[in_vertex_index]
    );
}

@group(0)
@binding(0)
var texture: texture_2d<f32>;

@group(0)
@binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, tex_sampler, in.tex_coords);
}
