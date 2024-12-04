package de.snowii.extractor.extractors

import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import de.snowii.extractor.Extractor
import java.io.FileReader
import net.minecraft.registry.BuiltinRegistries
import net.minecraft.registry.RegistryKey
import net.minecraft.registry.RegistryKeys
import net.minecraft.registry.entry.RegistryEntry
import net.minecraft.server.MinecraftServer
import net.minecraft.util.Identifier
import net.minecraft.util.math.ChunkPos
import net.minecraft.util.math.noise.DoublePerlinNoiseSampler
import net.minecraft.util.math.noise.InterpolatedNoiseSampler
import net.minecraft.util.math.random.Xoroshiro128PlusPlusRandom
import net.minecraft.world.gen.chunk.GenerationShapeConfig
import net.minecraft.world.gen.densityfunction.DensityFunction
import net.minecraft.world.gen.noise.NoiseParametersKeys

class DensityFunctionTest(
        val name: String,
        val seed: Long,
        val x: Int,
        val z: Int,
        val registry_key: String
) : Extractor.Extractor {

    companion object {
        public fun readTests(): List<Extractor.Extractor> {
            // The CWD is the ./run folder
            val reader = FileReader("../density_function_tests.json")
            val gson = Gson()
            val tests = gson.fromJson(reader, Array<DensityFunctionTest>::class.java)
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
        val noise_params = lookup.getOrThrow(RegistryKeys.NOISE_PARAMETERS)
        val funcs = lookup.getOrThrow(RegistryKeys.DENSITY_FUNCTION)
        val func_key =
                RegistryKey.of(
                        RegistryKeys.DENSITY_FUNCTION,
                        Identifier.ofVanilla(this.registry_key)
                )
        val func = funcs.getOrThrow(func_key).value()

        // Overworld shape config
        val shape = GenerationShapeConfig(-64, 384, 1, 2)

        val rand = Xoroshiro128PlusPlusRandom(seed)
        val splitter = rand.nextSplitter()

        val visitor =
                object : DensityFunction.DensityFunctionVisitor {
                    override fun apply(noise: DensityFunction.Noise): DensityFunction.Noise {
                        val entry: RegistryEntry<DoublePerlinNoiseSampler.NoiseParameters> =
                                noise.noiseData()
                        val key = entry.key.orElseThrow()
                        val sampler =
                                NoiseParametersKeys.createNoiseSampler(noise_params, splitter, key)
                        return DensityFunction.Noise(entry, sampler)
                    }

                    override fun apply(func: DensityFunction): DensityFunction {
                        if (func is InterpolatedNoiseSampler) {
                            val random = splitter.split(Identifier.ofVanilla("terrain"))
                            return func.copyWithRandom(random)
                        }
                        return func
                    }
                }

        val function = func.apply(visitor)
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

                    val sample = function.sample(pos)
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
