package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.block.Block
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer
import net.minecraft.util.math.BlockPos
import net.minecraft.world.EmptyBlockView
import java.util.*

/**
 * this is just an array that contains the collision shape indices of each block state
 *
 * this could in theory be part of [BlockStates] but that's by far the largest file already
 */
class BlockStateCollisionShapes : Extractor.Extractor {
    override fun fileName(): String {
        return "block_state_collision_shapes.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val json = JsonArray()

        var idx = 0

        for (block in Registries.BLOCK) {
            for (state in block.stateManager.states) {
                val collisionShapeIdxsJson = JsonArray()
                for (box in state.getCollisionShape(EmptyBlockView.INSTANCE, BlockPos.ORIGIN).boundingBoxes) {
                    collisionShapeIdxsJson.add(idx)
                    idx++
                }
                json.add(collisionShapeIdxsJson)
            }
        }

        return json
    }
}
