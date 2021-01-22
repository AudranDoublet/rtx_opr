SRC=\
	./cubetracer/shaders/initial/miss.rmiss \
	./cubetracer/shaders/initial/raygen.rgen \
	./cubetracer/shaders/initial/anyhit.rahit \
	./cubetracer/shaders/initial/closesthit.rchit \
	./cubetracer/shaders/shadow/miss.rmiss \
	./cubetracer/shaders/shadow/raygen.rgen \
	./cubetracer/shaders/shadow/anyhit.rahit \
	./cubetracer/shaders/path_tracing/miss.rmiss \
	./cubetracer/shaders/path_tracing/raygen.rgen \
	./cubetracer/shaders/reconstruct.comp \
	./cubetracer/shaders/reproject.comp

OBJ=$(SRC:=.spv)

all: $(OBJ)

%.spv: %
	glslangValidator -V -o $@ $^
