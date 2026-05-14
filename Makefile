GRADLE_VERSION ?= 9.5.0
RUST_LOG ?= DEBUG

.PHONY: all
all: disco-plugin

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: papermc
papermc: gradlew cargo
	./gradlew :papermc:build

.PHONY: disco-plugin
disco-plugin: gradlew cargo
	./gradlew :disco-plugin:build

.PHONY: cargo
cargo:
	cargo build --release

.PHONY: run
run: disco-plugin
	RUST_LOG=$(RUST_LOG) ./gradlew :disco-plugin:runServer

.PHONY: clean clean-all
clean:
	cargo clean
	rm -rf Cargo.lock ./build/ ./*/bin/
clean-all: clean
	rm -rf ./run/ ./.gradle/ ./gradle/ gradlew gradlew.bat
