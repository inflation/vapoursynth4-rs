#!/usr/bin/env bash

set -euxo pipefail

export PREFIX="$PWD/vapoursynth/build"
export VAPOURSYNTH_LIB_PATH="$PREFIX/lib"

if [ ! -f "$VAPOURSYNTH_LIB_PATH/libvapoursynth.so" ]; then
    rm -rf vapoursynth && git clone --depth 1 --branch R70 https://github.com/vapoursynth/vapoursynth

    if [ "$RUNNER_OS" == "macOS" ]; then
        brew install zimg automake libtool
    fi

    pushd vapoursynth

    if [ ! -f "$VAPOURSYNTH_LIB_PATH/libzimg.a" ]; then
        git clone --depth 1 --branch release-3.0.5 https://github.com/sekrit-twc/zimg
        pushd zimg
        ./autogen.sh
        ./configure --prefix="$PREFIX"
        make -j"$(nproc)"
        make install
        popd
    fi

    pip install Cython

    export PKG_CONFIG_PATH="$VAPOURSYNTH_LIB_PATH/pkgconfig"
    ./autogen.sh
    ./configure CFLAGS="-g -O0 -w" --prefix="$PREFIX"
    make -j"$(nproc)"
    make install
    popd
fi

echo "VAPOURSYNTH_LIB_PATH=${VAPOURSYNTH_LIB_PATH}" >> $GITHUB_ENV
echo "LD_LIBRARY_PATH=${VAPOURSYNTH_LIB_PATH}" >> $GITHUB_ENV
echo "DYLD_LIBRARY_PATH=${VAPOURSYNTH_LIB_PATH}" >> $GITHUB_ENV
