# eframe in android

This part of the repo will build an android app using a `cdylib` named `eframe_template_android`. This `cdylib` will be used by the android app to render the UI.

The chain is: `eframe_template` used in `eframe_template_android` used in `eframe_template` (android app)

## Building the android app

```sh
cargo install cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# install android ndk and sdk manually - you can follow steps on https://golb.n4n5.dev/android


# install gradle by following instructions at https://gradle.org/install/
# after the gradle install you can now install gradlew by running
make gradle

# will compile the cdylib and copy it to the android app
make

# run your android emulator

# this will install the app on the emulator
make run-on-device

```
