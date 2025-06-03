import path from "path";
import {existsSync, readFileSync} from "fs";
import {loadEnv, defineConfig} from "vite";
import vue from "@vitejs/plugin-vue";
import checkerPlugin from "vite-plugin-checker";
import svgLoaderPlugin from "vite-svg-loader";
import IconsPlugin from "unplugin-icons/vite";
import packageJson from "./package.json";
import postcss from "./postcss.config.js";
import ViteGitRevisionPlugin from "./build-src/vite_git_revision_plugin";

// can't import a random file as a string :(
// importing via node_modules at least lest us sort of import it from the module rather
// than using a relative path
const lessVars = readFileSync(
  "./node_modules/@si/vue-lib/src/tailwind/less_vars.less",
  "utf-8",
);

// fixes dev server handling of periods in paths
// see https://github.com/vitejs/vite/issues/2415
const dotPathFixPlugin = () => ({
  name: "dot-path-fix-plugin",
  configureServer: (server) => {
    server.middlewares.use((req, _res, next) => {
      const reqPath = req.url.split("?", 2)[0];
      if (
        !req.url.startsWith("/@") && // virtual files provided by vite plugins
        !req.url.startsWith("/api/") && // api proxy, configured below
        !existsSync(`./public${reqPath}`) && // files served directly from public folder
        !existsSync(`.${reqPath}`) // actual files
      ) {
        req.url = "/";
      }
      next();
    });
  },
});

// see https://vitejs.dev/config/ for more info
export default (opts: { mode: string }) => {
  // load config so we can keep the dev port to run there, and potentially other things in the future
  // 3rd arg (prefix) loads all env vars instead of just VITE_APP_*
  const config = loadEnv(opts.mode, process.cwd(), "");

  // defineConfig is a no-op but provides typing info for the options
  return defineConfig({
    plugins: [
      dotPathFixPlugin(),
      vue(),
      svgLoaderPlugin(),

      // using "raw" as icon compiler (rather than `vue3`) because we need raw svgs for use in konva
      // our Icon component knows how to deal with raw SVGs
      IconsPlugin({compiler: "raw"}),

      process.env.NODE_ENV !== "production" &&
      checkerPlugin({
        vueTsc: true,
        eslint: {
          lintCommand: packageJson.scripts.lint,
          // I _think_ we only want to pop up an error on the screen for proper errors
          // otherwise we can get a lot of unused var errors when you comment something out temporarily
          dev: {logLevel: ["error"]},
        },
      }),

      ViteGitRevisionPlugin({}),
    ],
    css: {
      postcss,
      preprocessorOptions: {
        less: {additionalData: lessVars},
      },
    },
    server: {
      host: config.DEV_HOST,
      allowedHosts: true,
      port: parseInt(config.DEV_PORT),
      strictPort: true,
      fs: {
        strict: true,
      },
      proxy: {
        "/api": {
          target: config.DEV_API_PROXY_URL,
          ws: true,
        },
      },
      headers: {
        'Cross-Origin-Opener-Policy': 'same-origin',
        'Cross-Origin-Embedder-Policy': 'credentialless',
      },
    },
    optimizeDeps: {
      exclude: ['@sqlite.org/sqlite-wasm'],
    },
    preview: {
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
        {find: "util", replacement: "util-browser"},
      ],
    },
    build: {
      manifest: "manifest.json",
      rollupOptions: {
        input: {
          main: path.resolve(__dirname, "index.html"),
          worker: path.resolve(__dirname, "src/workers/webworker.ts"), // Add worker as an entry point
        },
        output: {
          entryFileNames: (chunk) => {
            if (chunk.name === "worker") {
              return "assets/webworker.js"; // Specify output path for web worker
            }
            return "assets/[name]-[hash].js";
          },
          sourcemap: "inline",
          format: "es",
          globals: {
            react: "React",
            "react-dom": "ReactDOM",
          },
        },
      },
    },
  });
};
