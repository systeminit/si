import { arg, queryField, objectType, inputObjectType } from "@nexus/schema";
import * as jwtLib from "jsonwebtoken";
import { AuthenticationError } from "apollo-server";
import * as traceApi from "@opentelemetry/api";

import { environment } from "@/environment";
import { resolverSpan } from "@/tracing/resolver";

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
  async resolve(_root, { input }: any, { dataSources: { grpc } }: any) {
    const tracer = traceApi.trace.getTracer("si-graphql-api");
    const span = resolverSpan("login");
    return tracer.withSpan(span, async () => {
      const g = grpc.service("account");
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
        span.end();
        return {
          jwt,
          userId: result.response.userId.value,
          billingAccountId: result.response.billingAccountId.value,
        };
      } else {
        span.setAttribute("error", true);
        span.setAttribute("error.message", "authentication failed");
        span.end();
        throw new AuthenticationError("authentication failed");
      }
    });
  },
});

export const loginTypes = [UserLoginRequest, UserLoginReply, UserLogin];
