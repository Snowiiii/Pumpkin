import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Pumpkin",
  description:
    "Empowering everyone to host fast and efficient Minecraft servers",
  lang: "en-US",
  base: "/Pumpkin/",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    sidebar: [
      {
        text: "About",
        items: [
          { text: "Introduction", link: "/about/introduction" },
          { text: "Quick Start", link: "/about/quick-start" },
          {
            text: "Contributing",
            link: "https://github.com/Snowiiii/Pumpkin/blob/master/CONTRIBUTING.md",
          },
        ],
      },
      {
        text: "Plugins",
        items: [
          { text: "About Plugins", link: "/plugins/about" },
          {
            text: "Getting Started in Rust",
            link: "/plugins/getting-started-rs",
          },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/Snowiiii/Pumpkin" },
      { icon: "discord", link: "https://discord.gg/RNm224ZsDq" },
    ],

    logo: "/assets/icon.png",
  },
  head: [["link", { rel: "icon", href: "/assets/favicon.ico" }]],
});
