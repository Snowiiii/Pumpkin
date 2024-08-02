### Pumpkin Protocol
Contains all Serverbound(Client->Server) and Clientbound(Server->Client) Packets.
The packets are sorted by their folder architecture you find:

Serverbound Packets under:
  `server`
Clientbound Packets under:
  `client`
Than they both have folders for packets for all their diffrent states:

### States:
**Handshake**: Always the first packet being send from the Client. This begins also determins the next state, usally to indicate if the player thans perform a Status Request, Join the Server or wants to be transfered.

**Status**: Indicates the Client wants to see a Status response (MOTD).

**Login**: The Login sequence. Indicates the Client wants to join to the Server

**Config**: A sequence of Configuration packets beining mostly send from the Server to the Client. (Features, Resource Pack, Server Links...)

**Play**: The final state which indicate the Player is now ready to Join in also used to handle all other Gameplay packets.

### How to add a Packet ?
You can find all Minecraft Java packets at https://wiki.vg/Protocol. There you also can see in which [State](State) they are.
You also can see all the information the Packets has which we can either Write or Read depending if its Serverbound or Clientbound
#### Adding a Serverbound Packet
Serverbound Packets do use the trait `ServerPacket` which has an packet id (use hex) and can read incoming Client packets.
https://wiki.vg/Protocol gives you the Packet structure which you can than read. Feel free to orientate on already existing Packets.
Please use a name structure like `SPacketName` for the struct, The 'S' representing its Serverbound
#### Adding a Clientbound Packet
Clientbound Packets do use the trait `ClientPacket` which has an packet id (use hex) and can write outgoining Server packets.
https://wiki.vg/Protocol gives you the Packet structure which you can than write. Feel free to orientate on already existing Packets.
Please use a name structure like `CPacketName` for the struct, The 'C' representing its Clientbound