#!/usr/bin/env bash

set -euxo pipefail

env | sort

if [ "$RUNNER_OS" == "macOS" ]; then
    brew install zimg automake libtool
fi

PREFIX="$PWD/vapoursynth-build"

if [ ! -d "$PREFIX" ]; then
    pip install Cython

    git clone --depth 1 --branch release-3.0.5 --recursive --shallow-submodules \
        https://github.com/sekrit-twc/zimg
    cd zimg
    ./autogen.sh
    ./configure --prefix="$PREFIX"
    make -j"$(nproc)"
    make install

    cd ../vapoursynth
    export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig"
    ./autogen.sh
    ./configure CFLAGS="-g -O0 -w" --prefix="$PREFIX"
    make -j"$(nproc)"
    make install
fi

ls -lR "$PREFIX"

if [ "$RUNNER_OS" == "Linux" ]; then
    sudo sh -c "echo $PREFIX/lib > /etc/ld.so.conf.d/vapoursynth.conf"
    sudo ldconfig
fi
