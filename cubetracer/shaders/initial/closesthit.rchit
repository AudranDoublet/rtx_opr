#version 460
#extension GL_NV_ray_tracing : require
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_GOOGLE_include_directive : enable

#include "../triangle_data.h"

#define G_UNIFORM_SET 0
#include "../global_uniforms.h"
#undef G_UNIFORM_SET

#include "../payloads.h"
layout(location = PAYLOAD_IDX_GENERAL) rayPayloadInNV InitialPayload PAYLOAD_GENERAL;

hitAttributeNV vec3 attribs;

void main() {
    // FIXME: maybe we should avoid deref multiple times and store the struct?? 
    const float lod = gl_RayTmaxNV / 10.0;
    const vec3 normal = BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].normal;
    uint material = BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].material;

    vec3 orig =  BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tex_orig
                    +  BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tex_u * attribs.x
                    +  BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tex_v * attribs.y;
    vec3 color_modifier = vec3(1.0);

    PAYLOAD_GENERAL.hit_position = gl_WorldRayOriginNV + gl_WorldRayDirectionNV * gl_HitTNV;// + normal * 0.001;

    // if material is 2 (grass color overlay) and if the overlay isn't transparent,
    //  switch to material 1 and use the overlay texture
    if (material == 2 && textureLod(UNI_TEXTURE_ARRAY, orig + vec3(0, 0, 3), lod).a > 0.5) {
        material = 1;
        orig.z += 3;
    }

    // if material is 1 (need to apply grass color), retrieve the grass color as color_modifier
    if (material == 1) {
        int x = int(round(PAYLOAD_GENERAL.hit_position.x)) & 15;
        int z = int(round(PAYLOAD_GENERAL.hit_position.z)) & 15;

        color_modifier = BLAS_CHUNK_COLUMN_COLOR[gl_InstanceID].colors[x + z*16].xyz;
    }

    const mat3 transform = mat3(
        BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].tangeant,
        BLAS_TRIANGLE_DATA[gl_InstanceID].data[gl_PrimitiveID].bitangeant,
        normal
    );

    // normal deformation
    const vec3 normal_deformed = transform * (2 * textureLod(UNI_TEXTURE_ARRAY, orig + vec3(0, 0, 1), lod).xyz - vec3(1.0));
    const vec3 illum = max(dot(-UNI_SCENE.sunDirection, normal_deformed), 0.0) * textureLod(UNI_TEXTURE_ARRAY, orig, lod).xyz * color_modifier;
    const vec3 coeffs = textureLod(UNI_TEXTURE_ARRAY, orig + vec3(0, 0, 2), lod).xyz;

    PAYLOAD_GENERAL.hit = true;
    PAYLOAD_GENERAL.normal = normal_deformed;
    PAYLOAD_GENERAL.distance = gl_RayTmaxNV;
    PAYLOAD_GENERAL.illumination = illum;

    PAYLOAD_GENERAL.mer = coeffs;
}
