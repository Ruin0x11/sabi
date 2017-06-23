#version 150

in uvec2 position;

in vec2 tex_offset;
in uvec2 map_coord;
in vec2 tex_ratio;
in uvec2 sprite_size;

uniform mat4 matrix;
uniform uvec2 tile_size;
uniform vec2 angle;
uniform float time;

out highp vec2 v_TexCoords;

vec2 sprite_texture(vec2 pos) {
  float u = pos.x * tex_ratio.x + tex_offset.x;
  float v = 1.0 - (pos.y * tex_ratio.y + tex_offset.y);
  return vec2(u, v);
}

vec2 sprite_offset(vec2 size) {
  return (vec2(48, 48) - size);
}

void main() {
  float angle = 1.2;
  // vec2 soffset = sprite_offset(sprite_size * vec2(-sin(angle) * 2, 1.0));
  vec2 soffset = sprite_offset(sprite_size + vec2(0, 48));
  mat4 rot = mat4(1.0, 0.0, 0.0, 0.0,  -sin(angle) * 2, 1.0, 0.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0 );
  mat4 scale = mat4(1.0, 0.0, 0.0, 0.0,  0.0, cos(angle) /2 + 0.5, 0.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0 );
  // vec4 before = vec4(map_coord * tile_size + position * sprite_size + soffset + vec2(0, -24), 0.0, 1.0);
  vec4 asd =  rot * scale * (vec4(position * sprite_size + soffset, 0.0, 1.0)) + vec4(map_coord * tile_size * vec2(2.0, 2.0) + position * sprite_size + vec2(8, 0), 0.0, 1.0);
  gl_Position = matrix * asd;
  v_TexCoords = sprite_texture(position);
}
