package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import de.snowii.extractor.Extractor
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer


class Sounds : Extractor.Extractor {
    override fun fileName(): String {
        return "sounds.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val soundJson = JsonArray()
        for (sound in Registries.SOUND_EVENT) {
            soundJson.add(
                Registries.SOUND_EVENT.getId(sound)!!.path,
            )
        }

        return soundJson
    }
}
