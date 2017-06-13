#version 150 core

uniform vec2 u_resolution;
uniform float u_time;

out lowp vec4 out_color;

void main() {
  vec2 st = gl_FragCoord.xy/u_resolution;

  float y = 1.0 - st.y;
  float percent = 1.0;
  vec3 color = y * vec3(0.0, 0.33, 1.0);

  out_color = vec4(color, 1.0);
}
