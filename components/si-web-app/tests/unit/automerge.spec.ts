import * as Automerge from "automerge";

describe("automerge", () => {
  test("foo", () => {
    let init: Record<string, any> = { foo: "bar" };
    let d1 = Automerge.from(init);
    let d3 = Automerge.change(d1, doc => {
      doc.foo = "bing";
      doc.snoop = "doggy";
    });
    console.log("okay", { d4 });
    console.log("confliucts", { conflicts: Automerge.con) });
    expect(true).toBe(false);
  });
});
