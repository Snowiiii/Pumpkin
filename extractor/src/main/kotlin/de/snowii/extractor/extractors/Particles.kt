package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.registry.DynamicRegistryManager
import net.minecraft.registry.Registries
import net.minecraft.registry.RegistryKeys


class Particles : Extractor.Extractor {
    override fun fileName(): String {
        return "particles.json"
    }

    override fun extract(registryManager: DynamicRegistryManager.Immutable): JsonElement {
        val particlesJson = JsonArray()

        for (particle in Registries.PARTICLE_TYPE) {
            val particleJson = JsonObject()
            particleJson.addProperty("id", Registries.PARTICLE_TYPE.getRawId(particle))
            particleJson.addProperty("name", Registries.PARTICLE_TYPE.getId(particle)!!.toString())
            particlesJson.add(particleJson)
        }

        return particlesJson
    }
}
