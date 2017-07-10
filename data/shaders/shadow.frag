#version 150

in vec4 v_Color;
in highp vec2 v_TexCoords;

uniform lowp sampler2D tex;

out lowp vec4 out_color;

void main() {
  out_color = vec4(v_Color.rgb, texture(tex, v_TexCoords).r);
}
