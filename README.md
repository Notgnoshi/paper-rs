# paper-rs PoC

An attempt at making a Paper Minecraft plugin in Rust

## Dependencies

### Rust 1.95

<https://rust-lang.org/tools/install/>

### Cap'n Proto 1.0.1

```sh
# Fedora
sudo dnf install capnproto capnproto-devel
# Ubuntu
sudo apt install capnproto libcapnp-dev
```

And the Java Cap'n Proto compiler:

```sh
git clone https://github.com/capnproto/capnproto-java.git /tmp/capnproto-java
pushd /tmp/capnproto-java
make
PREFIX=$HOME/.local make install
popd
```

You may want to unset `PREFIX` and `sudo make install` to the default `/usr/local` if you want to
use capnproto-java system-wide.

### Java 25 and Gradle

```sh
# Fedora
sudo dnf install java-25-openjdk-devel
# Ubuntu 26.04
sudo apt install openjdk-25-jdk gradle
```

On Ubuntu 24.04 the default repos don't ship `openjdk-25-jdk`, so install the Temurin JDK instead:
<https://adoptium.net/installation/linux#_deb_installation_on_debian_or_ubuntu>.

Fedora doesn't package Gradle. Install the binary distribution to `~/.local/`:

```sh
GRADLE_VERSION=9.5.0
curl -fsSL -o "/tmp/gradle.zip" \
    "https://services.gradle.org/distributions/gradle-${GRADLE_VERSION}-bin.zip"
unzip "/tmp/gradle.zip" -d "$HOME/.local/share/"
ln -sf "$HOME/.local/share/gradle-${GRADLE_VERSION}/bin/gradle" "$HOME/.local/bin/gradle"
```

Gradle is only needed once, to bootstrap the gitignored `./gradlew` wrapper. After the wrapper is in
place, `./gradlew` self-manages its Gradle distribution and Gradle does not need to be on `$PATH`.
