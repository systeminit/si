import { assertEquals } from "@std/assert";
import { FunctionKind, Project } from "./project.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "./config.ts";

Deno.test("SharedFunctionsProjectModule - path generation", async () => {
  const tempDir = await Deno.makeTempDir();
  try {
    const project = new Project(tempDir);

    // Test base path
    const basePath = project.sharedFunctions.moduleBasePath();
    assertEquals(basePath.path, `${tempDir}/shared-functions`);

    // Test action function paths
    const actionBasePath = project.sharedFunctions.sharedFuncBasePath(
      FunctionKind.Action,
    );
    assertEquals(actionBasePath.path, `${tempDir}/shared-functions/actions`);

    const actionCodePath = project.sharedFunctions.sharedFuncCodePath(
      "aws-create",
      FunctionKind.Action,
    );
    assertEquals(
      actionCodePath.path,
      `${tempDir}/shared-functions/actions/aws-create.ts`,
    );

    const actionMetadataPath = project.sharedFunctions.sharedFuncMetadataPath(
      "aws-create",
      FunctionKind.Action,
    );
    assertEquals(
      actionMetadataPath.path,
      `${tempDir}/shared-functions/actions/aws-create.metadata.json`,
    );

    // Test authentication function paths
    const authBasePath = project.sharedFunctions.sharedFuncBasePath(
      FunctionKind.Auth,
    );
    assertEquals(
      authBasePath.path,
      `${tempDir}/shared-functions/authentication`,
    );

    // Test qualification function paths
    const qualBasePath = project.sharedFunctions.sharedFuncBasePath(
      FunctionKind.Qualification,
    );
    assertEquals(
      qualBasePath.path,
      `${tempDir}/shared-functions/qualifications`,
    );

    // Test format version path
    const formatVersionPath = project.sharedFunctions.formatVersionPath();
    assertEquals(
      formatVersionPath.path,
      `${tempDir}/shared-functions/.format-version`,
    );
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("SharedFunctionsProjectModule - create and read shared function", async () => {
  const tempDir = await Deno.makeTempDir();
  try {
    const project = new Project(tempDir);

    // Create the shared-functions directory structure
    const actionBasePath = project.sharedFunctions.sharedFuncBasePath(
      FunctionKind.Action,
    );
    await actionBasePath.mkdir({ recursive: true });

    // Write a shared function
    const codePath = project.sharedFunctions.sharedFuncCodePath(
      "aws-create",
      FunctionKind.Action,
    );
    const metadataPath = project.sharedFunctions.sharedFuncMetadataPath(
      "aws-create",
      FunctionKind.Action,
    );

    const code = `export async function main(input: any) {
  console.log("Creating AWS resource", input);
  return { success: true };
}`;

    const metadata = {
      name: "aws-create",
      displayName: "AWS Resource Creation",
      description: "Generic AWS resource creation",
    };

    await codePath.writeTextFile(code);
    await metadataPath.writeTextFile(JSON.stringify(metadata, null, 2));

    // Read them back
    const readCode = await codePath.readTextFile();
    assertEquals(readCode, code);

    const readMetadata = JSON.parse(await metadataPath.readTextFile());
    assertEquals(readMetadata.name, "aws-create");
    assertEquals(readMetadata.displayName, "AWS Resource Creation");
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("SharedFunctionsProjectModule - format version", async () => {
  const tempDir = await Deno.makeTempDir();
  try {
    const project = new Project(tempDir);

    const formatVersionPath = project.sharedFunctions.formatVersionPath();
    const basePath = project.sharedFunctions.moduleBasePath();

    // Create directory
    await basePath.mkdir({ recursive: true });

    // Write format version
    await formatVersionPath.writeTextFile(
      SCHEMA_FILE_FORMAT_VERSION.toString(),
    );

    // Read it back
    const version = parseInt(await formatVersionPath.readTextFile());
    assertEquals(version, SCHEMA_FILE_FORMAT_VERSION);
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("SharedFunctionsProjectModule - all function kinds", async () => {
  const tempDir = await Deno.makeTempDir();
  try {
    const project = new Project(tempDir);

    const functionKinds = [
      { kind: FunctionKind.Action, dirName: "actions" },
      { kind: FunctionKind.Auth, dirName: "authentication" },
      { kind: FunctionKind.Codegen, dirName: "codeGenerators" },
      { kind: FunctionKind.Management, dirName: "management" },
      { kind: FunctionKind.Qualification, dirName: "qualifications" },
    ];

    for (const { kind, dirName } of functionKinds) {
      const basePath = project.sharedFunctions.sharedFuncBasePath(kind);
      assertEquals(basePath.path, `${tempDir}/shared-functions/${dirName}`);

      const codePath = project.sharedFunctions.sharedFuncCodePath(
        "test-func",
        kind,
      );
      assertEquals(
        codePath.path,
        `${tempDir}/shared-functions/${dirName}/test-func.ts`,
      );
    }
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("SharedFunctionsProjectModule - relative paths", async () => {
  const tempDir = await Deno.makeTempDir();
  try {
    const project = new Project(tempDir);

    const codeRelPath = project.sharedFunctions.sharedFuncCodeRelativePath(
      "aws-create",
      FunctionKind.Action,
    );
    assertEquals(codeRelPath.path, "actions/aws-create.ts");

    const metadataRelPath =
      project.sharedFunctions.sharedFuncMetadataRelativePath(
        "aws-create",
        FunctionKind.Action,
      );
    assertEquals(metadataRelPath.path, "actions/aws-create.metadata.json");
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});

Deno.test("SharedFunctionsProjectModule - name normalization", async () => {
  const tempDir = await Deno.makeTempDir();
  try {
    const project = new Project(tempDir);

    // Test that special characters are normalized
    const codePath = project.sharedFunctions.sharedFuncCodePath(
      "aws::create::instance",
      FunctionKind.Action,
    );

    // Should normalize "::" to "-"
    assertEquals(
      codePath.path,
      `${tempDir}/shared-functions/actions/aws--create--instance.ts`,
    );
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});
