import { defineConfig } from "vitepress";

async function load() {
  return defineConfig({
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
  });
}

export default load();
