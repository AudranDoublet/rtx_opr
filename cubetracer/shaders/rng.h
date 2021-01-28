#ifndef  _RNG_H_
#define  _RNG_H_

#extension GL_GOOGLE_include_directive : enable
#include "constants.h"

// http://www.reedbeta.com/blog/quick-and-easy-gpu-random-numbers-in-d3d11/
uint update_seed(in out uint seed) {
    seed = (seed ^ 61) ^ (seed >> 16);
    seed *= 9;
    seed = seed ^ (seed >> 4);
    seed *= 0x27d4eb2d;
    seed = seed ^ (seed >> 15);
    return seed;
}

// source: https://gist.github.com/patriciogonzalezvivo/670c22f3966e662d2f83
float rand(vec2 n) { 
	return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
}

float floatConstruct( uint m ) {
    const uint ieeeMantissa = 0x007FFFFFu; // binary32 mantissa bitmask
    const uint ieeeOne      = 0x3F800000u; // 1.0 in IEEE binary32

    m &= ieeeMantissa;                     // Keep only mantissa bits (fractional part)
    m |= ieeeOne;                          // Add fractional part to 1.0

    float  f = uintBitsToFloat( m );       // Range [1:2]
    return f - 1.0;                        // Range [0:1]
}


float noise(in out  uint seed) {
    vec2 p = vec2(floatConstruct(update_seed(seed)), floatConstruct(update_seed(seed)));
	vec2 ip = floor(p);
	vec2 u = fract(p);
	u = u*u*(3.0-2.0*u);
	
	float res = mix(
		mix(rand(ip),rand(ip+vec2(1.0,0.0)),u.x),
		mix(rand(ip+vec2(0.0,1.0)),rand(ip+vec2(1.0,1.0)),u.x),u.y);
	return res*res;
}

vec3 sampleCosHemisphere(vec2 uv)
{
    float theta = 2.0 * C_PI * uv.x;
    float r = sqrt(uv.y);

    vec2 disk = vec2(cos(theta), sin(theta)) * r;
    return vec3(disk.x, sqrt(max(0.0, 1.0 - dot(disk, disk))), disk.y);
}

// source: https://github.com/NVIDIA/Q2RTX/blob/master/src/refresh/vkpt/shader/utils.glsl
mat3 construct_ONB_frisvad(vec3 normal)
{
    precise mat3 ret;
    ret[1] = normal;
    if(normal.z < -0.999805696f) {
        ret[0] = vec3(0.0f, -1.0f, 0.0f);
        ret[2] = vec3(-1.0f, 0.0f, 0.0f);
    }
    else {
        precise float a = 1.0f / (1.0f + normal.z);
        precise float b = -normal.x * normal.y * a;
        ret[0] = vec3(1.0f - normal.x * normal.x * a, b, -normal.x);
        ret[2] = vec3(b, 1.0f - normal.y * normal.y * a, -normal.y);
    }
    return ret;
}

vec3 sampleCosHemisphere(vec3 normal, vec2 uv) {
    const vec3 dir = sampleCosHemisphere(uv);
    const mat3 basis = construct_ONB_frisvad(normal);
    return normalize(basis * dir);
}

// Eric Heitz, Sampling the GGX Distribution of Visible Normals, Journal of Computer Graphics Techniques (JCGT), vol. 7, no. 4, 1–13, 2018
// Input Ve: view direction
// Input alpha: roughness parameters
// Input U1, U2: uniform random numbers
// Output Ne: normal sampled with PDF D_Ve(Ne) = G1(Ve) * max(0, dot(Ve, Ne)) * D(Ne) / Ve.z
vec3 sampleGGXVNDF(vec3 Ve, float alpha, float U1, float U2)
{
    // Section 3.2: transforming the view direction to the hemisphere configuration
    vec3 Vh = normalize(vec3(alpha* Ve.x, alpha * Ve.y, Ve.z));

    // Section 4.1: orthonormal basis (with special case if cross product is zero)
    float lensq = Vh.x * Vh.x + Vh.y * Vh.y;
    vec3 T1 = lensq > 0 ? vec3(-Vh.y, Vh.x, 0) * inversesqrt(lensq) : vec3(1,0,0);
    vec3 T2 = cross(Vh, T1);

    // Section 4.2: parameterization of the projected area
    float r = sqrt(U1);
    float phi = 2.0 * C_PI * U2;
    float t1 = r * cos(phi);
    float t2 = r * sin(phi);
    float s = 0.5 * (1.0 + Vh.z);
    t2 = (1.0 - s)*sqrt(1.0 - t1*t1) + s*t2;

    // Section 4.3: reprojection onto hemisphere
    vec3 Nh = t1*T1 + t2*T2 + sqrt(max(0.0, 1.0 - t1*t1 - t2*t2))*Vh;

    // Section 3.4: transforming the normal back to the ellipsoid configuration
    vec3 Ne = normalize(vec3(alpha * Nh.x, alpha * Nh.y, max(0.0, Nh.z)));
    return Ne;
}
#endif // _RNG_H_
