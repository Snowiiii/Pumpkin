package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
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
            val screenJson = JsonObject()
            screenJson.addProperty("id", Registries.SCREEN_HANDLER.getRawId(screen))
            screenJson.addProperty("name", Registries.SCREEN_HANDLER.getId(screen)!!.toString())
            screensJson.add(screenJson)
        }

        return screensJson
    }
}
