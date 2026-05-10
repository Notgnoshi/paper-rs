GRADLE_VERSION ?= 9.5.0
RUST_LOG ?= DEBUG

.PHONY: all
all: disco-plugin

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: paper-shim
paper-shim: gradlew cargo
	./gradlew :paper-shim:build

.PHONY: disco-plugin
disco-plugin: gradlew cargo
	./gradlew :disco-plugin:build

.PHONY: cargo
cargo:
	cargo build --release

.PHONY: run
run: disco-plugin
	RUST_LOG=$(RUST_LOG) ./gradlew :disco-plugin:runServer -Pnative-lib=$(abspath target/release/libdisco_ffi.so)

.PHONY: clean clean-all
clean:
	cargo clean
	rm -rf Cargo.lock ./build/ ./*/bin/
clean-all: clean
	rm -rf ./run/ ./.gradle/ ./gradle/ gradlew gradlew.bat
