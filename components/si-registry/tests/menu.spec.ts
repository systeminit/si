import { entityMenu } from "../src/menu";
import { SchematicKind } from "../src/registryEntry";

describe("menu", () => {
  describe("entityMenu", () => {
    test("returns the entity menu", () => {
      const result = entityMenu({
        schematicKind: SchematicKind.Deployment,
        rootEntityType: "application",
      });
      expect(result).toEqual(
        expect.objectContaining({
          list: expect.arrayContaining([
            expect.objectContaining({
              name: "application",
              kind: "category",
              items: expect.arrayContaining([
                expect.objectContaining({
                  entityType: "service",
                  name: "service",
                }),
              ]),
            }),
            expect.objectContaining({
              name: "compute",
              kind: "category",
              items: expect.arrayContaining([
                expect.objectContaining({
                  entityType: "kubernetesCluster",
                  name: "kubernetes",
                }),
              ]),
            }),
            expect.objectContaining({
              name: "provider",
              kind: "category",
              items: expect.arrayContaining([
                expect.objectContaining({
                  entityType: "cloudProvider",
                  name: "cloud",
                }),
              ]),
            }),
          ]),
        }),
      );
    });
  });
});
