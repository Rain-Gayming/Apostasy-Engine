#version 450

layout(location = 0) in uint data_lo;
layout(location = 1) in uint data_hi;

layout(location = 0) out vec3 fragColor;

layout(push_constant) uniform Push {
    mat4 view;
    mat4 proj;
} pc;

mat4 model = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
);

void main() {
  uint x    = (data_lo >> 0u)  & 0x3Fu;  // 6 bits
  uint y    = (data_lo >> 6u)  & 0x3Fu;  // 6 bits
  uint z    = (data_lo >> 12u) & 0x3Fu;  // 6 bits
  uint face = (data_lo >> 18u) & 0x7u;   // 3 bits

  vec3 position = vec3(float(x), float(y), float(z));
  gl_Position = pc.proj * pc.view * model * vec4(position, 1.0);
  fragColor = vec3(0.1 * face, 0.1 * face, 0.1 * face);
}
