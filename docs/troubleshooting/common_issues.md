### Common Issues

1.  ### I can place blocks inside me

    See [#49](https://github.com/Snowiiii/Pumpkin/issues/49)

    **Issue:** Players are able to place blocks in them.

    **Cause:** The server is currently not calculating hit boxes for blocks, we're working on that.

2.  ### Failed to verify username

    **Issue:** Some players reported having issues logging into the Server, including a "Failed to verify username" error.

    **Cause:** This has to do with Authentication, Usually with the prevent proxy connections setting.

    **Fix:** Disable `prevent_proxy_connections` in `features.toml`
