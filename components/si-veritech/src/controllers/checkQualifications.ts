import WebSocket from "ws";
import Debug from "debug";
import { SiCtx } from "../siCtx";
const debug = Debug("veritech:controllers:checkQualifications");
import _ from "lodash";

import {
  SiEntity as Entity,
  Resource,
  SiEntity,
  OpSet,
  OpSource,
} from "si-entity";
import {
  Qualification,
  registry,
  allFieldsValidQualification,
  ValidatorKind,
} from "si-registry";

import intel from "../intel";

export interface CheckQualificationsRequest {
  entity: Entity;
  systemId: string;
}

export interface CheckQualificationsItem {
  name: string;
  qualified: boolean;
  output?: string;
  error?: string;
}

export type CheckQualificationCallback = (
  ctx: typeof SiCtx,
  qualification: Qualification,
  request: CheckQualificationsRequest,
) => Promise<CheckQualificationsItem>;

export async function allFieldsValid(
  _ctx: typeof SiCtx,
  q: Qualification,
  r: CheckQualificationsRequest,
): Promise<CheckQualificationsItem> {
  const output: Record<string, any> = {};
  let qualified = true;

  // Check that fields that have values are valid.
  for (const op of r.entity.ops) {
    const result = r.entity.validateProp(op as OpSet);
    const fieldName = op.path.join(".");
    if (result.success) {
      output[fieldName] = "success";
    } else {
      output[fieldName] = result.errors;
      qualified = false;
    }
  }

  // Check that there aren't fields that are required, but don't have values
  const editFields = r.entity.editFields();
  for (const editField of editFields) {
    if (editField.schema.validation) {
      for (const validation of editField.schema.validation) {
        if (validation.kind == ValidatorKind.Required) {
          let hasValue;
          if (editField.schema.type == "array") {
            const fullPath = _.concat([r.entity.entityType], editField.path);
            const arrayMetaLength =
              r.entity.arrayMeta[r.entity.pathToString(fullPath)];
            if (arrayMetaLength && arrayMetaLength.length > 0) {
              hasValue = true;
            } else {
              hasValue = false;
            }
          } else {
            hasValue =
              r.entity.hasValueFrom({
                path: editField.path,
                system: r.systemId,
                source: OpSource.Manual,
              }) ||
              r.entity.hasValueFrom({
                path: editField.path,
                system: r.systemId,
                source: OpSource.Inferred,
              }) ||
              r.entity.hasValueFrom({
                path: editField.path,
                system: r.systemId,
                source: OpSource.Expression,
              }) ||
              r.entity.hasValueFrom({
                path: editField.path,
                system: "baseline",
                source: OpSource.Manual,
              }) ||
              r.entity.hasValueFrom({
                path: editField.path,
                system: "baseline",
                source: OpSource.Inferred,
              }) ||
              r.entity.hasValueFrom({
                path: editField.path,
                system: "baseline",
                source: OpSource.Expression,
              });
          }
          if (hasValue) {
            if (!output[editField.path.join(".")]) {
              output[editField.path.join(".")] = "success";
            }
          } else {
            output[editField.path.join(".")] = [
              { message: `field is required` },
            ];
            qualified = false;
          }
        }
      }
    }
  }

  return {
    name: q.name,
    qualified,
    output: JSON.stringify(output),
  };
}

export async function checkQualifications(
  ws: WebSocket,
  req: string,
): Promise<void> {
  debug("/checkQualifications BEGIN");
  debug("request message: %O", req);
  const request: CheckQualificationsRequest = JSON.parse(req);
  request.entity = SiEntity.fromJson(request.entity);
  const schema = registry[request.entity.entityType];
  let intelFuncs = intel[request.entity.entityType];
  const checkPromises: Promise<void>[] = [];

  if (schema) {
    let qualifications = [allFieldsValidQualification];
    if (schema.qualifications) {
      qualifications = _.concat(qualifications, schema.qualifications);
    }
    const validNames = qualifications.map((q) => q.name);
    ws.send(JSON.stringify({ protocol: { validNames } }));
    if (intelFuncs && intelFuncs.checkQualifications) {
      if (!intelFuncs.checkQualifications[allFieldsValidQualification.name]) {
        intelFuncs.checkQualifications[
          allFieldsValidQualification.name
        ] = allFieldsValid;
      }
    } else {
      intelFuncs = {
        checkQualifications: {
          allFieldsValid: allFieldsValid,
        },
      };
    }
    for (const q of qualifications) {
      if (
        intelFuncs &&
        intelFuncs.checkQualifications &&
        intelFuncs.checkQualifications[q.name]
      ) {
        ws.send(JSON.stringify({ protocol: { start: q.name } }));
        const p = intelFuncs.checkQualifications[q.name](SiCtx, q, request)
          .then((item) => {
            ws.send(
              JSON.stringify({
                protocol: {
                  item,
                },
              }),
            );
          })
          .catch((e) => {
            ws.send(
              JSON.stringify({
                protocol: {
                  item: {
                    name: q.name,
                    qualified: false,
                    error: `${e}`,
                  },
                },
              }),
            );
          });
        checkPromises.push(p);
      } else {
        ws.send(JSON.stringify({ protocol: { start: q.name } }));
        ws.send(
          JSON.stringify({
            protocol: {
              name: q.name,
              qualified: false,
              error: `qualification check named ${q.name} is not implemented!`,
            },
          }),
        );
      }
    }
    await Promise.allSettled(checkPromises)
      .then(() => {
        ws.send(JSON.stringify({ protocol: { finished: null } }));
        ws.close(1000, "finished");
      })
      .catch((e) => debug("got an error trying to finalize things", { e }));
  } else {
    debug("closing, schema not found");
    ws.close(4004, `schema not found for ${request.entity.entityType}; bug!`);
  }
}
