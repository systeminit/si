import { defineNuxtConfig } from "nuxt3";

// https://v3.nuxtjs.org/docs/directory-structure/nuxt.config
export default defineNuxtConfig({
  publicRuntimeConfig: {
    space: process.env.CTF_SPACE_ID,
    accessToken: process.env.CTF_CDA_ACCESS_TOKEN,
    preview: process.env.CTF_PREVIEW,
  },
  googleFonts: {
    families: {
      Montserrat: true,
      "Source Code Pro": true,
    },
    display: "swap",
  },
  tailwindcss: {
    config: {
      plugins: [
        require("@tailwindcss/forms"),
        require("@tailwindcss/typography"),
      ],
    },
    theme: {
      fontFamily: {
        sans: ["Montserrat", "ui-sans-serif", "system-ui"],
        serif: ["ui-serif"],
        mono: ['"Source Code Pro"', "ui-monospace"],
      },
    },
  },
  buildModules: ["@pinia/nuxt"],
  modules: ["@nuxtjs/tailwindcss"],
  title: "System Initiative",
  meta: {
    link: [{ rel: "icon", type: "image/x-icon", href: "/favicon.png" }],
  },
});
