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

        for (item in server.registryManager.getOrThrow(RegistryKeys.ITEM).streamEntries().toList()) {
            val itemJson = JsonObject()
            val realItem: Item = item.value()

            itemJson.addProperty("id", Registries.ITEM.getRawId(realItem))
            itemJson.add(
                "components",
                ComponentMap.CODEC.encodeStart(
                    RegistryOps.of(JsonOps.INSTANCE, server.registryManager),
                    realItem.components
                ).getOrThrow()
            )

            itemsJson.add(Registries.ITEM.getId(realItem).path, itemJson)
        }
        return itemsJson
    }
}
