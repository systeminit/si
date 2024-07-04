import { defineConfig } from "vitepress";
import dotenv from "dotenv";

dotenv.config();

export default defineConfig({
  title: "System Initiative Docs",
  description: "Description goes here",
  markdown: {
    theme: {
      light: "github-light",
      dark: "github-dark",
    },
  },
  cleanUrls: true,
  themeConfig: {
    nav: [
      { text: "Home", link: "/" },
      { text: "Tutorials", link: "/tutorials/" },
      { text: "Reference Docs", link: "/reference/" },
      { text: "Changelog", link: "/changelog/" },
    ],
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
