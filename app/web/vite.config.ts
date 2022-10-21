import path from "path";
import { readFileSync } from "fs";
import { loadEnv, defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import checkerPlugin from "vite-plugin-checker";
import svgLoaderPlugin from "vite-svg-loader";
import IconsPlugin from "unplugin-icons/vite";
import packageJson from "./package.json";
import postcss from "./postcss.config.js";
import ViteGitRevisionPlugin from "./build-src/vite_git_revision_plugin";

const lessVars = readFileSync("./src/assets/style/less_vars.less", "utf-8");

// see https://vitejs.dev/config/ for more info
export default (opts: { mode: string }) => {
  // load config so we can keep the dev port to run there, and potentially other things in the future
  // 3rd arg (prefix) loads all env vars instead of just VITE_APP_*
  const config = loadEnv(opts.mode, process.cwd(), "");

  // defineConfig is a no-op but provides typing info for the options
  return defineConfig({
    plugins: [
      vue(),
      svgLoaderPlugin(),
      // IconsPlugin({ compiler: "vue3" }),
      IconsPlugin({ compiler: "raw" }),
      checkerPlugin({
        vueTsc: true,
        eslint: {
          lintCommand: packageJson.scripts.lint,
          // I _think_ we only want to pop up an error on the screen for proper errors
          // otherwise we can get a lot of unused var errors when you comment something out temporarily
          dev: { logLevel: ["error"] },
        },
      }),
      ViteGitRevisionPlugin({}),
    ],
    css: {
      postcss,
      preprocessorOptions: {
        less: { additionalData: lessVars },
      },
    },
    server: {
      port: parseInt(config.DEV_PORT),
      fs: {
        strict: true,
      },
      proxy: {
        "/api": {
          target: config.DEV_API_PROXY_URL,
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
    define: {},
  });
};
