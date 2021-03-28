# Introduction

Minecraft-like game with infinite maps rendered using path-tracing techniques and NVIDIA's RTX technology (Vulkan).
This project is a rewrite of a [project we did with OpenGL and raytracing without hardware acceleration](https://github.com/AudranDoublet/glopr).

This project is originally an assignment for our **advanced introduction to OpenGL** course, nothing serious.

The project is entirely written in Rust and GLSL (for Vulkan), using [Ash](https://github.com/MaikKlein/ash) for Vulkan bindings.

## Samples

Images are more talkative than long texts, so here are few samples of what our engines are capable of:

<img src="/data/samples/1.png" width="384" height="216"> <img src="/data/samples/4.png" width="384" height="216">
<img src="/data/samples/2.png" width="384" height="216"> <img src="/data/samples/5.png" width="384" height="216">
<img src="/data/samples/3.png" width="384" height="216"> <img src="/data/samples/6.png" width="384" height="216">

Here is a demonstration video that shows some scenes captured from the game (*you have to click on the image below*):

[![Demonstration Video](http://img.youtube.com/vi/OUKqvlPS1nk/0.jpg)](http://www.youtube.com/watch?v=OUKqvlPS1nk "Demonstration Video")

## Disclaimer

We made every resources used by the project except for the Textures that we took from the internet, a **big thanks** to people who contributed to those **beautiful** and **free** textures! :heart:

# Functionalities

RTX-GlOPR is a simple Minecraft-like game, which use path tracing algorithms for rendering.

We implemented a map generator:
* generation of different biomes with a global coherence (big oceans, warm / cold / temperate zones, beaches ...)
* generation of columns with coherent size (using perlin noise) and smooth transition between biomes (ex: between plains and mountains)
* generation of decorations: flowers, cactus, various trees, grass, ...

We implemented a minimalistic game engine using AABB collisions.

Finally, we implemented a rendering pipeline in Vulkan with a few steps:

FIXME schema

**Initial ray**, which is basically a ray casting for each screen pixel, to known the hitten object (and store the normal, material properties, material color, ...).
It can be noted that this could be replaced by traditional rasterization, which would probably be faster.

**Procedural skybox** is computed using Rayleigh diffusion, when a ray doesn't hit an object (initial ray and specular).

**Shadow ray** which just sends a ray from hitten point in initial ray towards the direction of the sun. This phase only handles sun shadows, not block lights.
In fact, these lights are not managed in raytracing but we "hope" to simply touch them with the path tracing phases.

The interest of this implementation is the performance: Minecraft is a game where light sources can be counted by hundreds or thousands, and it would be unthinkable to throw so many rays.

**Diffuse reflections** are implemented using Disney-Burley model. FIXME

**Specular reflections** are implemented using a Microfacet-based BRDF. FIXME

**Refraction** step sends rays through transparents surfaces (glasses and water). This step isn't done through path tracing for performance reasons.
A big flaw of this approach is that diffuse and specular lightning won't be seen behind a transparent surface, nevertheless it's still provides good results.

**Denoising** is extremly important for real-time path tracing. One of the state-of-the-art approach for this is SVGG and A-SVGF algorithms, which includes a temporal filtering
(accumulation of samples between each frames, using a reprojection) and a spatial filtering (à trous filter).

Our version implements only the temporal filter, because the rest of our graphics pipeline is not optimized enough, and we would probably have lost too much FPS. We didn't have time to optimize more for our assignment.

**Shadow maps** are used as part of the god rays rendering. It's basically implemented as a depth map of the scene from the sun view (orthographic projection). As for the initial ray, we implemented
this step using RTX, but using rasterization would probably have been better.

**God rays (atmospheric light scattering)** are due to small particles in the light-transmitting medium. To simulate the effect we samples some air points between the camera and the initial hit point. For each of
these points, we need to know whether or not their are shadowed. If they are, the point doesn't provide illumination, otherwise it does. The shadowmap is then very useful, because it wouldn't be feasible to launch so many shadow rays.

This part could benefit from a less basic implementation of shadowmaps.

Note: in fact, for performance reasons, god rays are computed with a resolution halved, and then upsampled with a bilateral filter.

**Reconstruction** produce the final image by combining all lights sources using Fresnel's law: direct illumination, denoised diffuse, denoised specular, refraction and god rays.

# Performances

FIXME

# Compile me

1. Install rustup: https://rustup.rs/
2. Run: rustup install nightly
3. In this directory, run: rustup override set nightly
4. Run cargo build --release

# Usage

Example:
```
cargo run --release -- game \
          --view-distance 10 \
          --layout fr
```

Main game parameters:
* view-distance: number of chunks seen in each direction
* layout: fr or us, main keyboard mapping
* world: world path to load
* flat: if presents, the map is flat
* seed: (number) world random seed; by default 0

# In game options

**Move** Z,Q,S,D (fr) or W,A,S,D (us)

**Break a block** Left click

**Place a block** Right click

**1,2,3,4,5,6,7,8,9,0** display pathtracing debug buffers

**Alt+1,2,3,4** change block in hand

**Toggle ambient light** L

**Set sun position** K

**Do daylight cycle** N

**Sneak** Left-shift (the player will be slower but won't fall)

**Toggle sprint** Left-control (the player will be faster)

**Toggle fly mode** Double click on space

# References

NVIDIA offers resources on RTX (including the official version of Minecraft RTX developed by them), which helped us a lot:
* [NVIDIA's Metalness-Emissivness-Roughness textures for Minecraft](https://www.nvidia.com/en-us/geforce/guides/minecraft-rtx-texturing-guide/)
* [technical presentation by members of the Minecraft RTX's team, which presents a pipeline similar to the one we use](https://www.youtube.com/watch?v=mDlmQYHApBU)
* [Q2RTX, the RTX version of Quake2 made by NVIDIA](https://github.com/NVIDIA/Q2RTX)
* [RTX tutorial for DX12](https://developer.nvidia.com/rtx/raytracing/dxr/DX12-Raytracing-tutorial-Part-1)

FIXME
[1] disney burley
[2] microfacet
[3] Q2RTX video on godrays
