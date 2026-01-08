import t from "tap";
import _ from "lodash";
import { DateTime } from "luxon";
import { expect } from "chai";

import { TosVersion } from "@si/ts-lib/src/terms-of-service";
import { request } from "./helpers/supertest-agents";
import { testSuiteAfter, testSuiteBefore } from "./helpers/test-suite-hooks";
import { createDummyUser } from "./helpers/dummy-factory";

t.before(testSuiteBefore);
t.teardown(testSuiteAfter);

t.test("User routes", async () => {
  const { user } = await createDummyUser();
  const { user: anotherUser } = await createDummyUser();
  const { user: userNeedsTos } = await createDummyUser({ tos: false });

  t.test("GET /whoami - check auth and load current user", async (t) => {
    t.test("fails if user is not logged in", async () => {
      await request.get("/whoami")
        .expectError("Unauthorized");
    });

    t.test("fetches user info", async () => {
      await request.get("/whoami")
        .set("spoof-auth", user.id)
        .expectOk()
        .expectBody({
          user: {
            // not checking everything here...
            id: user.id,
            email: user.email,
          },
        });
    });

  });

  t.test("PATCH /users/:userId - update user info", async (t) => {
    const GOOD_PARAMS = {
      firstName: "newfirst",
      lastName: "newlast",
      nickname: "newnickname",
      email: `newemail${+new Date()}@systeminit.dev`,
      pictureUrl: "https://placekitten.com/100/100",
      discordUsername: "newdiscord#9999",
      githubUsername: "newgithubusername",
    };

    t.test("cannot update user data without auth", async () => {
      await request.patch(`/users/${user.id}`)
        .send(GOOD_PARAMS)
        .expectError("Unauthorized");
    });
    t.test("cannot update user data of other users", async () => {
      await request.patch(`/users/${user.id}`)
        .set("spoof-auth", anotherUser.id)
        .send(GOOD_PARAMS)
        .expectError("Forbidden");
    });

    t.test("can update your own user info", async () => {
      await request.patch(`/users/${user.id}`)
        .set("spoof-auth", user.id)
        .send(GOOD_PARAMS)
        .expectOk()
        .expectBody({ user: GOOD_PARAMS });
    });

    t.test("can clear optional username fields with empty string", async () => {
      // First set the usernames
      await request.patch(`/users/${user.id}`)
        .set("spoof-auth", user.id)
        .send({
          discordUsername: "testuser#1234",
          githubUsername: "testuser",
        })
        .expectOk();

      // Then clear them with empty strings (should be converted to null)
      const response = await request.patch(`/users/${user.id}`)
        .set("spoof-auth", user.id)
        .send({
          discordUsername: "",
          githubUsername: "",
        })
        .expectOk();

      // Verify they were cleared (converted to null)
      expect(response.body.user.discordUsername).to.be.null;
      expect(response.body.user.githubUsername).to.be.null;
    });

    t.test("can clear optional username fields with null", async () => {
      // First set the usernames
      await request.patch(`/users/${user.id}`)
        .set("spoof-auth", user.id)
        .send({
          discordUsername: "testuser#1234",
          githubUsername: "testuser",
        })
        .expectOk();

      // Then clear them with null
      const response = await request.patch(`/users/${user.id}`)
        .set("spoof-auth", user.id)
        .send({
          discordUsername: null,
          githubUsername: null,
        })
        .expectOk();

      // Verify they were cleared
      expect(response.body.user.discordUsername).to.be.null;
      expect(response.body.user.githubUsername).to.be.null;
    });

    // check bad params
    _.each({
      "non-string firstName": { firstName: 1 },
      "non-string lastName": { lastName: true },
      "non-string nickname": { nickname: {} },
      "null nickname": { nickname: null }, // all other fields are nullable currently
      "non-string email": { email: 1 },
      "invalid email": { email: "bad-email@systeminit" },
      "non-string pictureUrl": { pictureUrl: 1 },
      "invalid pictureUrl": { pictureUrl: "not-a-url" },
      "non-string discordUsername": { discordUsername: false },
      "non-string githubUsername": { githubUsername: {} },
    }, (bodyOverride, description) => {
      t.test(`bad params - ${description}`, async () => {
        await request.patch(`/users/${user.id}`)
          .set("spoof-auth", user.id)
          .send({
            GOOD_PARAMS,
            ...bodyOverride,
          })
          .expectError("BadRequest");
      });
    });
  });

  t.test("POST /users/:userId/complete-tutorial-step - record tutorial progress", async (t) => {
    const STEP_NAME_1 = "fakeStep1";
    const STEP_NAME_2 = "fakeStep2";

    const GOOD_PARAMS = {
      step: STEP_NAME_1,
    };

    t.test("cannot access without auth", async () => {
      await request.post(`/users/${user.id}/complete-tutorial-step`)
        .send(GOOD_PARAMS)
        .expectError("Unauthorized");
    });
    t.test("cannot access for other users", async () => {
      await request.post(`/users/${user.id}/complete-tutorial-step`)
        .set("spoof-auth", anotherUser.id)
        .send(GOOD_PARAMS)
        .expectError("Forbidden");
    });

    t.test("can record tutorial progress for yourself", async () => {
      await request.post(`/users/${user.id}/complete-tutorial-step`)
        .set("spoof-auth", user.id)
        .send(GOOD_PARAMS)
        .expectOk()
        .expect((res) => {
          const completedStepsData = res.body.user.onboardingDetails.vroStepsCompletedAt;
          expect(completedStepsData).to.have.all.keys(STEP_NAME_1);
          expect(DateTime.fromISO(completedStepsData[STEP_NAME_1]).isValid).to.be.true;
        });
    });
    t.test("can record multiple tutorial steps", async () => {
      await request.post(`/users/${user.id}/complete-tutorial-step`)
        .set("spoof-auth", user.id)
        .send({ step: STEP_NAME_2 })
        .expectOk()
        .expect((res) => {
          const completedStepsData = res.body.user.onboardingDetails.vroStepsCompletedAt;
          expect(completedStepsData).to.have.all.keys(STEP_NAME_1, STEP_NAME_2);
          expect(DateTime.fromISO(completedStepsData[STEP_NAME_2]).isValid).to.be.true;
          expect(completedStepsData[STEP_NAME_2] > completedStepsData[STEP_NAME_1]).to.be.true;
        });
    });
  });

  t.test("POST /tos-agreement - record agreement to TOS", async (t) => {
    // version IDs are currently just a sortable string
    // and a single "latest version" is managed in the code
    // this will probably move to something in the db or env vars eventually...
    const VERSION_1 = TosVersion.v20230330;
    const VERSION_2 = VERSION_1.replace("2023", "2024");
    const GOOD_PARAMS = {
      tosVersionId: VERSION_1,
    };

    t.test("cannot access without auth", async () => {
      await request.post(`/tos-agreement`)
        .send(GOOD_PARAMS)
        .expectError("Unauthorized");
    });

    t.test("(confirm user needs TOS update)", async () => {
      await request.get("/whoami")
        .set("spoof-auth", userNeedsTos.id)
        .expectBody({ user: { needsTosUpdate: true } });
    });

    t.test("success - record agreement", async () => {
      await request.post(`/tos-agreement`)
        .set("spoof-auth", userNeedsTos.id)
        .set("X-Forwarded-For", "9.8.7.6")
        .send({ tosVersionId: VERSION_1 })
        .expectOk()
        .expectBody({
          userId: userNeedsTos.id,
          tosVersionId: VERSION_1,
          ipAddress: "9.8.7.6",
        });
    });

    t.test("confirm user no longer needs TOS update", async () => {
      await request.get("/whoami")
        .set("spoof-auth", userNeedsTos.id)
        .expectBody({ user: { needsTosUpdate: false } });
    });

    t.test("can record agreement to a newer version", async () => {
      await request.post(`/tos-agreement`)
        .set("spoof-auth", userNeedsTos.id)
        .send({ tosVersionId: VERSION_2 })
        .expectOk();
    });

    t.test("cannot record agreement to an earlier version", async () => {
      await request.post(`/tos-agreement`)
        .set("spoof-auth", userNeedsTos.id)
        .send({ tosVersionId: VERSION_1 })
        .expectError("Conflict");
    });

    // check bad params
    _.each({
      "no version": {},
      "null version": { tosVersionId: null },
      "non-string version": { tosVersionId: 123 },
    }, (bodyOverride, description) => {
      t.test(`bad params - ${description}`, async () => {
        await request.post("/tos-agreement")
          .set("spoof-auth", userNeedsTos.id)
          .send({
            ...bodyOverride,
          })
          .expectError("BadRequest");
      });
    });

    t.test("success - record agreement", async () => {
      await request.post(`/tos-agreement`)
        .set("spoof-auth", anotherUser.id)
        .send(GOOD_PARAMS)
        .expectOk();
    });

  });

});
