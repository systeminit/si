import request from "supertest";
import { SiEntity } from "si-entity";

import { app } from "../../src/app";
import { InferPropertiesRequest } from "../../src/controllers/inferProperties";

describe("controllers", () => {
  describe("inferProperties", () => {
    test("returns the entity directly if there is no callback", async () => {
      const entity = new SiEntity({ entityType: "noCallbacks" });

      const reqData: InferPropertiesRequest = {
        entityType: entity.entityType,
        entity,
        resources: [],
        predecessors: [],
      };
      const res = await request(app)
        .post("/inferProperties")
        .send(reqData)
        .set("Accept", "application/json");
      expect(res.status).toBe(200);
      expect(res.body.entity).toEqual({ ...entity });
    });

    test("returns 400 if the registry entry doesn't exist", async () => {
      const entity = new SiEntity({ entityType: "snopes" });

      const reqData: InferPropertiesRequest = {
        entityType: entity.entityType,
        entity,
        resources: [],
        predecessors: [],
      };
      const res = await request(app)
        .post("/inferProperties")
        .send(reqData)
        .set("Accept", "application/json");
      expect(res.status).toBe(400);
      expect(res.body).toEqual({
        code: 400,
        message: "Cannot find registry entry for snopes",
      });
    });

    test("returns the changed entity", async () => {
      const entity = new SiEntity({ entityType: "leftHandPath" });

      const reqData: InferPropertiesRequest = {
        entityType: entity.entityType,
        entity,
        resources: [],
        predecessors: [],
      };
      const res = await request(app)
        .post("/inferProperties")
        .send(reqData)
        .set("Accept", "application/json");
      expect(res.status).toBe(200);
      expect(res.body).toEqual({
        entity: {
          ...entity,
          ops: expect.arrayContaining([
            expect.objectContaining({
              op: "set",
              path: ["simpleString"],
              source: "inferred",
              system: "baseline",
              value: "chunky",
            }),
          ]),
          properties: expect.objectContaining({
            baseline: expect.objectContaining({
              simpleString: "chunky",
            }),
          }),
        },
      });
    });
  });
});
