#version 450

layout(location = 0) out vec4 outColor;
layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragTexCoord;

void main() {
  float brightness = dot(fragNormal, vec3(-1.0, 0.5, 0.9));
  vec3 regular_color = vec3(0.8, 0.8, 0.8);
  vec3 dark_color = vec3(0.2, 0.2, 0.2);
  vec4 color = vec4(mix(dark_color, regular_color, brightness), 1.0);
  outColor = color;
}
