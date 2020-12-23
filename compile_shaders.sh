#!/bin/bash

find ./cubetracer/shaders -not \( -name *.spv -o -name *.h \) -type f -exec glslangValidator -V -o {}.spv {} \;
