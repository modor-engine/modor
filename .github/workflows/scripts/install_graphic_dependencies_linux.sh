#!/bin/bash
set -xeu

VULKAN_SDK_VERSION="1.3.268"

sudo apt-get update -y -qq
wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-$VULKAN_SDK_VERSION-jammy.list https://packages.lunarg.com/vulkan/$VULKAN_SDK_VERSION/lunarg-vulkan-$VULKAN_SDK_VERSION-jammy.list
sudo apt-get update -y
sudo apt install -y vulkan-sdk
