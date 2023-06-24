sudo apt-get update -y -qq
wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-jammy.list https://packages.lunarg.com/vulkan/lunarg-vulkan-jammy.list
sudo apt-get update -y
sudo apt install -y libegl1-mesa libgl1-mesa-dri libxcb-xfixes0-dev mesa-vulkan-drivers vulkan-sdk
