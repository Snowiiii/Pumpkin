package de.snowii.extractor.extractors

import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import net.minecraft.recipe.Recipe
import net.minecraft.server.MinecraftServer

class Recipes : Extractor.Extractor {
    override fun fileName(): String {
        return "recipes.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val recipesJson = JsonArray()
        val gson = Gson()

        for (recipeRaw in server.recipeManager.values()) {
            val recipeJson = JsonObject()

            recipeJson.addProperty("name", recipeRaw.id.value.toString())
            val recipe: Recipe<*> = recipeRaw.value()
            recipeJson.addProperty("category", recipe.group)
            recipeJson.addProperty("group", recipe.group)
            recipeJson.addProperty("type", recipe.type.toString())
            val placementArray = JsonArray()
            for (slot in recipe.ingredientPlacement.placementSlots) {
                val placementJson = JsonObject()
                if (slot.isPresent) {
                    placementJson.addProperty("position", slot.orElseThrow().placerOutputPosition)
                }
                placementArray.add(placementJson)
            }
            recipeJson.add("placementSlots", placementArray)

            val ingredientArray = JsonArray()
            for (ingredient in recipe.ingredientPlacement.ingredients) {
                if (ingredient != null) {
                    val items = ingredient.matchingItems
                    val ingredientJson = JsonObject()
                    for (item in items) {
                        ingredientJson.addProperty("id", item.idAsString)
                    }
                    ingredientArray.add(ingredientJson)
                }
            }
            recipeJson.add("ingredients", ingredientArray)

            recipesJson.add(recipeJson)
        }
        return recipesJson
    }
}
