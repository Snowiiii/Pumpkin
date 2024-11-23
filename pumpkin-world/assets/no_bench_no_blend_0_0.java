package com.example.mixin;

import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.registry.BuiltinRegistries;
import net.minecraft.registry.RegistryKeys;
import net.minecraft.registry.RegistryWrapper;
import net.minecraft.registry.RegistryWrapper.WrapperLookup;
import net.minecraft.registry.entry.RegistryEntry.Reference;
import net.minecraft.server.MinecraftServer;
import net.minecraft.util.math.noise.DoublePerlinNoiseSampler.NoiseParameters;
import net.minecraft.world.gen.chunk.AquiferSampler;
import net.minecraft.world.gen.chunk.Blender;
import net.minecraft.world.gen.chunk.ChunkGeneratorSettings;
import net.minecraft.world.gen.chunk.ChunkNoiseSampler;
import net.minecraft.world.gen.chunk.GenerationShapeConfig;
import net.minecraft.world.gen.densityfunction.DensityFunctionTypes;
import net.minecraft.world.gen.noise.NoiseConfig;

import java.lang.reflect.Method;
import java.util.Arrays;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftServer.class)
public class ExampleMixin {
	private static AquiferSampler.FluidLevelSampler createFluidLevelSampler(ChunkGeneratorSettings settings) {
		AquiferSampler.FluidLevel fluidLevel = new AquiferSampler.FluidLevel(-54, Blocks.LAVA.getDefaultState());
		int i = settings.seaLevel();
		AquiferSampler.FluidLevel fluidLevel2 = new AquiferSampler.FluidLevel(i, settings.defaultFluid());
		return (x, y, z) -> y < Math.min(-54, i) ? fluidLevel : fluidLevel2;
	}

	private static int get_index(GenerationShapeConfig config, int x, int y, int z) {
		if (x < 0 || y < 0 || z < 0) {
			System.err.println("Bad local pos");
			System.exit(1);
		}
		return config.height() * 16 * x + 16 * y + z;
	}

	private static int[] populate_noise(ChunkNoiseSampler sampler, GenerationShapeConfig config, ChunkGeneratorSettings settings) {
		int[] result = new int[16 * 16 * config.height()];

		try {
			// Find sampleBlockState because the name is obfuscated
			Method sampleBlockState = null;
			for (Method method: ChunkNoiseSampler.class.getDeclaredMethods()) {
				if (method.getReturnType().equals(BlockState.class) && method.getParameterCount() == 0) {
					sampleBlockState = method;
				}
			}
			sampleBlockState.setAccessible(true);

			int i = 0;
			int j = 0;

			sampler.sampleStartDensity();

			int k = config.horizontalCellBlockCount();
			int l = config.verticalCellBlockCount();

			int m = 16 / k;
			int n = 16 / k;

			int cellHeight = config.height() / l;
			int minimumCellY = config.minimumY() / l;

			for (int o = 0; o < m; o++) {
				sampler.sampleEndDensity(o);

				for (int p = 0; p < n; p++) {
					for (int r = cellHeight - 1; r >= 0; r--) {
						sampler.onSampledCellCorners(r, p);

						for (int s = l - 1; s >= 0; s--) {
							int t = (minimumCellY + r) * l + s;
							double d = (double)s / (double)l;
							sampler.interpolateY(t, d);

							for (int w = 0; w < k; w++) {
								int x = i + o * k + w;
								int y = x & 15;
								double e = (double)w / (double)k;
								sampler.interpolateX(x, e);

								for (int z = 0; z < k; z++) {
									int aa = j + p * k + z;
									int ab = aa & 15;
									double f = (double)z / (double)k;
									sampler.interpolateZ(aa, f);
									BlockState blockState = (BlockState) sampleBlockState.invoke(sampler);
									if (blockState == null) {
										blockState = settings.defaultBlock();
									}
									int index = get_index(config, y, t - config.minimumY(), ab);
									result[index] = Block.getRawIdFromState(blockState);
								}
							}
						}
					}
				}

				sampler.swapBuffers();
			}

			sampler.stopInterpolation();

			return result;
		} catch (Exception e) {
			System.out.println(e);
			System.exit(1);
		}
		return null;
	}

	@Inject(at = @At("HEAD"), method = "loadWorld")
	private void init(CallbackInfo info) {
		WrapperLookup lookup = BuiltinRegistries.createWrapperLookup();
		RegistryWrapper<ChunkGeneratorSettings> wrapper = lookup.getOrThrow(RegistryKeys.CHUNK_GENERATOR_SETTINGS);
		RegistryWrapper<NoiseParameters> noise_params = lookup.getOrThrow(RegistryKeys.NOISE_PARAMETERS);

		Reference<ChunkGeneratorSettings> ref = wrapper.getOrThrow(ChunkGeneratorSettings.OVERWORLD);
		ChunkGeneratorSettings settings = ref.value();
		NoiseConfig config = NoiseConfig.create(settings, noise_params, 0);

		GenerationShapeConfig shape = GenerationShapeConfig.create(-64, 384, 1, 2);
		ChunkNoiseSampler test_sampler = new ChunkNoiseSampler(16 / shape.horizontalCellBlockCount(), config, 0, 0, shape,
		new DensityFunctionTypes.Beardifying() {
			public double maxValue() {
				return 0.0;
			}

			public double minValue() {
				return 0.0;
			}

			public double sample(NoisePos pos) {
				return 0.0;
			}

			public void fill(double[] densities, EachApplier applier) {
				Arrays.fill(densities, 0.0);
			}
		}, settings, createFluidLevelSampler(settings), Blender.getNoBlending());

		int[] list = populate_noise(test_sampler, shape, settings);
		System.err.print("[");
		for (int i: list) {
			System.err.print(i + ",");
		}
		System.err.println("]");
	}
}
