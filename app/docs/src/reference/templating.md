---
outline:
  level: [1, 2, 3, 4]
---

# Writing Templates in System Initiative

Templates in System Initiative define how to search, modify, and converge
components in your workspace. Each template is a TypeScript file that exports a
default async function receiving a `TemplateContext` object.

---

## Structure of a Template

A minimal template looks like this:

```ts
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.name("example-template");
  ctx.changeSet("dev");
  ctx.search(["schema:*"]);
  ctx.logger.info("Template running");
}
```

### Key Concepts

- **Name** — Identifies the template run.
- **Change Set** — Defines where changes are applied.
- **Search** — Defines which components the template operates on.
- **Logger** — Writes information to the output for debugging or tracking.

---

## Adding Input Parameters

Templates can define inputs validated by
[Zod](https://github.com/colinhacks/zod). Use this to make your template
reusable and safe.

```ts
import { z } from "zod";
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.inputs(
    z.object({
      environment: z.string().default("prod"),
      replicas: z.number().int().default(2),
    }),
  );

  const inputs = ctx.inputData();
  ctx.logger.info(
    `Environment: ${inputs.environment}, Replicas: ${inputs.replicas}`,
  );
}
```

---

## Searching and Filtering Components

Use `search` to find components, then filter or transform them.

```ts
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.name("filter-components");
  ctx.changeSet("dev");

  ctx.search(["schema:AWS*", "component:prod-*"]);

  ctx.transform((workingSet) => {
    return workingSet.filter((c) => c.name.includes("web"));
  });

  ctx.logger.info(`Filtered ${ctx.workingSet().length} components`);
}
```

---

## Modifying Components

You can set attributes, delete them, or create subscriptions between components.

```ts
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.name("modify-components");
  ctx.changeSet("dev");
  ctx.search(["schema:*"]);

  const set = ctx.workingSet();

  for (const c of set) {
    ctx.setAttribute(c, "/domain/replicas", 3);
    ctx.setSubscription(c, "/domain/connection", {
      component: "database",
      path: "/domain/connectionString",
    });
  }

  ctx.logger.info(`Updated ${set.length} components`);
}
```

---

## Creating and Copying Components

You can generate new components or clone existing ones.

```ts
import { TemplateContext } from "jsr:@systeminit/si";

export default async function (ctx: TemplateContext) {
  ctx.name("create-components");
  ctx.changeSet("staging");

  const workspaceId = ctx.workspaceId();
  const changeSetId = await ctx.getHeadChangeSetId();
  const schemaId = await ctx.getSchemaIdByName(
    workspaceId,
    changeSetId,
    "AWS EC2 Instance",
  );

  const newComponent = ctx.newComponent("staging-instance", schemaId);
  ctx.setAttribute(newComponent, "/domain/instanceType", "t3.small");
  ctx.logger.info("Created new staging instance");
}
```

---

## Naming Patterns

Rename components automatically using a pattern.

```ts
ctx.namePattern({
  pattern: /prod-(.+)/g,
  replacement: "staging-$1",
});
```

---

## Good Practices

- Use clear, descriptive names for templates and change sets.
- Keep transformations small and test them incrementally.
- Validate all inputs with Zod.
- Use `ctx.logger` to describe what your template does at each step.
- Cache baselines for large workspaces to improve performance.
- Avoid hardcoding workspace IDs or component names when possible.

---

## Running a Template

Once your template is ready, run it with the `si-tmpl` binary.

```bash
SI_API_TOKEN=<token> si-tmpl run ./tmpl/example.ts --key unique-key
```

For verbose logs:

```bash
si-tmpl run ./tmpl/example.ts --key unique-key --verbose 3
```

You can also cache your workspace for faster runs:

```bash
si-tmpl run ./tmpl/cache.ts --key cache-gen --cache-baseline ./cache/baseline.yaml --cache-baseline-only
```

Then use it later:

```bash
si-tmpl run ./tmpl/example.ts --key example-run --baseline ./cache/baseline.yaml
```
