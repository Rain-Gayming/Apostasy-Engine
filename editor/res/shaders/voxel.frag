#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 1) in flat uint fragTexId;
layout(location = 2) in flat uint fragAtlasTiles;
layout(location = 3) in flat uint fragFace;
layout(location = 4) in flat uint fragQuadW;
layout(location = 5) in flat uint fragQuadH;

layout(binding = 0) uniform sampler2D atlas;

layout(location = 0) out vec4 outColor;


void main() {
    float tile_size = 1.0 / float(fragAtlasTiles);
    uint tx = fragTexId % fragAtlasTiles;
    uint ty = fragTexId / fragAtlasTiles;

    vec2 tiled_uv = vec2(
        fragUV.x * float(fragQuadW),
        fragUV.y * float(fragQuadH)
    );

    vec2 local_uv;
if (fragFace == 0u) {        // +X
    local_uv = fract(vec2(tiled_uv.y, 1.0 - tiled_uv.x));
} else if (fragFace == 1u) { // -X
    local_uv = fract(vec2(1.0 - tiled_uv.y, 1.0 - tiled_uv.x));
} else if (fragFace == 2u) { // +Y
    local_uv = fract(vec2(tiled_uv.x, tiled_uv.y));
} else if (fragFace == 3u) { // -Y
    local_uv = fract(vec2(tiled_uv.x, tiled_uv.y));
} else if (fragFace == 4u) { // +Z
    local_uv = fract(vec2(tiled_uv.x, 1.0 - tiled_uv.y));
} else {                     // -Z
    local_uv = fract(vec2(1.0 - tiled_uv.x,  tiled_uv.y));
}

    vec2 uv = vec2(float(tx), float(ty)) * tile_size + local_uv * tile_size;

    float shade = 1.0;
    if (fragFace == 0u || fragFace == 1u) shade = 0.8;
    if (fragFace == 4u || fragFace == 5u) shade = 0.6;
    if (fragFace == 3u)                   shade = 0.4;

    vec4 color = texture(atlas, uv);
    outColor = vec4(color.rgb * shade, color.a);
}
