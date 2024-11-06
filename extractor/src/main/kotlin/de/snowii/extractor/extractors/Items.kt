package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.mojang.serialization.JsonOps
import de.snowii.extractor.Extractor
import net.minecraft.component.ComponentMap
import net.minecraft.item.Item
import net.minecraft.registry.DynamicRegistryManager
import net.minecraft.registry.RegistryKeys
import net.minecraft.registry.RegistryOps


class Items : Extractor.Extractor {
    override fun fileName(): String {
        return "items.json"
    }


    override fun extract(registryManager: DynamicRegistryManager.Immutable): JsonElement {
        val itemsJson = JsonArray()

        for (item in registryManager.getOrThrow(RegistryKeys.ITEM).streamEntries().toList()) {
            val itemJson = JsonObject()

            itemJson.addProperty("id", item.key.orElseThrow().value.toString())
            itemJson.addProperty("name", item.key.orElseThrow().value.toString())
            val realItem: Item = item.value()
            itemJson.addProperty("translation_key", realItem.translationKey)
            itemJson.addProperty("max_stack", realItem.maxCount)
            itemJson.addProperty("max_durability", realItem.defaultStack.maxDamage)
            itemJson.addProperty("break_sound", realItem.breakSound.id().toString())

            itemJson.add(
                "components",
                ComponentMap.CODEC.encodeStart(
                    RegistryOps.of(JsonOps.INSTANCE, registryManager),
                    realItem.components
                ).getOrThrow()
            )

            itemsJson.add(itemJson)
        }
        return itemsJson
    }
}
