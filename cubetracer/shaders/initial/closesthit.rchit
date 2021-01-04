#version 460
#extension GL_NV_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_GOOGLE_include_directive : enable

#include "../triangle_data.h"
#include "payload.h"

layout(location = 0) rayPayloadInNV InitialPayload payload;
//layout(location = 1) rayPayloadNV bool shadowed;

hitAttributeNV vec3 attribs;

layout(binding = 0, set = 0) uniform accelerationStructureNV topLevelAS;
layout(binding = 3, set = 0) uniform Uniforms {
    vec3 sunDirection;
} scene;

layout (binding = 4, set = 0) uniform sampler2DArray texture_array;
layout (binding = 5, set = 0) buffer BlasTriangleData {
    TriangleData data[];
} blas_triangle_data[];
layout (binding = 6, set = 0) buffer ChunkTextures {
    vec3 data[];
} blas_textures[];


const uint CULL_MASK = 0xff;
const float T_MIN = 0.01;
const float T_MAX = 100.0;

void main() {
    // FIXME: maybe we should avoid deref multiple times and store the struct?? 
    const mat3 transform = mat3(
        blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].tangeant,
        blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].bitangeant,
        blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].normal
    );

    const vec3 orig = blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].tex_orig
                    +  blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].tex_u * attribs.x
                    +  blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].tex_v * attribs.y;

    const float lod = gl_RayTmaxNV / 10.0;

    // normal deformation
    const vec3 normal = transform * (2 * textureLod(texture_array, orig + vec3(0, 0, 1), lod).xyz - vec3(1.0));

    const vec3 illum = max(dot(-scene.sunDirection, normal), 0.0) * textureLod(texture_array, orig, lod).xyz;

    payload.hit = true;
    payload.normal = normal;
    payload.distance = gl_RayTmaxNV;
    payload.hit_position = gl_WorldRayOriginNV + gl_WorldRayDirectionNV * gl_HitTNV;
    payload.illumination = illum;
}
