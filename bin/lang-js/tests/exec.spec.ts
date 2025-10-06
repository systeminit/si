import { assertEquals } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { makeExec } from "../src/sandbox/exec.ts";

Deno.test("exec", async (t) => {
  await t.step("exec a command", async () => {
    const e = makeExec("p");
    const r = await e.waitUntilEnd("echo", ["poop"]);
    console.log(r);
    assertEquals(r.all, "poop");
  });

  await t.step("watch a command", async (t) => {
    await t.step("until it succeeds", async () => {
      const e = makeExec("p");
      const getCurrentTime = await e.waitUntilEnd("date", ["+%s"]);
      const startSeconds = parseInt(String(getCurrentTime.all || "3"));
      const waitUntil = startSeconds + 3;
      const r = await e.watch({
        cmd: "date",
        args: ["+%s"],
        retryMs: 2000,
        callback: async (r) => {
          const elapsed = parseInt(String(r.all || "3"));
          return elapsed > waitUntil;
        },
      });
      assertEquals(r.result.exitCode, 0);
    });

    await t.step("fail immediately if the command fails", async () => {
      const e = makeExec("p");
      let didIt = "nope";
      const r = await e.watch({
        cmd: "bigGunsBangBang",
        args: ["+%s"],
        retryMs: 2000,
        callback: async (r) => {
          console.log(r);
          didIt = "yep";
          return true;
        },
      });
      assertEquals(r.result.failed, true);
      assertEquals(r.failed, "commandFailed");
      assertEquals(didIt, "nope");
    });

    await t.step("fail if the deadline is exceeded", async () => {
      const e = makeExec("p");
      const getCurrentTime = await e.waitUntilEnd("date", ["+%s"]);
      const startSeconds = parseInt(String(getCurrentTime.all || "3"));
      const waitUntil = startSeconds + 30000;
      const r = await e.watch({
        cmd: "date",
        args: ["+%s"],
        retryMs: 2000,
        maxRetryCount: 2,
        callback: async (r) => {
          const elapsed = parseInt(String(r.all || "3"));
          return elapsed > waitUntil;
        },
      });

      assertEquals(r.result.exitCode, 0);
      assertEquals(r.failed, "deadlineExceeded");
    });
  });
});
