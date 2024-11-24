package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer

class BlockEntities : Extractor.Extractor {
    override fun fileName(): String {
        return "block_entities.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val blockEntitiesJson = JsonArray()
        for (blockEntity in Registries.BLOCK_ENTITY_TYPE) {
            val blockEntityJson = JsonObject()

            blockEntityJson.addProperty("id", Registries.BLOCK_ENTITY_TYPE.getRawId(blockEntity))
            blockEntityJson.addProperty("ident", Registries.BLOCK_ENTITY_TYPE.getId(blockEntity).toString())
            blockEntityJson.addProperty("name", Registries.BLOCK_ENTITY_TYPE.getId(blockEntity)!!.path)

            blockEntitiesJson.add(blockEntityJson)
        }
        return blockEntitiesJson
    }
}
