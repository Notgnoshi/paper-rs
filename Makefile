GRADLE_VERSION := 9.5.0

.PHONY: all
all: disco-plugin

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: paper-shim
paper-shim: gradlew
	./gradlew :paper-shim:build

.PHONY: disco-plugin
disco-plugin: paper-shim
	./gradlew :disco-plugin:build

.PHONY: run
run: gradlew
	./gradlew :disco-plugin:runServer

.PHONY: clean clean-all
clean:
	rm -rf ./build ./run
clean-all: clean
	rm -rf ./.gradle/ ./gradle/ gradlew gradlew.bat
