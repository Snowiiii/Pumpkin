package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import io.netty.buffer.ByteBuf
import net.minecraft.network.NetworkState
import net.minecraft.network.listener.PacketListener
import net.minecraft.network.packet.PacketType
import net.minecraft.network.state.*
import net.minecraft.server.MinecraftServer


class Packet : Extractor.Extractor {
    override fun fileName(): String {
        return "packets.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val packetsJson = JsonArray()

        serializeFactory(HandshakeStates.C2S_FACTORY, packetsJson)
        serializeFactory(QueryStates.C2S_FACTORY, packetsJson)
        serializeFactory(QueryStates.S2C_FACTORY, packetsJson)
        serializeFactory(LoginStates.C2S_FACTORY, packetsJson)
        serializeFactory(LoginStates.S2C_FACTORY, packetsJson)
        serializeFactory(ConfigurationStates.C2S_FACTORY, packetsJson)
        serializeFactory(ConfigurationStates.S2C_FACTORY, packetsJson)
        serializeFactory(PlayStateFactories.C2S, packetsJson)
        serializeFactory(PlayStateFactories.S2C, packetsJson)

        return packetsJson
    }

    private fun <T : PacketListener?, B : ByteBuf?> serializeFactory(
        factory: NetworkState.Factory<T, B>,
        json: JsonArray
    ) {
        factory.forEachPacketType { type: PacketType<*>, i: Int ->
            val packetJson = JsonObject()
            packetJson.addProperty("name", type.id().path)
            packetJson.addProperty("phase", factory.phase().id)
            packetJson.addProperty("side", factory.side().getName())
            packetJson.addProperty("id", i)
            json.add(packetJson)
        }
    }
}
