package de.snowii.extractor

import com.google.gson.GsonBuilder
import com.google.gson.JsonElement
import de.snowii.extractor.extractors.*
import net.fabricmc.api.ModInitializer
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents
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
    private val modID: String = "pumpkin_extractor"
    private val logger: Logger = LoggerFactory.getLogger(modID)

    override fun onInitialize() {
        logger.info("Starting Pumpkin Extractor")
        val extractors = arrayOf(
            Sounds(),
            Recipes(),
            Particles(),
            SyncedRegistries(),
            Packets(),
            Screens(),
            Items(),
            Blocks(),
        )

        val outputDirectory: Path
        try {
            outputDirectory = Files.createDirectories(Paths.get("pumpkin_extractor_output"))
        } catch (e: IOException) {
            logger.info("Failed to create output directory.", e)
            return
        }

        val gson = GsonBuilder().setPrettyPrinting().disableHtmlEscaping().serializeNulls().create()

        ServerLifecycleEvents.SERVER_STARTED.register(ServerLifecycleEvents.ServerStarted { server: MinecraftServer ->
            for (ext in extractors) {
                try {
                    val out = outputDirectory.resolve(ext.fileName())
                    val fileWriter = FileWriter(out.toFile(), StandardCharsets.UTF_8)
                    gson.toJson(ext.extract(server), fileWriter)
                    fileWriter.close()
                    logger.info("Wrote " + out.toAbsolutePath())
                } catch (e: java.lang.Exception) {
                    logger.error(("Extractor for \"" + ext.fileName()) + "\" failed.", e)
                }
            }
        })
    }

    interface Extractor {
        fun fileName(): String

        @Throws(Exception::class)
        fun extract(server: MinecraftServer): JsonElement
    }
}
