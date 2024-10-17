in vec2 v_uv;

out vec4 tex;

uniform sampler2D source_texture;

void main() {
  tex = vec4(texture(source_texture, v_uv).rgb, 1.0);

  tex = pow(tex, vec4(1.0 / 2.2));
}
