#version 150

in uvec2 position;
in uvec2 map_coord;
in vec4 color;
in int tile_index;

uniform uvec2 tile_size;
uniform mat4 matrix;
uniform vec2 tex_ratio;

out lowp vec4 v_Color;
out highp vec2 v_TexCoords;

vec2 tile_offset(int index) {
  float ax = (index % 4);
  float ay = (index / 4);
  return vec2(ax, ay);
}

vec2 tex_coords(vec2 pos) {
  vec2 new_pos = pos + tile_offset(tile_index);
  float u = new_pos.x * tex_ratio.x;
  float v = 1.0 - (new_pos.y * tex_ratio.y);
  return vec2(u, v);
}

void main() {
  v_Color = color / 255.0;
  gl_Position = matrix * vec4(map_coord * tile_size + position * tile_size, 0.0, 1.0);
  v_TexCoords = tex_coords(position);
}
