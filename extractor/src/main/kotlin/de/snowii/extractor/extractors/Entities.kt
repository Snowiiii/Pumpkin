package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import de.snowii.extractor.Extractor
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer


class Entities : Extractor.Extractor {
    override fun fileName(): String {
        return "entities.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val entitiesJson = JsonArray()
        for (entity in Registries.ENTITY_TYPE) {
            entitiesJson.add(
                Registries.ENTITY_TYPE.getId(entity)!!.path,
            )
        }

        return entitiesJson
    }
}
