const WIDTH: f32 = 800f;
const HEIGHT: f32 = 600f;

fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> mat4x4f {
 return mat4x4f(
        2 / (right - left), 0, 0, 0,
        0, 2 / (top - bottom), 0, 0, 
        0, 0, 2 / (far - near), 0,
        (right + left) / (left - right), (top + bottom) / (bottom - top), near / (near - far), 1
    );
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let points_arr = array(
      vec2f(0, 100),
      vec2f(0, 0),
      vec2f(100,  0),
      vec2f(100,  0),
      vec2f(100, 100),
      vec2f(0,  100),
    );
    let tex_coords_arr = array(
      vec2f(0, 1),
      vec2f(0, 0),
      vec2f(1, 0),
      vec2f(1, 0),
      vec2f(1, 1),
      vec2f(0, 1),
    );

    let ortho_mat = ortho(0, WIDTH, HEIGHT, 0, 0.1, 100);
    let pos = points_arr[in_vertex_index];

    return VertexOutput(
        ortho_mat * vec4<f32>(pos.x, pos.y, 1.0, 1.0),
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
