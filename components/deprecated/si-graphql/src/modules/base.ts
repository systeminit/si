import { GraphQLModule } from "@graphql-modules/core";
import { DocumentNode } from "graphql";
import { checkAuthentication } from "@/modules/auth";
import { GqlRoot, GqlArgs, GqlContext, GqlInfo } from "@/app.module";
import {
  FindComponentInput,
  GetComponentsInput,
} from "@/modules/components/queries";
import { Component, ComponentObject } from "@/datalayer/component";
import { Entity } from "@/datalayer/entity";

export function buildGetComponentFunction<T extends ComponentObject>(
  component: Component<T>,
): (
  o: GqlRoot,
  a: GetComponentsInput,
  c: GqlContext,
  i: GqlInfo,
) => Promise<T[]> {
  const f = async function(
    _obj: GqlRoot,
    args: GetComponentsInput,
    _context: GqlContext,
    info: GqlInfo,
  ): Promise<T[]> {
    const user = await checkAuthentication(info);
    return component.filterAll(args, user);
  };
  return f;
}

export function buildFindComponentFunction<T extends ComponentObject>(
  component: Component<T>,
): (
  obj: GqlRoot,
  args: FindComponentInput,
  context: GqlContext,
  info: GqlInfo,
) => Promise<T[]> {
  const f = async function(
    _obj: GqlRoot,
    args: FindComponentInput,
    _context: GqlContext,
    info: GqlInfo,
  ): Promise<T[]> {
    const user = await checkAuthentication(info);
    return component.find(args, user);
  };
  return f;
}

interface CreateModuleArgs {
  componentName: string;
  component: Component<any>;
  entity: Entity<any>;
  typeDefs: DocumentNode;
  createEntity: (
    o: GqlRoot,
    a: GqlArgs,
    c: GqlContext,
    i: GqlInfo,
  ) => Promise<any>;
}

export function createComponentModule(args: CreateModuleArgs): GraphQLModule {
  const getComponent = buildGetComponentFunction(args.component);
  const getComponentName = `get${args.componentName}Components`;
  const findComponent = buildFindComponentFunction(args.component);
  const findComponentName = `find${args.componentName}Components`;
  const createEntityName = `create${args.componentName}`;

  const Query = {};
  Query[getComponentName] = getComponent;
  Query[findComponentName] = findComponent;

  console.log(Query);

  const Mutation = {};
  Mutation[createEntityName] = args.createEntity;

  const module = new GraphQLModule({
    typeDefs: args.typeDefs,
    resolvers: {
      Query,
      Mutation,
    },
  });
  return module;
}
