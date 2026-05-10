GRADLE_VERSION := 9.5.0

.PHONY: all
all: disco-plugin

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: paper-shim
paper-shim: gradlew cargo
	./gradlew :paper-shim:build

.PHONY: disco-plugin
disco-plugin: gradlew cargo bindings
	./gradlew :disco-plugin:build

.PHONY: cargo
cargo:
	cargo build --release

.PHONY: bindings
bindings: cargo
	rm -rf build/generated/c disco-plugin/src/main/java/io/disco/ffi
	mkdir -p build/generated/c disco-plugin/src/main/java
	cbindgen --config cbindgen.toml --crate disco-ffi --output build/generated/c/disco.h
	jextract --target-package io.disco.ffi --header-class-name DiscoFfi \
	    --output disco-plugin/src/main/java \
	    build/generated/c/disco.h

.PHONY: run
run: disco-plugin
	./gradlew :disco-plugin:runServer -PnativeLib=$(abspath target/release/libdisco_ffi.so)

.PHONY: clean clean-all
clean:
	cargo clean
	rm -rf Cargo.lock ./build/ ./run/ ./disco-plugin/src/main/java/io/disco/ffi ./*/bin/
clean-all: clean
	rm -rf ./.gradle/ ./gradle/ gradlew gradlew.bat
