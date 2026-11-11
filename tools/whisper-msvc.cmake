# Set for every Windows build in .cargo/config.toml (CMAKE_TOOLCHAIN_FILE_x86_64_pc_windows_msvc).
# The cmake crate that whisper-rs-sys builds whisper.cpp with writes its own release flags without
# optimisation (it asks cc for opt-level 0 and leaves the rest to CMake, then overrides CMake's
# /O2), so with MSVC whisper.cpp was compiled with /Od and ran about ten times slower. This puts the
# optimisation of a normal CMake release build back.
foreach(lang C CXX)
  if(NOT CMAKE_${lang}_FLAGS_RELEASE MATCHES "/O2")
    set(CMAKE_${lang}_FLAGS_RELEASE "${CMAKE_${lang}_FLAGS_RELEASE} /O2 /Ob2 /DNDEBUG" CACHE STRING "" FORCE)
  endif()
endforeach()
