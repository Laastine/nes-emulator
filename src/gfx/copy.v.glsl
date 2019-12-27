out vec2 v_uv;

vec2[4] VERTICES = vec2[](
  vec2(-1.0, -1.0),
  vec2( 1.0, -1.0),
  vec2( 1.0,  1.0),
  vec2(-1.0,  1.0)
);

void main() {
  vec2 pos = VERTICES[gl_VertexID];

  gl_Position = vec4(pos, 0.0, 1.0);
  v_uv = pos * 0.5 + 0.5;
}
