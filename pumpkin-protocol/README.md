### Pumpkin Protocol
Contains all serverbound and clientbound packets.

**Clientbound** meaning that it's a packet being sent from the server to the client. (Server->Client)

**Serverbound** meaning that it's a packet the client sends to the server. (Client->Server)

Packets in the Pumpkin protocol are organized by functionality and state.

`server`: Contains definitions for serverbound packets.

`client`: Contains definitions for clientbound packets.

### States:
**Handshake**: Always the first packet being send from the Client. This begins also determins the next state, usally to indicate if the player thans perform a Status Request, Join the Server or wants to be transfered.

**Status**: Indicates the Client wants to see a Status response (MOTD).

**Login**: The Login sequence. Indicates the Client wants to join to the Server

**Config**: A sequence of Configuration packets beining mostly send from the Server to the Client. (Features, Resource Pack, Server Links...)

**Play**: The final state which indicate the Player is now ready to Join in also used to handle all other Gameplay packets.

### How to add a Packet ?
You can find all Minecraft Java packets at https://wiki.vg/Protocol. There you also can see in which [State](State) they are.
You also can see all the information the Packets has which we can either Write or Read depending if its Serverbound or Clientbound
#### Adding a Packet
Adding a Packet is easy. First you have to dereive serde Serialize for Clientbound Packets or Deserialize for Serverbound packets.
```rust
#[derive(Serialize)]
```
Next you have set the packet id using the packet macro
```rust
#[packet(0x1D)]
```
Now you can create the Field. Please start the Packet name with "C" if its Clientbound and with "S" if its Serverbound.
Example:
```rust
pub struct CPlayDisconnect {
    reason: TextComponent,
    more fields...
}
```
Also don't forgot to impl a new function for Clientbound Packets so we can actaully send then by putting in the values :D
Example:
```rust
impl CPlayDisconnect {
    pub fn new(reason: TextComponent) -> Self {
        Self { reason }
    }
}
```
At the End everything should come together,
Thats a Clientbound Packet
```rust
#[derive(Serialize)]
#[packet(0x1D)]
pub struct CPlayDisconnect {
    reason: TextComponent,
}

impl CPlayDisconnect {
    pub fn new(reason: TextComponent) -> Self {
        Self { reason }
    }
}
```
Thats a Serverbound packet
```rust
#[derive(Deserialize)]
#[packet(0x1D)]
pub struct CPlayDisconnect {
    reason: TextComponent,
}
``