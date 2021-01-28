SRC=\
	./cubetracer/shaders/initial/miss.rmiss \
	./cubetracer/shaders/initial/raygen.rgen \
	./cubetracer/shaders/initial/anyhit.rahit \
	./cubetracer/shaders/initial/closesthit.rchit \
	./cubetracer/shaders/shadow/miss.rmiss \
	./cubetracer/shaders/shadow/raygen.rgen \
	./cubetracer/shaders/shadow/anyhit.rahit \
	./cubetracer/shaders/path_tracing/miss.rmiss \
	./cubetracer/shaders/path_tracing/diffuse.rgen \
	./cubetracer/shaders/path_tracing/specular.rgen \
	./cubetracer/shaders/reconstruct.comp \
	./cubetracer/shaders/reconstruct.comp \
	./cubetracer/shaders/reproject.comp \
	./cubetracer/shaders/shadow_map/raygen.rgen \
	./cubetracer/shaders/shadow_map/miss.rmiss \
	./cubetracer/shaders/shadow_map/closesthit.rchit \
	./cubetracer/shaders/god_rays.comp \
	./cubetracer/shaders/god_rays_reconstruct.comp \
	./cubetracer/shaders/refract/raygen.rgen \

OBJ=$(SRC:=.spv)

all: $(OBJ)

%.spv: %
	glslangValidator -V -o $@ $^
