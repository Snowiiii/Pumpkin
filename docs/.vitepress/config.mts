import { defineConfig } from "vitepress";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Pumpkin",
  description:
    "Empowering everyone to host fast and efficient Minecraft servers",
  lang: "en",
  base: "/Pumpkin/",
  locales: {
    root: {
      label: 'English',
      lang: 'en',
    },
    es: {
      label: 'Español',
      lang: 'es',
      link: '/Pumpkin/es',
    }
  },
   // default /fr/ -- shows on navbar translations menu, can be external
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    search: {
      provider: "local",
    },
    nav: [
      {
        text: "Documentation",
        link: '/about/introduction'
      }
    ],
    sidebar: [
      {
        text: "About",
        items: [
          { text: "Introduction", link: "/en/about/introduction" },
          { text: "Quick Start", link: "/en/about/quick-start" },
          { text: "Benchmarks", link: "/en/about/benchmarks" },
        ],
      },
      {
        text: "Configuration",
        items: [
          { text: "Introduction", link: "/en/config/introduction" },
          { text: "Basic", link: "/en/config/basic" },
          { text: "Proxy", link: "/en/config/proxy"},
          { text: "Authentication", link: "/en/config/authentication"},
          { text: "Compression", link: "/en/config/compression"},
          { text: "Resource Pack", link: "/en/config/resource-pack"},
          { text: "Commands", link: "/en/config/commands"},
          { text: "RCON", link: "/en/config/rcon"},
          { text: "PVP", link: "/en/config/pvp"},
          { text: "Logging", link: "/en/config/logging"},
          { text: "Query", link: "/en/config/query"},
          { text: "LAN Broadcast", link: "/en/config/lan-broadcast"},
        ],
      },
      {
        text: "Developers",
        items: [
          { text: "Contributing", link: "/en/developer/contributing", },
          { text: "Introduction", link: "/en/developer/introduction" },
          { text: "Networking", link: "/en/developer/networking" },
          { text: "Authentication", link: "/en/developer/authentication" },
          { text: "RCON", link: "/en/developer/rcon" },
          { text: "World", link: "/en/developer/world"},
        ],
      },      
      {
        text: "Troubleshooting",
        items: [
          { text: "Common Issues", link: "/en/troubleshooting/common_issues.md" },
        ],
      },
    ],
    

    socialLinks: [
      { icon: "github", link: "https://github.com/Snowiiii/Pumpkin" },
      { icon: "discord", link: "https://discord.gg/RNm224ZsDq" },
    ],

    logo: "/assets/icon.png",
    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2024-present Aleksandr Medvedev",
    },
    editLink: {
      pattern: "https://github.com/Snowiiii/Pumpkin/blob/master/docs/:path",
      text: "Edit this page on GitHub",
    },
    lastUpdated: {
      text: "Updated at",
      formatOptions: {
        dateStyle: "medium",
        timeStyle: "medium",
      },
    },
    outline: "deep"
  },
  head: [["link", { rel: "icon", href: "/Pumpkin/assets/favicon.ico" }]],
});
