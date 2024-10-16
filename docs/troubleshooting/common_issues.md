### Common Issues

1.  ### Broken Chunk Lighting

    See [#93](https://github.com/Snowiiii/Pumpkin/issues/93)

    **Issue:** Broken chunk lighting on your Minecraft server.

    **Cause:** The server is currently not calculating lighting for chunks, we're working on that.

    **Temporary Fix:** Use a full-bright resource pack. This will temporarily resolve the issue by making all chunks appear brightly lit. You can find many full-bright resource packs online.

2.  ### I can place blocks inside me

    See [#49](https://github.com/Snowiiii/Pumpkin/issues/49)

    **Issue:** Players are able to place blocks in them.

    **Cause:** The server is currently not calculating hit boxes for blocks, we're working on that.

3.  The Server is unresponsive

    **Issue:** You have to wait before reconnecting or can't do basic things while chunks are loading.

    **Cause:** The server has currently blocking issues, we're working on that.

4.  ### Failed to verify username

    **Issue:** Some players reported having issues logging into the Server, including a "Failed to verify username" error.

    **Cause:** This has to do with Authentication, Usually with the prevent proxy connections setting.

    **Fix:** Disable `prevent_proxy_connections` in `features.toml`
