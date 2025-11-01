import { Logger } from "@logtape/logtape";
import { Context } from "./context.ts";
import { basename, isAbsolute, resolve, toFileUrl } from "@std/path";

export interface TemplateContextOptions {
  // Invocation Key for idempotency control
  key: string;
}

export class TemplateContext {
  public readonly logger: Logger;
  private _name: string;
  private _invocationKey: string;
  private _changeSet: string;
  private _search: string[];

  constructor(templatePath: string, options: TemplateContextOptions) {
    const ctx = Context.instance();
    this.logger = ctx.logger;

    // Extract default name from template path (filename without extension)
    this._name = basename(templatePath, ".ts");
    // Set the invocation key from the CLI options
    this._invocationKey = options.key;
    this._changeSet = `${this._name}-${this._invocationKey}`;
    this._search = [];
  }

  /**
   * Get or set the template name.
   *
   * @param newName - Optional new name to set
   * @returns The current name if no argument provided, otherwise void
   */
  name(newName?: string): string | void {
    if (newName !== undefined) {
      this.logger.debug(`Setting Name: ${newName}`);
      this._name = newName;
    } else {
      return this._name;
    }
  }

  /**
   * Get or set the changeSet name. The default is the template file name, minus
   * the extension, plus the invocation key from the command line arguments.
   *
   * @param newChangeSet - Optional new change set name to set
   * @returns The current name if no argument provided, otherwise void
   */
  changeSet(newChangeSet?: string): string | void {
    if (newChangeSet !== undefined) {
      this.logger.debug(`Setting Change Set: ${newChangeSet}`);
      this._changeSet = newChangeSet;
    } else {
      return this._changeSet;
    }
  }

  /**
   * Get the invocation key.
   *
   * @returns The invocation key
   */
  invocationKey(): string {
    return this._invocationKey;
  }

  /**
   * Get or set the search strings.
   *
   * @param newSearch - Optional new search array to set
   * @returns The current search array if no argument provided, otherwise void
   */
  search(newSearch?: string[]): string[] | void {
    if (newSearch !== undefined) {
      this.logger.debug(`Setting Search: ${JSON.stringify(newSearch)}`);
      this._search = newSearch;
    } else {
      return this._search;
    }
  }
}

function createTemplateContext(templatePath: string, options: TemplateContextOptions): TemplateContext {
  return new TemplateContext(templatePath, options);
}

export async function runTemplate(
  template: string,
  options: TemplateContextOptions,
) {
  const ctx = Context.instance();

  const specifier = /^https?:\/\//.test(template)
    ? template
    : toFileUrl(isAbsolute(template) ? template : resolve(template)).href;
  ctx.logger.info(`Loading Template: ${specifier}`);

  const mod = await import(specifier);
  const run = typeof mod === "function"
    ? mod
    : typeof (mod as any).default === "function"
    ? (mod as any).default
    : (mod as any).run;

  if (typeof run !== "function") {
    console.error(
      "The module must export a function (default) or a named `run(ctx)`.",
    );
    Deno.exit(1);
  }
  const tctx = createTemplateContext(template, options);
  await run(tctx);
}
