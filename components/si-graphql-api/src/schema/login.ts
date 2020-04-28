import { arg, queryField, objectType, inputObjectType } from "nexus";
import * as jwtLib from "jsonwebtoken";
import { AuthenticationError } from "apollo-server";

import { registryGenerator } from "@/schema/registryGenerator";
import { environment } from "@/environment";
import { Context } from "@/.";

const UserLoginRequest = inputObjectType({
  name: "UserLoginRequest",
  definition(t) {
    t.string("email", { required: true });
    t.string("password", { required: true });
    t.string("billingAccountName", { required: true });
  },
});

const UserLoginReply = objectType({
  name: "UserLoginReply",
  definition(t) {
    t.string("jwt");
    t.string("userId");
    t.string("billingAccountId");
  },
});

const UserLogin = queryField("userLogin", {
  type: UserLoginReply,
  args: { input: arg({ type: UserLoginRequest }) },
  async resolve(_root, { input }, { dataSources: { grpc } }: Context) {
    const g = grpc.service("si-account");
    const grpcInput = {
      email: { value: input.email },
      password: { value: input.password },
      billingAccountName: { value: input.billingAccountName },
    };
    const req = new g.Request("userLoginInternal", grpcInput).withRetry(0);
    const result = await req.exec();
    if (result.response.authenticated.value == true) {
      const jwt = jwtLib.sign(
        {
          userId: result.response.userId.value,
          billingAccountId: result.response.billingAccountId.value,
        },
        environment.jwtKey,
        {
          expiresIn: "1 days",
          audience: "https://app.systeminit.com",
          issuer: "https://app.systeminit.com",
        },
      );
      return {
        jwt,
        userId: result.response.userId.value,
        billingAccountId: result.response.billingAccountId.value,
      };
    } else {
      throw new AuthenticationError("authentication failed");
    }
  },
});

export const loginTypes = [UserLoginRequest, UserLoginReply, UserLogin];
