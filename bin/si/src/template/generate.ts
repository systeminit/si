import type { Context } from "../context.ts";
import { dirname, join } from "@std/path";
import { ensureDir } from "@std/fs";

export interface GenerateTemplateOptions {
  name: string;
  outputDir?: string;
}

export async function callGenerateTemplate(
  ctx: Context,
  options: GenerateTemplateOptions,
): Promise<void> {
  const templateName = options.name;
  const outputDir = options.outputDir || Deno.cwd();
  const templatePath = join(outputDir, `${templateName}.ts`);

  ctx.logger.info("Generating template structure", {
    name: templateName,
    path: templatePath,
  });

  // Try multiple paths to find the template (handles running from source vs compiled binary)
  const possiblePaths = [
    // When running from source
    new URL("../../data/templates/template.ts.tmpl", import.meta.url)
      .pathname,
    // Relative to current working directory
    join(Deno.cwd(), "data/templates/template.ts.tmpl"),
    // Relative to binary location
    join(dirname(Deno.execPath()), "data/templates/template.ts.tmpl"),
  ];

  let templateContent = "";

  for (const path of possiblePaths) {
    try {
      templateContent = await Deno.readTextFile(path);
      break;
    } catch {
      // Try next path
    }
  }

  if (!templateContent) {
    ctx.logger.error("Could not find template shell file");
    throw new Error(
      `Failed to find template shell file. Tried paths: ${
        possiblePaths.join(", ")
      }`,
    );
  }

  await ensureDir(dirname(templatePath));

  await Deno.writeTextFile(templatePath, templateContent);

  ctx.logger.info("Template generated successfully", {
    path: templatePath,
  });

  ctx.analytics.trackEvent("template_generate", {
    templateName,
    outputPath: templatePath,
  });

  console.log(`\nTemplate generated: ${templatePath}`);
  console.log(`\nNext steps:`);
  console.log(`1. Edit ${templatePath} to customize your template`);
  console.log(`2. Run your template with: si template run ${templatePath}`);
}
