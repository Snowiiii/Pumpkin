### Networking

Most of the Networking code in Pumpkin can be found at [Pumpkin-Protocol](https://github.com/Snowiiii/Pumpkin/tree/master/pumpkin-protocol)

Serverbound: Client→Server

Clientbound: Server→Client

### Structure

Packets in the Pumpkin protocol are organized by functionality and state.

`server`: Contains definitions for serverbound packets.

`client`: Contains definitions for clientbound packets.

### States

**Handshake**: Always the first packet being sent from the Client. This also determines the next state, usually to indicate if the player wants to perform a Status Request, join the server or wants to be transferred.

**Status**: Indicates the Client wants to see a Status response (MOTD).

**Login**: The Login sequence. Indicates the Client wants to join to the Server.

**Config**: A sequence of Configuration packets is mostly sent from the Server to the Client. (Features, Resource Pack, Server Links, etc.)

**Play**: The final state, which indicates the Player is now ready to join, is also used to handle all other Gameplay packets.

### Minecraft Protocol

You can find all Minecraft Java packets at https://wiki.vg/Protocol. There you also can see in which [State](#States) they are.
You can also can see all the information the Packets have, which we can either Write or Read depending on whether they are Serverbound or Clientbound.

### Adding a Clientbound Packet

1. Adding a Packet is easy. First, you have to derive serde Serialize for packets.

```rust
#[derive(Serialize)]
```

2. Next, you have set the packet using the packet macro, This uses the Packet ID from the enum order

```rust
#[client_packet(ClientboundPlayPackets::Disconnect as i32)]
```

3. Now you can create the Struct.

> [!IMPORTANT]
> Please start the Packet name with "C" for Clientbound.
> Also please add the State to the packet if its a Packet sent in multiple States, For example there are 3 Disconnect Packets.
>
> - CLoginDisconnect
> - CConfigDisconnect
> - CPlayDisconnect

Create fields within your packet structure to represent the data that will be sent to the client.

> [!IMPORTANT]
> Use descriptive field names and appropriate data types.

Example:

```rust
pub struct CPlayDisconnect {
    reason: TextComponent,
    more fields...
}
```

4. Also don't forgot to impl a new function for Clientbound Packets so we can actually send them by putting in the values.

Example:

```rust
impl CPlayDisconnect {
    pub fn new(reason: TextComponent) -> Self {
        Self { reason }
    }
}
```

5. In the end, everything should come together.

```rust
#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::Disconnect as i32)]
pub struct CPlayDisconnect {
    reason: TextComponent,
}

impl CPlayDisconnect {
    pub fn new(reason: TextComponent) -> Self {
        Self { reason }
    }
}
```

6. You can also Serialize the Packet manually, which can be useful if the Packet is more complex.

```diff
-#[derive(Serialize)]

+ impl ClientPacket for CPlayDisconnect {
+    fn write(&self, bytebuf: &mut crate::bytebuf::ByteBuffer) {
+       bytebuf.put_slice(&self.reason.encode());
+    }
```

7. You can now send the Packet. See [Sending Packets](#sending-packets)

### Adding a Serverbound Packet

1. Adding a Packet is easy. First, you have to derive serde Deserialize for packets.

```rust
#[derive(Deserialize)]
```

2. Now you can create the Struct.

> [!IMPORTANT]
> Please start the Packet name with "S" for Serverbound.
> Also please add the State to the packet if its a Packet sent in multiple States.

Create fields within your packet structure to represent the data that will be sent to the client.

> [!IMPORTANT]
> Use descriptive field names and appropriate data types.

Example:

```rust
pub struct SPlayerPosition {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub ground: bool,
}
```

3. In the end, everything should come together.

```rust
#[derive(Deserialize)]
pub struct SPlayerPosition {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub ground: bool,
}
```

4. You can also Deserialize the Packet manually, which can be useful if the Packet is more complex

```diff
-#[derive(Deserialize)]

+ impl ServerPacket for SPlayerPosition {
+    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
+       Ok(Self {
+           x: bytebuf.get_f64()?,
+           feet_y: bytebuf.get_f64()?,
+           z: bytebuf.get_f64()?,
+           ground: bytebuf.get_bool()?,
+       })
+    }
```

5. You can listen for the Packet. See [Receive Packets](#receiving-packets)

### Client

Pumpkin has stores Client and Players separately. Everything that is not in the Play State is a Simple Client. Here are the Differences

**Client**

- Can only be in Status/Login/Transfer/Config State
- Is not a living entity
- Has small resource consumption

**Player**

- Can only be in Play State
- Is a living entity in a world
- Has more data, Consumes more resources

#### Sending Packets

Example:

```rust
// Works only in Status State
client.send_packet(&CStatusResponse::new("{ description: "A Description"}"));
```

#### Receiving Packets

For Clients:
`src/client/mod.rs`

```diff
// Put the Packet into the right State
 fn handle_mystate_packet(
  &self,
    server: &Arc<Server>,
    packet: &mut RawPacket,
) -> Result<(), DeserializerError> {
    let bytebuf = &mut packet.bytebuf;
    if let Some(packet) = ServerboundStatusPackets::from_i32(packet.id.0) {
        match packet {
            ServerboundStatusPackets::StatusRequest => {
                self.handle_status_request(server, SStatusRequest::read(bytebuf)?)
                    .await;
            }
            ServerboundStatusPackets::MyPacket => {
+                self.handle_my_packet(MyPacket::read(bytebuf)?)
+                    .await;
            }
        };
    } else {
        _ => {
            log::error!(
                "Failed to handle packet id {} while in ... state",
                packet.id.0
            );
            Ok(())
        }
    }
}
```

For Players:
`src/entity/player.rs`

```diff
// Players only have Play State
 fn handle_play_packet(
  &self,
    server: &Arc<Server>,
    packet: &mut RawPacket,
) -> Result<(), DeserializerError> {
    let bytebuf = &mut packet.bytebuf;
    if let Some(packet) = ServerboundPlayPackets::from_i32(packet.id.0) {
        ServerboundPlayPackets::ChatMessage => {
            self.handle_chat_message(SChatMessage::read(bytebuf)?).await;
        }
       ServerboundPlayPackets::MyPacket => {
+           self.handle_mypacket(server, MyPacket::read(bytebuf)?).await;
        }
        _ => {
            log::error!(
                "Failed to handle packet id {} while in ... state",
                packet.id.0
            );
        }
        Ok(())
    }
}
```

### Compression

Minecraft Packets **can** use the ZLib compression for decoding/encoding. There is usually a threshold set when compression is applied, this most often affects Chunk Packets.

### Porting

To port to a new minecraft version, you can compare difference in Protocol on [wiki.vg](https://wiki.vg/index.php?title=Protocol&action=history)

Also change the `CURRENT_MC_PROTOCOL` in `src/lib.rs`
