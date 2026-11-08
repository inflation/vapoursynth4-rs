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
        MSYS2_ARG_CONV_EXCL='*' pwsh -NoLogo -NoProfile -NonInteractive -Command "$(cat <<'POWERSHELL'
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Download($url, $file, $sha256) {
    Invoke-WebRequest -Uri $url -OutFile $file
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $sha256) {
        throw "Checksum mismatch for $file"
    }
}

$version = '1.4.341.0'
$sdk = "$env:RUNNER_TEMP/vulkan-sdk"
$installer = "$env:RUNNER_TEMP/vulkan-sdk.exe"
Download "https://sdk.lunarg.com/sdk/download/$version/windows/vulkansdk-windows-X64-$version.exe" $installer '5072ac63f0b00bc8c132bc0052bac0456f61983bd9d5dd50f614e190472db875'
$installerArgs = "--root `"$sdk`" --accept-licenses --default-answer --confirm-command install"
$process = Start-Process -FilePath $installer -ArgumentList $installerArgs -Wait -PassThru -WindowStyle Hidden
if ($process.ExitCode -ne 0) { throw "Vulkan SDK installer exited with $($process.ExitCode)" }
if (-not (Test-Path "$sdk/Include/vulkan/vulkan_core.h")) { throw 'Vulkan headers not found' }

$archive = "$env:RUNNER_TEMP/lavapipe.zip"
$lavapipe = "$env:RUNNER_TEMP/lavapipe"
Download 'https://github.com/jakoch/rasterizers/releases/download/20261006/lavapipe-x64-26.2.4.zip' $archive 'dd6d69c0df698bacf6633711912e8eb1cf6e70b2796d46240c7ee3708e27a438'
Expand-Archive -LiteralPath $archive -DestinationPath $lavapipe
$driver = "$lavapipe/lvp_icd.json"
@{
    file_format_version = '1.0.0'
    ICD = @{
        library_path = (Resolve-Path "$lavapipe/bin/vulkan_lvp.dll").Path
        api_version = '1.4.0'
    }
} | ConvertTo-Json -Compress | Set-Content -LiteralPath $driver
$env:VK_DRIVER_FILES = $driver
& "$sdk/Bin/vulkaninfoSDK.exe" --summary
if ($LASTEXITCODE -ne 0) { throw "vulkaninfo exited with $LASTEXITCODE" }
"VULKAN_SDK=$sdk" >> $env:GITHUB_ENV
"VK_DRIVER_FILES=$driver" >> $env:GITHUB_ENV
"$sdk/Bin" >> $env:GITHUB_PATH
POWERSHELL
)"
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
