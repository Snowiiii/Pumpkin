package de.snowii.extractor

import com.google.gson.Gson
import com.google.gson.GsonBuilder
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.google.gson.reflect.TypeToken
import de.snowii.extractor.extractors.*
import java.io.FileWriter
import java.io.IOException
import java.nio.charset.StandardCharsets
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths
import java.util.TreeMap
import net.fabricmc.api.ModInitializer
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents
import net.minecraft.server.MinecraftServer
import org.slf4j.Logger
import org.slf4j.LoggerFactory

class Extractor : ModInitializer {
    private val modID: String = "pumpkin_extractor"
    private val logger: Logger = LoggerFactory.getLogger(modID)

    override fun onInitialize() {
        logger.info("Starting Pumpkin Extractor")
        var extractors =
                listOf(
                        Sounds(),
                        Recipes(),
                        Particles(),
                        SyncedRegistries(),
                        Packets(),
                        Screens(),
                        Tags(),
                        Items(),
                        Blocks(),
                )

        val chunk_tests = ChunkTest.readTests()
        extractors += chunk_tests
        val density_function_tests = DensityFunctionTest.readTests()
        extractors += density_function_tests
        val config_function_tests = ConfigFunctionTest.readTests()
        extractors += config_function_tests

        val outputDirectory: Path
        try {
            outputDirectory = Files.createDirectories(Paths.get("pumpkin_extractor_output"))
        } catch (e: IOException) {
            logger.info("Failed to create output directory.", e)
            return
        }

        val gson = GsonBuilder().disableHtmlEscaping().create()

        ServerLifecycleEvents.SERVER_STARTING.register(
                ServerLifecycleEvents.ServerStarting { server: MinecraftServer ->
                    for (ext in extractors) {
                        try {
                            val out = outputDirectory.resolve(ext.fileName())
                            val fileWriter = FileWriter(out.toFile(), StandardCharsets.UTF_8)
                            val extracted = ext.extract(server)
                            gson.toJson(deterministic_json(gson, extracted) ?: extracted, fileWriter)
                            fileWriter.close()
                            logger.info("Wrote " + out.toAbsolutePath())
                        } catch (e: java.lang.Exception) {
                            logger.error(("Extractor for \"" + ext.fileName()) + "\" failed.", e)
                        }
                    }

                    throw java.lang.Exception(
                            "THIS EXCEPTION WAS INSERTED TO STOP USELESS WORK, DONT WORRY ABOUT IT"
                    )
                }
        )
    }

    fun deterministic_json(gson: Gson, element: JsonElement): Any? {
        if (element is JsonObject) {
            val json = gson.toJson(element)
            val map_type = object : TypeToken<TreeMap<String, JsonElement>>() {}.type
            val map = gson.fromJson<TreeMap<String, JsonElement>>(json, map_type)
            val keys = map.keys.asSequence()
            val loose_map = TreeMap<String, Any>()
            for (key in keys) {
                val value = map[key]!!
                loose_map[key] = this.deterministic_json(gson, value) ?: value
            }
            return loose_map
        } else if (element is JsonArray) {
            val checked_array = ArrayList<Any>()
            for (el in element) {
                checked_array.add(deterministic_json(gson, el) ?: el)
            }
            return checked_array
        }
        return null
    }

    interface Extractor {
        fun fileName(): String

        @Throws(Exception::class) fun extract(server: MinecraftServer): JsonElement
    }
}
