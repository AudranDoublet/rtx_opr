#ifndef  _SHADING_H_
#define  _SHADING_H_

#extension GL_GOOGLE_include_directive : enable
#include "constants.h"

// https://en.wikipedia.org/wiki/Schlick%27s_approximation
float fresnelSchlickSameMaterial(float cosTheta) {
    return pow(1 - cosTheta, 5);
}


// http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html
float ndfGGXThrowbridgeReitz(float alpha, float cosTheta_N_H) {
    float D = alpha / (cosTheta_N_H * cosTheta_N_H * (alpha*alpha - 1) + 1);
    return D*D / C_PI;
}

float geomShadowingGGX(float alpha, float cosTheta_N_L, float cosTheta_N_V) {

    // http://graphicrants.blogspot.com/2013/08/specular-brdf-reference.html
    // float alpha2 = alpha*alpha;
    // return 2 * cosTheta_N_V / (cosTheta_N_V + sqrt(alpha2 + (1 - alpha2) * cosTheta_N_V * cosTheta_N_V));

    // http://perso.univ-lyon1.fr/jean-claude.iehl/Public/educ/M1IMAGE/html/group__brdf.html
    float tan2_theta_N_V = 1 / (cosTheta_N_V*cosTheta_N_V) - 1;
    float lambda_N_V = 1 + alpha*alpha * tan2_theta_N_V;
    float tan2_theta_N_L = 1 / (cosTheta_N_L * cosTheta_N_L) - 1;
    float lambda_N_L = 1 + alpha*alpha * tan2_theta_N_L;
    float G2= 2 / (sqrt(lambda_N_V) + sqrt(lambda_N_L));

    return G2;
}

// lightDir: light direction (hitPoint - lightPos) or UNI_SCENE.sunDirection
// N: normal
// L: light direction
// V: View direction (normalize(UNI_CAMERA.origin.xyz - hitPoint))
float GGXMicrofacetBRDF(const float roughness, const vec3 N, const vec3 L, const vec3 V) {
    const float cosTheta_N_L = max(0, dot(N, L));
    if (cosTheta_N_L ==  0)
        return 0;

    const float alpha = roughness * roughness;
    const vec3 H = normalize(V + L); // half vector viewDir/lightDir

    const float cosTheta_N_V = max(0, dot(N, V));
    const float cosTheta_N_H = max(0, dot(N, H));

    const float F = fresnelSchlickSameMaterial(max(0, dot(V, H)));
    const float D = ndfGGXThrowbridgeReitz(alpha, cosTheta_N_H);
    const float G2 = geomShadowingGGX(alpha, cosTheta_N_L, cosTheta_N_V);

    return (F * D * G2) / (4.0 * cosTheta_N_L * cosTheta_N_V);
}

void diffuseBurleySun(const vec3 hitPoint, const vec3 N, const float roughness, out vec3 diffuse, out vec3 specular) {
    diffuse = UNI_SUN.color.rgb;

    const vec3 V = normalize(vec3(UNI_CAMERA.origin.xyz) - hitPoint);
    const vec3 L = -UNI_SUN.direction;

    specular = diffuse * GGXMicrofacetBRDF(roughness, N, L, V);
    diffuse *= max(dot(L, N), 0.0);
}

vec3 composeColor(const vec3 hitPoint, const vec3 N, const vec3 albedo, const vec3 mer) {
    vec3 diffuse;
    vec3 specular;
    diffuseBurleySun(hitPoint, N, mer.b, diffuse, specular);


    diffuse = albedo * (1 - specular);
    specular = mix(vec3(1), albedo, mer.r) * specular;

    return diffuse + specular;
}

#endif // _SHADING_H_
