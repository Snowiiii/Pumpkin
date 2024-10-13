### Authentication

### Why Authentication

Minecraft is the most Popular game out there, And is is very easy to play it without paying for it. In Fact you don't pay for the Game, You pay for an Minecraft Account.
People who don't bough the Game but play online are using [Cracked Accounts](#cracked-accounts)

#### Cracked Accounts

- Don't cost any Money
- Everyone can set their own Nickname
- Have no UUID
- Have no Skin/Cape
- Not Secure

The Problem is that everyone can name themself how they want, Allowing to Join the Server as a Staff Member for example and having extended permissions,
Cracked accounts are also often used for Botting and [Denial of Service](https://de.wikipedia.org/wiki/Denial_of_Service) Attacks.

### Cracked Server

By default the `online_mode` is enabled in the configuration, This enables Authentication disallowing [Cracked Accounts](#cracked-accounts). When you are willing to allow Cracked Accounts, you can dissable `online_mode`
in the `configuration.toml`

### How Mojang Authentication works

To ensure a player has a premium accounts:

1. A client with a premium account sends a login request to the Mojang session server.
2. **Mojang's servers** verify the client's credentials and add the player to the their Servers
3. Now our server will send a Request to the Session servers and check if the Player has joined the Session Server.
4. If the request was successfull, It will give use more information about the Player (e.g. UUID, Name, Skin/Cape...)

### Custom Authentication Server

Pumpkin does support custom Authentication servers, You can replace the Authentication URL in `features.toml`.

#### How Pumpkin Authentication Works

1. **GET Request:** Pumpkin sends a GET request to the specified authentication URL.

2. **Status Code 200:** If the authentication is successful, the server responds with a status code of 200.

3. **Parse JSON Game Profile:** Pumpkin parses the JSON game profile returned in the response.

#### Game Profile

```rust
struct GameProfile {
    id: UUID,
    name: String,
    properties: Vec<Property>,
    profile_actions: Option<Vec<ProfileAction>>, // Optional, Only present when actions are applied
}
```

##### Property

```rust
struct Property {
    name: String,
    value: String, // Base64 encoded
    signature: Option<String>, // Optional, Base64 encoded
}
```

##### Profile Action

```rust
enum ProfileAction {
    FORCED_NAME_CHANGE,
    USING_BANNED_SKIN,
}
```
