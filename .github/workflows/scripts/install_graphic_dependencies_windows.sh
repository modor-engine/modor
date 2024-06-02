#!/bin/bash
set -xeu

DXC_RELEASE="v1.7.2308"
DXC_FILENAME="dxc_2023_08_14.zip"
curl.exe -L --retry 5 https://github.com/microsoft/DirectXShaderCompiler/releases/download/$DXC_RELEASE/$DXC_FILENAME -o dxc.zip
7z.exe e dxc.zip -odxc bin/x64/{dxc.exe,dxcompiler.dll,dxil.dll}

# We need to use cygpath to convert PWD to a windows path as we're using bash.
cygpath --windows "$PWD/dxc" >> "$GITHUB_PATH"
