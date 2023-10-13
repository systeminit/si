import {makeExec} from "../src/sandbox/exec";

describe("exec", () => {
    test("exec a command", async () => {
        const e = makeExec("p");
        const r = await e.waitUntilEnd("echo", ["poop"]);
        expect(r.all).toBe("poop");
    });

    describe("watch a command", () => {
        test("until it succeeds", async () => {
            const e = makeExec("p");
            const getCurrentTime = await e.waitUntilEnd("date", ["+%s"]);
            const startSeconds = getCurrentTime.all || "3";
            const waitUntil = startSeconds + 3;
            const r = await e.watch({
                cmd: "date", args: ["+%s"], retryMs: 2000, callback: async (r) => {
                    const elapsed = r.all || "3";
                    return elapsed > waitUntil;
                }
            });
            expect(r.result.exitCode).toBe(0);
        });

        test("fail immediately if the command fails", async () => {
            const e = makeExec("p");
            let didIt = "nope";
            const r = await e.watch({
                cmd: "bigGunsBangBang", args: ["+%s"], retryMs: 2000, callback: async (r) => {
		    console.log(r)
                    didIt = "yep";
                    return true;
                }
            });
            expect(r.result.failed).toBe(true);
            expect(r.failed).toBe("commandFailed");
            expect(didIt).toBe("nope");
        });

        test("fail if the deadline is exceeded", async () => {
            const e = makeExec("p");
            const getCurrentTime = await e.waitUntilEnd("date", ["+%s"]);
            const startSeconds = getCurrentTime.all || "3";
            const waitUntil = startSeconds + 30000;
            const r = await e.watch({
                cmd: "date", args: ["+%s"], retryMs: 2000, maxRetryCount: 2, callback: async (r) => {
                    const elapsed = r.all || "3";
                    return elapsed > waitUntil;
                }
            });
            expect(r.result.exitCode).toBe(0);
            expect(r.failed).toBe("deadlineExceeded");
        });
    });
});
