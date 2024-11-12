package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.registry.tag.TagPacketSerializer
import net.minecraft.server.MinecraftServer


class Tags : Extractor.Extractor {
    override fun fileName(): String {
        return "tags.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val tagsJson = JsonArray()

        val tags = TagPacketSerializer.serializeTags(server.combinedDynamicRegistries)

        for (tag in tags.entries) {
            val tagGroupTagsJson = JsonObject()
            tagGroupTagsJson.addProperty("name", tag.key.value.toString())
            val tagValues =
                tag.value.toRegistryTags(server.combinedDynamicRegistries.combinedRegistryManager.getOrThrow(tag.key))
            for (value in tagValues.tags) {
                val tagGroupTagsJsonArray = JsonArray()
                for (tagVal in value.value) {
                    tagGroupTagsJsonArray.add(tagVal.key.orElseThrow().value.toString())
                }
                tagGroupTagsJson.add(value.key.id.toString(), tagGroupTagsJsonArray)
            }
            tagsJson.add(tagGroupTagsJson)
        }

        return tagsJson
    }


}
