import {
  FuncsApi,
  SchemasApi,
  SchemaVariantFunc,
} from "@systeminit/api-client";
import { ApiContext } from "../../../api.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "../../../config.ts";
import { Context } from "../../../context.ts";
import { FunctionMetadata, SchemaMetadata } from "../../../generators.ts";
import * as materialize from "../../../materialize.ts";
import { getLogger } from "../../../logger.ts";
import { AbsoluteFilePath, Project } from "../../../project.ts";

const logger = getLogger();

export async function callRemoteSchemaPull(
  ctx: Context,
  project: Project,
  apiCtx: ApiContext,
  schemaNames: string[],
): Promise<PullFullSchemaResult[]> {
  logger.info("Pulling remote schemas to local system");
  logger.info("---");
  logger.info("");

  const api = {
    schemas: new SchemasApi(apiCtx.config),
    funcs: new FuncsApi(apiCtx.config),
  };

  const changeSetCoord = {
    workspaceId: apiCtx.workspace.id,
    changeSetId: "HEAD",
  };

  const results = [];
  const pulledSchemaNames = [];
  const notFoundSchemaNames = [];
  for (const schemaName of schemaNames) {
    const result = await pullSchemaByName(
      project,
      api,
      changeSetCoord,
      schemaName,
    );
    if (result) {
      results.push(result);
      pulledSchemaNames.push(schemaName);
    } else {
      notFoundSchemaNames.push(schemaName);
    }
  }

  ctx.analytics.trackEvent("remote_schema_pull", {
    schemaNames: pulledSchemaNames,
    notFoundSchemaNames,
  });

  // TODO(fnichol): If some schemas were not found, should we error out? At the
  // moment we aren't

  logger.info("");
  logger.info("---");
  logger.info("Successfully pulled schemas:");
  for (const schemaName of pulledSchemaNames) {
    logger.info(` - ${schemaName}`);
  }

  return results;
}

type Api = {
  schemas: SchemasApi;
  funcs: FuncsApi;
};

type SchemaAndVariantData = {
  schema: {
    id: string;
    name: string;
  };
  variant: {
    id: string;
    displayName: string;
    description?: string | null;
    category: string;
    link?: string | null;
  };
  func: {
    schemaId: string;
    actionIds: string[];
    codegenIds: string[];
    managementIds: string[];
    qualificationIds: string[];
  };
};

type ChangeSetCoordinate = {
  workspaceId: string;
  changeSetId: string;
};

type FuncCoord = ChangeSetCoordinate & {
  funcId: string;
};

async function pullSchemaByName(
  project: Project,
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  schemaName: string,
): Promise<PullFullSchemaResult | undefined> {
  logger.info("Pulling schema {schemaName}", { schemaName });

  const data = await getSchemaAndVariantBySchemaName(
    api,
    changeSetCoord,
    schemaName,
  );

  if (!data) {
    // TODO(fnichol): log error, throw, or other here?
    return undefined;
  }

  const formatVersionBody = SCHEMA_FILE_FORMAT_VERSION.toString();

  const metadata = schemaMetadata(data);
  const metadataBody = JSON.stringify(metadata, null, 2);

  const { codeBody } = await fetchFuncMetadataAndCode(api, {
    funcId: data.func.schemaId,
    ...changeSetCoord,
  });

  await materialize.materializeSchemaBase(project, schemaName);
  const { formatVersionPath } = await materialize
    .materializeSchemaFormatVersion(
      project,
      schemaName,
      formatVersionBody,
    );
  const { metadataPath, codePath } = await materialize.materializeSchema(
    project,
    schemaName,
    metadataBody,
    codeBody,
  );

  // Render all action funcs
  if (data.func.actionIds.length > 0) {
    await materialize.materializeSchemaActionBase(project, schemaName);
  }
  const actionPaths = [];
  for (const funcId of data.func.actionIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeSchemaAction(
      project,
      schemaName,
      metadata.name,
      metadataBody,
      codeBody,
    );
    actionPaths.push(paths);
  }

  // Render all codegen funcs
  if (data.func.codegenIds.length > 0) {
    await materialize.materializeSchemaCodegenBase(project, schemaName);
  }
  const codegenPaths = [];
  for (const funcId of data.func.codegenIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeSchemaCodegen(
      project,
      schemaName,
      metadata.name,
      metadataBody,
      codeBody,
    );
    codegenPaths.push(paths);
  }

  // Render all management funcs
  if (data.func.managementIds.length > 0) {
    await materialize.materializeSchemaManagementBase(project, schemaName);
  }
  const managementPaths = [];
  for (const funcId of data.func.managementIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeSchemaManagement(
      project,
      schemaName,
      metadata.name,
      metadataBody,
      codeBody,
    );
    managementPaths.push(paths);
  }

  // Render all qualification funcs
  if (data.func.qualificationIds.length > 0) {
    await materialize.materializeSchemaQualificationBase(project, schemaName);
  }
  const qualificationPaths = [];
  for (const funcId of data.func.qualificationIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeSchemaQualification(
      project,
      schemaName,
      metadata.name,
      metadataBody,
      codeBody,
    );
    qualificationPaths.push(paths);
  }

  return {
    formatVersionPath,
    metadataPath,
    codePath,
    actionPaths,
    codegenPaths,
    managementPaths,
    qualificationPaths,
  };
}

async function fetchFuncMetadataAndCode(
  api: Api,
  funcCoord: FuncCoord,
): Promise<{ metadata: FunctionMetadata; codeBody: string }> {
  const { data: func } = await api.funcs.getFunc({ ...funcCoord });

  return {
    metadata: {
      name: func.name,
      displayName: func.displayName,
      description: func.description,
    },
    codeBody: func.code,
  };
}

async function getSchemaAndVariantBySchemaName(
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  schemaName: string,
): Promise<SchemaAndVariantData | undefined> {
  const response = await api.schemas.findSchema(
    {
      schema: schemaName,
      ...changeSetCoord,
    },
    {
      // Not found is an expected response
      validateStatus: (status) =>
        (status >= 200 && status < 300) || status == 404,
    },
  );
  const { status: schemaStatus, data: schema } = response;

  if (schemaStatus == 404) {
    // TODO(fnichol): log and return or throw?
    logger.error("remote schema not found named {schemaName}", {
      schemaName,
    });
    return undefined;
  }

  const schemaCoord = {
    schemaId: schema.schemaId,
    ...changeSetCoord,
  };

  const { data: variant } = await api.schemas.getDefaultVariant({
    ...schemaCoord,
  });
  // TODO(fnichol): handle HTTP/202 to try again as variant could be building

  const funcs = new SchemaVariantFuncs(variant.variantFuncs);

  return {
    schema: {
      id: schema.schemaId,
      name: schema.schemaName,
    },
    variant: {
      id: variant.variantId,
      displayName: variant.displayName,
      description: variant.description,
      category: variant.category,
      link: variant.link,
    },
    func: {
      schemaId: variant.assetFuncId,
      actionIds: funcs.actionIds(),
      codegenIds: funcs.codegenIds(),
      managementIds: funcs.managementIds(),
      qualificationIds: funcs.qualificationIds(),
    },
  };
}

function schemaMetadata(data: SchemaAndVariantData): SchemaMetadata {
  return {
    name: data.schema.name,
    category: data.variant.category,
    description: data.variant.description,
    documentation: data.variant.link,
  };
}

class SchemaVariantFuncs {
  constructor(public readonly funcs: SchemaVariantFunc[]) {}

  public actionFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter((svf) => svf.funcKind.kind === "action");
  }

  public actionIds(): string[] {
    return this.actionFuncs().map((func) => func.id);
  }

  public codegenFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter(
      (svf) =>
        svf.funcKind.kind === "other" &&
        svf.funcKind.funcKind === "CodeGeneration",
    );
  }

  public codegenIds(): string[] {
    return this.codegenFuncs().map((func) => func.id);
  }

  public managementFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter((svf) => svf.funcKind.kind === "management");
  }

  public managementIds(): string[] {
    return this.managementFuncs().map((func) => func.id);
  }

  public qualificationFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter(
      (svf) =>
        svf.funcKind.kind === "other" &&
        svf.funcKind.funcKind === "Qualification",
    );
  }

  public qualificationIds(): string[] {
    return this.qualificationFuncs().map((func) => func.id);
  }
}

export interface PullFullSchemaResult {
  formatVersionPath: AbsoluteFilePath;
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
  actionPaths: { metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }[];
  codegenPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  managementPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  qualificationPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
}
