// this takes sandbox.ts and builds it and its dependencies into a single file
// called bundle.ts that is then imported during deno run for function
// execution. This is to ensure we don't need to download imports on every
// function execution when running in firecracker

import * as esbuild from "https://deno.land/x/esbuild@v0.20.0/mod.js";
import { dirname, fromFileUrl, join } from "https://deno.land/std/path/mod.ts";
import { denoPlugins } from "jsr:@luca/esbuild-deno-loader@^0.11.1";

async function buildSandbox(outputPath?: string) {
  const baseDir = dirname(fromFileUrl(import.meta.url));

  try {
    const result = await esbuild.build({
      plugins: [
        ...denoPlugins({}),
      ],
      absWorkingDir: baseDir,
      entryPoints: ["./sandbox.ts"],
      bundle: true,
      format: "esm",
      platform: "node",
      write: false,
      banner: {
        js:
          `import { createRequire } from 'node:module';const require = createRequire(import.meta.url);// INJECTION_POINT - DO NOT REMOVE THIS LINE`,
      },
      define: {
        "import.meta.main": "false",
        "process.env.NODE_ENV": '"production"',
        "global": "globalThis",
      },
      sourcemap: false,
      minify: true,
      minifyWhitespace: true,
      minifyIdentifiers: true,
      minifySyntax: true,
      treeShaking: true,
      charset: "utf8",
      legalComments: "none",
      keepNames: false,
      drop: ["debugger"],
    });

    if (!result.outputFiles?.[0]) {
      throw new Error("No output generated");
    }

    const bundleContent = result.outputFiles[0].text;
    const bundlePath = outputPath;
    if (!bundlePath) {
      throw new Error("Output path was not provided as an argument.");
    }
    await Deno.mkdir(dirname(bundlePath), { recursive: true });
    await Deno.writeTextFile(
      bundlePath,
      `export const SANDBOX_BUNDLE=${JSON.stringify(bundleContent)};`,
    );

    console.log(`✅ Bundle size: ${bundleContent.length} bytes`);
  } catch (error) {
    console.error("❌ Build failed:", error);
    throw error;
  }
}

if (import.meta.main) {
  const outputPath = Deno.args[0];
  console.log("🏃 Running build script...");
  await buildSandbox(outputPath);
}
