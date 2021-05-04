import WebSocket from "ws";
import Debug from "debug";
import { SiCtx } from "../siCtx";
const debug = Debug("veritech:controllers:checkQualifications");

import { SiEntity as Entity, Resource, SiEntity } from "si-entity";
import { registry } from "si-registry";

import intel from "../intel";

export interface CheckQualificationsRequest {
  entityType: string;
  entity: Entity;
  resources: Resource[];
  predecessors: {
    entity: Entity;
    resources: Resource[];
  }[];
  systemId: string;
}

export interface CheckQualificationsItem {
  name: string;
  qualified: boolean;
  output?: string;
  error?: string;
}

export async function checkQualifications(
  ws: WebSocket,
  req: string,
): Promise<void> {
  debug("/checkQualifications BEGIN");
  debug("request message: %O", req);
  const request: CheckQualificationsRequest = JSON.parse(req);
  request.entity = SiEntity.fromJson(request.entity);
  const schema = registry[request.entityType];
  const intelFuncs = intel[request.entityType];
  const checkPromises: Promise<void>[] = [];
  if (schema) {
    if (schema.qualifications?.length) {
      const validNames = schema.qualifications.map((q) => q.name);
      ws.send(JSON.stringify({ protocol: { validNames } }));
      for (const q of schema.qualifications) {
        if (
          intelFuncs.checkQualifications &&
          intelFuncs.checkQualifications[q.name]
        ) {
          ws.send(JSON.stringify({ protocol: { start: q.name } }));
          const p = intelFuncs.checkQualifications[q.name](SiCtx, q, request)
            .then((item) => {
              debug("returning the message", item);
              ws.send(
                JSON.stringify({
                  protocol: {
                    item,
                  },
                }),
              );
            })
            .catch((e) => {
              debug("failed a check from a throw", e);
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
    }
    await Promise.allSettled(checkPromises)
      .then(() => {
        debug("trying to finish up");
        ws.send(JSON.stringify({ protocol: { finished: null } }));
        ws.close(1000, "finished");
        debug("finished");
      })
      .catch((e) => debug("got an error trying to finalize things", { e }));
  } else {
    debug("closing, schema not found");
    ws.close(4004, `schema not found for ${request.entityType}; bug!`);
  }
}
