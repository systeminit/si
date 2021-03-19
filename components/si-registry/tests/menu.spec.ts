import { entityMenu } from "../src/menu";

describe("menu", () => {
  describe("entityMenu", () => {
    test("returns the entity menu", () => {
      const result = entityMenu();
      expect(result).toEqual(
        expect.objectContaining({
          list: expect.arrayContaining([
            expect.objectContaining({
              name: "application",
              items: expect.arrayContaining([
                expect.objectContaining({
                  entityType: "service",
                  displayName: "service",
                }),
              ]),
            }),
            expect.objectContaining({
              name: "helm",
            }),
            expect.objectContaining({
              name: "kubernetes",
            }),
          ]),
        }),
      );
    });
  });
});
