#version 150

in highp vec2 v_TexCoords;

uniform lowp sampler2D tex;

out lowp vec4 color;

void main(void) {
  if (texture(tex, v_TexCoords).a < 0.1) {
    discard;
  }

  color = vec4(0, 0, 0, 1.0);
}
