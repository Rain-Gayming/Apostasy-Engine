#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 1) in flat uint fragTexId;
layout(location = 2) in flat uint fragAtlasTiles;

layout(binding = 0) uniform sampler2D atlas;

layout(location = 0) out vec4 outColor;

void main() {
    float tile_size = 1.0 / float(fragAtlasTiles);

    float tx = float(fragTexId % fragAtlasTiles) * tile_size;
    float ty = float(fragTexId / fragAtlasTiles) * tile_size;

    // wrap UV within the tile
    vec2 uv = vec2(tx, ty) + fract(fragUV) * tile_size;

    outColor = texture(atlas, uv);
}
