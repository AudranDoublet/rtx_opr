#!/bin/bash

find ./cubetracer/shaders -not \( -name *.spv -o -name *.h -o -name *.glsl \) -type f -exec glslangValidator -V -o {}.spv {} \;
