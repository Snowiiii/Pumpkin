package de.snowii.extractor.extractors

import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.registry.Registries
import net.minecraft.server.MinecraftServer


class Particles : Extractor.Extractor {
    override fun fileName(): String {
        return "particles.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val particlesJson = JsonObject()
        for (particle in Registries.PARTICLE_TYPE) {
            particlesJson.addProperty(
                Registries.PARTICLE_TYPE.getId(particle)!!.toString(),
                Registries.PARTICLE_TYPE.getRawId(particle)
            )
        }

        return particlesJson
    }
}
