### Extractor

Pumpkin maintains a FabricMC mod to generate the protocol information. This mod is written in Kotlin and uses the Gradle build toolchain to run FabricMC.

It is found under the `extractor` folder on the repository.

> [!IMPORTANT]
> Pumpkin does not distribute any copyrightable information (game assets, programs), only information necessary to interoperate between clients (protocol information).

### Generate

1. Install the [Gradle](https://gradle.org/install/) build tool.

2. Clone the git [repository](https://github.com/Snowiiii/Pumpkin) if you have not already.

This will fail but will generate the `./extractor/run` folder:
```
cd extractor    
gradle runServer
```

> [!NOTE]
> All dependencies such as Minecraft server and FabricMC will be downloaded when running this command.


3. Set `eula=false` in `extractor/run/eula.txt` to `eula=true` and then run this again:

```
gradle runServer
```

4. Wait until the last extractor completes to cancel the server

5. The generated files will be under `./run/pumpkin_extractor_output`

### Add an Extractor

1. Copy the `Particles.kt` file
2. Rename the file to your new registry
3. Modify the file to add the required information according the internal structure of the Minecraft `Registry`
4. Add the new filename to the `extractors` variable in the `Extractor.kt` file
5. Run the steps from the [Generate](#generate) section to generate the new output
6. Copy the new data to the assets folder and make a new PR with the changes

### Useful Links

- [Kotlin](https://kotlinlang.org/) - Programming language which the extractor is written in 
- [FabricMC](https://fabricmc.net/) - FabricMC is a popular mod loader for Minecraft
