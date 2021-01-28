#ifndef  _SHADING_H_
#define  _SHADING_H_

#extension GL_GOOGLE_include_directive : enable
#include "constants.h"

// https://en.wikipedia.org/wiki/Schlick%27s_approximation
float fresnelSchlick(float NoL, float F0) {
    return 1 + (1 - F0) * pow(1 - NoL, 5);
}

float fresnelSchlick(float NoL) {
    return fresnelSchlick(NoL, 0.04);
}



// http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html
float NDF_GGX(float alpha, float NoH) {
    float D = alpha / (NoH * NoH * (alpha*alpha - 1) + 1);
    return D*D / C_PI;
}

float G1_GGX(float alpha, float NoV) {
    float alpha2 = alpha*alpha;
    return 2 * NoV / (NoV + sqrt(alpha2 + (1 - alpha2) * NoV * NoV));
}

// lightDir: light direction (hitPoint - lightPos) or UNI_SCENE.sunDirection
// N: normal
// L: light direction
// V: View direction (normalize(UNI_CAMERA.origin.xyz - hitPoint))
float GGXMicrofacetBRDF(const float roughness, const vec3 N, const vec3 L, const vec3 V, float NoL) {
    const float NoV = max(0, dot(N, V));
    if (NoL == 0 || NoV == 0) {
        return 0;
    }

    const float alpha = roughness * roughness;
    const vec3 H = normalize(V + L); // half vector viewDir/lightDir

    const float NoH = max(0, dot(N, H));

    const float F = fresnelSchlick(max(0, dot(V, H)));
    const float D = NDF_GGX(alpha, NoH);
    const float G2 = G1_GGX(alpha, NoL) * G1_GGX(alpha, NoV);

    return (F * D * G2) / (4.0 * NoL * NoV);
}

void diffuseBurleySun(const vec3 hitPoint, const vec3 N, const float roughness, 
        out vec3 diffuse, 
        out vec3 specular,
        out float NoL) {
    diffuse = UNI_SUN.color.rgb;

    const vec3 V = normalize(vec3(UNI_CAMERA.origin.xyz) - hitPoint);
    const vec3 L = -UNI_SUN.direction;
    NoL = max(0, dot(N, L));

    specular = diffuse * GGXMicrofacetBRDF(roughness, N, L, V, NoL);
    diffuse *= NoL;
}

float sunIllum(vec3 N) {
    return max(0, dot(N, -UNI_SUN.direction));
}

vec3 composeColor(
        vec3 diffuse,
        vec3 specular,
        float NoL
        ) {


    return diffuse * fresnelSchlick(NoL) + specular;
}

#endif // _SHADING_H_
