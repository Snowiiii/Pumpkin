### Common Issues

1. ### Broken Chunk Lighting

   See [#93](https://github.com/Snowiiii/Pumpkin/issues/93)

   **Issue:** Broken chunk lighting in your Minecraft server.

   **Cause:** The server is currently not calculating lighting for chunks, we working on that.

   **Temporary Fix:** Use a full-bright resource pack. This will temporarily resolve the issue by making all chunks appear brightly lit. You can find many full-bright resource packs online.

2. ### I can place blocks inside me

   See [#49](https://github.com/Snowiiii/Pumpkin/issues/49)

   **Issue:** Players are able to place block in them.

   **Cause:** The server is currently not calculating hitboxes for blocks, we working on that.

3. ### Server is unresponsive

   **Issue:** You have to wait before reconnect or can't do basic things while chunks are loading.

   **Cause:** The server has currently blocking issues, we working on that.

4. ### Failed to verify username

   **Issue:** Some players reported having issues loggin into the Server, Having "Failed to verify username" error.

   **Cause:** This has to do with Authentication, Usally with the prevent proxy connections setting.

   **Fix:** Disable `prevent_proxy_connections` in `features.toml`
