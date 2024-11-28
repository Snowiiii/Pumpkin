package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import de.snowii.extractor.Extractor
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer

class Screens : Extractor.Extractor {
    override fun fileName(): String {
        return "screens.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val screensJson = JsonArray()
        for (screen in Registries.SCREEN_HANDLER) {
            screensJson.add(
                Registries.SCREEN_HANDLER.getId(screen)!!.path,
            )
        }

        return screensJson
    }
}
