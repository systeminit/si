# AI-Agent Development Guide

## Project Purpose

This library automates AWS infrastructure using System Initiative. It interacts
with SI's assets that represent AWS CloudFormation schemas, managed via AWS
Cloud Control API.

## Build/Lint/Test Commands

- Run all tests: `deno test --allow-all` (note: the full test suite can take
  several minutes)
- Run single test:
  `deno test --allow-all --filter=test_function_name mod_test.ts`
- Run specific tests: `deno test --allow-all --filter="test_pattern"`
- Format code: `deno fmt` or `buck2 run :fix-format`
- Check format: `buck2 run :check-format`
- Lint: `deno lint`
- Publish package: `deno publish` (update version in deno.json first)

## Code Style Guidelines

- Use TypeScript with explicit types
- Format: Standard Deno formatting (2-space indentation)
- Imports: Use JSR modules (@openai/openai, @std/assert), and npm modules if needed
- Error handling: Use try/catch with explicit Error types
- Testing: Use Deno.test() with @std/assert for assertions
- Documentation: JSDoc style comments for functions and modules
- Naming: camelCase for variables/functions, PascalCase for types/schemas
- JSON Schema: Define validation schemas with explicit types and required fields
- Follow AWS CloudFormation schema conventions when modeling resources
- Ensure correct JSON output formatting for AWS Cloud Control API compatibility
