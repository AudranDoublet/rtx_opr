#ifndef  _GLOBAL_UNIFORMS_H
#define  _GLOBAL_UNIFORMS_H

#include "triangle_data.h"

#ifdef G_UNIFORM_INC_NV
layout(set=G_UNIFORM_SET, binding = 0) uniform accelerationStructureNV UNI_TLAS;
#endif

layout(set=G_UNIFORM_SET, binding = 1) uniform SCamera{
    dmat4 screen_to_world;
    dmat4 prev_world_to_screen;
    dvec4 origin;
} UNI_CAMERA;

layout(set=G_UNIFORM_SET, binding = 2) uniform SScene {
    uint rendered_buffer;
    uint updated;
} UNI_SCENE;

layout (set=G_UNIFORM_SET, binding = 3) uniform sampler2DArray UNI_TEXTURE_ARRAY;

layout (set=G_UNIFORM_SET, binding = 4) buffer BlasTriangleData {
    TriangleData data[];
} BLAS_TRIANGLE_DATA[];

layout (set=G_UNIFORM_SET, binding = 5) buffer ChunkTextures {
    vec3 data[];
} BLAS_TEXTURES[];
layout (set=G_UNIFORM_SET, binding = 7) buffer ChunkColumnColor {
    vec3 colors[];
} BLAS_CHUNK_COLUMN_COLOR[];

layout(set=G_UNIFORM_SET, binding = 6) uniform SSun{
    mat4 projection;
    vec3 direction;
} UNI_SUN;
#endif // _GLOBAL_UNIFORMS_H
