import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import eslintPlugin from "vite-plugin-eslint";
import path from "path";
import svgLoaderPlugin from "vite-svg-loader";
import IconsPlugin from "unplugin-icons/vite";

import postcss from "./postcss.config.mjs";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    svgLoaderPlugin(),
    IconsPlugin({ compiler: "vue3" }),
    eslintPlugin(),
  ],
  css: {
    postcss,
  },
  server: {
    port: 8080,
    fs: {
      strict: true,
    },
    proxy: {
      "/api": {
        target: "http://127.0.0.1:5156",
        ws: true,
      },
    },
  },
  resolve: {
    alias: [
      {
        find: "@",
        replacement: path.resolve(__dirname, "src"),
      },
    ],
  },
  define: {
    "process.env": process.env,
  },
});
