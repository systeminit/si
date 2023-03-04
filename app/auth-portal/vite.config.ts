/* eslint-disable import/no-extraneous-dependencies */
import path from "path";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import checkerPlugin from "vite-plugin-checker";
import packageJson from "./package.json";

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: 9000,
    strictPort: true,
  },
  plugins: [
    vue(),

    ...(process.env.NODE_ENV !== "production"
      ? [
          checkerPlugin({
            vueTsc: true,
            eslint: {
              lintCommand: packageJson.scripts.lint,
              // I _think_ we only want to pop up an error on the screen for proper errors
              // otherwise we can get a lot of unused var errors when you comment something out temporarily
              dev: { logLevel: ["error"] },
            },
          }),
        ]
      : []),
  ],
  resolve: {
    alias: [
      {
        find: "@",
        replacement: path.resolve(__dirname, "src"),
      },
    ],
  },
});
