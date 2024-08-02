#!/bin/bash

# This script decompiles the official Minecraft server JAR file.
# Be careful, as releasing the decompiled source code is against Minecraft's EULA.

# configure which version we want to decompile
MC_VERSION="1.21"

# clone DecompilerMC for automatic decompilation
# look here for more informations:
# - https://minecraft.fandom.com/wiki/Tutorials/See_Minecraft%27s_code
# - https://github.com/hube12/DecompilerMC
git clone https://github.com/hube12/DecompilerMC.git
echo "DecompilerMC cloned successfully."
echo "Decompilation might take a while, please be patient."
cd DecompilerMC
python3.12 main.py -mcv $MC_VERSION -s server -na -f -rmap -rjar -dm -dj -dd -dec -q -c
mv src ..