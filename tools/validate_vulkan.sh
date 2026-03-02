#!/usr/bin/env bash
# Validation helper: runs the top-level binary with Vulkan validation layers enabled
# and writes stderr (validation layer messages) to vulkan_validation.log
set -euo pipefail
cd "$(dirname "$0")/.."

# Export common Vulkan validation/debug env vars
export VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation
export VK_LOADER_DEBUG=all
# RADV-specific flags to get GPU faults on AMD (optional tuning)
export RADV_DEBUG=cs,fail_on_va=1,gpuvm=1
# Vulkan validation will print to stderr; capture to file and also tee to console
LOGFILE=vulkan_validation.log
echo "Running with validation; logs -> $LOGFILE"
# Run cargo run and capture stderr
cargo run 2> >(tee "$LOGFILE" >&2)
