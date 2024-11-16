package de.snowii.extractor.extractors

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
        val soundJson = JsonObject()
        for (sound in Registries.SOUND_EVENT) {
            soundJson.addProperty(
                Registries.SOUND_EVENT.getId(sound)!!.toString(),
                Registries.SOUND_EVENT.getRawId(sound)
            )
        }

        return soundJson
    }
}
