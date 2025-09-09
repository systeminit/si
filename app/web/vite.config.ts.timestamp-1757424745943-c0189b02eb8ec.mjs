// vite.config.ts
import path2 from "path";
import { existsSync, readFileSync } from "fs";
import { loadEnv, defineConfig } from "file:///home/john/working-folder/2-repos/si/node_modules/.pnpm/vite@5.4.19_@types+node@18.19.59_less@4.1.3_terser@5.24.0/node_modules/vite/dist/node/index.js";
import vue from "file:///home/john/working-folder/2-repos/si/node_modules/.pnpm/@vitejs+plugin-vue@5.1.4_vite@5.4.19_@types+node@18.19.59_less@4.1.3_terser@5.24.0__vue@3.5.13_typescript@5.0.4_/node_modules/@vitejs/plugin-vue/dist/index.mjs";
import checkerPlugin from "file:///home/john/working-folder/2-repos/si/node_modules/.pnpm/vite-plugin-checker@0.6.4_eslint@8.57.1_optionator@0.9.4_typescript@5.0.4_vite@5.4.19_@_ab8576112484cfdff876d4950f52440a/node_modules/vite-plugin-checker/dist/esm/main.js";
import svgLoaderPlugin from "file:///home/john/working-folder/2-repos/si/node_modules/.pnpm/vite-svg-loader@3.6.0/node_modules/vite-svg-loader/index.js";
import IconsPlugin from "file:///home/john/working-folder/2-repos/si/node_modules/.pnpm/unplugin-icons@0.17.4_@vue+compiler-sfc@3.5.13_vue-template-compiler@2.7.14/node_modules/unplugin-icons/dist/vite.mjs";

// package.json
var package_default = {
  name: "web",
  private: true,
  version: "0.0.0",
  type: "module",
  scripts: {
    clean: "rm -rf ./dist ./lib ./target && pnpm vite:clean",
    build: "vite build",
    "build:check": "vue-tsc --noEmit",
    "build:clean": "pnpm run clean && npm run build",
    "build:watch": "pnpm run clean && tsc --watch",
    "build:pedantically": "pnpm run build:check && npm run build",
    watch: "pnpm run build:watch",
    lint: "eslint src --ext .ts,.js,.cjs,.vue",
    "lint:fix": "pnpm run lint --fix",
    "lint:strict": "npm run lint --max-warnings=0",
    "lint:summary": "pnpm run lint --format summary-chart",
    fmt: "pnpm run lint:fix",
    "fmt:check": "pnpm run lint",
    check: "pnpm run lint:strict && npm run build:check",
    dev: "pnpm run start",
    start: "vite",
    serve: "vite preview --port 8080",
    test: "echo 'Sorry, no tests yet!'",
    "vite:clean": "rm -rf ./node_modules/.vite",
    "cypress:run": "cypress run",
    "cypress:open": "cypress open"
  },
  dependencies: {
    "@codemirror/autocomplete": "^6.4.2",
    "@codemirror/commands": "^6.1.2",
    "@codemirror/lang-javascript": "^6.1.2",
    "@codemirror/lang-json": "^6.0.1",
    "@codemirror/lang-yaml": "^6.1.1",
    "@codemirror/language": "^6.3.1",
    "@codemirror/legacy-modes": "^6.3.1",
    "@codemirror/lint": "^6.1.0",
    "@codemirror/search": "^6.3.0",
    "@codemirror/state": "^6.1.4",
    "@codemirror/view": "^6.7.1",
    "@fsegurai/codemirror-theme-github-dark": "^6.2.0",
    "@fsegurai/codemirror-theme-github-light": "^6.2.0",
    "@headlessui/vue": "^1.7.10",
    "@honeycombio/opentelemetry-web": "^0.3.0",
    "@lezer/highlight": "^1.1.3",
    "@opentelemetry/api": "^1.8.0",
    "@opentelemetry/auto-instrumentations-web": "^0.39.0",
    "@opentelemetry/exporter-otlp-http": "^0.26.0",
    "@opentelemetry/exporter-trace-otlp-http": "^0.57.2",
    "@opentelemetry/instrumentation": "^0.57.2",
    "@opentelemetry/instrumentation-document-load": "^0.38.0",
    "@opentelemetry/instrumentation-fetch": "^0.57.2",
    "@opentelemetry/instrumentation-long-task": "^0.38.0",
    "@opentelemetry/instrumentation-user-interaction": "^0.38.0",
    "@opentelemetry/resources": "^1.30.1",
    "@opentelemetry/sdk-trace-base": "^1.30.1",
    "@opentelemetry/sdk-trace-web": "^1.30.1",
    "@opentelemetry/semantic-conventions": "^1.30.0",
    "@replit/codemirror-vim": "^6.0.11",
    "@si/ts-lib": "workspace:*",
    "@si/vue-lib": "workspace:*",
    "@sqlite.org/sqlite-wasm": "3.50.3-build1",
    "@tanstack/vue-form": "^1.9.0",
    "@tanstack/vue-query": "^5.67.3",
    "@tanstack/vue-table": "^8.20.5",
    "@tanstack/vue-virtual": "^3.13.6",
    "@types/async": "^3.2.15",
    "@typescript/vfs": "^1.5.3",
    "@vueuse/core": "^12.0.0",
    "@vueuse/head": "^1.1.15",
    async: "^3.2.4",
    axios: "^1.8.4",
    buffer: "^6.0.3",
    clsx: "^1.2.1",
    codemirror: "^6.0.1",
    comlink: "^4.4.2",
    d3: "^7.9.0",
    "date-fns": "^2.29.2",
    elkjs: "^0.10.0",
    "fast-json-patch": "^3.1.1",
    "floating-vue": "^2.0.0-beta.20",
    fontfaceobserver: "^2.3.0",
    fzf: "^0.5.2",
    graphology: "^0.25.4",
    "graphology-layout-forceatlas2": "^0.10.1",
    "graphology-layout-noverlap": "^0.4.2",
    "is-promise": "^4.0.0",
    "javascript-time-ago": "^2.5.7",
    joi: "^17.11.0",
    "js-base64": "^3.7.5",
    "js-beautify": "^1.14.9",
    "js-confetti": "^0.11.0",
    "js-md5": "^0.8.3",
    "jwt-decode": "^3.1.2",
    konva: "^8.3.13",
    less: "^4.1.3",
    "libsodium-wrappers": "^0.7.13",
    "local-storage-fallback": "^4.1.3",
    "lodash-es": "^4.17.21",
    mitt: "^3.0.1",
    "p-queue": "^8.1.0",
    pinia: "^2.2.4",
    plur: "^5.1.0",
    "posthog-js": "^1.155.0",
    "quick-lru": "^7.0.1",
    "reconnecting-websocket": "^4.4.0",
    sigma: "3.0.0-beta.5",
    "sourcemapped-stacktrace": "^1.1.11",
    tinycolor2: "^1.4.2",
    typescript: "^5.0.4",
    ulid: "^2.3.0",
    "util-browser": "^0.0.2",
    validator: "^13.7.0",
    "vanilla-picker": "^2.12.1",
    vue: "^3.5.13",
    "vue-html-secure": "^1.0.10",
    "vue-konva": "^3.0.1",
    "vue-markdown-render": "^2.2.1",
    "vue-router": "^4.4.5",
    "vue-safe-teleport": "^0.1.2",
    "vue-toastification": "2.0.0-rc.5",
    xterm: "^5.3.0",
    "xterm-addon-fit": "^0.8.0",
    "xterm-addon-web-links": "^0.9.0",
    "y-indexeddb": "^9.0.12",
    "y-websocket": "^1.5.0",
    yjs: "^13.6.8",
    "yjs-codemirror-plugin": "workspace:*"
  },
  devDependencies: {
    "@iconify/json": "^2.2.166",
    "@si/eslint-config": "workspace:*",
    "@si/tsconfig": "workspace:*",
    "@types/codemirror": "^5.60.5",
    "@types/d3": "^7.4.3",
    "@types/fontfaceobserver": "^2.1.0",
    "@types/javascript-time-ago": "^2.0.3",
    "@types/js-beautify": "^1.14.1",
    "@types/libsodium-wrappers": "^0.7.11",
    "@types/lodash-es": "^4.17.12",
    "@types/node": "^18.19.59",
    "@types/tinycolor2": "^1.4.3",
    "@types/validator": "^13.7.2",
    "@types/wicg-file-system-access": "^2023.10.5",
    "@vitejs/plugin-vue": "^5.1.4",
    cypress: "^14.5.3",
    "cypress-vite": "^1.5.0",
    eslint: "^8.57.1",
    "graphology-types": "^0.24.7",
    "unplugin-icons": "^0.17.4",
    vite: "^5.4.19",
    "vite-plugin-checker": "^0.6.4",
    "vite-svg-loader": "^3.4.0",
    "vue-tsc": "^1.8.27"
  }
};

// postcss.config.js
import config from "file:///home/john/working-folder/2-repos/si/lib/vue-lib/src/tailwind/postcss.config.cjs";
var postcss_config_default = config;

// build-src/vite_git_revision_plugin.ts
import { execSync } from "child_process";
import path from "path";
var defaultOptions = {
  shortSha: true
};
var vite_git_revision_plugin_default = (options) => {
  options = Object.assign(defaultOptions, options || {});
  function runGitCommand(command, fallback) {
    const cmd = [
      "git",
      ...options.gitWorkTree ? [
        `--git-dir=${path.join(options.gitWorkTree, ".git")}`,
        `--work-tree=${options.gitWorkTree}`
      ] : [],
      command
    ].join(" ");
    try {
      const result = execSync(cmd).toString().replace(/[\s\r\n]+$/, "");
      return result;
    } catch (err) {
      if (fallback !== void 0) return fallback;
      throw err;
    }
  }
  const gitBranch = runGitCommand("rev-parse --abbrev-ref HEAD", "unknown");
  const gitSha = runGitCommand(
    `rev-parse ${options.shortSha ? "--short" : ""} HEAD`,
    "unknown"
  );
  return {
    name: "vite:git-revision",
    config(config2) {
      config2.define.__VITE_GIT_BRANCH__ = JSON.stringify(gitBranch);
      config2.define.__VITE_GIT_SHA__ = JSON.stringify(gitSha);
    }
  };
};

// vite.config.ts
import * as child_process from "child_process";
var __vite_injected_original_dirname = "/home/john/working-folder/2-repos/si/app/web";
var lessVars = readFileSync(
  "./node_modules/@si/vue-lib/src/tailwind/less_vars.less",
  "utf-8"
);
var dotPathFixPlugin = () => ({
  name: "dot-path-fix-plugin",
  configureServer: (server) => {
    server.middlewares.use((req, _res, next) => {
      const reqPath = req.url.split("?", 2)[0];
      if (!req.url.startsWith("/@") && // virtual files provided by vite plugins
      !req.url.startsWith("/api/") && // api proxy, configured below
      !existsSync(`./public${reqPath}`) && // files served directly from public folder
      !existsSync(`.${reqPath}`)) {
        req.url = "/";
      }
      next();
    });
  }
});
var gitHashFile = (file) => {
  const cmd = `git hash-object '${file}'`;
  return child_process.execSync(cmd).toString().trim();
};
var webWorkerPath = path2.resolve(__vite_injected_original_dirname, "src/workers/webworker.ts");
var webWorkerHash = JSON.stringify(gitHashFile(webWorkerPath));
var sharedWorkerPath = path2.resolve(__vite_injected_original_dirname, "src/workers/shared_webworker.ts");
var sharedWorkerHash = JSON.stringify(gitHashFile(sharedWorkerPath));
var headCommitHash = JSON.stringify(
  child_process.execSync("git rev-parse HEAD").toString().trim()
);
var vite_config_default = (opts) => {
  const config2 = loadEnv(opts.mode, process.cwd(), "");
  return defineConfig({
    // NOTE: these constants only update at build time, or if you restart the vite server
    define: {
      __COMMIT_HASH__: headCommitHash,
      __SHARED_WORKER_HASH__: sharedWorkerHash,
      __WEBWORKER_HASH__: webWorkerHash
    },
    plugins: [
      dotPathFixPlugin(),
      vue(),
      svgLoaderPlugin(),
      // using "raw" as icon compiler (rather than `vue3`) because we need raw svgs for use in konva
      // our Icon component knows how to deal with raw SVGs
      IconsPlugin({ compiler: "raw" }),
      process.env.NODE_ENV !== "production" && checkerPlugin({
        vueTsc: true,
        eslint: {
          lintCommand: package_default.scripts.lint,
          // I _think_ we only want to pop up an error on the screen for proper errors
          // otherwise we can get a lot of unused var errors when you comment something out temporarily
          dev: { logLevel: ["error"] }
        }
      }),
      vite_git_revision_plugin_default({})
    ],
    css: {
      postcss: postcss_config_default,
      preprocessorOptions: {
        less: { additionalData: lessVars }
      }
    },
    server: {
      hmr: false,
      host: config2.DEV_HOST,
      allowedHosts: true,
      port: parseInt(config2.DEV_PORT),
      strictPort: true,
      fs: {
        strict: true
      },
      proxy: {
        "/api": {
          target: config2.DEV_API_PROXY_URL,
          ws: true
        }
      },
      headers: {
        "Cross-Origin-Opener-Policy": "same-origin",
        "Cross-Origin-Embedder-Policy": "credentialless"
      }
    },
    optimizeDeps: {
      exclude: ["@sqlite.org/sqlite-wasm"]
    },
    preview: {
      proxy: {
        "/api": {
          target: config2.DEV_API_PROXY_URL,
          ws: true
        }
      }
    },
    resolve: {
      alias: [
        {
          find: "@",
          replacement: path2.resolve(__vite_injected_original_dirname, "src")
        },
        { find: "util", replacement: "util-browser" }
      ]
    },
    build: {
      manifest: "manifest.json",
      sourcemap: "inline",
      rollupOptions: {
        input: {
          main: path2.resolve(__vite_injected_original_dirname, "index.html"),
          worker: webWorkerPath,
          sharedWorker: sharedWorkerPath
        },
        output: {
          entryFileNames: (chunk) => {
            if (chunk.name === "worker") {
              return "assets/webworker.js";
            }
            if (chunk.name === "sharedWorker") {
              return "assets/shared_webworker.js";
            }
            return "assets/[name]-[hash].js";
          },
          format: "es",
          globals: {
            react: "React",
            "react-dom": "ReactDOM"
          }
        }
      }
    }
  });
};
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiLCAicGFja2FnZS5qc29uIiwgInBvc3Rjc3MuY29uZmlnLmpzIiwgImJ1aWxkLXNyYy92aXRlX2dpdF9yZXZpc2lvbl9wbHVnaW4udHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvaG9tZS9qb2huL3dvcmtpbmctZm9sZGVyLzItcmVwb3Mvc2kvYXBwL3dlYlwiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9maWxlbmFtZSA9IFwiL2hvbWUvam9obi93b3JraW5nLWZvbGRlci8yLXJlcG9zL3NpL2FwcC93ZWIvdml0ZS5jb25maWcudHNcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfaW1wb3J0X21ldGFfdXJsID0gXCJmaWxlOi8vL2hvbWUvam9obi93b3JraW5nLWZvbGRlci8yLXJlcG9zL3NpL2FwcC93ZWIvdml0ZS5jb25maWcudHNcIjtpbXBvcnQgcGF0aCBmcm9tIFwicGF0aFwiO1xuaW1wb3J0IHsgZXhpc3RzU3luYywgcmVhZEZpbGVTeW5jIH0gZnJvbSBcImZzXCI7XG5pbXBvcnQgeyBsb2FkRW52LCBkZWZpbmVDb25maWcgfSBmcm9tIFwidml0ZVwiO1xuaW1wb3J0IHZ1ZSBmcm9tIFwiQHZpdGVqcy9wbHVnaW4tdnVlXCI7XG5pbXBvcnQgY2hlY2tlclBsdWdpbiBmcm9tIFwidml0ZS1wbHVnaW4tY2hlY2tlclwiO1xuaW1wb3J0IHN2Z0xvYWRlclBsdWdpbiBmcm9tIFwidml0ZS1zdmctbG9hZGVyXCI7XG5pbXBvcnQgSWNvbnNQbHVnaW4gZnJvbSBcInVucGx1Z2luLWljb25zL3ZpdGVcIjtcbmltcG9ydCBwYWNrYWdlSnNvbiBmcm9tIFwiLi9wYWNrYWdlLmpzb25cIjtcbmltcG9ydCBwb3N0Y3NzIGZyb20gXCIuL3Bvc3Rjc3MuY29uZmlnLmpzXCI7XG5pbXBvcnQgVml0ZUdpdFJldmlzaW9uUGx1Z2luIGZyb20gXCIuL2J1aWxkLXNyYy92aXRlX2dpdF9yZXZpc2lvbl9wbHVnaW5cIjtcbmltcG9ydCAqIGFzIGNoaWxkX3Byb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcblxuLy8gY2FuJ3QgaW1wb3J0IGEgcmFuZG9tIGZpbGUgYXMgYSBzdHJpbmcgOihcbi8vIGltcG9ydGluZyB2aWEgbm9kZV9tb2R1bGVzIGF0IGxlYXN0IGxlc3QgdXMgc29ydCBvZiBpbXBvcnQgaXQgZnJvbSB0aGUgbW9kdWxlIHJhdGhlclxuLy8gdGhhbiB1c2luZyBhIHJlbGF0aXZlIHBhdGhcbmNvbnN0IGxlc3NWYXJzID0gcmVhZEZpbGVTeW5jKFxuICBcIi4vbm9kZV9tb2R1bGVzL0BzaS92dWUtbGliL3NyYy90YWlsd2luZC9sZXNzX3ZhcnMubGVzc1wiLFxuICBcInV0Zi04XCIsXG4pO1xuXG4vLyBmaXhlcyBkZXYgc2VydmVyIGhhbmRsaW5nIG9mIHBlcmlvZHMgaW4gcGF0aHNcbi8vIHNlZSBodHRwczovL2dpdGh1Yi5jb20vdml0ZWpzL3ZpdGUvaXNzdWVzLzI0MTVcbmNvbnN0IGRvdFBhdGhGaXhQbHVnaW4gPSAoKSA9PiAoe1xuICBuYW1lOiBcImRvdC1wYXRoLWZpeC1wbHVnaW5cIixcbiAgY29uZmlndXJlU2VydmVyOiAoc2VydmVyKSA9PiB7XG4gICAgc2VydmVyLm1pZGRsZXdhcmVzLnVzZSgocmVxLCBfcmVzLCBuZXh0KSA9PiB7XG4gICAgICBjb25zdCByZXFQYXRoID0gcmVxLnVybC5zcGxpdChcIj9cIiwgMilbMF07XG4gICAgICBpZiAoXG4gICAgICAgICFyZXEudXJsLnN0YXJ0c1dpdGgoXCIvQFwiKSAmJiAvLyB2aXJ0dWFsIGZpbGVzIHByb3ZpZGVkIGJ5IHZpdGUgcGx1Z2luc1xuICAgICAgICAhcmVxLnVybC5zdGFydHNXaXRoKFwiL2FwaS9cIikgJiYgLy8gYXBpIHByb3h5LCBjb25maWd1cmVkIGJlbG93XG4gICAgICAgICFleGlzdHNTeW5jKGAuL3B1YmxpYyR7cmVxUGF0aH1gKSAmJiAvLyBmaWxlcyBzZXJ2ZWQgZGlyZWN0bHkgZnJvbSBwdWJsaWMgZm9sZGVyXG4gICAgICAgICFleGlzdHNTeW5jKGAuJHtyZXFQYXRofWApIC8vIGFjdHVhbCBmaWxlc1xuICAgICAgKSB7XG4gICAgICAgIHJlcS51cmwgPSBcIi9cIjtcbiAgICAgIH1cbiAgICAgIG5leHQoKTtcbiAgICB9KTtcbiAgfSxcbn0pO1xuXG5cbmNvbnN0IGdpdEhhc2hGaWxlID0gKGZpbGU6IHN0cmluZykgPT4ge1xuICBjb25zdCBjbWQgPSBgZ2l0IGhhc2gtb2JqZWN0ICcke2ZpbGV9J2A7XG4gIHJldHVybiBjaGlsZF9wcm9jZXNzLmV4ZWNTeW5jKGNtZCkudG9TdHJpbmcoKS50cmltKCk7XG59XG5jb25zdCB3ZWJXb3JrZXJQYXRoID0gcGF0aC5yZXNvbHZlKF9fZGlybmFtZSwgXCJzcmMvd29ya2Vycy93ZWJ3b3JrZXIudHNcIik7XG5jb25zdCB3ZWJXb3JrZXJIYXNoID0gSlNPTi5zdHJpbmdpZnkoZ2l0SGFzaEZpbGUod2ViV29ya2VyUGF0aCkpO1xuXG5jb25zdCBzaGFyZWRXb3JrZXJQYXRoID0gcGF0aC5yZXNvbHZlKF9fZGlybmFtZSwgXCJzcmMvd29ya2Vycy9zaGFyZWRfd2Vid29ya2VyLnRzXCIpO1xuY29uc3Qgc2hhcmVkV29ya2VySGFzaCA9IEpTT04uc3RyaW5naWZ5KGdpdEhhc2hGaWxlKHNoYXJlZFdvcmtlclBhdGgpKTtcblxuY29uc3QgaGVhZENvbW1pdEhhc2ggPSBKU09OLnN0cmluZ2lmeShcbiAgY2hpbGRfcHJvY2Vzcy5leGVjU3luYyhcImdpdCByZXYtcGFyc2UgSEVBRFwiKS50b1N0cmluZygpLnRyaW0oKVxuKTtcblxuLy8gc2VlIGh0dHBzOi8vdml0ZWpzLmRldi9jb25maWcvIGZvciBtb3JlIGluZm9cbmV4cG9ydCBkZWZhdWx0IChvcHRzOiB7IG1vZGU6IHN0cmluZyB9KSA9PiB7XG4gIC8vIGxvYWQgY29uZmlnIHNvIHdlIGNhbiBrZWVwIHRoZSBkZXYgcG9ydCB0byBydW4gdGhlcmUsIGFuZCBwb3RlbnRpYWxseSBvdGhlciB0aGluZ3MgaW4gdGhlIGZ1dHVyZVxuICAvLyAzcmQgYXJnIChwcmVmaXgpIGxvYWRzIGFsbCBlbnYgdmFycyBpbnN0ZWFkIG9mIGp1c3QgVklURV9BUFBfKlxuICBjb25zdCBjb25maWcgPSBsb2FkRW52KG9wdHMubW9kZSwgcHJvY2Vzcy5jd2QoKSwgXCJcIik7XG5cbiAgcmV0dXJuIGRlZmluZUNvbmZpZyh7XG4gICAgLy8gTk9URTogdGhlc2UgY29uc3RhbnRzIG9ubHkgdXBkYXRlIGF0IGJ1aWxkIHRpbWUsIG9yIGlmIHlvdSByZXN0YXJ0IHRoZSB2aXRlIHNlcnZlclxuICAgIGRlZmluZToge1xuICAgICAgX19DT01NSVRfSEFTSF9fOiAgaGVhZENvbW1pdEhhc2gsXG4gICAgICBfX1NIQVJFRF9XT1JLRVJfSEFTSF9fOiBzaGFyZWRXb3JrZXJIYXNoLFxuICAgICAgX19XRUJXT1JLRVJfSEFTSF9fOiB3ZWJXb3JrZXJIYXNoLFxuICAgIH0sXG4gICAgcGx1Z2luczogW1xuICAgICAgZG90UGF0aEZpeFBsdWdpbigpLFxuICAgICAgdnVlKCksXG4gICAgICBzdmdMb2FkZXJQbHVnaW4oKSxcblxuICAgICAgLy8gdXNpbmcgXCJyYXdcIiBhcyBpY29uIGNvbXBpbGVyIChyYXRoZXIgdGhhbiBgdnVlM2ApIGJlY2F1c2Ugd2UgbmVlZCByYXcgc3ZncyBmb3IgdXNlIGluIGtvbnZhXG4gICAgICAvLyBvdXIgSWNvbiBjb21wb25lbnQga25vd3MgaG93IHRvIGRlYWwgd2l0aCByYXcgU1ZHc1xuICAgICAgSWNvbnNQbHVnaW4oe2NvbXBpbGVyOiBcInJhd1wifSksXG5cbiAgICAgIHByb2Nlc3MuZW52Lk5PREVfRU5WICE9PSBcInByb2R1Y3Rpb25cIiAmJlxuICAgICAgICBjaGVja2VyUGx1Z2luKHtcbiAgICAgICAgICB2dWVUc2M6IHRydWUsXG4gICAgICAgICAgZXNsaW50OiB7XG4gICAgICAgICAgICBsaW50Q29tbWFuZDogcGFja2FnZUpzb24uc2NyaXB0cy5saW50LFxuICAgICAgICAgICAgLy8gSSBfdGhpbmtfIHdlIG9ubHkgd2FudCB0byBwb3AgdXAgYW4gZXJyb3Igb24gdGhlIHNjcmVlbiBmb3IgcHJvcGVyIGVycm9yc1xuICAgICAgICAgICAgLy8gb3RoZXJ3aXNlIHdlIGNhbiBnZXQgYSBsb3Qgb2YgdW51c2VkIHZhciBlcnJvcnMgd2hlbiB5b3UgY29tbWVudCBzb21ldGhpbmcgb3V0IHRlbXBvcmFyaWx5XG4gICAgICAgICAgICBkZXY6IHsgbG9nTGV2ZWw6IFtcImVycm9yXCJdIH0sXG4gICAgICAgICAgfSxcbiAgICAgICAgfSksXG5cbiAgICAgIFZpdGVHaXRSZXZpc2lvblBsdWdpbih7fSksXG4gICAgXSxcbiAgICBjc3M6IHtcbiAgICAgIHBvc3Rjc3MsXG4gICAgICBwcmVwcm9jZXNzb3JPcHRpb25zOiB7XG4gICAgICAgIGxlc3M6IHsgYWRkaXRpb25hbERhdGE6IGxlc3NWYXJzIH0sXG4gICAgICB9LFxuICAgIH0sXG4gICAgc2VydmVyOiB7XG4gICAgICBobXI6IGZhbHNlLFxuICAgICAgaG9zdDogY29uZmlnLkRFVl9IT1NULFxuICAgICAgYWxsb3dlZEhvc3RzOiB0cnVlLFxuICAgICAgcG9ydDogcGFyc2VJbnQoY29uZmlnLkRFVl9QT1JUKSxcbiAgICAgIHN0cmljdFBvcnQ6IHRydWUsXG4gICAgICBmczoge1xuICAgICAgICBzdHJpY3Q6IHRydWUsXG4gICAgICB9LFxuICAgICAgcHJveHk6IHtcbiAgICAgICAgXCIvYXBpXCI6IHtcbiAgICAgICAgICB0YXJnZXQ6IGNvbmZpZy5ERVZfQVBJX1BST1hZX1VSTCxcbiAgICAgICAgICB3czogdHJ1ZSxcbiAgICAgICAgfSxcbiAgICAgIH0sXG4gICAgICBoZWFkZXJzOiB7XG4gICAgICAgIFwiQ3Jvc3MtT3JpZ2luLU9wZW5lci1Qb2xpY3lcIjogXCJzYW1lLW9yaWdpblwiLFxuICAgICAgICBcIkNyb3NzLU9yaWdpbi1FbWJlZGRlci1Qb2xpY3lcIjogXCJjcmVkZW50aWFsbGVzc1wiLFxuICAgICAgfSxcbiAgICB9LFxuICAgIG9wdGltaXplRGVwczoge1xuICAgICAgZXhjbHVkZTogW1wiQHNxbGl0ZS5vcmcvc3FsaXRlLXdhc21cIl0sXG4gICAgfSxcbiAgICBwcmV2aWV3OiB7XG4gICAgICBwcm94eToge1xuICAgICAgICBcIi9hcGlcIjoge1xuICAgICAgICAgIHRhcmdldDogY29uZmlnLkRFVl9BUElfUFJPWFlfVVJMLFxuICAgICAgICAgIHdzOiB0cnVlLFxuICAgICAgICB9LFxuICAgICAgfSxcbiAgICB9LFxuICAgIHJlc29sdmU6IHtcbiAgICAgIGFsaWFzOiBbXG4gICAgICAgIHtcbiAgICAgICAgICBmaW5kOiBcIkBcIixcbiAgICAgICAgICByZXBsYWNlbWVudDogcGF0aC5yZXNvbHZlKF9fZGlybmFtZSwgXCJzcmNcIiksXG4gICAgICAgIH0sXG4gICAgICAgIHsgZmluZDogXCJ1dGlsXCIsIHJlcGxhY2VtZW50OiBcInV0aWwtYnJvd3NlclwiIH0sXG4gICAgICBdLFxuICAgIH0sXG4gICAgYnVpbGQ6IHtcbiAgICAgIG1hbmlmZXN0OiBcIm1hbmlmZXN0Lmpzb25cIixcbiAgICAgIHNvdXJjZW1hcDogXCJpbmxpbmVcIixcbiAgICAgIHJvbGx1cE9wdGlvbnM6IHtcbiAgICAgICAgaW5wdXQ6IHtcbiAgICAgICAgICBtYWluOiBwYXRoLnJlc29sdmUoX19kaXJuYW1lLCBcImluZGV4Lmh0bWxcIiksXG4gICAgICAgICAgd29ya2VyOiB3ZWJXb3JrZXJQYXRoLFxuICAgICAgICAgIHNoYXJlZFdvcmtlcjogc2hhcmVkV29ya2VyUGF0aCxcbiAgICAgICAgfSxcbiAgICAgICAgb3V0cHV0OiB7XG4gICAgICAgICAgZW50cnlGaWxlTmFtZXM6IChjaHVuaykgPT4ge1xuICAgICAgICAgICAgaWYgKGNodW5rLm5hbWUgPT09IFwid29ya2VyXCIpIHtcbiAgICAgICAgICAgICAgcmV0dXJuIFwiYXNzZXRzL3dlYndvcmtlci5qc1wiOyAvLyBTcGVjaWZ5IG91dHB1dCBwYXRoIGZvciB3ZWIgd29ya2VyXG4gICAgICAgICAgICB9XG4gICAgICAgICAgICBpZiAoY2h1bmsubmFtZSA9PT0gXCJzaGFyZWRXb3JrZXJcIikge1xuICAgICAgICAgICAgICByZXR1cm4gXCJhc3NldHMvc2hhcmVkX3dlYndvcmtlci5qc1wiO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgcmV0dXJuIFwiYXNzZXRzL1tuYW1lXS1baGFzaF0uanNcIjtcbiAgICAgICAgICB9LFxuICAgICAgICAgIGZvcm1hdDogXCJlc1wiLFxuICAgICAgICAgIGdsb2JhbHM6IHtcbiAgICAgICAgICAgIHJlYWN0OiBcIlJlYWN0XCIsXG4gICAgICAgICAgICBcInJlYWN0LWRvbVwiOiBcIlJlYWN0RE9NXCIsXG4gICAgICAgICAgfSxcbiAgICAgICAgfSxcbiAgICAgIH0sXG4gICAgfSxcbiAgfSk7XG59O1xuIiwgIntcbiAgXCJuYW1lXCI6IFwid2ViXCIsXG4gIFwicHJpdmF0ZVwiOiB0cnVlLFxuICBcInZlcnNpb25cIjogXCIwLjAuMFwiLFxuICBcInR5cGVcIjogXCJtb2R1bGVcIixcbiAgXCJzY3JpcHRzXCI6IHtcbiAgICBcImNsZWFuXCI6IFwicm0gLXJmIC4vZGlzdCAuL2xpYiAuL3RhcmdldCAmJiBwbnBtIHZpdGU6Y2xlYW5cIixcbiAgICBcImJ1aWxkXCI6IFwidml0ZSBidWlsZFwiLFxuICAgIFwiYnVpbGQ6Y2hlY2tcIjogXCJ2dWUtdHNjIC0tbm9FbWl0XCIsXG4gICAgXCJidWlsZDpjbGVhblwiOiBcInBucG0gcnVuIGNsZWFuICYmIG5wbSBydW4gYnVpbGRcIixcbiAgICBcImJ1aWxkOndhdGNoXCI6IFwicG5wbSBydW4gY2xlYW4gJiYgdHNjIC0td2F0Y2hcIixcbiAgICBcImJ1aWxkOnBlZGFudGljYWxseVwiOiBcInBucG0gcnVuIGJ1aWxkOmNoZWNrICYmIG5wbSBydW4gYnVpbGRcIixcbiAgICBcIndhdGNoXCI6IFwicG5wbSBydW4gYnVpbGQ6d2F0Y2hcIixcbiAgICBcImxpbnRcIjogXCJlc2xpbnQgc3JjIC0tZXh0IC50cywuanMsLmNqcywudnVlXCIsXG4gICAgXCJsaW50OmZpeFwiOiBcInBucG0gcnVuIGxpbnQgLS1maXhcIixcbiAgICBcImxpbnQ6c3RyaWN0XCI6IFwibnBtIHJ1biBsaW50IC0tbWF4LXdhcm5pbmdzPTBcIixcbiAgICBcImxpbnQ6c3VtbWFyeVwiOiBcInBucG0gcnVuIGxpbnQgLS1mb3JtYXQgc3VtbWFyeS1jaGFydFwiLFxuICAgIFwiZm10XCI6IFwicG5wbSBydW4gbGludDpmaXhcIixcbiAgICBcImZtdDpjaGVja1wiOiBcInBucG0gcnVuIGxpbnRcIixcbiAgICBcImNoZWNrXCI6IFwicG5wbSBydW4gbGludDpzdHJpY3QgJiYgbnBtIHJ1biBidWlsZDpjaGVja1wiLFxuICAgIFwiZGV2XCI6IFwicG5wbSBydW4gc3RhcnRcIixcbiAgICBcInN0YXJ0XCI6IFwidml0ZVwiLFxuICAgIFwic2VydmVcIjogXCJ2aXRlIHByZXZpZXcgLS1wb3J0IDgwODBcIixcbiAgICBcInRlc3RcIjogXCJlY2hvICdTb3JyeSwgbm8gdGVzdHMgeWV0ISdcIixcbiAgICBcInZpdGU6Y2xlYW5cIjogXCJybSAtcmYgLi9ub2RlX21vZHVsZXMvLnZpdGVcIixcbiAgICBcImN5cHJlc3M6cnVuXCI6IFwiY3lwcmVzcyBydW5cIixcbiAgICBcImN5cHJlc3M6b3BlblwiOiBcImN5cHJlc3Mgb3BlblwiXG4gIH0sXG4gIFwiZGVwZW5kZW5jaWVzXCI6IHtcbiAgICBcIkBjb2RlbWlycm9yL2F1dG9jb21wbGV0ZVwiOiBcIl42LjQuMlwiLFxuICAgIFwiQGNvZGVtaXJyb3IvY29tbWFuZHNcIjogXCJeNi4xLjJcIixcbiAgICBcIkBjb2RlbWlycm9yL2xhbmctamF2YXNjcmlwdFwiOiBcIl42LjEuMlwiLFxuICAgIFwiQGNvZGVtaXJyb3IvbGFuZy1qc29uXCI6IFwiXjYuMC4xXCIsXG4gICAgXCJAY29kZW1pcnJvci9sYW5nLXlhbWxcIjogXCJeNi4xLjFcIixcbiAgICBcIkBjb2RlbWlycm9yL2xhbmd1YWdlXCI6IFwiXjYuMy4xXCIsXG4gICAgXCJAY29kZW1pcnJvci9sZWdhY3ktbW9kZXNcIjogXCJeNi4zLjFcIixcbiAgICBcIkBjb2RlbWlycm9yL2xpbnRcIjogXCJeNi4xLjBcIixcbiAgICBcIkBjb2RlbWlycm9yL3NlYXJjaFwiOiBcIl42LjMuMFwiLFxuICAgIFwiQGNvZGVtaXJyb3Ivc3RhdGVcIjogXCJeNi4xLjRcIixcbiAgICBcIkBjb2RlbWlycm9yL3ZpZXdcIjogXCJeNi43LjFcIixcbiAgICBcIkBmc2VndXJhaS9jb2RlbWlycm9yLXRoZW1lLWdpdGh1Yi1kYXJrXCI6IFwiXjYuMi4wXCIsXG4gICAgXCJAZnNlZ3VyYWkvY29kZW1pcnJvci10aGVtZS1naXRodWItbGlnaHRcIjogXCJeNi4yLjBcIixcbiAgICBcIkBoZWFkbGVzc3VpL3Z1ZVwiOiBcIl4xLjcuMTBcIixcbiAgICBcIkBob25leWNvbWJpby9vcGVudGVsZW1ldHJ5LXdlYlwiOiBcIl4wLjMuMFwiLFxuICAgIFwiQGxlemVyL2hpZ2hsaWdodFwiOiBcIl4xLjEuM1wiLFxuICAgIFwiQG9wZW50ZWxlbWV0cnkvYXBpXCI6IFwiXjEuOC4wXCIsXG4gICAgXCJAb3BlbnRlbGVtZXRyeS9hdXRvLWluc3RydW1lbnRhdGlvbnMtd2ViXCI6IFwiXjAuMzkuMFwiLFxuICAgIFwiQG9wZW50ZWxlbWV0cnkvZXhwb3J0ZXItb3RscC1odHRwXCI6IFwiXjAuMjYuMFwiLFxuICAgIFwiQG9wZW50ZWxlbWV0cnkvZXhwb3J0ZXItdHJhY2Utb3RscC1odHRwXCI6IFwiXjAuNTcuMlwiLFxuICAgIFwiQG9wZW50ZWxlbWV0cnkvaW5zdHJ1bWVudGF0aW9uXCI6IFwiXjAuNTcuMlwiLFxuICAgIFwiQG9wZW50ZWxlbWV0cnkvaW5zdHJ1bWVudGF0aW9uLWRvY3VtZW50LWxvYWRcIjogXCJeMC4zOC4wXCIsXG4gICAgXCJAb3BlbnRlbGVtZXRyeS9pbnN0cnVtZW50YXRpb24tZmV0Y2hcIjogXCJeMC41Ny4yXCIsXG4gICAgXCJAb3BlbnRlbGVtZXRyeS9pbnN0cnVtZW50YXRpb24tbG9uZy10YXNrXCI6IFwiXjAuMzguMFwiLFxuICAgIFwiQG9wZW50ZWxlbWV0cnkvaW5zdHJ1bWVudGF0aW9uLXVzZXItaW50ZXJhY3Rpb25cIjogXCJeMC4zOC4wXCIsXG4gICAgXCJAb3BlbnRlbGVtZXRyeS9yZXNvdXJjZXNcIjogXCJeMS4zMC4xXCIsXG4gICAgXCJAb3BlbnRlbGVtZXRyeS9zZGstdHJhY2UtYmFzZVwiOiBcIl4xLjMwLjFcIixcbiAgICBcIkBvcGVudGVsZW1ldHJ5L3Nkay10cmFjZS13ZWJcIjogXCJeMS4zMC4xXCIsXG4gICAgXCJAb3BlbnRlbGVtZXRyeS9zZW1hbnRpYy1jb252ZW50aW9uc1wiOiBcIl4xLjMwLjBcIixcbiAgICBcIkByZXBsaXQvY29kZW1pcnJvci12aW1cIjogXCJeNi4wLjExXCIsXG4gICAgXCJAc2kvdHMtbGliXCI6IFwid29ya3NwYWNlOipcIixcbiAgICBcIkBzaS92dWUtbGliXCI6IFwid29ya3NwYWNlOipcIixcbiAgICBcIkBzcWxpdGUub3JnL3NxbGl0ZS13YXNtXCI6IFwiMy41MC4zLWJ1aWxkMVwiLFxuICAgIFwiQHRhbnN0YWNrL3Z1ZS1mb3JtXCI6IFwiXjEuOS4wXCIsXG4gICAgXCJAdGFuc3RhY2svdnVlLXF1ZXJ5XCI6IFwiXjUuNjcuM1wiLFxuICAgIFwiQHRhbnN0YWNrL3Z1ZS10YWJsZVwiOiBcIl44LjIwLjVcIixcbiAgICBcIkB0YW5zdGFjay92dWUtdmlydHVhbFwiOiBcIl4zLjEzLjZcIixcbiAgICBcIkB0eXBlcy9hc3luY1wiOiBcIl4zLjIuMTVcIixcbiAgICBcIkB0eXBlc2NyaXB0L3Zmc1wiOiBcIl4xLjUuM1wiLFxuICAgIFwiQHZ1ZXVzZS9jb3JlXCI6IFwiXjEyLjAuMFwiLFxuICAgIFwiQHZ1ZXVzZS9oZWFkXCI6IFwiXjEuMS4xNVwiLFxuICAgIFwiYXN5bmNcIjogXCJeMy4yLjRcIixcbiAgICBcImF4aW9zXCI6IFwiXjEuOC40XCIsXG4gICAgXCJidWZmZXJcIjogXCJeNi4wLjNcIixcbiAgICBcImNsc3hcIjogXCJeMS4yLjFcIixcbiAgICBcImNvZGVtaXJyb3JcIjogXCJeNi4wLjFcIixcbiAgICBcImNvbWxpbmtcIjogXCJeNC40LjJcIixcbiAgICBcImQzXCI6IFwiXjcuOS4wXCIsXG4gICAgXCJkYXRlLWZuc1wiOiBcIl4yLjI5LjJcIixcbiAgICBcImVsa2pzXCI6IFwiXjAuMTAuMFwiLFxuICAgIFwiZmFzdC1qc29uLXBhdGNoXCI6IFwiXjMuMS4xXCIsXG4gICAgXCJmbG9hdGluZy12dWVcIjogXCJeMi4wLjAtYmV0YS4yMFwiLFxuICAgIFwiZm9udGZhY2VvYnNlcnZlclwiOiBcIl4yLjMuMFwiLFxuICAgIFwiZnpmXCI6IFwiXjAuNS4yXCIsXG4gICAgXCJncmFwaG9sb2d5XCI6IFwiXjAuMjUuNFwiLFxuICAgIFwiZ3JhcGhvbG9neS1sYXlvdXQtZm9yY2VhdGxhczJcIjogXCJeMC4xMC4xXCIsXG4gICAgXCJncmFwaG9sb2d5LWxheW91dC1ub3ZlcmxhcFwiOiBcIl4wLjQuMlwiLFxuICAgIFwiaXMtcHJvbWlzZVwiOiBcIl40LjAuMFwiLFxuICAgIFwiamF2YXNjcmlwdC10aW1lLWFnb1wiOiBcIl4yLjUuN1wiLFxuICAgIFwiam9pXCI6IFwiXjE3LjExLjBcIixcbiAgICBcImpzLWJhc2U2NFwiOiBcIl4zLjcuNVwiLFxuICAgIFwianMtYmVhdXRpZnlcIjogXCJeMS4xNC45XCIsXG4gICAgXCJqcy1jb25mZXR0aVwiOiBcIl4wLjExLjBcIixcbiAgICBcImpzLW1kNVwiOiBcIl4wLjguM1wiLFxuICAgIFwiand0LWRlY29kZVwiOiBcIl4zLjEuMlwiLFxuICAgIFwia29udmFcIjogXCJeOC4zLjEzXCIsXG4gICAgXCJsZXNzXCI6IFwiXjQuMS4zXCIsXG4gICAgXCJsaWJzb2RpdW0td3JhcHBlcnNcIjogXCJeMC43LjEzXCIsXG4gICAgXCJsb2NhbC1zdG9yYWdlLWZhbGxiYWNrXCI6IFwiXjQuMS4zXCIsXG4gICAgXCJsb2Rhc2gtZXNcIjogXCJeNC4xNy4yMVwiLFxuICAgIFwibWl0dFwiOiBcIl4zLjAuMVwiLFxuICAgIFwicC1xdWV1ZVwiOiBcIl44LjEuMFwiLFxuICAgIFwicGluaWFcIjogXCJeMi4yLjRcIixcbiAgICBcInBsdXJcIjogXCJeNS4xLjBcIixcbiAgICBcInBvc3Rob2ctanNcIjogXCJeMS4xNTUuMFwiLFxuICAgIFwicXVpY2stbHJ1XCI6IFwiXjcuMC4xXCIsXG4gICAgXCJyZWNvbm5lY3Rpbmctd2Vic29ja2V0XCI6IFwiXjQuNC4wXCIsXG4gICAgXCJzaWdtYVwiOiBcIjMuMC4wLWJldGEuNVwiLFxuICAgIFwic291cmNlbWFwcGVkLXN0YWNrdHJhY2VcIjogXCJeMS4xLjExXCIsXG4gICAgXCJ0aW55Y29sb3IyXCI6IFwiXjEuNC4yXCIsXG4gICAgXCJ0eXBlc2NyaXB0XCI6IFwiXjUuMC40XCIsXG4gICAgXCJ1bGlkXCI6IFwiXjIuMy4wXCIsXG4gICAgXCJ1dGlsLWJyb3dzZXJcIjogXCJeMC4wLjJcIixcbiAgICBcInZhbGlkYXRvclwiOiBcIl4xMy43LjBcIixcbiAgICBcInZhbmlsbGEtcGlja2VyXCI6IFwiXjIuMTIuMVwiLFxuICAgIFwidnVlXCI6IFwiXjMuNS4xM1wiLFxuICAgIFwidnVlLWh0bWwtc2VjdXJlXCI6IFwiXjEuMC4xMFwiLFxuICAgIFwidnVlLWtvbnZhXCI6IFwiXjMuMC4xXCIsXG4gICAgXCJ2dWUtbWFya2Rvd24tcmVuZGVyXCI6IFwiXjIuMi4xXCIsXG4gICAgXCJ2dWUtcm91dGVyXCI6IFwiXjQuNC41XCIsXG4gICAgXCJ2dWUtc2FmZS10ZWxlcG9ydFwiOiBcIl4wLjEuMlwiLFxuICAgIFwidnVlLXRvYXN0aWZpY2F0aW9uXCI6IFwiMi4wLjAtcmMuNVwiLFxuICAgIFwieHRlcm1cIjogXCJeNS4zLjBcIixcbiAgICBcInh0ZXJtLWFkZG9uLWZpdFwiOiBcIl4wLjguMFwiLFxuICAgIFwieHRlcm0tYWRkb24td2ViLWxpbmtzXCI6IFwiXjAuOS4wXCIsXG4gICAgXCJ5LWluZGV4ZWRkYlwiOiBcIl45LjAuMTJcIixcbiAgICBcInktd2Vic29ja2V0XCI6IFwiXjEuNS4wXCIsXG4gICAgXCJ5anNcIjogXCJeMTMuNi44XCIsXG4gICAgXCJ5anMtY29kZW1pcnJvci1wbHVnaW5cIjogXCJ3b3Jrc3BhY2U6KlwiXG4gIH0sXG4gIFwiZGV2RGVwZW5kZW5jaWVzXCI6IHtcbiAgICBcIkBpY29uaWZ5L2pzb25cIjogXCJeMi4yLjE2NlwiLFxuICAgIFwiQHNpL2VzbGludC1jb25maWdcIjogXCJ3b3Jrc3BhY2U6KlwiLFxuICAgIFwiQHNpL3RzY29uZmlnXCI6IFwid29ya3NwYWNlOipcIixcbiAgICBcIkB0eXBlcy9jb2RlbWlycm9yXCI6IFwiXjUuNjAuNVwiLFxuICAgIFwiQHR5cGVzL2QzXCI6IFwiXjcuNC4zXCIsXG4gICAgXCJAdHlwZXMvZm9udGZhY2VvYnNlcnZlclwiOiBcIl4yLjEuMFwiLFxuICAgIFwiQHR5cGVzL2phdmFzY3JpcHQtdGltZS1hZ29cIjogXCJeMi4wLjNcIixcbiAgICBcIkB0eXBlcy9qcy1iZWF1dGlmeVwiOiBcIl4xLjE0LjFcIixcbiAgICBcIkB0eXBlcy9saWJzb2RpdW0td3JhcHBlcnNcIjogXCJeMC43LjExXCIsXG4gICAgXCJAdHlwZXMvbG9kYXNoLWVzXCI6IFwiXjQuMTcuMTJcIixcbiAgICBcIkB0eXBlcy9ub2RlXCI6IFwiXjE4LjE5LjU5XCIsXG4gICAgXCJAdHlwZXMvdGlueWNvbG9yMlwiOiBcIl4xLjQuM1wiLFxuICAgIFwiQHR5cGVzL3ZhbGlkYXRvclwiOiBcIl4xMy43LjJcIixcbiAgICBcIkB0eXBlcy93aWNnLWZpbGUtc3lzdGVtLWFjY2Vzc1wiOiBcIl4yMDIzLjEwLjVcIixcbiAgICBcIkB2aXRlanMvcGx1Z2luLXZ1ZVwiOiBcIl41LjEuNFwiLFxuICAgIFwiY3lwcmVzc1wiOiBcIl4xNC41LjNcIixcbiAgICBcImN5cHJlc3Mtdml0ZVwiOiBcIl4xLjUuMFwiLFxuICAgIFwiZXNsaW50XCI6IFwiXjguNTcuMVwiLFxuICAgIFwiZ3JhcGhvbG9neS10eXBlc1wiOiBcIl4wLjI0LjdcIixcbiAgICBcInVucGx1Z2luLWljb25zXCI6IFwiXjAuMTcuNFwiLFxuICAgIFwidml0ZVwiOiBcIl41LjQuMTlcIixcbiAgICBcInZpdGUtcGx1Z2luLWNoZWNrZXJcIjogXCJeMC42LjRcIixcbiAgICBcInZpdGUtc3ZnLWxvYWRlclwiOiBcIl4zLjQuMFwiLFxuICAgIFwidnVlLXRzY1wiOiBcIl4xLjguMjdcIlxuICB9XG59XG4iLCAiY29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2Rpcm5hbWUgPSBcIi9ob21lL2pvaG4vd29ya2luZy1mb2xkZXIvMi1yZXBvcy9zaS9hcHAvd2ViXCI7Y29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2ZpbGVuYW1lID0gXCIvaG9tZS9qb2huL3dvcmtpbmctZm9sZGVyLzItcmVwb3Mvc2kvYXBwL3dlYi9wb3N0Y3NzLmNvbmZpZy5qc1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9pbXBvcnRfbWV0YV91cmwgPSBcImZpbGU6Ly8vaG9tZS9qb2huL3dvcmtpbmctZm9sZGVyLzItcmVwb3Mvc2kvYXBwL3dlYi9wb3N0Y3NzLmNvbmZpZy5qc1wiO2ltcG9ydCBjb25maWcgZnJvbSBcIkBzaS92dWUtbGliL3RhaWx3aW5kL3Bvc3Rjc3MuY29uZmlnLmNqc1wiO1xuXG5leHBvcnQgZGVmYXVsdCBjb25maWc7XG4iLCAiY29uc3QgX192aXRlX2luamVjdGVkX29yaWdpbmFsX2Rpcm5hbWUgPSBcIi9ob21lL2pvaG4vd29ya2luZy1mb2xkZXIvMi1yZXBvcy9zaS9hcHAvd2ViL2J1aWxkLXNyY1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9maWxlbmFtZSA9IFwiL2hvbWUvam9obi93b3JraW5nLWZvbGRlci8yLXJlcG9zL3NpL2FwcC93ZWIvYnVpbGQtc3JjL3ZpdGVfZ2l0X3JldmlzaW9uX3BsdWdpbi50c1wiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9pbXBvcnRfbWV0YV91cmwgPSBcImZpbGU6Ly8vaG9tZS9qb2huL3dvcmtpbmctZm9sZGVyLzItcmVwb3Mvc2kvYXBwL3dlYi9idWlsZC1zcmMvdml0ZV9naXRfcmV2aXNpb25fcGx1Z2luLnRzXCI7LypcbiAgdml0ZSBwbHVnaW4gdG8gbWFrZSBjdXJyZW50IGdpdCBicmFuY2gvaGFzaC9ldGMgYXZhaWxhYmxlXG4gIE5PVEUgLSBzdGFydGVkIGZyb20gZnJvbSBodHRwczovL2dpdGh1Yi5jb20vcWR1bGQvdml0ZS1wbHVnaW4tZ2l0LXJldmlzaW9uXG4gIGJ1dCBtb2R1bGUgaXMgbm90IHBvcHVsYXIvbWFpbnRhaW5lZCBzbyB3ZSBjYW4gY3VzdG9taXplIGEgYml0IGhlcmUgZm9yIG91ciBvd24gbmVlZHNcbiovXG5cbi8qIGVzbGludC1kaXNhYmxlIEB0eXBlc2NyaXB0LWVzbGludC9uby1leHBsaWNpdC1hbnkgKi9cblxuaW1wb3J0IHsgZXhlY1N5bmMgfSBmcm9tIFwiY2hpbGRfcHJvY2Vzc1wiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCB7IFBsdWdpbiB9IGZyb20gXCJ2aXRlXCI7XG5cbmludGVyZmFjZSBWaXRlR2l0UmV2aXNpb25QbHVnaW4ge1xuICAvLyBnaXQgd29yayB0cmVlXG4gIGdpdFdvcmtUcmVlPzogYW55O1xuICBzaG9ydFNoYT86IGJvb2xlYW47XG59XG5cbmNvbnN0IGRlZmF1bHRPcHRpb25zOiBWaXRlR2l0UmV2aXNpb25QbHVnaW4gPSB7XG4gIHNob3J0U2hhOiB0cnVlLFxufTtcblxuZXhwb3J0IGRlZmF1bHQgKG9wdGlvbnM6IFZpdGVHaXRSZXZpc2lvblBsdWdpbik6IFBsdWdpbiA9PiB7XG4gIG9wdGlvbnMgPSBPYmplY3QuYXNzaWduKGRlZmF1bHRPcHRpb25zLCBvcHRpb25zIHx8IHt9KTtcblxuICBmdW5jdGlvbiBydW5HaXRDb21tYW5kKGNvbW1hbmQ6IHN0cmluZywgZmFsbGJhY2s/OiBzdHJpbmcpIHtcbiAgICBjb25zdCBjbWQgPSBbXG4gICAgICBcImdpdFwiLFxuICAgICAgLi4uKG9wdGlvbnMuZ2l0V29ya1RyZWVcbiAgICAgICAgPyBbXG4gICAgICAgICAgICBgLS1naXQtZGlyPSR7cGF0aC5qb2luKG9wdGlvbnMuZ2l0V29ya1RyZWUsIFwiLmdpdFwiKX1gLFxuICAgICAgICAgICAgYC0td29yay10cmVlPSR7b3B0aW9ucy5naXRXb3JrVHJlZX1gLFxuICAgICAgICAgIF1cbiAgICAgICAgOiBbXSksXG4gICAgICBjb21tYW5kLFxuICAgIF0uam9pbihcIiBcIik7XG4gICAgdHJ5IHtcbiAgICAgIGNvbnN0IHJlc3VsdCA9IGV4ZWNTeW5jKGNtZClcbiAgICAgICAgLnRvU3RyaW5nKClcbiAgICAgICAgLnJlcGxhY2UoL1tcXHNcXHJcXG5dKyQvLCBcIlwiKTtcbiAgICAgIHJldHVybiByZXN1bHQ7XG4gICAgfSBjYXRjaCAoZXJyKSB7XG4gICAgICBpZiAoZmFsbGJhY2sgIT09IHVuZGVmaW5lZCkgcmV0dXJuIGZhbGxiYWNrO1xuICAgICAgdGhyb3cgZXJyO1xuICAgIH1cbiAgfVxuXG4gIC8vIFRPRE86IGN1cnJlbnRseSBmYWlsaW5nIG9uIGRvY2tlciBidWlsZHMgYmVjYXVzZSB0aGUgY29waWVkIGZpbGVzIGFyZSBubyBsb25nZXIgaW4gYSBnaXQgcmVwb1xuICAvLyB3ZSdsbCB3YW50IHRvIGRvIHNvbWV0aGluZyB0byBtYWtlIGl0IGF2YWlsYWJsZSwgbGlrZSBkdW1wIHRoZSBpbmZvIHRvIGEgZmlsZS4uLlxuICBjb25zdCBnaXRCcmFuY2ggPSBydW5HaXRDb21tYW5kKFwicmV2LXBhcnNlIC0tYWJicmV2LXJlZiBIRUFEXCIsIFwidW5rbm93blwiKTtcbiAgY29uc3QgZ2l0U2hhID0gcnVuR2l0Q29tbWFuZChcbiAgICBgcmV2LXBhcnNlICR7b3B0aW9ucy5zaG9ydFNoYSA/IFwiLS1zaG9ydFwiIDogXCJcIn0gSEVBRGAsXG4gICAgXCJ1bmtub3duXCIsXG4gICk7XG5cbiAgcmV0dXJuIHtcbiAgICBuYW1lOiBcInZpdGU6Z2l0LXJldmlzaW9uXCIsXG4gICAgY29uZmlnKGNvbmZpZzogYW55KSB7XG4gICAgICAvLyB0aGVzZSB2YXJpYWJsZXMgd2lsbCBiZSByZXBsYWNlZCBpbiB0aGUgYnVpbGQgcHJvY2Vzc1xuICAgICAgY29uZmlnLmRlZmluZS5fX1ZJVEVfR0lUX0JSQU5DSF9fID0gSlNPTi5zdHJpbmdpZnkoZ2l0QnJhbmNoKTtcbiAgICAgIGNvbmZpZy5kZWZpbmUuX19WSVRFX0dJVF9TSEFfXyA9IEpTT04uc3RyaW5naWZ5KGdpdFNoYSk7XG4gICAgfSxcbiAgfTtcbn07XG4iXSwKICAibWFwcGluZ3MiOiAiO0FBQXNULE9BQU9BLFdBQVU7QUFDdlUsU0FBUyxZQUFZLG9CQUFvQjtBQUN6QyxTQUFTLFNBQVMsb0JBQW9CO0FBQ3RDLE9BQU8sU0FBUztBQUNoQixPQUFPLG1CQUFtQjtBQUMxQixPQUFPLHFCQUFxQjtBQUM1QixPQUFPLGlCQUFpQjs7O0FDTnhCO0FBQUEsRUFDRSxNQUFRO0FBQUEsRUFDUixTQUFXO0FBQUEsRUFDWCxTQUFXO0FBQUEsRUFDWCxNQUFRO0FBQUEsRUFDUixTQUFXO0FBQUEsSUFDVCxPQUFTO0FBQUEsSUFDVCxPQUFTO0FBQUEsSUFDVCxlQUFlO0FBQUEsSUFDZixlQUFlO0FBQUEsSUFDZixlQUFlO0FBQUEsSUFDZixzQkFBc0I7QUFBQSxJQUN0QixPQUFTO0FBQUEsSUFDVCxNQUFRO0FBQUEsSUFDUixZQUFZO0FBQUEsSUFDWixlQUFlO0FBQUEsSUFDZixnQkFBZ0I7QUFBQSxJQUNoQixLQUFPO0FBQUEsSUFDUCxhQUFhO0FBQUEsSUFDYixPQUFTO0FBQUEsSUFDVCxLQUFPO0FBQUEsSUFDUCxPQUFTO0FBQUEsSUFDVCxPQUFTO0FBQUEsSUFDVCxNQUFRO0FBQUEsSUFDUixjQUFjO0FBQUEsSUFDZCxlQUFlO0FBQUEsSUFDZixnQkFBZ0I7QUFBQSxFQUNsQjtBQUFBLEVBQ0EsY0FBZ0I7QUFB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
