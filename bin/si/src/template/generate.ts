import type { Context } from "../context.ts";
import { dirname, join } from "@std/path";
import { ensureDir } from "@std/fs";
import { loadTemplateShell } from "../template-loader.ts";

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

  // Load the template shell using the centralized template loader
  let templateContent: string;
  try {
    templateContent = await loadTemplateShell();
  } catch (error) {
    ctx.logger.error("Could not find template shell file", {
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }

  await ensureDir(dirname(templatePath));

  await Deno.writeTextFile(templatePath, templateContent);

  ctx.logger.info("Template generated successfully", {
    path: templatePath,
  });

  ctx.analytics.trackEvent("template generate", {
    templateName,
  });

  console.log(`\nTemplate generated: ${templatePath}`);
  console.log(`\nNext steps:`);
  console.log(`1. Edit ${templatePath} to customize your template`);
  console.log(`2. Run your template with: si template run ${templatePath}`);
}
