import { assertEquals, assertExists } from "@std/assert";
import { generateDummySpecs } from "./pipeline.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { FuncSpec } from "../../bindings/FuncSpec.ts";

Deno.test("generateDummySpecs - should generate specs", async () => {
  const specs = await generateDummySpecs({
    forceUpdateExistingPackages: false,
    moduleIndexUrl: "http://localhost:5157",
    docLinkCache: "doc-link-cache.json",
    inferred: "inferred.json",
  });

  assertExists(specs, "specs should exist");
  assertEquals(Array.isArray(specs), true, "specs should be an array");
  assertEquals(specs.length, 2, "should have 2 specs (Server and Database)");
});

Deno.test("generateDummySpecs - Server spec structure", async () => {
  const specs = await generateDummySpecs({
    forceUpdateExistingPackages: false,
    moduleIndexUrl: "http://localhost:5157",
    docLinkCache: "doc-link-cache.json",
    inferred: "inferred.json",
  });

  const serverSpec = specs.find((s) => s.name === "Dummy::Server");
  assertExists(serverSpec, "Server spec should exist");

  // Validate top-level structure
  assertEquals(serverSpec.kind, "module", "should be a module");
  assertEquals(serverSpec.name, "Dummy::Server", "should have correct name");
  assertExists(serverSpec.version, "should have version");
  assertExists(serverSpec.description, "should have description");
  assertExists(serverSpec.schemas, "should have schemas");

  // Validate schemas array
  assertEquals(
    serverSpec.schemas.length,
    1,
    "should have exactly one schema",
  );

  const schema = serverSpec.schemas[0];
  assertExists(schema, "schema should exist");
  assertEquals(schema.name, "Dummy::Server", "schema name should match");
  assertExists(schema.data, "schema should have data");
  assertEquals(
    schema.data.category,
    "Dummy::Server",
    "should have correct category",
  );

  // Validate variants
  assertEquals(
    schema.variants.length,
    1,
    "should have exactly one variant",
  );

  const variant = schema.variants[0];
  assertExists(variant, "variant should exist");
  assertExists(variant.domain, "variant should have domain");
  assertExists(variant.resourceValue, "variant should have resourceValue");
  assertExists(variant.secrets, "variant should have secrets");
  assertExists(variant.actionFuncs, "variant should have actionFuncs");
  assertExists(
    variant.managementFuncs,
    "variant should have managementFuncs",
  );

  // Validate action funcs
  assertEquals(
    variant.actionFuncs.length,
    4,
    "should have 4 action funcs (create, update, delete, refresh)",
  );

  const actionKinds = variant.actionFuncs.map((af) => af.kind);
  assertEquals(
    actionKinds.includes("create"),
    true,
    "should have create action",
  );
  assertEquals(
    actionKinds.includes("update"),
    true,
    "should have update action",
  );
  assertEquals(
    actionKinds.includes("delete"),
    true,
    "should have delete action",
  );
  assertEquals(
    actionKinds.includes("refresh"),
    true,
    "should have refresh action",
  );

  // Validate management funcs
  assertEquals(
    variant.managementFuncs.length,
    2,
    "should have 2 management funcs (discover, import)",
  );

  // Validate domain properties
  assertExists(variant.domain.data, "domain should have data");
  assertEquals(
    variant.domain.kind,
    "object",
    "domain should be an object",
  );
  assertExists(
    variant.domain.entries,
    "domain should have entries (properties)",
  );

  // Validate resourceValue properties
  assertExists(variant.resourceValue.data, "resourceValue should have data");
  assertEquals(
    variant.resourceValue.kind,
    "object",
    "resourceValue should be an object",
  );
  assertExists(
    variant.resourceValue.entries,
    "resourceValue should have entries",
  );
});

Deno.test("generateDummySpecs - Database spec structure", async () => {
  const specs = await generateDummySpecs({
    forceUpdateExistingPackages: false,
    moduleIndexUrl: "http://localhost:5157",
    docLinkCache: "doc-link-cache.json",
    inferred: "inferred.json",
  });

  const databaseSpec = specs.find((s) => s.name === "Dummy::Database");
  assertExists(databaseSpec, "Database spec should exist");

  assertEquals(
    databaseSpec.kind,
    "module",
    "should be a module",
  );
  assertEquals(
    databaseSpec.name,
    "Dummy::Database",
    "should have correct name",
  );
  assertExists(databaseSpec.schemas, "should have schemas");

  const schema = databaseSpec.schemas[0];
  const variant = schema.variants[0];

  // Validate it has the same structure as Server
  assertExists(variant.domain, "variant should have domain");
  assertExists(variant.resourceValue, "variant should have resourceValue");
  assertEquals(
    variant.actionFuncs.length,
    4,
    "should have 4 action funcs",
  );
  assertEquals(
    variant.managementFuncs.length,
    2,
    "should have 2 management funcs",
  );
});

Deno.test("generateDummySpecs - func definitions", async () => {
  const specs = await generateDummySpecs({
    forceUpdateExistingPackages: false,
    moduleIndexUrl: "http://localhost:5157",
    docLinkCache: "doc-link-cache.json",
    inferred: "inferred.json",
  });

  const serverSpec = specs.find((s) => s.name === "Dummy::Server");
  assertExists(serverSpec, "Server spec should exist");

  const variant = serverSpec.schemas[0].variants[0];

  // Check action func specs
  for (const actionFunc of variant.actionFuncs) {
    assertExists(actionFunc.funcUniqueId, "action func should have uniqueId");
    assertExists(actionFunc.kind, "action func should have kind");

    // Find the corresponding func spec in the package
    const funcSpec: FuncSpec | undefined = serverSpec.funcs.find(
      (f) => f.uniqueId === actionFunc.funcUniqueId,
    );
    assertExists(funcSpec, `func spec for ${actionFunc.kind} should exist`);
    assertExists(funcSpec.name, "func spec should have name");
    assertExists(funcSpec.data, "func spec should have data");
    assertEquals(
      funcSpec.data.backendKind,
      "jsAction",
      "func should be jsAction",
    );
    assertExists(funcSpec.data.codeBase64, "func should have codeBase64");
  }

  // Check management func specs
  for (const mgmtFunc of variant.managementFuncs) {
    assertExists(mgmtFunc.funcUniqueId, "management func should have uniqueId");

    const funcSpec: FuncSpec | undefined = serverSpec.funcs.find(
      (f) => f.uniqueId === mgmtFunc.funcUniqueId,
    );
    assertExists(funcSpec, "management func spec should exist");
    assertExists(funcSpec.data, "func spec should have data");
    assertEquals(
      funcSpec.data.backendKind,
      "management",
      "func should be management",
    );
    assertExists(funcSpec.data.codeBase64, "func should have codeBase64");
  }
});

Deno.test("generateDummySpecs - validates spec shape for serialization", async () => {
  const specs = await generateDummySpecs({
    forceUpdateExistingPackages: false,
    moduleIndexUrl: "http://localhost:5157",
    docLinkCache: "doc-link-cache.json",
    inferred: "inferred.json",
  });

  // Validate each spec can be serialized to JSON
  for (const spec of specs) {
    let jsonString: string;
    try {
      jsonString = JSON.stringify(spec);
      assertExists(jsonString, "spec should be serializable to JSON");
    } catch (e) {
      throw new Error(`Failed to serialize ${spec.name}: ${e}`);
    }

    // Validate it can be parsed back
    let parsed: ExpandedPkgSpec;
    try {
      parsed = JSON.parse(jsonString);
      assertExists(parsed, "spec should be parseable from JSON");
      assertEquals(parsed.name, spec.name, "parsed spec should match original");
    } catch (e) {
      throw new Error(`Failed to parse ${spec.name}: ${e}`);
    }
  }
});