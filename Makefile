GRADLE_VERSION := 9.5.0

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
	./gradlew :disco-plugin:runServer -PnativeLib=$(abspath target/release/libdisco_ffi.so)

.PHONY: clean clean-all
clean:
	cargo clean
	rm -rf ./build ./run
clean-all: clean
	rm -rf ./.gradle/ ./gradle/ gradlew gradlew.bat
