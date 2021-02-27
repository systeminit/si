import { registry, findProp } from "../src/index";

describe("index", () => {
  test("exports registry", () => {
    expect(registry).not.toBeNull();
  });

  test("exports findProp", () => {
    expect(findProp).not.toBeNull();
  });
});
