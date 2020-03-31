import { GraphqlHintLoader } from "@/graphql-hint";
import { protobufLoader } from "@/protobuf";
import { services } from "@/services";
import {
  SchemaGenerator,
  transformOutputMethod,
  transformOutputField,
  transformOutputType,
  transformInputField,
  transformInputType,
  transformInputMethod,
} from "@/schema/generator";

test("transformOutputField", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const input = { value: "learn to fly" };
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  ) as protobuf.Method;
  createEntity.resolve();
  const requestType = createEntity.resolvedRequestType as protobuf.Type;
  const propsField = requestType.fields["props"];
  propsField.resolve();
  const deploymentType = propsField.resolvedType as protobuf.Type;
  const output = transformOutputField(input, deploymentType.fields.apiVersion);
  expect(output).toEqual("learn to fly");
  done();
});

test("transformOutputFieldWithMap", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const input = { alpha: "bravo", charlie: "delta" };
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  ) as protobuf.Method;
  createEntity.resolve();
  const requestType = createEntity.resolvedRequestType as protobuf.Type;
  const propsField = requestType.fields["props"];
  propsField.resolve();
  const deploymentType = propsField.resolvedType as protobuf.Type;
  const metadataField = deploymentType.fields["metadata"];
  metadataField.resolve();
  const metadataType = metadataField.resolvedType as protobuf.Type;
  const labelsField = metadataType.fields["labels"];

  const output = transformOutputField(input, labelsField);
  expect(output).toEqual([
    { key: "alpha", value: "bravo" },
    { key: "charlie", value: "delta" },
  ]);
  done();
});

test("transformOutputType", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const input = { apiVersion: { value: "learn to fly" } };
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  ) as protobuf.Method;
  createEntity.resolve();
  const requestType = createEntity.resolvedRequestType as protobuf.Type;
  const propsField = requestType.fields["props"];
  propsField.resolve();
  const deploymentType = propsField.resolvedType as protobuf.Type;
  transformOutputType(input, deploymentType);
  expect(input).toEqual({ apiVersion: "learn to fly" });
  done();
});

test("transformOutputMethod", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  );
  const input = {
    entity: {
      name: "foo-fighters-1555",
      displayName: "Foo Fighters 1555",
      description: "it has the best of you",
      workspaceId: "workspace:borntoresistirefuse",
      object: {
        apiVersion: { value: "not a fool" },
      },
    },
  };

  const output = transformOutputMethod(input, createEntity as protobuf.Method);

  expect(output).toEqual({
    entity: {
      name: "foo-fighters-1555",
      displayName: "Foo Fighters 1555",
      description: "it has the best of you",
      workspaceId: "workspace:borntoresistirefuse",
      object: {
        apiVersion: "not a fool",
      },
    },
  });
  done();
});

test("transformInputField", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const input = "learn to fly";
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  ) as protobuf.Method;
  createEntity.resolve();
  const requestType = createEntity.resolvedRequestType as protobuf.Type;
  const propsField = requestType.fields["props"];
  propsField.resolve();
  const deploymentType = propsField.resolvedType as protobuf.Type;
  const output = transformInputField(input, deploymentType.fields.apiVersion);
  expect(output).toEqual({ value: input });
  done();
});

test("transformInputFieldWithMap", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const input = [
    { key: "alpha", value: "bravo" },
    { key: "charlie", value: "delta" },
  ];
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  ) as protobuf.Method;
  createEntity.resolve();
  const requestType = createEntity.resolvedRequestType as protobuf.Type;
  const propsField = requestType.fields["props"];
  propsField.resolve();
  const deploymentType = propsField.resolvedType as protobuf.Type;
  const metadataField = deploymentType.fields["metadata"];
  metadataField.resolve();
  const metadataType = metadataField.resolvedType as protobuf.Type;
  const labelsField = metadataType.fields["labels"];

  const output = transformInputField(input, labelsField);
  expect(output).toEqual({ alpha: "bravo", charlie: "delta" });
  done();
});

test("transformInputType", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const input = { apiVersion: "learn to fly" };
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  ) as protobuf.Method;
  createEntity.resolve();
  const requestType = createEntity.resolvedRequestType as protobuf.Type;
  const propsField = requestType.fields["props"];
  propsField.resolve();
  const deploymentType = propsField.resolvedType as protobuf.Type;
  transformInputType(input, deploymentType);
  expect(input).toEqual({ apiVersion: { value: "learn to fly" } });
  done();
});

test("transformInputMethod", async done => {
  const graphqlHintLoader = new GraphqlHintLoader({
    services,
  });
  const sg = new SchemaGenerator(services, protobufLoader, graphqlHintLoader);
  const createEntity = protobufLoader.root.lookup(
    ".si.kubernetes.deployment.KubernetesDeployment.CreateEntity",
  );

  const input = {
    name: "foo-fighters-1555",
    displayName: "Foo Fighters 1555",
    description: "it has the best of you",
    workspaceId: "workspace:borntoresistirefuse",
    props: {
      apiVersion: "not a fool",
    },
  };
  const output = transformInputMethod(input, createEntity as protobuf.Method);

  expect(output).toEqual({
    name: "foo-fighters-1555",
    displayName: "Foo Fighters 1555",
    description: "it has the best of you",
    workspaceId: "workspace:borntoresistirefuse",
    props: {
      apiVersion: { value: "not a fool" },
    },
  });
  done();
});
