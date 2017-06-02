#version 150

in uvec2 position;
in uvec2 map_coord;
in vec4 color;

uniform uvec2 tile_size;
uniform mat4 matrix;

out lowp vec4 v_Color;

void main() {
  v_Color = color / 255.0;
  gl_Position = matrix * vec4(map_coord * tile_size + position * tile_size, 0.0, 1.0);
}
