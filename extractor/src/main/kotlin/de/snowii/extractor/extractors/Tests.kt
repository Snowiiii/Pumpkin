
package de.snowii.extractor.extractors

import com.google.gson.Gson
import com.google.gson.GsonBuilder
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import de.snowii.extractor.Extractor
import net.minecraft.block.Block
import net.minecraft.block.BlockState
import net.minecraft.block.Blocks
import net.minecraft.registry.BuiltinRegistries
import net.minecraft.registry.RegistryKeys
import net.minecraft.registry.RegistryWrapper
import net.minecraft.registry.RegistryWrapper.WrapperLookup
import net.minecraft.registry.entry.RegistryEntry.Reference
import net.minecraft.server.MinecraftServer
import net.minecraft.util.math.noise.DoublePerlinNoiseSampler.NoiseParameters
import net.minecraft.util.math.ChunkPos
import net.minecraft.world.gen.chunk.AquiferSampler
import net.minecraft.world.gen.chunk.Blender
import net.minecraft.world.gen.chunk.ChunkGeneratorSettings
import net.minecraft.world.gen.chunk.ChunkNoiseSampler
import net.minecraft.world.gen.chunk.GenerationShapeConfig
import net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos;
import net.minecraft.world.gen.densityfunction.DensityFunction.EachApplier;
import net.minecraft.world.gen.densityfunction.DensityFunctionTypes
import net.minecraft.world.gen.noise.NoiseConfig

import java.lang.reflect.Method
import java.util.Arrays
import kotlin.reflect.full.createType
import kotlin.reflect.full.declaredFunctions
import kotlin.reflect.jvm.javaMethod
import kotlin.reflect.KFunction

class Tests : Extractor.Extractor {
    override fun fileName(): String = "chunk.json"

	private fun createFluidLevelSampler(settings: ChunkGeneratorSettings): AquiferSampler.FluidLevelSampler {
		val fluidLevel = AquiferSampler.FluidLevel(-54, Blocks.LAVA.getDefaultState());
		val i = settings.seaLevel();
		val fluidLevel2 = AquiferSampler.FluidLevel(i, settings.defaultFluid());
		return AquiferSampler.FluidLevelSampler {_, y, _ -> if (y < Math.min(-54, i)) fluidLevel else fluidLevel2};
	}

	private fun get_index(config: GenerationShapeConfig, x: Int, y: Int, z: Int): Int {
		if (x < 0 || y < 0 || z < 0) {
			System.err.println("Bad local pos");
			System.exit(1);
		}
		return config.height() * 16 * x + 16 * y + z
	}

	// This is basically just what NoiseChunkGenerator is doing
	private fun populate_noise(start_x: Int, start_z: Int, sampler: ChunkNoiseSampler, config: GenerationShapeConfig, settings: ChunkGeneratorSettings): IntArray? {
		val result = IntArray(16 * 16 * config.height())

		for (method: KFunction<*> in sampler::class.declaredFunctions) {
			if (method.name.equals("sampleBlockState")) {
				sampler.sampleStartDensity()
				val k = config.horizontalCellBlockCount()
				val l = config.verticalCellBlockCount()

				val m = 16 / k
				val n = 16 / k

				val cellHeight = config.height() / l
				val minimumCellY = config.minimumY() / l

				for (o in 0..<m) {
					sampler.sampleEndDensity(o)
					for (p in 0..<n) {
						for (r in (0..<cellHeight).reversed()) {
							sampler.onSampledCellCorners(r, p)
							for (s in (0..<l).reversed()) {
								val t = (minimumCellY + r) * l + s
								val d = s.toDouble() / l.toDouble()
								sampler.interpolateY(t, d)
								for (w in 0..<k) {
									val x = start_x + o * k + w
									val y = x and 15
									val e = w.toDouble() / k.toDouble()
									sampler.interpolateX(x, e)
									for (z in 0..<k) {
										val aa = start_z + p * k + z
										val ab = aa and 15
										val f = z.toDouble() / k.toDouble()
										sampler.interpolateZ(aa, f)
										var blockstate = method.call(sampler) as BlockState?
										if (blockstate == null) {
											blockstate = settings.defaultBlock()
										}
										val index = this.get_index(config, y, t - config.minimumY(), ab)
										result[index] = Block.getRawIdFromState(blockstate)
									}
								}
							}
						}
					}
					sampler.swapBuffers()
				}
				sampler.stopInterpolation()
				return result
			}
		}
		System.err.println("No valid method found for block state sampler!");
		return null;
	}

    // Dumps a chunk to an array of block state ids
    override fun extract(server: MinecraftServer): JsonElement {
        val topLevelJson = JsonArray()
		val seed = 0L
		val chunk_pos = ChunkPos(7, 4)

		val lookup = BuiltinRegistries.createWrapperLookup()
		val wrapper = lookup.getOrThrow(RegistryKeys.CHUNK_GENERATOR_SETTINGS)
		val noise_params = lookup.getOrThrow(RegistryKeys.NOISE_PARAMETERS)

		val ref = wrapper.getOrThrow(ChunkGeneratorSettings.OVERWORLD)
		val settings = ref.value()
		val config = NoiseConfig.create(settings, noise_params, seed)

		// Overworld shape config
		val shape = GenerationShapeConfig(-64, 384, 1, 2)
		val test_sampler = ChunkNoiseSampler(16 / shape.horizontalCellBlockCount(), config, chunk_pos.startX, chunk_pos.startZ,
		shape, object: DensityFunctionTypes.Beardifying{
			override fun maxValue(): Double = 0.0
			override fun minValue(): Double = 0.0
			override fun sample(pos: NoisePos): Double = 0.0
			override fun fill(densities: DoubleArray, applier: EachApplier) {densities.fill(0.0)}
		}, settings, createFluidLevelSampler(settings), Blender.getNoBlending())

		val data = populate_noise(chunk_pos.startX, chunk_pos.startZ, test_sampler, shape, settings)
		data?.forEach { state ->
            topLevelJson.add(state)
        }

        return topLevelJson
    }
}
