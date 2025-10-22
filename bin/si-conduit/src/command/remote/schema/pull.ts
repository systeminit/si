import { FuncsApi, SchemasApi } from "@systeminit/api-client";
import { ApiContext } from "../../../api.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "../../../config.ts";
import { Context } from "../../../context.ts";
import { SchemaMetadata } from "../../../generators.ts";
import * as materialize from "../../../materialize.ts";
import { getLogger } from "../../../logger.ts";
import { Project } from "../../../project.ts";

const logger = getLogger();

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
  };
};

type ChangeSetCoordinate = {
  workspaceId: string;
  changeSetId: string;
};

type FuncCoord = ChangeSetCoordinate & {
  funcId: string;
};

export async function callRemoteSchemaPull(
  ctx: Context,
  project: Project,
  apiCtx: ApiContext,
  schemaNames: string[],
) {
  const api = {
    schemas: new SchemasApi(apiCtx.config),
    funcs: new FuncsApi(apiCtx.config),
  };

  const changeSetCoord = {
    workspaceId: apiCtx.workspace.id,
    changeSetId: "HEAD",
  };

  for (const schemaName of schemaNames) {
    await pullSchemaByName(ctx, project, api, changeSetCoord, schemaName);
  }
}

async function pullSchemaByName(
  ctx: Context,
  project: Project,
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  schemaName: string,
) {
  const data = await getSchemaAndVariantBySchemaName(
    api,
    changeSetCoord,
    schemaName,
  );

  if (!data) {
    // FIXME: log error, throw, or other here?
    return;
  }

  const formatVersionBody = SCHEMA_FILE_FORMAT_VERSION.toString();

  const metadata = schemaMetadata(data);
  const metadataBody = JSON.stringify(metadata, null, 2);

  const codeBody = await fetchFuncCodeBody(api, {
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

  await materialize.materializeSchemaActionBase(project, schemaName);
}

async function fetchFuncCodeBody(
  api: Api,
  funcCoord: FuncCoord,
): Promise<string> {
  const { data: func } = await api.funcs.getFunc({ ...funcCoord });

  return func.code;
}

async function getSchemaAndVariantBySchemaName(
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  schemaName: string,
): Promise<SchemaAndVariantData | undefined> {
  const { status: schemaStatus, data: schema } = await api.schemas.findSchema(
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

  if (schemaStatus == 404) {
    // FIXME: log or throw?
    logger.error("schema not found named {schemaName}", { schemaName });
    return;
    // throw new Error(`FIXME: schema not found: ${schemaName}`);
  }

  const schemaCoord = {
    schemaId: schema.schemaId,
    ...changeSetCoord,
  };

  const { data: variant } = await api.schemas.getDefaultVariant({
    ...schemaCoord,
  });
  // FIXME: handle HTTP/202 to try again as variant could be building

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
