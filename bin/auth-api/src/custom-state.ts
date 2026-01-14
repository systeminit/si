import Router from "@koa/router";
import Koa from "koa";
import { AuthToken, Workspace } from "@prisma/client";
import { UserWithTosStatus } from "./services/users.service";
import { AuthTokenData } from "./services/auth.service";

// types for the things we add to our koa ctx
export type CustomAppContext = object;
export type CustomAppState = {
  clientIp: string;
  token?: AuthTokenData;
  authUser?: UserWithTosStatus;
  authWorkspace?: Workspace;
  // For automation tokens, we look this up so we can check if they're revoked
  authToken?: AuthToken;
};

export type CustomRouteContext = Koa.ParameterizedContext<
  CustomAppState,
  Router.RouterParamContext<CustomAppState, CustomAppContext>
>;
