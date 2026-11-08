#!/usr/bin/env bash
set -euo pipefail

case "$(uname -s)" in
    Linux)
        sudo apt-get update
        sudo apt-get install -y --no-install-recommends libvulkan-dev mesa-vulkan-drivers vulkan-tools
        driver=$(find /usr/share/vulkan/icd.d -name 'lvp_icd*.json' -print -quit)
        ;;
    Darwin)
        brew install vulkan-headers vulkan-loader vulkan-tools
        cd "$RUNNER_TEMP"
        curl -fsSLO https://github.com/rerun-io/lavapipe-build/releases/download/v0.4.0/lavapipe-macos-arm64.tar.gz
        echo "50c14d456e2212736d8a3cf2f2629da4e027b9fb50863604211cc6de048fcb40  lavapipe-macos-arm64.tar.gz" | shasum -a 256 -c -
        tar -xzf lavapipe-macos-arm64.tar.gz
        driver="$RUNNER_TEMP/lavapipe-macos-arm64/lvp_icd.aarch64.json"
        echo "DYLD_LIBRARY_PATH=$(brew --prefix vulkan-loader)/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" >> "$GITHUB_ENV"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        cd "$RUNNER_TEMP"
        version=1.4.341.0
        sdk="$PWD/vulkan-sdk"
        sdk_windows=$(cygpath -w "$sdk")
        curl -fsSL "https://sdk.lunarg.com/sdk/download/$version/windows/vulkansdk-windows-X64-$version.exe" -o vulkan-sdk.exe
        echo "5072ac63f0b00bc8c132bc0052bac0456f61983bd9d5dd50f614e190472db875  vulkan-sdk.exe" | sha256sum -c -
        MSYS2_ARG_CONV_EXCL='*' ./vulkan-sdk.exe --root "$sdk_windows" --accept-licenses --default-answer --confirm-command install
        test -f "$sdk/Include/vulkan/vulkan_core.h"

        curl -fsSL https://github.com/jakoch/rasterizers/releases/download/20261006/lavapipe-x64-26.2.4.zip -o lavapipe.zip
        echo "dd6d69c0df698bacf6633711912e8eb1cf6e70b2796d46240c7ee3708e27a438  lavapipe.zip" | sha256sum -c -
        unzip -q lavapipe.zip -d lavapipe
        test -f lavapipe/bin/vulkan_lvp.dll
        library=$(cygpath -m "$PWD/lavapipe/bin/vulkan_lvp.dll")
        printf '{"file_format_version":"1.0.0","ICD":{"library_path":"%s","api_version":"1.4.0"}}\n' "$library" > lavapipe/lvp_icd.json
        driver=$(cygpath -w "$PWD/lavapipe/lvp_icd.json")

        # The Vulkan loader ignores VK_DRIVER_FILES in elevated processes.
        MSYS2_ARG_CONV_EXCL='*' reg.exe add 'HKLM\SOFTWARE\Khronos\Vulkan\Drivers' /v "$driver" /t REG_DWORD /d 0 /f
        VK_DRIVER_FILES="$driver" "$sdk/Bin/vulkaninfoSDK.exe" --summary
        {
            echo "VULKAN_SDK=$sdk_windows"
            echo "VK_DRIVER_FILES=$driver"
        } >> "$GITHUB_ENV"
        cygpath -w "$sdk/Bin" >> "$GITHUB_PATH"
        exit
        ;;
    *)
        echo "Unsupported platform: $(uname -s)" >&2
        exit 1
        ;;
esac

test -f "$driver"
VK_DRIVER_FILES="$driver" XDG_RUNTIME_DIR="$RUNNER_TEMP" vulkaninfo --summary
echo "VK_DRIVER_FILES=$driver" >> "$GITHUB_ENV"
