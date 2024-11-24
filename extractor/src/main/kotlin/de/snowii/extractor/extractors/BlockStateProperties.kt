package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.block.Block
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer
import java.util.*

/**
 * this is just an array that contains the block property values of each block state
 *
 * this could in theory be part of [BlockStates] but that's by far the largest file already
 */
class BlockStateProperties : Extractor.Extractor {
    override fun fileName(): String {
        return "block_state_properties.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val json = JsonArray()

        for (block in Registries.BLOCK) {
            for (state in block.stateManager.states) {
                val propertiesJson = JsonArray()

                for (p in state.properties) {
                    propertiesJson.add(state.get(p).toString())
                }

                json.add(propertiesJson)
            }
        }

        return json
    }
}
