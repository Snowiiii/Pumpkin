#!/bin/bash

# This script decompiles the official Minecraft server JAR file.
# Be careful, as releasing the decompiled source code is against Minecraft's EULA.

# configure which version we want to decompile
MC_VERSION="1.21"

# remove folders if they already exist
if [ -d "DecompilerMC" ]; then rm -r DecompilerMC; fi
if [ -d "src" ]; then rm -r src; fi
if [ -d "target" ]; then rm -r target; fi

# clone DecompilerMC for automatic decompilation
# look here for more informations:
# - https://minecraft.fandom.com/wiki/Tutorials/See_Minecraft%27s_code
# - https://github.com/hube12/DecompilerMC
git clone https://github.com/hube12/DecompilerMC.git
echo "DecompilerMC cloned successfully."

# decompile the server JAR file
echo "Decompilation might take a while, please be patient."
cd DecompilerMC
python3.12 main.py -mcv $MC_VERSION -s server -na -f -rmap -rjar -dm -dj -dd -dec -q -c

# move it to the correct directory
mv src ..
cd ..

# move the custom made pom.xml
cp pom.xml src/$MC_VERSION/server

# install the dependencies
cd ../src/$MC_VERSION/server
# mvn clean install