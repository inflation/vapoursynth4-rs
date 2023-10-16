#!/usr/bin/env bash

set -euxo pipefail

if [ ! -d "vapoursynth" ]; then
    git clone --branch R64 --depth 1 https://github.com/vapoursynth/vapoursynth.git 
fi

if [ "$RUNNER_OS" == "Linux" ]; then
    sudo apt-get install -y libzimg-dev
elif [ "$RUNNER_OS" == "macOS" ]; then
    brew install zimg automake
fi

PREFIX="$PWD/target/vapoursynth"

if [ ! -d "target/vapoursynth" ]; then
    pip install Cython

    cd vapoursynth
    ./autogen.sh
    ./configure CFLAGS="-g -O0 -w" --prefix="$PREFIX"
    make -j"$(nproc)"
    make install
fi

if [ "$RUNNER_OS" == "Linux" ]; then
    sudo sh -c "echo $PREFIX/lib > /etc/ld.so.conf.d/vapoursynth.conf"
    sudo ldconfig
fi
