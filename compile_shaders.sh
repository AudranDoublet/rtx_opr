#!/bin/bash

find ./cubetracer/shaders -not -name *.spv -type f -exec glslangValidator -V -o {}.spv {} \;
