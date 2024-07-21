import { defineConfig } from "vitepress";
import dotenv from "dotenv";
import path from "path";

dotenv.config();

export default defineConfig({
  title: "System Initiative Docs",
  description: "The documentation for System Initiative https://systeminit.com",
  markdown: {
    theme: {
      light: "github-light",
      dark: "github-dark",
    },
  },
  lastUpdated: true,
  outDir: path.join(__dirname, "../../dist"),
  ignoreDeadLinks: false,
  cleanUrls: true,
  themeConfig: {
    logo: {
      dark: "/si-logo-symbol-dark-mode.svg", 
      light: "/si-logo-symbol.svg",
      alt: "System Initiative",
    },
    nav: [
      { text: "Home", link: "/" },
      { text: "Tutorials", link: "/tutorials/" },
      { text: "Reference Guides", link: "/reference/" },
      { text: "Changelog", link: "/changelog/" },
      { text: "Log In", link: 'https://auth.systeminit.com/login', target: '__self', },
    ],
    socialLinks: [
      {
        icon: 'github', link: "https://github.com/systeminit/si",
      },
      {
        icon: "discord", link: "https://discord.com/invite/system-init",
      },
    ],
    sidebar: [
      {
        text: "Tutorials",
        collapsed: false,
        link: "/tutorials/",
        items: [
          {
            text: "Getting Started",
            link: "/tutorials/getting-started",
          },
          {
            text: "Creating a new Asset",
            link: "/tutorials/creating-new-assets",
          }
        ]
      },
      {
        text: "Reference Guides",
        link: "/reference/",
        collapsed: false,
        items: [
          {
            text: "Vocabulary",
            link: "/reference/vocabulary",
          },
          {
            text: "TypeScript Function API",
            link: "/reference/typescript/README",
          },
        ]
      },
      {
        text: "Changelog",
        link: "/changelog/",
      }
    ],
    editLink: {
      pattern: 'https://github.com/systeminit/si/edit/main/app/docs/src/:path',
      text: 'Edit this page on GitHub',
    },
    search: {
      provider: "local",
    },
  },
  vite: {
    server: {
      host: process.env.DEV_HOST,
      port: parseInt(process.env.DEV_PORT!, 10),
    },
  },
});
