import { defineConfig } from "vitepress";
import dotenv from "dotenv";
import path from "path";

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
  outDir: path.join(__dirname, "../../dist"),
  ignoreDeadLinks: true,
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
