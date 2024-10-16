### RCON

### What is RCON

RCON (Remote Console) is a protocol designed by Valve to allow administrators to control and manage game servers remotely.
It provides a way to execute commands on a server from a different location, such as a phone or a separate computer.

### Why RCON

- Convenience: Manage your server from anywhere with an internet connection.
- Flexibility: Execute commands without needing to be physically present at the server's location.
- Efficiency: Automate tasks and streamline server management.

### SSH vs RCON

**SSH**

- Offers strong encryption to protect data transmitted between the client and server
- Primarily designed for secure remote login and execution of commands on a remote machine
- Common for managing Linux/Unix systems, configuring networks, and running scripts.
- Provides a shell-like environment, allowing you to execute various commands and interact with the remote system

**RCON**

- Specifically designed for remote administration of game servers, allowing you to control and manage the server's settings and operations
- Typically less secure than SSH, as it often relies on plain text passwords
- Primarily used by game server administrators to manage game servers
- Limited set of game-specific commands

### Packets

RCON is a very simple Protocol it has a few Packets. That's how a RCON Packet looks

| Field | Description                                     |
| ----- | ----------------------------------------------- |
| ID    | For example used when auth failed or succeed    |
| Type  | That way we know what Packet this is            |
| Body  | A Message (String) (e.g. A Command, A Password) |

#### Server Bound Packets <sub><sub>(Client→Server)</sub></sub>

| Type | Packet      |
| ---- | ----------- |
| 2    | Auth        |
| 3    | ExecCommand |

#### Client Bound Packets <sub><sub>(Server→Client)</sub></sub>

| Type | Packet       |
| ---- | ------------ |
| 2    | AuthResponse |
| 0    | Output       |

### How RCON works

1. Authentication:

   - The RCON client sends an authentication packet with the desired password.
   - The server verifies the password and responds with an authentication response packet.
   - If successful, the response packet includes the same ID as the client send. If unsuccessful, the ID is -1.

2. Command Execution:

   - The authenticated client can now send command execution packets, Each packet contains the command to be executed.
   - The server processes the command and sends back an output packet containing the result or any error messages.
