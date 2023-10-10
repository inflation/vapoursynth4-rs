#!/usr/bin/env bash

set -euxo pipefail

if [ ! -d "vapoursynth" ]; then
    git clone --branch R64 --depth 1 https://github.com/vapoursynth/vapoursynth.git 
fi

sudo apt-get install -y libzimg-dev
sudo pip install Cython

if [ ! -d "target/vapoursynth" ]; then
    cd vapoursynth
    export PREFIX=$(readlink -f "$PWD/../target/vapoursynth")
    ./autogen.sh
    ./configure CFLAGS="-g -O0 -w" --prefix="$PREFIX"
    make -j"$(nproc)"
    sudo make install
fi

sudo sh -c "echo $PREFIX/lib > /etc/ld.so.conf.d/vapoursynth.conf"
sudo ldconfig
