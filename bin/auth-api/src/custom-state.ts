import Router from '@koa/router';
import Koa from 'koa';
import { Workspace } from '@prisma/client';
import { UserWithTosStatus } from "./services/users.service";

// types for the things we add to our koa ctx
export type CustomAppContext = {
};
export type CustomAppState = {
  clientIp: string,
  authUser?: UserWithTosStatus,
  authWorkspace?: Workspace,
};

export type CustomRouteContext = Koa.ParameterizedContext<
CustomAppState,
Router.RouterParamContext<CustomAppState, CustomAppContext>
>;
