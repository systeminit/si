import {
  type LoadResponse,
  RequestedModuleType,
  ResolutionMode,
  Workspace,
} from "@deno/loader";
import { basename, dirname, join } from "@std/path";
import { Context } from "../context.ts";

/**
 * Transpiles a TypeScript file to JavaScript at runtime using @deno/loader.
 * Writes the transpiled code to a temporary file next to the original template.
 * Returns the path to the temporary JavaScript file.
 *
 * The temp file is placed in the same directory as the original template,
 * which preserves relative import paths.
 */
export async function transpileTemplate(
  specifier: string,
): Promise<string> {
  const ctx = Context.instance();

  try {
    ctx.logger.debug(`Transpiling template: {specifier}`, { specifier });

    // Create a workspace and loader (no special config needed)
    const workspace = new Workspace();
    const loader = await workspace.createLoader();

    // Resolve the specifier
    const resolvedUrl = loader.resolveSync(
      specifier,
      "file:///", // referrer (base for relative imports)
      ResolutionMode.Import,
    );

    ctx.logger.debug(`Resolved template URL: {url}`, { url: resolvedUrl });

    // Load and transpile the module
    const response = await loader.load(
      resolvedUrl,
      RequestedModuleType.Default,
    );

    if (response.kind === "module") {
      ctx.logger.debug(
        `Transpiled template successfully (media type: {mediaType})`,
        { mediaType: response.mediaType },
      );

      // Convert Uint8Array to string
      const decoder = new TextDecoder();
      const transpiledCode = decoder.decode(response.code);

      // Determine the output path for the transpiled file
      // Place it in the same directory as the original template
      const templatePath = specifier.startsWith("file://")
        ? new URL(specifier).pathname
        : specifier;
      const templateDir = dirname(templatePath);
      const templateBasename = basename(templatePath, ".ts");
      const timestamp = Date.now();
      const tempFilePath = join(
        templateDir,
        `.${templateBasename}.transpiled-${timestamp}.js`,
      );

      // Write the transpiled code to the temp file
      await Deno.writeTextFile(tempFilePath, transpiledCode);
      ctx.logger.debug(`Wrote transpiled code to: {path}`, {
        path: tempFilePath,
      });

      return tempFilePath;
    } else if (response.kind === "external") {
      throw new Error(
        `Cannot transpile external module: ${response.specifier}`,
      );
    } else {
      const _assertNever: never = response;
      throw new Error(
        `Unhandled response kind: ${(response as LoadResponse).kind}`,
      );
    }
  } catch (error) {
    ctx.logger.error(
      `Failed to transpile template: {message}`,
      { message: error instanceof Error ? error.message : String(error) },
    );
    throw error;
  }
}
