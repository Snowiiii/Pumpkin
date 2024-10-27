package de.snowii.extractor

import com.google.gson.GsonBuilder
import com.google.gson.JsonElement
import de.snowii.extractor.extractors.*
import net.fabricmc.api.ModInitializer
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents
import net.minecraft.registry.DynamicRegistryManager
import net.minecraft.server.MinecraftServer
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import java.io.FileWriter
import java.io.IOException
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths


class Extractor : ModInitializer {
    val MOD_ID: String = "valence_extractor"
    val LOGGER: Logger = LoggerFactory.getLogger(MOD_ID)

    override fun onInitialize() {
        LOGGER.info("Starting Pumpkin Extractor")
        val extractors = arrayOf(
            Sounds(),
            Recipes(),
            Particles(),
            Packet(),
            Items(),
            Blocks(),
        )

        val outputDirectory: Path
        try {
            outputDirectory = Files.createDirectories(Paths.get("pumpkin_extractor_output"))
        } catch (e: IOException) {
            LOGGER.info("Failed to create output directory.", e)
            return
        }

        val gson = GsonBuilder().setPrettyPrinting().disableHtmlEscaping().serializeNulls().create()

        ServerLifecycleEvents.SERVER_STARTED.register(ServerLifecycleEvents.ServerStarted { server: MinecraftServer ->
            for (ext in extractors) {
                try {
                    val out = outputDirectory.resolve(ext.fileName())
                    val fileWriter = FileWriter(out.toFile(), StandardCharsets.UTF_8)
                    gson.toJson(ext.extract(server.registryManager), fileWriter)
                    fileWriter.close()
                    LOGGER.info("Wrote " + out.toAbsolutePath())
                } catch (e: java.lang.Exception) {
                    LOGGER.error(("Extractor for \"" + ext.fileName()) + "\" failed.", e)
                }
            }
        })
    }

    interface Extractor {
        fun fileName(): String

        @Throws(Exception::class)
        fun extract(registryManager: DynamicRegistryManager.Immutable): JsonElement
    }
}
