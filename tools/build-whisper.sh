#!/bin/sh
# Builds the whisper plugin with Vulkan (AMD, NVIDIA and Intel graphics cards) and puts the program
# where its plugin.json starts it: dist/cargo/release/loomasplugins-whisper.
# Needs the Vulkan SDK (VULKAN_SDK set, glslc on the path) and libclang (LIBCLANG_PATH, or LLVM in
# its usual place on Windows). On Windows the nested CMake folders of whisper.cpp pass the 260
# character limit inside dist/cargo, so the build uses a short folder ($WHISPER_TARGET_DIR,
# default C:/lw on Windows and /tmp/loomas-whisper elsewhere).
# Usage: tools/build-whisper.sh [--cpu]   (--cpu: without Vulkan, in dist/cargo like the others)
set -e
if [ "$1" = "--cpu" ]; then
  cargo build --release -p loomasplugins-whisper
  exit 0
fi
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*)
    windows=1
    : "${WHISPER_TARGET_DIR:=C:/lw}"
    : "${LIBCLANG_PATH:=C:/Program Files/LLVM/bin}"
    if [ -z "$VULKAN_SDK" ]; then
      VULKAN_SDK="$(ls -d /c/VulkanSDK/*/ 2>/dev/null | sort | tail -1)"
    fi
    ;;
  *)
    windows=0
    : "${WHISPER_TARGET_DIR:=/tmp/loomas-whisper}"
    ;;
esac
[ -n "$VULKAN_SDK" ] || { echo "the Vulkan SDK is missing: install it and set VULKAN_SDK" >&2; exit 1; }
export LIBCLANG_PATH VULKAN_SDK
export PATH="$VULKAN_SDK/Bin:$VULKAN_SDK/bin:$PATH"
CARGO_TARGET_DIR="$WHISPER_TARGET_DIR" cargo build --release -p loomasplugins-whisper --features vulkan
mkdir -p dist/cargo/release
program="loomasplugins-whisper"
[ "$windows" = 1 ] && program="$program.exe"
cp "$WHISPER_TARGET_DIR/release/$program" "dist/cargo/release/$program"
echo "built dist/cargo/release/$program with Vulkan"
