package de.snowii.extractor.extractors

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
        val blockEntitiesJson = JsonObject()
        for (blockEntity in Registries.BLOCK_ENTITY_TYPE) {
            val identifier = Registries.BLOCK_ENTITY_TYPE.getId(blockEntity)!!.toString()
            val id = Registries.BLOCK_ENTITY_TYPE.getRawId(blockEntity)
            blockEntitiesJson.addProperty(identifier, id)
        }
        return blockEntitiesJson
    }
}
