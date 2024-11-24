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
        val shapes: LinkedHashMap<Box, Int> = LinkedHashMap()

        for (block in Registries.BLOCK) {
            for (state in block.stateManager.states) {
                for (box in state.getCollisionShape(EmptyBlockView.INSTANCE, BlockPos.ORIGIN).boundingBoxes) {
                    shapes.putIfAbsent(box, shapes.size)
                }
            }
        }

        val shapesJson = JsonArray()
        for (shape in shapes.keys) {
            val shapeJson = JsonObject()
            shapeJson.addProperty("min_x", shape.minX)
            shapeJson.addProperty("min_y", shape.minY)
            shapeJson.addProperty("min_z", shape.minZ)
            shapeJson.addProperty("max_x", shape.maxX)
            shapeJson.addProperty("max_y", shape.maxY)
            shapeJson.addProperty("max_z", shape.maxZ)
            shapesJson.add(shapeJson)
        }

        return shapesJson
    }
}
