# CLAUDE.md - CF-DB Project Guidelines

## Commands

- **Run**: `deno run mod.ts`
- **Lint**: `deno lint`
- **Format**: `deno fmt`
- **Update Schema**: `./update-schema.sh` (updates CloudFormation schemas)
- **Test Single File**: `deno test [filename]`

## Code Style Guidelines

- **Imports**: Use explicit imports, grouped by external/internal, with npm:
  prefix for Node modules
- **Types**: Use TypeScript types extensively, leverage union types where
  appropriate
- **Naming**: camelCase for variables/functions, PascalCase for types/classes
- **Error Handling**: Create custom error classes extending Error, throw with
  descriptive messages
- **Logging**: Use logger from './logger.ts' with appropriate levels (.debug,
  .verbose, .info)
- **Style**: 2-space indentation, trailing commas in multiline structures

## Conventions

- Use lodash for utility functions (`import _ from "npm:lodash"`)
- Prefer functional programming patterns when processing data
- TypeScript's `Extend<T, F>` utility type for extending interfaces
- Dereference schemas to resolve references before processing

## Important Implementation Notes

- When using as a library (via JSR), schema paths should be resolved relative to
  the source file
- Use `import.meta.url` with `new URL()` to resolve paths correctly when
  importing
- Default schema path is in `./cloudformation-schema` relative to the module
