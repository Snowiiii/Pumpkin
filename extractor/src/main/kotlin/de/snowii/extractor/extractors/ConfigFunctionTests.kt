package de.snowii.extractor.extractors

import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import de.snowii.extractor.Extractor
import java.io.FileReader
import net.minecraft.registry.BuiltinRegistries
import net.minecraft.registry.RegistryKeys
import net.minecraft.server.MinecraftServer
import net.minecraft.util.math.ChunkPos
import net.minecraft.world.gen.chunk.ChunkGeneratorSettings
import net.minecraft.world.gen.chunk.GenerationShapeConfig
import net.minecraft.world.gen.densityfunction.DensityFunction
import net.minecraft.world.gen.noise.NoiseConfig

class ConfigFunctionTest(
        val name: String,
        val seed: Long,
        val x: Int,
        val z: Int,
        val function: String
) : Extractor.Extractor {

    companion object {
        public fun readTests(): List<Extractor.Extractor> {
            // The CWD is the ./run folder
            val reader = FileReader("../config_function_tests.json")
            val gson = Gson()
            val tests = gson.fromJson(reader, Array<ConfigFunctionTest>::class.java)
            return tests.toList()
        }
    }

    override fun fileName(): String = this.name + ".json"

    // Samples the density function over a chunk
    override fun extract(server: MinecraftServer): JsonElement {
        val topLevelJson = JsonArray()
        val seed = this.seed
        val chunk_pos = ChunkPos(this.x, this.z)

        val lookup = BuiltinRegistries.createWrapperLookup()
        val wrapper = lookup.getOrThrow(RegistryKeys.CHUNK_GENERATOR_SETTINGS)
        val noise_params = lookup.getOrThrow(RegistryKeys.NOISE_PARAMETERS)
        val ref = wrapper.getOrThrow(ChunkGeneratorSettings.OVERWORLD)
        val settings = ref.value()
        val config = NoiseConfig.create(settings, noise_params, seed)

        // Overworld shape config
        val shape = GenerationShapeConfig(-64, 384, 1, 2)

        val router = config.noiseRouter
        var function: DensityFunction? = null
        if (this.function == "final_density") {
            function = router.finalDensity()
        }

        for (x in 0..16) {
            for (y in shape.minimumY()..(shape.height() + shape.minimumY())) {
                for (z in 0..16) {
                    val xx = chunk_pos.startX + x
                    val zz = chunk_pos.startZ + z

                    val pos =
                            object : DensityFunction.NoisePos {
                                override fun blockX(): Int = xx
                                override fun blockY(): Int = y
                                override fun blockZ(): Int = zz
                            }

                    val sample = function!!.sample(pos)
                    val sample_json = JsonArray()
                    sample_json.add(xx)
                    sample_json.add(y)
                    sample_json.add(zz)
                    sample_json.add(sample)

                    topLevelJson.add(sample_json)
                }
            }
        }

        return topLevelJson
    }
}
