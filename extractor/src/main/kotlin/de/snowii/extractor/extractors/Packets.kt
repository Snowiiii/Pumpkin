package de.snowii.extractor.extractors

import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import de.snowii.extractor.Extractor
import io.netty.buffer.ByteBuf
import net.minecraft.network.NetworkPhase
import net.minecraft.network.NetworkState
import net.minecraft.network.PacketByteBuf
import net.minecraft.network.listener.ClientPacketListener
import net.minecraft.network.listener.ServerCrashSafePacketListener
import net.minecraft.network.packet.PacketType
import net.minecraft.network.state.*
import net.minecraft.server.MinecraftServer


class Packets : Extractor.Extractor {
    override fun fileName(): String {
        return "packets.json"
    }

    override fun extract(server: MinecraftServer): JsonElement {
        val packetsJson = JsonObject()

        val clientBound = arrayOf(
            QueryStates.S2C_FACTORY,
            LoginStates.S2C_FACTORY,
            ConfigurationStates.S2C_FACTORY,
            PlayStateFactories.S2C
        )

        val serverBound = arrayOf(
            HandshakeStates.C2S_FACTORY,
            QueryStates.C2S_FACTORY,
            LoginStates.C2S_FACTORY,
            ConfigurationStates.C2S_FACTORY,
            PlayStateFactories.C2S
        )
        val serverBoundJson = serializeServerBound(serverBound)
        val clientBoundJson = serializeClientBound(clientBound)

        packetsJson.add("serverbound", serverBoundJson)
        packetsJson.add("clientbound", clientBoundJson)

        return packetsJson
    }


    private fun serializeServerBound(
        packets: Array<NetworkState.Factory<out ServerCrashSafePacketListener, out ByteBuf>>
    ): JsonObject {
        val handshakeArray = JsonArray()
        val statusArray = JsonArray()
        val loginArray = JsonArray()
        val configArray = JsonArray()
        val playArray = JsonArray()

        for (factory in packets) {
            factory.forEachPacketType { type: PacketType<*>, _: Int ->
                when (factory.phase()!!) {
                    NetworkPhase.HANDSHAKING -> handshakeArray.add(type.id().path)
                    NetworkPhase.PLAY -> playArray.add(type.id().path)
                    NetworkPhase.STATUS -> statusArray.add(type.id().path)
                    NetworkPhase.LOGIN -> loginArray.add(type.id().path)
                    NetworkPhase.CONFIGURATION -> configArray.add(type.id().path)
                }
            }
        }
        val finalJson = JsonObject()
        finalJson.add("handshake", handshakeArray)
        finalJson.add("status", statusArray)
        finalJson.add("login", loginArray)
        finalJson.add("config", configArray)
        finalJson.add("play", playArray)
        return finalJson
    }

    private fun serializeClientBound(
        packets: Array<NetworkState.Factory<out ClientPacketListener, out PacketByteBuf>>
    ): JsonObject {
        val statusArray = JsonArray()
        val loginArray = JsonArray()
        val configArray = JsonArray()
        val playArray = JsonArray()

        for (factory in packets) {
            factory.forEachPacketType { type: PacketType<*>, _: Int ->
                when (factory.phase()!!) {
                    NetworkPhase.HANDSHAKING -> error("Client bound Packet should have no handshake")
                    NetworkPhase.PLAY -> playArray.add(type.id().path)
                    NetworkPhase.STATUS -> statusArray.add(type.id().path)
                    NetworkPhase.LOGIN -> loginArray.add(type.id().path)
                    NetworkPhase.CONFIGURATION -> configArray.add(type.id().path)
                }
            }
        }
        val finalJson = JsonObject()
        finalJson.add("status", statusArray)
        finalJson.add("login", loginArray)
        finalJson.add("config", configArray)
        finalJson.add("play", playArray)
        return finalJson
    }
}
