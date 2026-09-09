#!/usr/bin/env bash
# Install VapourSynth from the official binary wheels.
#
# Since R74 the Windows installer, the portable zip and the PyPI wheel all ship
# the same payload, so this is the same build end users get on every platform.

set -euxo pipefail

VS_VERSION="${VS_VERSION:-79}"

python -m pip install --upgrade pip
python -m pip install "vapoursynth==${VS_VERSION}"

VS_DIR="$(python -c 'import vapoursynth, os; print(os.path.dirname(vapoursynth.__file__))')"

case "$(uname -s)" in
    Linux*)
        ln -sf "$VS_DIR/libvapoursynth.so.4" "$VS_DIR/libvapoursynth.so"
        echo "LD_LIBRARY_PATH=${VS_DIR}" >> "$GITHUB_ENV"
        ;;
    Darwin*)
        ln -sf "$VS_DIR/libvapoursynth.4.dylib" "$VS_DIR/libvapoursynth.dylib"
        echo "DYLD_LIBRARY_PATH=${VS_DIR}" >> "$GITHUB_ENV"
        ;;
    *)
        # Put the DLLs where the loader will find them at test time.
        echo "$VS_DIR" >> "$GITHUB_PATH"
        ;;
esac

echo "VAPOURSYNTH_LIB_PATH=${VS_DIR}" >> "$GITHUB_ENV"
