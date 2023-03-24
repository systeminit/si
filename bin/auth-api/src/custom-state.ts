import Router from '@koa/router';
import Koa from 'koa';
import { UserWithTosStatus } from "./services/users.service";

// types for the things we add to our koa ctx
export type CustomAppContext = {
};
export type CustomAppState = {
  clientIp: string,
  authUser?: UserWithTosStatus,
  // workspace?: Workspace
};

export type CustomRouteContext = Koa.ParameterizedContext<
CustomAppState,
Router.RouterParamContext<CustomAppState, CustomAppContext>
>;
