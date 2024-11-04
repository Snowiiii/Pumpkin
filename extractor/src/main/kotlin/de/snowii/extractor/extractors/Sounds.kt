package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer


class Sounds : Extractor.Extractor {
    override fun fileName(): String {
        return "sounds.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val itemsJson = JsonArray()

        for (sound in Registries.SOUND_EVENT) {
            val itemJson = JsonObject()
            itemJson.addProperty("id", Registries.SOUND_EVENT.getRawId(sound))
            itemJson.addProperty("name", Registries.SOUND_EVENT.getId(sound)!!.toString())
            itemsJson.add(itemJson)
        }

        return itemsJson
    }
}
