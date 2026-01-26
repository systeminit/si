/* eslint-disable import/no-extraneous-dependencies */
import path from "path";
import { readFileSync } from "fs";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import checkerPlugin from "vite-plugin-checker";
import IconsPlugin from "unplugin-icons/vite";
import svgLoaderPlugin from "vite-svg-loader";
import { visualizer as VisualizerPlugin } from "rollup-plugin-visualizer";
// import MarkdownPlugin from "vite-plugin-md";
import {
  plugin as MarkdownPlugin,
  Mode as MdPluginMode,
} from "vite-plugin-markdown";
import packageJson from "./package.json";
// eslint-disable-next-line import/extensions
import postcss from "./postcss.config.js";

const lessVars = readFileSync(
  "./node_modules/@si/vue-lib/src/tailwind/less_vars.less",
  "utf-8",
);

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: 9000,
    strictPort: true,
  },
  plugins: [
    vue({}),
    MarkdownPlugin({ mode: [MdPluginMode.VUE] }), // TODO(Wendy) - we may want to replace this with a custom Markdown system to meet our needs

    svgLoaderPlugin(),
    IconsPlugin({ compiler: "raw" }),

    process.env.NODE_ENV !== "production" &&
      checkerPlugin({
        vueTsc: true,
        // NOTE: ESLint checker disabled due to incompatibility between
        // vite-plugin-checker@0.12.0 and ESLint 9
        // ESLint can still be run manually via: pnpm lint
        // eslint: {
        //   lintCommand: packageJson.scripts.lint,
        //   dev: { logLevel: ["error"] },
        // },
      }),

    // https://github.com/btd/rollup-plugin-visualizer/issues/176
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    !!process.env.RUN_BUILD_ANALYZER &&
      VisualizerPlugin({
        open: true,
        filename: "dist/stats.html",
        brotliSize: true,
      }),
  ],
  css: {
    postcss,
    preprocessorOptions: {
      less: { additionalData: lessVars },
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
  build: {
    manifest: true,
  },
});
