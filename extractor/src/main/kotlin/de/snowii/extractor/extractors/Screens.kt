package de.snowii.extractor.extractors

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
        val screensJson = JsonObject()
        for (screen in Registries.SCREEN_HANDLER) {
            screensJson.addProperty(
                Registries.SCREEN_HANDLER.getId(screen)!!.toString(),
                Registries.SCREEN_HANDLER.getRawId(screen)
            )
        }

        return screensJson
    }
}
