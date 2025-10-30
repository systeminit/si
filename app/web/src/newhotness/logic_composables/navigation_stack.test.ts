import { beforeEach, expect, test } from "vitest";
import { breadcrumbs, prevPage, push, reset } from "./navigation_stack";

beforeEach(() => {
  reset();
});

test("prevPage returns undefined when stack is empty", () => {
  const result = prevPage();
  expect(result).toBeUndefined();
});

test("prevPage returns undefined when only one page in stack", () => {
  push(
    "/workspace/changeset",
    "new-hotness",
    {
      workspacePk: "ws1",
      changeSetId: "cs1",
    },
    {},
  );

  const result = prevPage();
  expect(result).toBeUndefined();
});

test("prevPage returns previous page with full context", () => {
  // Push first page (explore view with viewId)
  push(
    "/workspace/changeset",
    "new-hotness",
    { workspacePk: "ws1", changeSetId: "cs1" },
    { viewId: "my-view", grid: "1", sortBy: "latest" },
  );

  // Push second page (component details)
  push(
    "/workspace/changeset/component/c",
    "new-hotness-component",
    { workspacePk: "ws1", changeSetId: "cs1", componentId: "comp1" },
    { viewId: "my-view", grid: "1", sortBy: "latest" },
  );

  const result = prevPage();

  expect(result).toEqual({
    url: "/workspace/changeset",
    name: "new-hotness",
    params: { workspacePk: "ws1", changeSetId: "cs1" },
    query: { viewId: "my-view", grid: "1", sortBy: "latest" },
  });
});

test("push does not add duplicate entries", () => {
  push(
    "/workspace/changeset",
    "new-hotness",
    { workspacePk: "ws1" },
    {
      viewId: "view1",
    },
  );
  push(
    "/workspace/changeset",
    "new-hotness",
    { workspacePk: "ws1" },
    {
      viewId: "view1",
    },
  );

  expect(breadcrumbs.length).toBe(1);
});

test("push adds entry when params differ", () => {
  push("/workspace/changeset", "new-hotness", { workspacePk: "ws1" }, {});
  push("/workspace/changeset", "new-hotness", { workspacePk: "ws2" }, {});

  expect(breadcrumbs.length).toBe(2);
});

test("push adds entry when query differs", () => {
  push(
    "/workspace/changeset",
    "new-hotness",
    { workspacePk: "ws1" },
    {
      viewId: "view1",
    },
  );
  push(
    "/workspace/changeset",
    "new-hotness",
    { workspacePk: "ws1" },
    {
      viewId: "view2",
    },
  );

  expect(breadcrumbs.length).toBe(2);
});

test("stack maintains LIMIT of 5 entries", () => {
  for (let i = 0; i < 10; i++) {
    push(`/page${i}`, "route", { id: `${i}` }, {});
  }

  expect(breadcrumbs.length).toBe(5);
  // Should keep the last 5 entries
  expect(breadcrumbs[0]?.url).toBe("/page5");
  expect(breadcrumbs[4]?.url).toBe("/page9");
});

test("prevPage works correctly with multiple navigations preserving viewId", () => {
  // Simulate: Explore (with viewId) -> Component Details -> Go Back
  push(
    "/explore",
    "new-hotness",
    { workspacePk: "ws1", changeSetId: "cs1" },
    {
      viewId: "custom-view",
    },
  );
  push(
    "/component",
    "new-hotness-component",
    {
      workspacePk: "ws1",
      changeSetId: "cs1",
      componentId: "c1",
    },
    { viewId: "custom-view" },
  );

  const previous = prevPage();

  expect(previous?.query.viewId).toBe("custom-view");
  expect(previous?.name).toBe("new-hotness");
});
