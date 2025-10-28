# Logging Guidelines

This document defines the logging standards for si-tmpl to ensure clean,
log-oriented output suitable for both human consumption and automated
processing.

## Core Principles

1. **No fancy formatting**: No emojis, ASCII boxes, step numbers, or visual
   decorations in log messages
2. **Structured logging only**: Always use LogTape's template mechanism, never
   string concatenation
3. **Level-appropriate content**: INFO for pipeline stages, DEBUG for consumer
   debugging, TRACE for internal details
4. **Dry-run transparency**: Dry-run output should mirror execution output (with
   attributes at INFO level)

## LogTape Template Syntax

Always use LogTape's structured logging:

```typescript
// ✅ CORRECT: LogTape templates
logger.info("Loading template: {specifier}", { specifier });
logger.debug(
  "Baseline loaded: {componentCount} components, {schemaCount} schemas",
  { componentCount, schemaCount },
);
logger.debug("Setting attributes on {name} {*}", component.attributes);

// ❌ WRONG: String concatenation
logger.info("Loading template: " + specifier);
logger.info(`Loading template: ${specifier}`);
logger.debug("Baseline loaded: " + componentCount + " components");
```

### Object Dumping

Use `{*}` to dump entire objects:

```typescript
logger.debug("Input data {*}", inputData);
logger.trace("Complete payload {*}", payload);
```

## Log Levels

### INFO Level (Verbosity 2)

**Purpose**: Pipeline progress visible to end users

**What to log**:

- One line per major pipeline stage BEFORE execution
- During execution: one line per component with progressive counter
- Summary lines after completion

**Format**: `"Stage description: {context}", { context }`

**Examples**:

```typescript
logger.info("Loading template: {specifier}", { specifier });
logger.info("Building baseline: {count} search strings", { count });
logger.info("Computing pending changes");
logger.info("Creating component {name} ({current}/{total})", {
  name,
  current,
  total,
});
logger.info("Execution complete: {succeeded} succeeded, {failed} failed", {
  succeeded,
  failed,
});
```

**Special case - Dry-run at INFO level**: In dry-run mode, attributes MUST be
shown at INFO level (not just DEBUG):

```typescript
if (dryRun) {
  logger.info("Creating component {name} ({current}/{total})", {
    name,
    current,
    total,
  });
  logger.info("Setting attributes on {name} {*}", component.attributes);
}
```

### DEBUG Level (Verbosity 3)

**Purpose**: Consumer debugging - detailed summaries and component-level
operations

**What to log**:

- Summary data after stages complete
- Component-level decisions and filtering results
- Attributes during execution (in normal mode, not dry-run)
- Key data transformations

**Format**: Structured key-value pairs

**Examples**:

```typescript
logger.debug(
  "Baseline loaded: {componentCount} components, {schemaCount} schemas",
  { componentCount, schemaCount },
);
logger.debug("Component action: {action} {name}", { action: "Create", name });
logger.debug(
  "Pending changes: {creates} creates, {updates} updates, {deletes} deletes",
  { creates, updates, deletes },
);
logger.debug("Setting attributes on {name} {*}", component.attributes);
```

### TRACE Level (Verbosity 4)

**Purpose**: Developer debugging - internal execution details

**What to log**:

- Cache hits/misses
- API call details
- Subscription checks
- Per-component operations in loops
- Complete payloads
- Any repetitive operations

**Examples**:

```typescript
logger.trace("Search cache hit for query: {query}", { query });
logger.trace("Component cache hit for ID: {id}", { id });
logger.trace("Subscription already set on {name} at {path}", { name, path });
logger.trace("Renamed {oldName} to {newName}", { oldName, newName });
logger.trace("Complete payload {*}", payload);
```

## Anti-Patterns to Avoid

### ❌ Fancy Formatting

```typescript
// DON'T DO THIS
logger.info("\n=== Starting Template Converge ===\n");
logger.info("Step 1: Getting or creating change set...");
logger.info("✅ Template converge complete!");
logger.info("📦 CREATE (27 components):");
```

### ❌ String Concatenation

```typescript
// DON'T DO THIS
logger.info("Loading template: " + specifier);
logger.info(`Found ${count} components`);
logger.debug("Name: " + name + ", ID: " + id);
```

### ❌ Multi-line Formatted Output

```typescript
// DON'T DO THIS
logger.info(`
  • Component: ${name}
    Attributes (${count}):
      "/si/name": "${name}"
      "/si/type": "component"
`);
```

### ❌ Wrong Verbosity Level

```typescript
// DON'T DO THIS
logger.info("Search cache hit for query: " + query); // Should be TRACE
logger.debug("Computing pending changes"); // Should be INFO
logger.trace("Execution complete: 27 succeeded"); // Should be INFO
```

## Dry-Run Mode

Dry-run output must mirror what execution would show, with one key difference:

**At INFO level**: Attributes MUST be shown (they're normally only at DEBUG
during execution)

```typescript
// Dry-run at INFO level
if (dryRun) {
  logger.info("Creating component {name} ({current}/{total})", {
    name,
    current,
    total,
  });
  logger.info("Setting attributes on {name} {*}", component.attributes); // ← INFO in dry-run
}

// Normal execution at INFO level
if (!dryRun) {
  logger.info("Creating component {name} ({current}/{total})", {
    name,
    current,
    total,
  });
  // Attributes logged at DEBUG level only
  logger.debug("Setting attributes on {name} {*}", component.attributes); // ← DEBUG in normal mode
}
```

Close dry-run with summary:

```typescript
logger.info(
  "Dry run complete: {creates} creates, {updates} updates, {deletes} deletes",
  { creates, updates, deletes },
);
```

## Console Output

**NEVER use `console.log`, `console.error`, or `console.warn`** except in these
cases:

- Module initialization errors (before Context/logger is initialized)
- Fatal errors that prevent logger initialization

Always prefer `logger.error()` over `console.error()`.

## Progressive Counters

When executing changes, show progress with counters:

```typescript
for (let i = 0; i < changes.length; i++) {
  const change = changes[i];
  const current = i + 1;
  const total = changes.length;

  logger.info("Creating component {name} ({current}/{total})", {
    name: change.name,
    current,
    total,
  });
}
```

## File-by-File Guidance

### src/template/converge.ts

- INFO: Stage start messages only
- DEBUG: Stage completion summaries
- Remove all `===` headers, step numbers, emojis

### src/template/execute.ts

- INFO: Per-component operations with counters
- INFO (dry-run only): Attributes
- DEBUG (normal execution): Attributes
- TRACE: Full API payloads

### src/template/context.ts

- Move cache hits to TRACE
- Move "subscription already set" to TRACE
- Keep meaningful state changes at DEBUG

### src/template/baseline.ts

- INFO: "Building baseline" stage message
- DEBUG: Summary (component/schema counts)
- TRACE: Per-component loading

### src/template/pending_changes.ts

- INFO: Summary only
- DEBUG: Per-component actions

### src/template/names.ts

- INFO: Pattern application messages
- TRACE: Per-component renames

### src/template/input.ts

- INFO: File loading message
- DEBUG: Input data dump

## Testing Checklist

When refactoring logging:

- [ ] Run at verbosity 2 (INFO) - should be ~15-20 lines for pipeline, plus
      execution progress
- [ ] Run at verbosity 3 (DEBUG) - should add summaries and attribute details
- [ ] Run at verbosity 4 (TRACE) - should show all internal operations
- [ ] Run with `--dry-run` at INFO level - should show attributes
- [ ] Verify no `===`, emojis, or step numbers in output
- [ ] Verify all logs use LogTape templates (`{param}` syntax)
- [ ] Verify no string concatenation or template literals for data
