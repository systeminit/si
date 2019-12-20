import { arg, queryField, objectType, inputObjectType } from "nexus";
import * as jwtLib from "jsonwebtoken";
import { AuthenticationError } from "apollo-server";

import { environment } from "@/environment";
import { Context } from "@/.";

const LoginRequest = inputObjectType({
  name: "LoginRequest",
  definition(t) {
    t.string("email", { required: true });
    t.string("password", { required: true });
    t.string("billingAccountShortName", { required: true });
  },
});

const LoginReply = objectType({
  name: "LoginReply",
  definition(t) {
    t.string("jwt");
    t.string("userId");
    t.string("billingAccountId");
  },
});

const Login = queryField("login", {
  type: LoginReply,
  args: { input: arg({ type: LoginRequest }) },
  async resolve(_root, { input }, { dataSources: { grpc } }: Context) {
    const g = grpc.service("Account");
    const req = new g.Request("login", input);
    const result = await req.exec();
    if (result.response.authenticated == true) {
      const jwt = jwtLib.sign(
        {
          userId: result.response.userId,
          billingAccountId: result.response.billingAccountId,
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
        userId: result.response.userId,
        billingAccountId: result.response.billingAccountId,
      };
    } else {
      throw new AuthenticationError("authentication failed");
    }
  },
});

export const loginTypes = [LoginRequest, LoginReply, Login];
