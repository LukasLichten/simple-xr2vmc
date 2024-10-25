.phony: all build run clean test help 

all: build run

build: 
	cargo build --release

run:
	./target/release/simple-xr2vmc

clean:
	cargo clean

test: 
	# cargo test -p sample_plugin
	echo "TODO"

help:
	@echo "Makefile for build Simple-XR2VMC"
	@echo "make:             Runs 'make build' and 'make run'"
	@echo "make build:       Builds (release mode)"
	@echo "make run:         Runs it (release mode)"
	@echo "make clean:       Runs cargo clean"
	@echo "make test:        TODO Runs tests"
	@echo "make help:        Prints this info"
