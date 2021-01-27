#ifndef  _CONSTANTS_H_
#define  _CONSTANTS_H_

#define C_PI 3.1415926535897932384626433832795

// PATH TRACING CONSTANTS
#define C_PT_PROBA (1.0/(2.0 * C_PI)) // probabily of taking a given ray within a cos hemisphere
#define C_PT_MIN_CONTRIB (1e-4) // min contrib coeff, if reached, we stop bouncing on that pixel

// VOLUMETRIC LIGHTING
#define C_SUN_DISTANCE 256.0
#define C_SUN_COLOR    vec3(1.0, 1.0, 1.0)

#endif // _CONSTANTS_H_
