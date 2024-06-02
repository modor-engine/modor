#!/bin/bash
set -xeu

DXC_RELEASE="v1.7.2308"
DXC_FILENAME="dxc_2023_08_14.zip"
WARP_VERSION="1.0.8"
MESA_VERSION="23.3.1"

curl.exe -L --retry 5 https://github.com/microsoft/DirectXShaderCompiler/releases/download/$DXC_RELEASE/$DXC_FILENAME -o dxc.zip
7z.exe e dxc.zip -odxc bin/x64/{dxc.exe,dxcompiler.dll,dxil.dll}
cygpath --windows "$PWD/dxc" >> "$GITHUB_PATH" # We need to use cygpath to convert PWD to a windows path as we're using bash.

curl.exe -L --retry 5 https://www.nuget.org/api/v2/package/Microsoft.Direct3D.WARP/$WARP_VERSION -o warp.zip
7z.exe e warp.zip -owarp build/native/amd64/d3d10warp.dll
curl.exe -L --retry 5 https://github.com/pal1000/mesa-dist-win/releases/download/$MESA_VERSION/mesa3d-$MESA_VERSION-release-msvc.7z -o mesa.7z
7z.exe e mesa.7z -omesa x64/{opengl32.dll,libgallium_wgl.dll,libglapi.dll,vulkan_lvp.dll,lvp_icd.x86_64.json}

# We need to use cygpath to convert PWD to a windows path as we're using bash.
echo "VK_DRIVER_FILES=`cygpath --windows $PWD/mesa/lvp_icd.x86_64.json`" >> "$GITHUB_ENV"
echo "GALLIUM_DRIVER=llvmpipe" >> "$GITHUB_ENV"
echo "WGPU_DX12_COMPILER=dxc" >> "$GITHUB_ENV"
