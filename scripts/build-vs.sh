#!/usr/bin/env bash
# Build VapourSynth from source, instrumented with AddressSanitizer.
#
# Only the sanitizer job needs this: ASan can only see into a dependency that was
# itself built with it, so the wheel used everywhere else (scripts/install-vs.sh)
# is not enough here.
#
# R79 uses Meson (autotools was dropped after R70) and requires a C++20 compiler.
# zimg and libp2p are declared as Meson wraps, so they are fetched and built as
# subprojects -- and b_sanitize instruments them too.
#
# Every path is version-scoped, so bumping VS_VERSION cannot reuse a cached build
# of the previous version.

set -euxo pipefail

VS_VERSION="${VS_VERSION:-R79}"

ROOT="$PWD/vapoursynth"
SRC="$ROOT/src-$VS_VERSION"
BUILD="$ROOT/meson-$VS_VERSION"
PREFIX="$ROOT/build-$VS_VERSION"

mkdir -p "$ROOT"

if [ ! -d "$SRC" ]; then
    git clone --depth 1 --branch "$VS_VERSION" \
        https://github.com/vapoursynth/vapoursynth "$SRC"
fi

# R79 needs meson >= 1.3.0 and builds the Python module, so install rather than
# relying on whatever the runner happens to ship.
python -m pip install --upgrade meson ninja cython

# Rust's -Z sanitizer=address links LLVM's ASan runtime. GCC's libasan is a
# different, incompatible one -- mixing them aborts every test with "Your
# application is linked against incompatible ASan runtimes".
export CC="${CC:-clang}"
export CXX="${CXX:-clang++}"
if ! command -v "$CC" >/dev/null 2>&1; then
    echo "clang is required so VapourSynth uses the same ASan runtime as Rust" >&2
    exit 1
fi

if [ ! -e "$PREFIX/.installed" ]; then
    rm -rf "$BUILD"
    # debugoptimized (-O2 -g), not debug: zimg AVX-512 kernels pass loop
    # variables to intrinsics that require a compile-time immediate, which
    # only holds once the optimiser folds them. ASan targets -O1/-O2 anyway.
    # LTO is on by default upstream and buys nothing here. b_lundef must be
    # off: it adds -Wl,--no-undefined, but an ASan shared library leaves the
    # runtime symbols to be resolved from the executable at load time.
    CFLAGS="${CFLAGS:-} -fno-omit-frame-pointer" \
    CXXFLAGS="${CXXFLAGS:-} -fno-omit-frame-pointer" \
    meson setup "$BUILD" "$SRC" \
        --prefix="$PREFIX" \
        --buildtype=debugoptimized \
        -Db_sanitize=address \
        -Db_lundef=false \
        -Db_lto=false \
        --wrap-mode=forcefallback
    meson install -C "$BUILD"
    touch "$PREFIX/.installed"
fi

# Meson installs the libraries next to the Python module, exactly like the wheel:
# <prefix>/lib/pythonX.Y/site-packages/vapoursynth/
VS_DIR="$(dirname "$(find "$PREFIX" -name 'libvapoursynth.so*' -print -quit)")"
SITE_PACKAGES="$(dirname "$VS_DIR")"

# Same development symlinks the wheel needs: only runtime sonames are installed,
# and the script library is named after the wheel rather than the autotools target.
ln -sf "$VS_DIR"/libvapoursynth.so.* "$VS_DIR/libvapoursynth.so"
ln -sf "$VS_DIR/libvsscript.so" "$VS_DIR/libvapoursynth-script.so"

{
    echo "VAPOURSYNTH_LIB_PATH=${VS_DIR}"
    echo "LD_LIBRARY_PATH=${VS_DIR}"
    echo "DYLD_LIBRARY_PATH=${VS_DIR}"
    echo "PYTHONPATH=${SITE_PACKAGES}"
} >> "$GITHUB_ENV"
