package de.snowii.extractor.extractors

import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.mojang.serialization.JsonOps
import de.snowii.extractor.Extractor
import net.minecraft.component.ComponentMap
import net.minecraft.item.Item
import net.minecraft.registry.Registries
import net.minecraft.registry.RegistryKeys
import net.minecraft.registry.RegistryOps
import net.minecraft.server.MinecraftServer


class Items : Extractor.Extractor {
    override fun fileName(): String {
        return "items.json"
    }


    override fun extract(server: MinecraftServer): JsonElement {
        val itemsJson = JsonObject()

        if (Item.BLOCK_ITEMS.values.size == Item.BLOCK_ITEMS.values.toSet().size)
            throw Exception("some items are associated with multiple blocks and neither Pumpkin nor this extractor can handle that")

        val blockItems = Item.BLOCK_ITEMS.toList().associate { (b, i) ->
            val itemId = Registries.ITEM.getRawId(i.asItem())
            val blockId = Registries.BLOCK.getRawId(b)
            itemId to blockId
        }

        for (item in server.registryManager.getOrThrow(RegistryKeys.ITEM).streamEntries().toList()) {
            val itemJson = JsonObject()
            val realItem: Item = item.value()

            val itemId = Registries.ITEM.getRawId(realItem)
            itemJson.addProperty("id", itemId)

            blockItems[itemId]?.let { blockId -> itemJson.addProperty("block_id", blockId) }

            itemJson.add(
                "components",
                ComponentMap.CODEC.encodeStart(
                    RegistryOps.of(JsonOps.INSTANCE, server.registryManager),
                    realItem.components
                ).getOrThrow()
            )

            itemsJson.add(Registries.ITEM.getId(realItem).toString(), itemJson)
        }
        return itemsJson
    }
}
