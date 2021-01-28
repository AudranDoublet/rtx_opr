#ifndef  _SHADING_H_
#define  _SHADING_H_

#extension GL_GOOGLE_include_directive : enable
#include "constants.h"

// https://en.wikipedia.org/wiki/Schlick%27s_approximation
vec3 fresnelSchlick(float NoL, vec3 F0) {
    return F0 + (1 - F0) * pow(1 - NoL, 5);
}

vec3 fresnelSchlick(float NoL) {
    return fresnelSchlick(NoL, vec3(0.04));
}

vec3 fresnelSchlick(float NoL, vec3 surfaceColor, float metalness) {
    vec3 F0 = vec3(0);
    F0 = mix(F0, surfaceColor, metalness);
    return fresnelSchlick(NoL, F0);
}

// http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html
float GGX_NDF(float alpha, float NoH) {
    float D = alpha / (NoH * NoH * (alpha*alpha - 1) + 1);
    return D*D / C_PI;
}

float G1_GGX(float roughness, float NoL) {
    float alpha2 = roughness*roughness;
    return 2 * NoL / (NoL+ sqrt(alpha2 + (1 - alpha2) * NoL * NoL));
}


vec3 GGXMicrofacetBRDF(const vec3 mer, const vec3 surfaceColor, const vec3 N, const vec3 L, const vec3 V, float NoL)
{
    float roughness = mer.z;

    vec3 H = normalize(L - V);

    float VoH = max(0, -dot(V, H));
    float NoV = max(0, -dot(N, V));
    float NoH = max(0, dot(N, H));

    if (NoL > 0)
    {

        float alpha = max(roughness*roughness, 0.02);
        const float G = G1_GGX(alpha, NoL) * G1_GGX(alpha, NoV);
        float D = GGX_NDF(alpha, NoH);

        const vec3 F = fresnelSchlick(VoH, surfaceColor, mer.r);

        return F * D * G / (4 * NoL * NoV);
    }

    return vec3(0);
}


void diffuseBurleySun(const vec3 hitPoint, const vec3 N, const vec3 mer, 
        const vec3 surfaceColor,
        out vec3 diffuse, 
        out vec3 specular,
        out float NoL) {
    diffuse = UNI_SUN.color.rgb;

    const vec3 V = normalize(hitPoint - vec3(UNI_CAMERA.origin.xyz));
    const vec3 L = -UNI_SUN.direction;
    NoL = max(0, dot(N, L));

    specular = diffuse * GGXMicrofacetBRDF(mer, surfaceColor, N, L, V, NoL);
    diffuse *= NoL;
}

vec3 sunIllum(vec3 N) {
    return UNI_SUN.color.rgb * max(0, dot(N, -UNI_SUN.direction));
}

#endif // _SHADING_H_
