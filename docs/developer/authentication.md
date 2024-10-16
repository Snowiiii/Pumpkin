### Authentication

### Why Authentication

Minecraft is the most popular game out there and is very easy to play without paying for it. In fact, you don't pay for the game; you pay for a Minecraft account. People who don't buy the game but play it online are using [Cracked Accounts](#cracked-accounts).

#### Cracked Accounts

- Don't cost any money.
- Everyone can set their own nickname.
- Have no UUID.
- Have no skin/cape.
- Not secure.

The problem is that everyone can name themselves however they want, which allows them to join the server as a staff member, for example, and have extended permissions. Cracked accounts are also often used for botting and [Denial of Service](https://de.wikipedia.org/wiki/Denial_of_Service) attacks.

### Cracked Server

By default, `online_mode` is enabled in the configuration. This enables authentication, disabling [Cracked Accounts](#cracked-accounts). If you want to allow cracked accounts, you can disable `online_mode` in the `configuration.toml`.

### How Mojang Authentication Works

To ensure a player has a premium account:

1. A client with a premium account sends a login request to the Mojang session server.
2. **Mojang's servers** verify the client's credentials and add the player to their servers.
3. Our server sends a request to the session servers to check if the player has joined the session server.
4. If the request is successful, it will provide more information about the player (e.g., UUID, name, skin/cape...).

### Custom Authentication Server

Pumpkin does support custom authentication servers. You can replace the authentication URL in `features.toml`.

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
    profile_actions: Option<Vec<ProfileAction>>, // Optional, only present when actions are applied
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
