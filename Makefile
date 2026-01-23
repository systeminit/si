all: build-binaries build-images tag-images

build-binaries:
	buck2 build @//mode/release edda pinga rebaser sdf veritech luminork forklift

build-images:
	./build.sh

tag-images:
	./tag.sh
