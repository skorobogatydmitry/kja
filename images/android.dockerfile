FROM rust:1-alpine

ENV ANDROID_API=35
ENV ANDROID_HOME="/opt/android"
ENV ANDROID_NDK_ROOT="${ANDROID_HOME}/ndk/29.0.14206865"
ENV NDK_TOOLCHAIN="${ANDROID_NDK_ROOT}/toolchains/llvm/prebuilt/linux-x86_64/bin"
ENV PATH="$PATH:${NDK_TOOLCHAIN}:${ANDROID_HOME}/build-tools/36.0.0:${ANDROID_HOME}/cmdline-tools/bin"
# Android binary names doesn't match cargo's conventions
ENV CC_aarch64_linux_android=aarch64-linux-android${ANDROID_API}-clang
ENV CXX_aarch64_linux_android=aarch64-linux-android${ANDROID_API}-clang++
ENV CC_armv7_linux_androideabi=armv7a-linux-androideabi${ANDROID_API}-clang
ENV CXX_armv7_linux_androideabi=armv7a-linux-androideabi${ANDROID_API}-clang++
ENV CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=aarch64-linux-android${ANDROID_API}-clang
ENV CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=armv7a-linux-androideabi${ANDROID_API}-clang

RUN rustup target add armv7-linux-androideabi aarch64-linux-android

RUN mkdir -p "${ANDROID_HOME}/cmdline-tools"
RUN apk add curl
RUN curl -sLo /tmp/clt.zip https://dl.google.com/android/repository/commandlinetools-linux-14742923_latest.zip
RUN unzip -d "${ANDROID_HOME}" /tmp/clt.zip

RUN apk add openjdk21

RUN yes | sdkmanager --licenses --sdk_root="${ANDROID_HOME}"
RUN sdkmanager --sdk_root="${ANDROID_HOME}" --install "build-tools;36.0.0" "ndk;29.0.14206865" "platforms;android-35"

# android SDK tools don't run otherwise
RUN apk add gcompat bash

ENTRYPOINT [ "/bin/bash", "-c", "/usr/local/cargo/bin/cargo ${INPUT_CARGO_ARGS:-build}" ]
