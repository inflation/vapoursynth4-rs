#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y --no-install-recommends libvulkan-dev mesa-vulkan-drivers vulkan-tools

driver=$(find /usr/share/vulkan/icd.d -name 'lvp_icd*.json' -print -quit)
test -f "$driver"
VK_DRIVER_FILES="$driver" XDG_RUNTIME_DIR="$RUNNER_TEMP" vulkaninfo --summary
echo "VK_DRIVER_FILES=$driver" >> "$GITHUB_ENV"
