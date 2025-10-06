import {
  assert,
  assertEquals,
  assertExists,
} from "https://deno.land/std@0.224.0/assert/mod.ts";
import { createSandbox } from "../src/sandbox.ts";
import { FunctionKind } from "../src/function.ts";

Deno.test("sandbox edge cases - all function kinds create valid sandboxes", () => {
  const kinds = [
    FunctionKind.ActionRun,
    FunctionKind.Before,
    FunctionKind.Management,
    FunctionKind.ResolverFunction,
    FunctionKind.Validation,
    FunctionKind.SchemaVariantDefinition,
  ];

  for (const kind of kinds) {
    const sandbox = createSandbox(kind, "test-execution-id");
    assertExists(sandbox);
    assertExists(sandbox._);
    assert(typeof sandbox === "object");
  }
});

Deno.test("sandbox edge cases - common utilities are available", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");

  // Check common utilities
  assertExists(sandbox._);
  assertExists(sandbox.Buffer);
  assertExists(sandbox.requestStorage);
  assertExists(sandbox.zlib);
  assertExists(sandbox.siExec);
  assertExists(sandbox.YAML);
  assertExists(sandbox.os);
  assertExists(sandbox.fs);
  assertExists(sandbox.path);
  assertExists(sandbox.Joi);
  assertExists(sandbox.toml);
  assertExists(sandbox.jsonpatch);
  assertExists(sandbox.layout);
  assertExists(sandbox.template);
  assertExists(sandbox.extLib);
});

Deno.test("sandbox edge cases - schema variant definition has builders", () => {
  const sandbox = createSandbox(
    FunctionKind.SchemaVariantDefinition,
    "test-id",
  );

  assertExists(sandbox.AssetBuilder);
  assertExists(sandbox.PropBuilder);
  assertExists(sandbox.SecretDefinitionBuilder);
  assertExists(sandbox.SecretPropBuilder);
  assertExists(sandbox.ValueFromBuilder);
  assertExists(sandbox.SocketDefinitionBuilder);
  assertExists(sandbox.MapKeyFuncBuilder);
  assertExists(sandbox.PropWidgetDefinitionBuilder);
  assertExists(sandbox.SiPropValueFromDefinitionBuilder);
});

Deno.test("sandbox edge cases - before function has special requestStorage", () => {
  const sandbox = createSandbox(FunctionKind.Before, "test-id");

  assertExists(sandbox.requestStorage);
  // Before functions should have a different requestStorage implementation
  assert(sandbox.requestStorage !== undefined);
});

Deno.test("sandbox edge cases - different execution IDs create isolated sandboxes", () => {
  const sandbox1 = createSandbox(FunctionKind.ActionRun, "exec-1");
  const sandbox2 = createSandbox(FunctionKind.ActionRun, "exec-2");

  // Sandboxes should be different objects
  assert(sandbox1 !== sandbox2);

  // But should have the same structure
  assertEquals(Object.keys(sandbox1).sort(), Object.keys(sandbox2).sort());
});

Deno.test("sandbox edge cases - lodash is available and functional", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const _ = sandbox._ as any;

  // Test some common lodash functions
  assertEquals(_.isArray([]), true);
  assertEquals(_.isString("test"), true);
  assertEquals(_.map([1, 2, 3], (n: number) => n * 2), [2, 4, 6]);
  assertEquals(_.get({ a: { b: { c: 1 } } }, "a.b.c"), 1);
});

Deno.test("sandbox edge cases - Joi validation is available", () => {
  const sandbox = createSandbox(FunctionKind.Validation, "test-id");
  const Joi = sandbox.Joi as any;

  const schema = Joi.object({
    name: Joi.string().required(),
    age: Joi.number().min(0),
  });

  const { error: error1 } = schema.validate({ name: "test", age: 25 });
  assertEquals(error1, undefined);

  const { error: error2 } = schema.validate({ age: 25 });
  assertExists(error2);
});

Deno.test("sandbox edge cases - YAML parsing works", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const YAML = sandbox.YAML as any;

  const yamlString = `
name: test
value: 123
nested:
  key: value
`;

  const parsed = YAML.parse(yamlString);
  assertEquals(parsed.name, "test");
  assertEquals(parsed.value, 123);
  assertEquals(parsed.nested.key, "value");

  const stringified = YAML.stringify(parsed);
  assert(stringified.includes("name"));
  assert(stringified.includes("test"));
});

Deno.test("sandbox edge cases - Buffer is available", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const Buffer = sandbox.Buffer as any;

  const buf = Buffer.from("hello world", "utf8");
  assertEquals(buf.toString(), "hello world");
  assertEquals(buf.toString("base64"), "aGVsbG8gd29ybGQ=");
});

Deno.test("sandbox edge cases - jsonpatch is functional", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const jsonpatch = sandbox.jsonpatch as any;

  const original = { name: "test", value: 1 };
  const patches = [
    { op: "replace", path: "/name", value: "updated" },
    { op: "add", path: "/newField", value: "new" },
  ];

  const result = jsonpatch.applyPatch(original, patches).newDocument;
  assertEquals(result.name, "updated");
  assertEquals(result.newField, "new");
});

Deno.test("sandbox edge cases - toml parsing works", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const toml = sandbox.toml as any;

  const tomlString = `
title = "Test"
[section]
key = "value"
number = 42
`;

  const parsed = toml.parse(tomlString);
  assertEquals(parsed.title, "Test");
  assertEquals(parsed.section.key, "value");
  assertEquals(parsed.section.number, 42);
});

Deno.test("sandbox edge cases - path utilities work", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const path = sandbox.path as any;

  assertEquals(path.join("a", "b", "c"), "a/b/c");
  assertEquals(path.basename("/path/to/file.txt"), "file.txt");
  assertEquals(path.dirname("/path/to/file.txt"), "/path/to");
  assertEquals(path.extname("file.txt"), ".txt");
});

Deno.test("sandbox edge cases - layout utilities exist", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const layout = sandbox.layout as any;

  assertExists(layout.createFrame);
  assertExists(layout.createComponent);
  assertExists(layout.createRow);
});

Deno.test("sandbox edge cases - template utilities exist", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const template = sandbox.template as any;

  assertExists(template.converge);
});

Deno.test("sandbox edge cases - extLib utilities exist", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const extLib = sandbox.extLib as any;

  assertExists(extLib);
  assert(typeof extLib === "object");
});

Deno.test("sandbox edge cases - zlib compression works", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");
  const zlib = sandbox.zlib as any;
  const Buffer = sandbox.Buffer as any;

  const input = "Hello World!";
  const compressed = zlib.gzipSync(Buffer.from(input));
  const decompressed = zlib.gunzipSync(compressed);

  assertEquals(decompressed.toString(), input);
});

Deno.test("sandbox edge cases - requestStorage basic operations", () => {
  const sandbox = createSandbox(FunctionKind.ActionRun, "test-id");

  // Check that requestStorage exists in sandbox
  assertExists(sandbox.requestStorage);
  assert(typeof sandbox.requestStorage === "object");

  // The actual requestStorage operations are tested in requestStorage.spec.ts
});
