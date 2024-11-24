package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.block.Block
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer
import net.minecraft.util.math.BlockPos
import net.minecraft.util.math.Box
import net.minecraft.world.EmptyBlockView
import java.util.*


class Blocks : Extractor.Extractor {
    override fun fileName(): String {
        return "blocks.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val blocksJson = JsonArray()

        for (block in Registries.BLOCK) {
            val blockJson = JsonObject()
            blockJson.addProperty("id", Registries.BLOCK.getRawId(block))
            blockJson.addProperty("name", Registries.BLOCK.getId(block).toString())
            blockJson.addProperty("translation_key", block.translationKey)
            blockJson.addProperty("hardness", block.hardness)
            blockJson.addProperty("item_id", Registries.ITEM.getRawId(block.asItem()))

            val propsJson = JsonArray()
            for (prop in block.stateManager.properties) {
                val propJson = JsonObject()

                propJson.addProperty("name", prop.name)

                val valuesJson = JsonArray()
                for (value in prop.values) {
                    valuesJson.add(value.toString().lowercase())
                }
                propJson.add("values", valuesJson)

                propsJson.add(propJson)
            }
            blockJson.add("properties", propsJson)
            blockJson.addProperty("default_state_id", Block.getRawIdFromState(block.defaultState))

            val minStateId = block.stateManager.states.minOf { Block.getRawIdFromState(it) }
            val maxStateId = block.stateManager.states.maxOf { Block.getRawIdFromState(it) }
            blockJson.addProperty("first_state_id", minStateId)
            blockJson.addProperty("last_state_id", maxStateId)

            blocksJson.add(blockJson)
        }

        return blocksJson
    }
}
