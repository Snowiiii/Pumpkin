### Pumpkin World
Contains everything World related for example

- Loading Chunks (Anvil Format)
- Generating Chunks
- Loading Blocks/Items

### Porting
When updating your Minecraft server to a newer version, you typically need to replace the files in the assets directory to ensure compatibility with the new version's resources.
Thankfully, vanilla Minecraft provides a way to extract these updated assets directly from the server JAR file itself.

1. Download the latest Minecraft server JAR file for the version you want to upgrade to.
2. Run `java -DbundlerMainClass=net.minecraft.data.Main -jar <minecraft_server>.jar --reports`.
3. This command will create a new folder named `reports` in the same directory as the server JAR. This folder contains the updated "assets" directory for the new version.
4. Copy the assets folder from the reports folder and replace the existing assets directory within your server directory.

For details see https://wiki.vg/Data_Generators
