const WIDTH: f32 = 800;
const HEIGHT: f32 = 600;
const SPRITE_TILE_SIZE: f32 = 0.125;

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


@group(1)
@binding(0)
var<uniform> obj_position: vec4f;

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
    let tex_coords = tex_coords_arr[in_vertex_index];

    return VertexOutput(
        ortho_mat * vec4<f32>(
          obj_position.x + pos.x, 
          obj_position.y + pos.y, 
          1.0, 1.0),
        vec2f(   
          (obj_position.z + tex_coords.x) * SPRITE_TILE_SIZE,
          (obj_position.w + tex_coords.y) * SPRITE_TILE_SIZE
        )
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
