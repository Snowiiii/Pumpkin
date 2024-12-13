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
        val tagsArray = ArrayList<JsonElement>()

        val tags = TagPacketSerializer.serializeTags(server.combinedDynamicRegistries)

        for (tag in tags.entries) {
            val tagGroupTagsJson = JsonObject()
            tagGroupTagsJson.addProperty("name", tag.key.value.path)
            val tagValues =
                tag.value.toRegistryTags(server.combinedDynamicRegistries.combinedRegistryManager.getOrThrow(tag.key))
            for (value in tagValues.tags) {
                val tagGroupTagsArray = ArrayList<String>()
                for (tagVal in value.value) {
                    tagGroupTagsArray.add(tagVal.key.orElseThrow().value.path)
                }
                val tagGroupTagsJsonArray = JsonArray()
                for (path in tagGroupTagsArray.sorted()) {
                    tagGroupTagsArray.add(path)
                }
                tagGroupTagsJson.add(value.key.id.path, tagGroupTagsJsonArray)
            }
            tagsArray.add(tagGroupTagsJson)
        }

        val tagsJson = JsonArray()

        for (el in tagsArray.sortedBy { it.asJsonObject.get("name").asString }) {
            tagsJson.add(el)
        }

        return tagsJson
    }


}
