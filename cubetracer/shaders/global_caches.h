#ifndef  _GLOBAL_CACHES_H
#define  _GLOBAL_CACHES_H


#define DEF_IMAGE(BINDING, TYPE, NAME) layout(set=G_CACHE_SET, binding=BINDING, TYPE) uniform image2D CACHE_ ## NAME

/* FIXME: What about compressing the images? as we're never using the alpha channels */

// GENERAL CACHES
DEF_IMAGE(0,  rgba8,   RESULT_IMAGE);

DEF_IMAGE(1,  r32f   , DENOISE_PREV_HISTORY_LENGTH);
DEF_IMAGE(2,  r32f,    DENOISE_NEW_HISTORY_LENGTH);
DEF_IMAGE(3,  rgba32f, DENOISE_PREV_MOMENTS);
DEF_IMAGE(4,  rgba32f, DENOISE_NEW_MOMENTS);

DEF_IMAGE(5,  rgba32f, NORMALS);

DEF_IMAGE(6,  r32f,    INIT_DISTANCES);
DEF_IMAGE(7,  r32f,    DENOISE_PREV_INITIAL_DISTANCES);

DEF_IMAGE(8,  rgba32f, DIRECT_ILLUM);
DEF_IMAGE(9,  rgba32f, DENOISE_PREV_DIFFUSE);

DEF_IMAGE(10, rgba32f, ORIGIN);
DEF_IMAGE(11, rgba32f, SHADOWS);
DEF_IMAGE(12, rgba32f, ILLUM_COEFFS);

// PATH TRACING CACHES
DEF_IMAGE(13, rgba32f, PT_ILLUM);

#endif // _GLOBAL_CACHES_H
