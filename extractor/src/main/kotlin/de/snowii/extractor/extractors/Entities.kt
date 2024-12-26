package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.entity.LivingEntity
import net.minecraft.entity.SpawnReason
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer

class Entities : Extractor.Extractor {
    override fun fileName(): String {
        return "entities.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val entitiesJson = JsonObject()
        for (entityType in Registries.ENTITY_TYPE) {
            val entity = entityType.create(server.overworld!!, SpawnReason.NATURAL) ?: continue
            val entityJson = JsonObject()
            entityJson.addProperty("id", Registries.ENTITY_TYPE.getRawId(entityType))
            if (entity is LivingEntity) {
                entityJson.addProperty("max_health", entity.maxHealth)

            }
            entityJson.addProperty("attackable", entity.isAttackable)
            entityJson.addProperty("summonable", entityType.isSummonable)
            entityJson.addProperty("fire_immune", entityType.isFireImmune)
            val dimension = JsonArray()
            dimension.add(entityType.dimensions.width)
            dimension.add(entityType.dimensions.height)
            entityJson.add("dimension", dimension)

            entitiesJson.add(
                Registries.ENTITY_TYPE.getId(entityType).path, entityJson
            )
        }

        return entitiesJson
    }
}
