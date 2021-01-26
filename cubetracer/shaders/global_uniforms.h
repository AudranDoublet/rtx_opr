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
    bool updated;
} UNI_CAMERA;

layout(set=G_UNIFORM_SET, binding = 2) uniform SScene {
    vec3 sunDirection;
    uint rendered_buffer;
} UNI_SCENE;

layout (set=G_UNIFORM_SET, binding = 3) uniform sampler2DArray UNI_TEXTURE_ARRAY;
layout (set=G_UNIFORM_SET, binding = 4) buffer BlasTriangleData {
    TriangleData data[];
} BLAS_TRIANGLE_DATA[];
layout (set=G_UNIFORM_SET, binding = 5) buffer ChunkTextures {
    vec3 data[];
} BLAS_TEXTURES[];

#endif // _GLOBAL_UNIFORMS_H
