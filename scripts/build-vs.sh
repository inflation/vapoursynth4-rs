#!/usr/bin/env bash

set -euxo pipefail

export PREFIX="$PWD/vapoursynth/build"
export LIB_PATH="$PREFIX/lib"
NPROC="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 2)"

if [ ! -f "$LIB_PATH/libvapoursynth.so" ]; then
    git clone --depth 1 --branch R70 https://github.com/vapoursynth/vapoursynth || true

    if [ "$(uname)" == "Darwin" ]; then
        brew install zimg automake libtool
    fi

    pushd vapoursynth

    if [ ! -f "$LIB_PATH/libzimg.a" ]; then
        git clone --depth 1 --branch release-3.0.5 https://github.com/sekrit-twc/zimg || true
        pushd zimg
        ./autogen.sh
        ./configure --prefix="$PREFIX"
        make -j"$NPROC"
        make install
        popd
    fi

    pip install Cython

    export PKG_CONFIG_PATH="$LIB_PATH/pkgconfig"
    ./autogen.sh
    ./configure CFLAGS="-g -O0 -w -DVS_USE_LATEST_API -DVSSCRIPT_USE_LATEST_API" \
        CXXFLAGS="-g -O0 -w -DVS_USE_LATEST_API -DVSSCRIPT_USE_LATEST_API" --prefix="$PREFIX"
    make -j"$NPROC"
    make install
    popd
fi

list=("$LIB_PATH"/python*/site-packages)
{
    echo "VAPOURSYNTH_LIB_PATH=${LIB_PATH}"
    echo "LD_LIBRARY_PATH=${LIB_PATH}"
    echo "DYLD_LIBRARY_PATH=${LIB_PATH}"
    echo "PYTHONPATH=${list[0]}"
} >> "$GITHUB_ENV"
