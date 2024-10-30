package de.snowii.extractor.extractors

import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.recipe.Recipe
import net.minecraft.registry.DynamicRegistryManager
import net.minecraft.registry.RegistryKeys

class Recipes : Extractor.Extractor {
    override fun fileName(): String {
        return "recipes.json"
    }

    override fun extract(registryManager: DynamicRegistryManager.Immutable): JsonElement {
        val recipesJson = JsonArray()
        val gson = Gson()

        for (recipeRaw in registryManager.getOrThrow(RegistryKeys.RECIPE).streamEntries().toList()) {
            val recipeJson = JsonObject()

            recipeJson.addProperty("id", recipeRaw.key.orElseThrow().value.toString())
            recipeJson.addProperty("name", recipeRaw.key.orElseThrow().value.toString())
            val recipe: Recipe<*> = recipeRaw.value()
            recipeJson.addProperty("group", recipe.group)
            recipeJson.addProperty("ingredients", gson.toJson(recipe.ingredientPlacement.ingredients))
            recipeJson.addProperty("placementSlots", gson.toJson(recipe.ingredientPlacement.placementSlots))

            recipesJson.add(recipeJson)
        }
        return recipesJson
    }
}
