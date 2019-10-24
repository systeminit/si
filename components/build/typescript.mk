include $(CURDIR)/../build/docker.mk

.PHONY: build build_release test run install

install:
	npm install

build: install
	npm run build

build_release: build

test: install 
	npm run test

run: start

start: install
	npm run start
