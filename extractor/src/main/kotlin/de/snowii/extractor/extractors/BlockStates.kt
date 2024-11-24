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


class BlockStates : Extractor.Extractor {
    override fun fileName(): String {
        return "block_states.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val statesJson = JsonArray()

        for (block in Registries.BLOCK) {
            val blockId = Registries.BLOCK.getRawId(block)

            for (state in block.stateManager.states) {
                val stateJson = JsonObject()

                val id = Block.getRawIdFromState(state)
                stateJson.addProperty("id", id)
                stateJson.addProperty("block_id", blockId)
                stateJson.addProperty("air", state.isAir)
                stateJson.addProperty("luminance", state.luminance)
                stateJson.addProperty("burnable", state.isBurnable)
                if (state.isOpaque) {
                    stateJson.addProperty("opacity", state.opacity)
                }
                stateJson.addProperty("sided_transparency", state.hasSidedTransparency())
                stateJson.addProperty("replaceable", state.isReplaceable)

                for (blockEntity in Registries.BLOCK_ENTITY_TYPE) {
                    if (blockEntity.supports(state)) {
                        stateJson.addProperty("block_entity_type", Registries.BLOCK_ENTITY_TYPE.getRawId(blockEntity))
                    }
                }

                statesJson.add(stateJson)
            }
        }

        return statesJson
    }
}
