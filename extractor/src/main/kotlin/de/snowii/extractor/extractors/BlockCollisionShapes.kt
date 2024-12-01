package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer
import net.minecraft.util.math.BlockPos
import net.minecraft.util.math.Box
import net.minecraft.world.EmptyBlockView
import java.util.*

class BlockCollisionShapes : Extractor.Extractor {
    override fun fileName(): String {
        return "block_shapes.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val shapes = getShapes().toList().sortedBy { (_, i) -> i }

        val shapesJson = JsonArray()
        for ((box, _) in shapes) {
            val shapeJson = JsonObject()
            shapeJson.addProperty("x1", box.minX)
            shapeJson.addProperty("y1", box.minY)
            shapeJson.addProperty("z1", box.minZ)
            shapeJson.addProperty("x2", box.maxX)
            shapeJson.addProperty("y2", box.maxY)
            shapeJson.addProperty("z2", box.maxZ)
            shapesJson.add(shapeJson)
        }

        return shapesJson
    }

    companion object {
        fun getShapes(): Map<Box, Int> {
            val shapes: LinkedHashMap<Box, Int> = LinkedHashMap()

            for (block in Registries.BLOCK) {
                for (state in block.stateManager.states) {
                    for (box in state.getCollisionShape(EmptyBlockView.INSTANCE, BlockPos.ORIGIN).boundingBoxes) {
                        shapes.putIfAbsent(box, shapes.size)
                    }
                }
            }

            return shapes
        }
    }
}
