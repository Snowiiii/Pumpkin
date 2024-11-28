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
        val topLevelJson = JsonObject()

        val blocksJson = JsonArray()

        val shapes: LinkedHashMap<Box, Int> = LinkedHashMap()

        for (block in Registries.BLOCK) {
            val blockJson = JsonObject()
            blockJson.addProperty("id", Registries.BLOCK.getRawId(block))
            blockJson.addProperty("name", Registries.BLOCK.getId(block).path)
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

            val statesJson = JsonArray()
            for (state in block.stateManager.states) {
                val stateJson = JsonObject()
                stateJson.addProperty("id", Block.getRawIdFromState(state))
                stateJson.addProperty("air", state.isAir)
                stateJson.addProperty("luminance", state.luminance)
                stateJson.addProperty("burnable", state.isBurnable)
                if (state.isOpaque) {
                    stateJson.addProperty("opacity", state.opacity)
                }
                stateJson.addProperty("sided_transparency", state.hasSidedTransparency())
                stateJson.addProperty("replaceable", state.isReplaceable)

                if (block.defaultState == state) {
                    blockJson.addProperty("default_state_id", Block.getRawIdFromState(state))
                }

                val collisionShapeIdxsJson = JsonArray()
                for (box in state.getCollisionShape(EmptyBlockView.INSTANCE, BlockPos.ORIGIN).boundingBoxes) {
                    val idx = shapes.putIfAbsent(box, shapes.size)
                    collisionShapeIdxsJson.add(Objects.requireNonNullElseGet(idx) { shapes.size - 1 })
                }

                stateJson.add("collision_shapes", collisionShapeIdxsJson)

                for (blockEntity in Registries.BLOCK_ENTITY_TYPE) {
                    if (blockEntity.supports(state)) {
                        stateJson.addProperty("block_entity_type", Registries.BLOCK_ENTITY_TYPE.getRawId(blockEntity))
                    }
                }

                statesJson.add(stateJson)
            }
            blockJson.add("states", statesJson)

            blocksJson.add(blockJson)
        }

        val blockEntitiesJson = JsonArray()
        for (blockEntity in Registries.BLOCK_ENTITY_TYPE) {
            blockEntitiesJson.add(Registries.BLOCK_ENTITY_TYPE.getId(blockEntity)!!.path)
        }

        val shapesJson = JsonArray()
        for (shape in shapes.keys) {
            val shapeJson = JsonObject()
            val min = JsonArray()
            min.add(shape.minX)
            min.add(shape.minY)
            min.add(shape.minZ)
            val max = JsonArray()
            max.add(shape.maxX)
            max.add(shape.maxY)
            max.add(shape.maxZ)
            shapeJson.add("min", min)
            shapeJson.add("max", max)
            shapesJson.add(shapeJson)
        }

        topLevelJson.add("block_entity_types", blockEntitiesJson)
        topLevelJson.add("shapes", shapesJson)
        topLevelJson.add("blocks", blocksJson)

        return topLevelJson
    }
}
