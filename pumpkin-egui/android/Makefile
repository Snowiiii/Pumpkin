.DEFAULT_GOAL := apk
.PHONY: jni apk run-on-device

gradle:
	cd java && gradle wrapper

jni:
	cargo ndk --target arm64-v8a -o ./java/app/src/main/jniLibs/ build --profile release --offline
	cargo ndk --target x86_64-linux-android -o ./java/app/src/main/jniLibs/ build --profile release --offline

apk: jni
	cd java && ./gradlew build --stacktrace

run-on-device: jni
	adb uninstall local.eframe_template || true

	cd java && ./gradlew installDebug
	adb shell am start -n local.eframe_template/.MainActivity
	adb logcat -v color -s eframe_template *:e

clean:
	rm -rf java/app/src/main/jniLibs/
	rm -rf java/app/build/
	rm -rf java/.gradle/
	rm -rf target/
	rm -rf dist/android-28