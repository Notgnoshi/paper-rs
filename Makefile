GRADLE_VERSION := 9.5.0

.PHONY: all
all: paper-shim

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: paper-shim
paper-shim: gradlew
	./gradlew :paper-shim:build

.PHONY: clean clean-all
clean:
	rm -rf ./build
clean-all: clean
	rm -rf ./.gradle/ ./gradle/ gradlew gradlew.bat
