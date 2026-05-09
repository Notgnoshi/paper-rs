GRADLE_VERSION := 9.5.0

.PHONY: all
all: gradlew
	./gradlew --version

gradlew:
	gradle wrapper --gradle-version $(GRADLE_VERSION)

.PHONY: clean
clean:
	rm -rf .gradle gradle gradlew gradlew.bat
