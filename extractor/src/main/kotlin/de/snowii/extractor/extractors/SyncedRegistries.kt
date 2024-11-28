package de.snowii.extractor.extractors

import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.mojang.serialization.Codec
import com.mojang.serialization.JsonOps
import de.snowii.extractor.Extractor
import net.minecraft.registry.*
import net.minecraft.server.MinecraftServer
import java.util.stream.Stream


class SyncedRegistries : Extractor.Extractor {
    override fun fileName(): String {
        return "synced_registries.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val registries: Stream<RegistryLoader.Entry<*>> = RegistryLoader.SYNCED_REGISTRIES.stream()
        val json = JsonObject()
        registries.forEach { entry ->
            json.add(
                entry.key().value.path,
                mapJson(entry, server.registryManager, server.combinedDynamicRegistries)
            )
        }
        return json
    }

    private fun <T> mapJson(
        registryEntry: RegistryLoader.Entry<T>,
        registryManager: DynamicRegistryManager.Immutable,
        combinedRegistries: CombinedDynamicRegistries<ServerDynamicRegistryType?>
    ): JsonObject {
        val codec: Codec<T> = registryEntry.elementCodec()
        val registry: Registry<T> = registryManager.getOrThrow(registryEntry.key())
        val json = JsonObject()
        registry.streamEntries().forEach { entry ->
            json.add(
                entry.key.orElseThrow().value.path,
                codec.encodeStart(
                    combinedRegistries.combinedRegistryManager.getOps(JsonOps.INSTANCE),
                    entry.value()
                ).getOrThrow()
            )
        }
        return json
    }

}
