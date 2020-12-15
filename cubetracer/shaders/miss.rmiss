#version 460
#extension GL_NV_ray_tracing : require

#define PI 3.1415926535897932384626433832795

layout(location = 0) rayPayloadInNV vec3 hitValue;

const uint CST_SKY_NUM_SAMPLES = 16;
const uint CST_SKY_NUM_SAMPLES_LIGHT = 8;
const vec3 CST_SKY_BETA_R = vec3(3.8e-6f, 13.5e-6f, 33.1e-6f); 
const vec3 CST_SKY_BETA_M = vec3(21e-6f); 
const float CST_SKY_EARTH_RADIUS = 6360000;
const float CST_SKY_ATMOSPHERE_RADIUS = 6420000;
const float CST_SKY_ATMOSPHERE_RADIUS2 = CST_SKY_ATMOSPHERE_RADIUS*CST_SKY_ATMOSPHERE_RADIUS;
const float CST_SKY_HR = 7994.0;
const float CST_SKY_HM = 1200.0;
const float CST_SUN_INTENSITY = 20.0;

layout(binding = 5, set = 0) uniform Uniforms {
    vec3 sunDirection;
} scene;

bool rayAtmosphereIntersect(const vec3 orig, const vec3 dir, const float radius, out float tmin, out float tmax) {
    float tca = -dot(orig, dir);
    float d2 = dot(orig, orig) - tca*tca;

    if (d2 > CST_SKY_ATMOSPHERE_RADIUS2)
        return false;

    float thc = sqrt(CST_SKY_ATMOSPHERE_RADIUS2 - d2);

    tmin = tca - thc;
    tmax = tca + thc;
    return true;
}

vec3 computeSkyLight(vec3 dir, const vec3 origin)
{
    const vec3 orig = vec3(0, CST_SKY_EARTH_RADIUS - origin.y, 0);
    float tmin, tmax;

    if (!rayAtmosphereIntersect(orig, dir, CST_SKY_ATMOSPHERE_RADIUS, tmin, tmax) || tmax < 0) return vec3(0);

    if (tmin < 0.0) tmin = 0.0;

    // mie and rayleigh contribution
    vec3 sumR = vec3(0);
    vec3 sumM = vec3(0);

    float segmentLength = (tmax - tmin) / CST_SKY_NUM_SAMPLES;
    float tCurrent = tmin;

    float opticalDepthR = 0, opticalDepthM = 0;
    float mu = dot(dir, -scene.sunDirection); // mu in the paper which is the cosine of the angle between the sun direction and the ray direction
    float phaseR = 3.f / (16.f * PI) * (1 + mu * mu);
    float g = 0.76f;
    float phaseM = 3.f / (8.f * PI) * ((1.f - g * g) * (1.f + mu * mu)) / ((2.f + g * g) * pow(1.f + g * g - 2.f * g * mu, 1.5f));

    for (int i = 0; i < CST_SKY_NUM_SAMPLES; i++) {
        vec3 samplePosition = orig + (tCurrent + segmentLength * 0.5f) * dir;
        float height = length(samplePosition) - CST_SKY_EARTH_RADIUS;

        // compute optical depth for light
        float hr = exp(-height / CST_SKY_HR) * segmentLength;
        float hm = exp(-height / CST_SKY_HM) * segmentLength;
        opticalDepthR += hr;
        opticalDepthM += hm;

        // light optical depth
        float t0Light, t1Light;
        rayAtmosphereIntersect(samplePosition, -scene.sunDirection, CST_SKY_ATMOSPHERE_RADIUS, t0Light, t1Light);
        float segmentLengthLight = t1Light / CST_SKY_NUM_SAMPLES_LIGHT, tCurrentLight = 0;
        float opticalDepthLightR = 0, opticalDepthLightM = 0;

        int j;
        for (j = 0; j < CST_SKY_NUM_SAMPLES_LIGHT; ++j) {
            vec3 samplePositionLight = samplePosition + (tCurrentLight + segmentLengthLight * 0.5f) * (-scene.sunDirection);
            float heightLight = length(samplePositionLight) - CST_SKY_EARTH_RADIUS;
            if (heightLight < 0) break;
            opticalDepthLightR += exp(-heightLight / CST_SKY_HR) * segmentLengthLight;
            opticalDepthLightM += exp(-heightLight / CST_SKY_HM) * segmentLengthLight;
            tCurrentLight += segmentLengthLight;
        }

        if (j == CST_SKY_NUM_SAMPLES_LIGHT) {
            vec3 tau = CST_SKY_BETA_R * (opticalDepthR + opticalDepthLightR) + CST_SKY_BETA_M * 1.1f * (opticalDepthM + opticalDepthLightM);
            vec3 attenuation = vec3(exp(-tau.x), exp(-tau.y), exp(-tau.z));
            sumR += attenuation * hr;
            sumM += attenuation * hm;
        }

        tCurrent += segmentLength;
    }

    return (sumR * CST_SKY_BETA_R * phaseR + sumM * CST_SKY_BETA_M * phaseM) * CST_SUN_INTENSITY;
}

void main() {
    // Cornflower blue ftw
    hitValue = computeSkyLight(normalize(gl_WorldRayDirectionNV), gl_WorldRayOriginNV);
}
