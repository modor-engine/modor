#!/bin/bash
set -xeu

MESA_VERSION="23.3.1"

curl.exe -L --retry 5 https://github.com/pal1000/mesa-dist-win/releases/download/$MESA_VERSION/mesa3d-$MESA_VERSION-release-msvc.7z -o mesa.7z
7z.exe e mesa.7z -omesa x64/{opengl32.dll,libgallium_wgl.dll,libglapi.dll,vulkan_lvp.dll,lvp_icd.x86_64.json}

# We need to use cygpath to convert PWD to a windows path as we're using bash.
echo "VK_DRIVER_FILES=`cygpath --windows $PWD/mesa/lvp_icd.x86_64.json`" >> "$GITHUB_ENV"
echo "GALLIUM_DRIVER=llvmpipe" >> "$GITHUB_ENV"
