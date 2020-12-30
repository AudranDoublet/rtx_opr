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
    // FIXME: maybe we should avoid deref 2 times and store the struct?? 
    vec3 normal = blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].normal;

    vec3 orig = blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].tex_orig
                    +  blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].tex_u * attribs.x
                    +  blas_triangle_data[gl_InstanceID].data[gl_PrimitiveID].tex_v * attribs.y;

    float lod = gl_RayTmaxNV / 10.0;
    vec3 illum = max(dot(-scene.sunDirection, normal), 0.0) * textureLod(texture_array, orig, lod).xyz;

    payload.hit = true;
    payload.normal = normal;
    payload.distance = gl_RayTmaxNV;
    payload.hit_position = gl_WorldRayOriginNV + gl_WorldRayDirectionNV * gl_HitTNV;
    payload.illumination = illum;

    // Cast new ray in light direction
    // vec3 origin = gl_WorldRayOriginNV + gl_WorldRayDirectionNV * gl_HitTNV;

    /*
    traceNV(
        topLevelAS, 
        gl_RayFlagsTerminateOnFirstHitNV | gl_RayFlagsOpaqueNV | gl_RayFlagsSkipClosestHitShaderNV, 
        CULL_MASK, 
        1, 0, 1, 
        origin, 
        T_MIN, 
        -scene.sunDirection, 
        T_MAX, 
        1);

        hitValue *= 0.3;
    }
    */
}
