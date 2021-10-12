import {
  evaluateInferenceLambda,
  evaluateFrom,
  InferContext,
} from "../src/index";
import { Inference } from "si-inference";
import { OpSource, OpType, SiEntity } from "si-entity";
import { ValueTypeError } from "../src/errors";
import _ from "lodash";

function dockerImageEntity(idNumber: number, name: string): SiEntity {
  const e = new SiEntity({ entityType: "dockerImage" });
  e.name = name;
  e.id = `entity:${idNumber}`;
  return e;
}

function k8sDeploymentEntity(idNumber: number, name: string): SiEntity {
  const e = new SiEntity({ entityType: "k8sDeployment" });
  e.name = name;
  e.id = `entity:${idNumber}`;
  return e;
}

function inferenceTestsEntity(idNumber: number, name: string): SiEntity {
  const e = new SiEntity({ entityType: "inferenceTests" });
  e.name = name;
  e.id = `entity:${idNumber}`;
  return e;
}

describe("evaluateFrom", () => {
  describe("single selection", () => {
    test("no system", () => {
      const entity = dockerImageEntity(0, "opeth");
      entity.computeProperties();
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const context = [{ entity }];
      const targetEntity = dockerImageEntity(1, "poop");
      const { dataResult } = evaluateFrom(inference, targetEntity, context);
      expect(dataResult).toEqual({
        baseline: expect.arrayContaining([
          { entityId: "entity:0", properties: {} },
        ]),
      });
    });
    test("single system", () => {
      const entity = dockerImageEntity(0, "opeth");
      entity.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "nginx",
      });
      entity.computeProperties();
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const context = [{ entity }];
      const targetEntity = dockerImageEntity(1, "poop");
      const { dataResult } = evaluateFrom(inference, targetEntity, context);
      expect(dataResult).toEqual({
        baseline: expect.arrayContaining([
          { entityId: "entity:0", properties: { image: "nginx" } },
        ]),
      });
    });

    test("multi system", () => {
      const entity = dockerImageEntity(0, "opeth");
      entity.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "nginx",
      });
      entity.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "postgres",
      });
      entity.computeProperties();
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const context = [{ entity }];
      const targetEntity = dockerImageEntity(1, "poop");
      const { dataResult } = evaluateFrom(inference, targetEntity, context);
      expect(dataResult).toEqual({
        baseline: [{ entityId: "entity:0", properties: { image: "nginx" } }],
        "system:a": [
          { entityId: "entity:0", properties: { image: "postgres" } },
        ],
      });
    });

    test("multi system, complex from", () => {
      const entity = dockerImageEntity(0, "opeth");
      entity.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "nginx",
      });
      entity.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "postgres",
      });
      entity.addOpSet({
        op: OpType.Set,
        path: ["ExposedPorts", "0"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "80/tcp",
      });

      entity.computeProperties();
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: [
              { path: ["image"] },
              { name: true },
              { path: ["ExposedPorts"] },
            ],
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const context = [{ entity }];
      const targetEntity = dockerImageEntity(1, "poop");
      const { dataResult } = evaluateFrom(inference, targetEntity, context);
      expect(dataResult).toEqual({
        baseline: [
          {
            name: "opeth",
            entityId: "entity:0",
            properties: { image: "nginx" },
          },
        ],
        "system:a": [
          {
            name: "opeth",
            entityId: "entity:0",
            properties: { image: "postgres", ExposedPorts: ["80/tcp"] },
          },
        ],
      });
    });
  });
  describe("multi selection", () => {
    test("single system", () => {
      const opeth = dockerImageEntity(0, "opeth");
      opeth.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "nginx",
      });
      opeth.computeProperties();
      const cinderella = dockerImageEntity(1, "cinderella");
      cinderella.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "smoothOperator",
      });
      cinderella.computeProperties();

      const inference: Inference = {
        name: "tool",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const context = [{ entity: opeth }, { entity: cinderella }];
      const targetEntity = dockerImageEntity(1, "poop");
      const { dataResult } = evaluateFrom(inference, targetEntity, context);
      expect(dataResult).toEqual({
        baseline: [
          { entityId: "entity:0", properties: { image: "nginx" } },
          { entityId: "entity:1", properties: { image: "smoothOperator" } },
        ],
      });
    });

    test("multi system", () => {
      const opeth = dockerImageEntity(0, "opeth");
      opeth.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "nginx",
      });
      opeth.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "postgres",
      });

      opeth.computeProperties();

      const cinderella = dockerImageEntity(1, "cinderella");
      cinderella.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "smoothOperator",
      });
      cinderella.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "carlosSainz",
      });
      cinderella.computeProperties();

      const inference: Inference = {
        name: "tool",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const context = [{ entity: opeth }, { entity: cinderella }];
      const targetEntity = dockerImageEntity(1, "poop");
      const { dataResult } = evaluateFrom(inference, targetEntity, context);
      expect(dataResult).toEqual({
        baseline: [
          { entityId: "entity:0", properties: { image: "nginx" } },
          { entityId: "entity:1", properties: { image: "smoothOperator" } },
        ],
        "system:a": [
          { entityId: "entity:0", properties: { image: "postgres" } },
          { entityId: "entity:1", properties: { image: "carlosSainz" } },
        ],
      });
    });

    test("multi system, complex from", () => {
      const opeth = dockerImageEntity(0, "opeth");
      opeth.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "nginx",
      });
      opeth.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "postgres",
      });
      opeth.addOpSet({
        op: OpType.Set,
        path: ["ExposedPorts", "0"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "80/tcp",
      });
      opeth.computeProperties();

      const cinderella = dockerImageEntity(1, "cinderella");
      cinderella.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Manual,
        system: "baseline",
        value: "smoothOperator",
      });
      cinderella.addOpSet({
        op: OpType.Set,
        path: ["image"],
        source: OpSource.Inferred,
        system: "system:a",
        value: "carlosSainz",
      });
      cinderella.computeProperties();

      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: [
              { path: ["image"] },
              { name: true },
              { path: ["ExposedPorts"] },
            ],
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const context = [{ entity: opeth }, { entity: cinderella }];
      const targetEntity = dockerImageEntity(1, "poop");
      const { dataResult } = evaluateFrom(inference, targetEntity, context);
      expect(dataResult).toEqual({
        baseline: [
          {
            name: "opeth",
            entityId: "entity:0",
            properties: { image: "nginx" },
          },
          {
            name: "cinderella",
            entityId: "entity:1",
            properties: { image: "smoothOperator" },
          },
        ],
        "system:a": [
          {
            name: "opeth",
            entityId: "entity:0",
            properties: { image: "postgres", ExposedPorts: ["80/tcp"] },
          },
          {
            name: "cinderella",
            entityId: "entity:1",
            properties: { image: "carlosSainz" },
          },
        ],
      });
    });
  });

  describe("targetEntity", () => {
    test("includes the targetEntity", () => {
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            targetEntity: true,
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const inferContext = [
        {
          entity: dockerImageEntity(0, "opeth"),
          secret: {},
        },
      ];
      const targetEntity = dockerImageEntity(1, "slinky");
      const test = _.cloneDeep(targetEntity);
      test.isTarget = true;
      const { inputs } = evaluateFrom(inference, targetEntity, inferContext);
      expect(inputs).toContainEqual(test);
    });
  });
  describe("entityType", () => {
    test("valid selector", () => {
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const inferContext = [
        {
          entity: dockerImageEntity(0, "opeth"),
          secret: {},
        },
        {
          entity: dockerImageEntity(1, "vanHalen"),
          secret: {},
        },
        {
          entity: k8sDeploymentEntity(2, "abeFromanSausageKingOfChicago"),
          secret: {},
        },
      ];
      const targetEntity = dockerImageEntity(1, "slinky");
      const { inputs } = evaluateFrom(inference, targetEntity, inferContext);
      expect(inputs).toContainEqual(inferContext[0].entity);
      expect(inputs).toContainEqual(inferContext[1].entity);
      expect(inputs).not.toContainEqual(inferContext[2].entity);
    });

    test("no valid selection", () => {
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityType: "poopCanoe",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const inferContext = [
        {
          entity: dockerImageEntity(0, "opeth"),
          secret: {},
        },
        {
          entity: dockerImageEntity(1, "vanHalen"),
          secret: {},
        },
        {
          entity: k8sDeploymentEntity(2, "abeFromanSausageKingOfChicago"),
          secret: {},
        },
      ];
      const targetEntity = dockerImageEntity(1, "slinky");
      const { inputs } = evaluateFrom(inference, targetEntity, inferContext);
      expect(inputs.length).toBe(0);
    });
  });

  describe("entityId", () => {
    test("valid selector", () => {
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityId: "entity:0",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const inferContext = [
        {
          entity: dockerImageEntity(0, "opeth"),
          secret: {},
        },
        {
          entity: dockerImageEntity(1, "vanHalen"),
          secret: {},
        },
        {
          entity: k8sDeploymentEntity(2, "abeFromanSausageKingOfChicago"),
          secret: {},
        },
      ];
      const targetEntity = dockerImageEntity(1, "slinky");
      const { inputs } = evaluateFrom(inference, targetEntity, inferContext);
      expect(inputs).toContainEqual(inferContext[0].entity);
      expect(inputs).not.toContainEqual(inferContext[1].entity);
      expect(inputs).not.toContainEqual(inferContext[2].entity);
    });

    test("no valid selection", () => {
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityId: "entity:22",
            data: { path: ["image"] },
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const inferContext = [
        {
          entity: dockerImageEntity(0, "opeth"),
          secret: {},
        },
        {
          entity: dockerImageEntity(1, "vanHalen"),
          secret: {},
        },
        {
          entity: k8sDeploymentEntity(2, "abeFromanSausageKingOfChicago"),
          secret: {},
        },
      ];
      const targetEntity = dockerImageEntity(1, "slinky");
      const { inputs } = evaluateFrom(inference, targetEntity, inferContext);
      expect(inputs.length).toBe(0);
    });
  });

  describe("combo selector", () => {
    test("entityId, entityType, and targetEntity", () => {
      const inference: Inference = {
        name: "opeth",
        kind: "lambda",
        from: [
          {
            entityId: "entity:2",
            data: { name: true },
          },
          {
            entityType: "dockerImage",
            data: [{ path: ["image"] }, { name: true }],
          },
          {
            targetEntity: true,
            data: [{ name: true }],
          },
        ],
        to: { path: ["image"] },
        code: "return data.image",
      };
      const inferContext = [
        {
          entity: dockerImageEntity(0, "opeth"),
          secret: {},
        },
        {
          entity: dockerImageEntity(1, "vanHalen"),
          secret: {},
        },
        {
          entity: k8sDeploymentEntity(2, "abeFromanSausageKingOfChicago"),
          secret: {},
        },
        {
          entity: k8sDeploymentEntity(3, "lobo"),
          secret: {},
        },
      ];
      const targetEntity = dockerImageEntity(1, "slinky");
      const targetResult = _.cloneDeep(targetEntity);
      targetResult.isTarget = true;
      const { inputs } = evaluateFrom(inference, targetEntity, inferContext);
      expect(inputs).toContainEqual(inferContext[0].entity);
      expect(inputs).toContainEqual(inferContext[1].entity);
      expect(inputs).toContainEqual(inferContext[2].entity);
      expect(inputs).toContainEqual(targetResult);
      expect(inputs).not.toContainEqual(inferContext[3].entity);
    });
  });
});

describe("evaluateInference", () => {
  describe("lambda", () => {
    test("fails when inference 'to' and schema do not match", () => {
      const opeth = dockerImageEntity(0, "opeth");
      opeth.computeProperties();

      const context: InferContext = [
        {
          entity: opeth,
        },
      ];

      const targetEntity = dockerImageEntity(1, "cinderella");
      targetEntity.computeProperties();

      const inference: Inference = {
        name: "failsToWork",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { name: true },
          },
        ],
        to: { path: ["snake"] },
        code: "firstEntity.name",
      };

      expect(() =>
        evaluateInferenceLambda(inference, targetEntity, context),
      ).toThrowError();
    });

    test("fails when inference 'from' and schema do not match", () => {
      const opeth = dockerImageEntity(0, "opeth");
      opeth.computeProperties();

      const context: InferContext = [
        {
          entity: opeth,
        },
      ];

      const targetEntity = dockerImageEntity(1, "cinderella");
      targetEntity.computeProperties();

      const inference: Inference = {
        name: "failsToWork",
        kind: "lambda",
        from: [
          {
            entityType: "dockerImage",
            data: { path: ["snake"] },
          },
        ],
        to: { path: ["image"] },
        code: "firstEntity.properties.snake",
      };

      expect(() =>
        evaluateInferenceLambda(inference, targetEntity, context),
      ).toThrowError();
    });

    describe("targetEntity, single system", () => {
      test("name result", () => {
        const context: InferContext = [];

        const targetEntity = dockerImageEntity(1, "cinderella");
        targetEntity.addOpSet({
          op: OpType.Set,
          path: ["image"],
          source: OpSource.Manual,
          system: "baseline",
          value: "nginx",
        });
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "imageToName",
          kind: "lambda",
          from: [
            {
              targetEntity: true,
              data: { path: ["image"] },
            },
          ],
          to: { name: true },
          code: "firstEntity.properties.image",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        // Shouldn't work, because the name is not `si-`
        expect(targetEntity.name).not.toBe("nginx");

        targetEntity.name = "si-6954";
        evaluateInferenceLambda(inference, targetEntity, context);
        expect(targetEntity.name).toBe("nginx");
      });
    });

    describe("single target entity, single system", () => {
      test("name result", () => {
        const opeth = dockerImageEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["image"],
          source: OpSource.Manual,
          system: "baseline",
          value: "nginx",
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = dockerImageEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "imageToImage",
          kind: "lambda",
          from: [
            {
              entityType: "dockerImage",
              data: { path: ["image"] },
            },
          ],
          to: { name: true },
          code: "firstEntity.properties.image",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        // Shouldn't work, because the name is not `si-`
        expect(targetEntity.name).not.toBe("nginx");

        targetEntity.name = "si-6954";
        evaluateInferenceLambda(inference, targetEntity, context);
        expect(targetEntity.name).toBe("nginx");
      });

      test("name error", () => {
        const opeth = dockerImageEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["image"],
          source: OpSource.Manual,
          system: "baseline",
          value: "nginx",
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = dockerImageEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "imageToImage",
          kind: "lambda",
          from: [
            {
              entityType: "dockerImage",
              data: { path: ["image"] },
            },
          ],
          to: { name: true },
          code: "f = { i: firstEntity.properties.image }",
        };

        expect(() =>
          evaluateInferenceLambda(inference, targetEntity, context),
        ).toThrowError();
      });

      test("string result", () => {
        const opeth = dockerImageEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["image"],
          source: OpSource.Manual,
          system: "baseline",
          value: "nginx",
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = dockerImageEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "imageToImage",
          kind: "lambda",
          from: [
            {
              entityType: "dockerImage",
              data: { path: ["image"] },
            },
          ],
          to: { path: ["image"] },
          code: "firstEntity.properties.image",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["image"],
          value: "nginx",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "dockerImage",
              },
            ],
            inference,
          },
        });
      });

      test("string error", () => {
        const opeth = dockerImageEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["image"],
          source: OpSource.Manual,
          system: "baseline",
          value: "nginx",
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = dockerImageEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "imageToImage",
          kind: "lambda",
          from: [
            {
              entityType: "dockerImage",
              data: { path: ["image"] },
            },
          ],
          to: { path: ["image"] },
          code: "f = { i: firstEntity.properties.image }",
        };

        expect(() =>
          evaluateInferenceLambda(inference, targetEntity, context),
        ).toThrowError();
      });

      test("number result", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["numberType"],
          source: OpSource.Manual,
          system: "baseline",
          value: "42",
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "numberToNumber",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { path: ["numberType"] },
            },
          ],
          to: { path: ["numberType"] },
          code: "firstEntity.properties.numberType",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["numberType"],
          value: "42",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference,
          },
        });

        // Prove this works when the inference function returns a
        // javascript number, not just a string of a number
        const inferenceNumber: Inference = {
          name: "numberToNumber",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { path: ["numberType"] },
            },
          ],
          to: { path: ["numberType"] },
          code: "42",
        };
        evaluateInferenceLambda(inferenceNumber, targetEntity, context);

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["numberType"],
          value: "42",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference: inferenceNumber,
          },
        });
      });

      test("number error", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["numberType"],
          source: OpSource.Manual,
          system: "baseline",
          value: "42",
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "numberToNumber",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { path: ["numberType"] },
            },
          ],
          to: { path: ["numberType"] },
          code: "'notANumber'",
        };

        expect(() =>
          evaluateInferenceLambda(inference, targetEntity, context),
        ).toThrowError();
      });

      test("boolean result", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["booleanType"],
          source: OpSource.Manual,
          system: "baseline",
          value: false,
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "booleanToBoolean",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { path: ["booleanType"] },
            },
          ],
          to: { path: ["booleanType"] },
          code: "firstEntity.properties.booleanType",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["booleanType"],
          value: false,
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference,
          },
        });
      });

      test("boolean error", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.addOpSet({
          op: OpType.Set,
          path: ["booleanType"],
          source: OpSource.Manual,
          system: "baseline",
          value: false,
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "booleanToBoolean",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { path: ["booleanType"] },
            },
          ],
          to: { path: ["booleanType"] },
          code: "guns",
        };

        expect(() =>
          evaluateInferenceLambda(inference, targetEntity, context),
        ).toThrowError();
      });

      describe("map", () => {
        test("scalar values result", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.addOpSet({
            op: OpType.Set,
            path: ["mapWithStringValues", "metallica"],
            source: OpSource.Manual,
            system: "baseline",
            value: "the thing that should not be",
          });
          opeth.addOpSet({
            op: OpType.Set,
            path: ["mapWithStringValues", "madness"],
            source: OpSource.Manual,
            system: "baseline",
            value: "you dwell",
          });
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "mapToMap",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { path: ["mapWithStringValues"] },
              },
            ],
            to: { path: ["mapWithStringValues"] },
            code: "firstEntity.properties.mapWithStringValues",
          };

          evaluateInferenceLambda(inference, targetEntity, context);

          const metallicaValueInference = _.cloneDeep(inference);
          metallicaValueInference.to.extraPath = ["metallica"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["mapWithStringValues", "metallica"],
            value: "the thing that should not be",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: metallicaValueInference,
            },
          });
        });

        test("scalar values error", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "mapToMap",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { name: true },
              },
            ],
            to: { path: ["mapWithStringValues"] },
            code: "m = { 'metallica': { 'should not': 'take an object' } }",
          };

          expect(() =>
            evaluateInferenceLambda(inference, targetEntity, context),
          ).toThrowError();
        });

        test("object values result", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.addOpSet({
            op: OpType.Set,
            path: ["mapWithObjectValues", "metallica", "stringValue"],
            source: OpSource.Manual,
            system: "baseline",
            value: "the thing that should not be",
          });
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "mapToMapObject",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { path: ["mapWithObjectValues"] },
              },
            ],
            to: { path: ["mapWithObjectValues"] },
            code: "firstEntity.properties.mapWithObjectValues",
          };

          evaluateInferenceLambda(inference, targetEntity, context);

          const metallicaValueInference = _.cloneDeep(inference);
          metallicaValueInference.to.extraPath = ["metallica", "stringValue"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["mapWithObjectValues", "metallica", "stringValue"],
            value: "the thing that should not be",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: metallicaValueInference,
            },
          });
        });
      });

      describe("object", () => {
        test("scalar values result", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.addOpSet({
            op: OpType.Set,
            path: ["objectWithScalarValues", "stringValue"],
            source: OpSource.Manual,
            system: "baseline",
            value: "slayer",
          });
          opeth.addOpSet({
            op: OpType.Set,
            path: ["objectWithScalarValues", "numberValue"],
            source: OpSource.Manual,
            system: "baseline",
            value: "42",
          });
          opeth.addOpSet({
            op: OpType.Set,
            path: ["objectWithScalarValues", "booleanValue"],
            source: OpSource.Manual,
            system: "baseline",
            value: true,
          });
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "objectToObject",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { path: ["objectWithScalarValues"] },
              },
            ],
            to: { path: ["objectWithScalarValues"] },
            code: "firstEntity.properties.objectWithScalarValues",
          };

          evaluateInferenceLambda(inference, targetEntity, context);

          const stringValueInference = _.cloneDeep(inference);
          stringValueInference.to.extraPath = ["stringValue"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["objectWithScalarValues", "stringValue"],
            value: "slayer",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: stringValueInference,
            },
          });

          const numberValueInference = _.cloneDeep(inference);
          numberValueInference.to.extraPath = ["numberValue"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["objectWithScalarValues", "numberValue"],
            value: "42",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: numberValueInference,
            },
          });

          const booleanValueInference = _.cloneDeep(inference);
          booleanValueInference.to.extraPath = ["booleanValue"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["objectWithScalarValues", "booleanValue"],
            value: true,
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: booleanValueInference,
            },
          });
        });
        test("scalar values result via raw data", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "objectToObject",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { name: true },
              },
            ],
            to: { path: ["objectWithScalarValues"] },
            code: "v = { stringValue: 'slayer', booleanValue: true, numberValue: '42' }",
          };

          evaluateInferenceLambda(inference, targetEntity, context);

          const stringValueInference = _.cloneDeep(inference);
          stringValueInference.to.extraPath = ["stringValue"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["objectWithScalarValues", "stringValue"],
            value: "slayer",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: stringValueInference,
            },
          });

          const numberValueInference = _.cloneDeep(inference);
          numberValueInference.to.extraPath = ["numberValue"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["objectWithScalarValues", "numberValue"],
            value: "42",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: numberValueInference,
            },
          });

          const booleanValueInference = _.cloneDeep(inference);
          booleanValueInference.to.extraPath = ["booleanValue"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["objectWithScalarValues", "booleanValue"],
            value: true,
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: booleanValueInference,
            },
          });
        });
        test("errors on bad return data", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "objectToObject",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { name: true },
              },
            ],
            to: { path: ["objectWithScalarValues"] },
            code: "v = { fakeButt: 'p', stringValue: 'slayer', booleanValue: true, numberValue: '42' }",
          };
          expect(() =>
            evaluateInferenceLambda(inference, targetEntity, context),
          ).toThrowError();
        });
      });

      describe("array", () => {
        test("scalar values", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.addOpSet({
            op: OpType.Set,
            path: ["arrayWithStringValues", "0"],
            source: OpSource.Manual,
            system: "baseline",
            value: "slayer",
          });
          opeth.addOpSet({
            op: OpType.Set,
            path: ["arrayWithStringValues", "1"],
            source: OpSource.Manual,
            system: "baseline",
            value: "neurosis",
          });
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "arrayToArray",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { path: ["arrayWithStringValues"] },
              },
            ],
            to: { path: ["arrayWithStringValues"] },
            code: "firstEntity.properties.arrayWithStringValues",
          };

          evaluateInferenceLambda(inference, targetEntity, context);

          const slayerValueInference = _.cloneDeep(inference);
          slayerValueInference.to.extraPath = ["0"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["arrayWithStringValues", "0"],
            value: "slayer",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: slayerValueInference,
            },
          });
          const neurosisValueInference = _.cloneDeep(inference);
          neurosisValueInference.to.extraPath = ["1"];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["arrayWithStringValues", "1"],
            value: "neurosis",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: neurosisValueInference,
            },
          });
        });

        test("bad values", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "arrayToArray",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { path: ["arrayWithStringValues"] },
              },
            ],
            to: { path: ["arrayWithStringValues"] },
            code: "f = { p: 0 }",
          };
          expect(() =>
            evaluateInferenceLambda(inference, targetEntity, context),
          ).toThrowError();
        });
      });
      describe("complex object", () => {
        test("full insanity", () => {
          const opeth = inferenceTestsEntity(0, "opeth");
          opeth.addOpSet({
            op: OpType.Set,
            path: ["complexArray", "0", "nestedArray0", "0", "0", "slayer"],
            source: OpSource.Manual,
            system: "baseline",
            value: "reign in blood",
          });
          opeth.addOpSet({
            op: OpType.Set,
            path: ["complexArray", "0", "nestedArray0", "1", "0", "neurosis"],
            source: OpSource.Manual,
            system: "baseline",
            value: "times of grace",
          });
          opeth.addOpSet({
            op: OpType.Set,
            path: ["complexArray", "1", "nestedArray0", "0", "0", "skid row"],
            source: OpSource.Manual,
            system: "baseline",
            value: "slave to the grind",
          });
          opeth.computeProperties();

          const context: InferContext = [
            {
              entity: opeth,
            },
          ];

          const targetEntity = inferenceTestsEntity(1, "cinderella");
          targetEntity.computeProperties();

          const inference: Inference = {
            name: "complexToComplex",
            kind: "lambda",
            from: [
              {
                entityType: "inferenceTests",
                data: { path: ["complexArray"] },
              },
            ],
            to: { path: ["complexArray"] },
            code: "firstEntity.properties.complexArray",
          };

          evaluateInferenceLambda(inference, targetEntity, context);

          const slayerValueInference = _.cloneDeep(inference);
          slayerValueInference.to.extraPath = [
            "0",
            "nestedArray0",
            "0",
            "0",
            "slayer",
          ];
          expect(targetEntity.ops).toContainEqual({
            op: OpType.Set,
            source: OpSource.Inferred,
            path: ["complexArray", "0", "nestedArray0", "0", "0", "slayer"],
            value: "reign in blood",
            system: "baseline",
            provenance: {
              context: [
                {
                  id: "entity:0",
                  entityType: "inferenceTests",
                },
              ],
              inference: slayerValueInference,
            },
          });
        });
      });
    });

    describe("cleans up after stale inference", () => {
      test("previous string result, now no result at all", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "stringWithValue",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { name: true },
            },
          ],
          to: { path: ["stringType"] },
          code: "firstEntity.name",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["stringType"],
          value: "opeth",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference: inference,
          },
        });

        const emptyContext: InferContext = [];

        evaluateInferenceLambda(inference, targetEntity, emptyContext);

        expect(targetEntity.ops).not.toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["stringType"],
          value: "opeth",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference: inference,
          },
        });
      });

      test("previous array result, now no result at all", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.updateFromOps({
          setOps: [
            {
              op: OpType.Set,
              path: ["arrayWithStringValues", "0"],
              source: OpSource.Manual,
              system: "baseline",
              value: "slave to the grind",
            },
            {
              op: OpType.Set,
              path: ["arrayWithStringValues", "1"],
              source: OpSource.Manual,
              system: "baseline",
              value: "guns n roses",
            },
          ],
        });
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "arrayWithStringValues",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { path: ["arrayWithStringValues"] },
            },
          ],
          to: { path: ["arrayWithStringValues"] },
          code: "firstEntity?.properties?.arrayWithStringValues;",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        const skidRowInference = _.cloneDeep(inference);
        skidRowInference.to.extraPath = ["0"];

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["arrayWithStringValues", "0"],
          value: "slave to the grind",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference: skidRowInference,
          },
        });

        const gunsInference = _.cloneDeep(inference);
        gunsInference.to.extraPath = ["1"];

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["arrayWithStringValues", "1"],
          value: "guns n roses",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference: gunsInference,
          },
        });

        const emptyContext: InferContext = [];

        evaluateInferenceLambda(inference, targetEntity, emptyContext);

        expect(targetEntity.ops).not.toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["arrayWithStringValues", "0"],
          value: "slave to the grind",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference: skidRowInference,
          },
        });
        expect(targetEntity.ops).not.toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["arrayWithStringValues", "1"],
          value: "guns n roses",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference: gunsInference,
          },
        });
      });
    });

    describe("supports conditional execution", () => {
      test("runs if the code block returns true", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "stringFromName",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { name: true },
            },
          ],
          to: { path: ["stringType"] },
          if: "true",
          code: "firstEntity.name",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        expect(targetEntity.ops).toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["stringType"],
          value: "opeth",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference,
          },
        });
      });

      test("does not run if the code block returns false", () => {
        const opeth = inferenceTestsEntity(0, "opeth");
        opeth.computeProperties();

        const context: InferContext = [
          {
            entity: opeth,
          },
        ];

        const targetEntity = inferenceTestsEntity(1, "cinderella");
        targetEntity.computeProperties();

        const inference: Inference = {
          name: "stringFromName",
          kind: "lambda",
          from: [
            {
              entityType: "inferenceTests",
              data: { name: true },
            },
          ],
          to: { path: ["stringType"] },
          if: "false",
          code: "firstEntity.name",
        };

        evaluateInferenceLambda(inference, targetEntity, context);

        expect(targetEntity.ops).not.toContainEqual({
          op: OpType.Set,
          source: OpSource.Inferred,
          path: ["stringType"],
          value: "opeth",
          system: "baseline",
          provenance: {
            context: [
              {
                id: "entity:0",
                entityType: "inferenceTests",
              },
            ],
            inference,
          },
        });
      });
    });
  });
});
