# Template Files

This directory contains template files used by the SI CLI.

## Templates

- **SI_Agent_Context.md.tmpl** - Context file for AI coding tools (CLAUDE.md, AGENTS.md, OPENCODE.md)
- **template.ts.tmpl** - TypeScript template shell for `si template generate` command

## Embedded Templates

To ensure templates work in all build environments (development, `deno compile`, Buck2), the template content is embedded directly into the TypeScript code.

### Updating Templates

When you modify a template file:

1. Edit the template file in this directory
2. Run the template generator to update the embedded version:
   ```bash
   deno task generate-templates
   ```
3. Commit both the template file and the generated `src/embedded-templates.ts`

### How It Works

The `src/template-loader.ts` module tries to load templates in this order:

1. From the file system (works in development)
2. From embedded constants (fallback for compiled binaries)

This ensures templates are always available regardless of how the binary is built.
