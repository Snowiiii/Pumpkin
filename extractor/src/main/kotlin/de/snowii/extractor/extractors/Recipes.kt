package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.mojang.serialization.JsonOps
import de.snowii.extractor.Extractor
import net.minecraft.recipe.Recipe
import net.minecraft.registry.RegistryOps
import net.minecraft.server.MinecraftServer

class Recipes : Extractor.Extractor {
    override fun fileName(): String {
        return "recipes.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val recipesJson = JsonArray()

        for (recipeRaw in server.recipeManager.values()) {
            val recipe = recipeRaw.value
            recipesJson.add(
                Recipe.CODEC.encodeStart(
                    RegistryOps.of(JsonOps.INSTANCE, server.registryManager),
                    recipe
                ).getOrThrow()
            )
        }
        return recipesJson
    }
}
