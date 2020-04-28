import { registry, PropMethod } from "si-registry";
import { registryGenerator as sg } from "./registryGenerator";

//const sg = new SiRegistryGenerator();

test("transformGraphqlToGrpc", done => {
  const kubernetesDeployment = registry.get("kubernetesDeploymentEntity");
  const kubernetesDeploymentCreateEntity = kubernetesDeployment.methods.attrs.find(
    v => (v.name = "create"),
  ) as PropMethod;
  const input = {
    name: "amonamarth",
    displayName: "Amon Amarth",
    description: "Viking Metal",
    workspaceId: "workspace:vikingmetalrules001",
    constraints: {
      kubernetesVersion: "V1_15",
    },
    properties: {
      kubernetesObject: {
        apiVersion: "apps/v1",
        kind: "Deployment",
        metadata: {
          name: "nginx-deployment",
          labels: [
            {
              key: "app",
              value: "nginx",
            },
          ],
        },
        spec: {
          replicas: 3,
          selector: {
            matchLabels: [
              {
                key: "app",
                value: "nginx",
              },
            ],
          },
          template: {
            metadata: {
              labels: [
                {
                  key: "app",
                  value: "nginx",
                },
              ],
            },
            spec: {
              containers: [
                {
                  name: "nginx",
                  image: "nginx:1.7.9",
                  ports: [{ containerPort: 80 }],
                },
              ],
            },
          },
        },
      },
    },
  };
  const response = sg.transformGraphqlToGrpc(
    input,
    kubernetesDeploymentCreateEntity,
    kubernetesDeployment,
  );
  // A well known type - string
  expect(response["name"]).toEqual({ value: "amonamarth" });
  // A well known type - number
  expect(
    response["properties"]["kubernetesObject"]["spec"]["replicas"],
  ).toEqual({ value: 3 });
  // Another, deeply nested well known type
  expect(response["properties"]["kubernetesObject"]["apiVersion"]).toEqual({
    value: "apps/v1",
  });
  // A well known type inside of a map
  expect(
    response["properties"]["kubernetesObject"]["metadata"]["labels"],
  ).toEqual({ app: { value: "nginx" } });
  done();
});

test("transformGrpcToGraphql", done => {
  const kubernetesDeployment = registry.get("kubernetesDeploymentEntity");
  const kubernetesDeploymentCreateEntity = kubernetesDeployment.methods.attrs.find(
    v => (v.name = "create"),
  ) as PropMethod;

  const input = {
    entity: {
      id: { value: "whatsup" },
      name: { value: "amonamarth" },
      displayName: { value: "Amon Amarth" },
      constraints: {
        kubernetesVersion: "V1_15",
      },
      properties: {
        kubernetesObject: {
          apiVersion: { value: "apps/v1" },
          kind: { value: "Deployment" },
          metadata: {
            name: { value: "nginx-deployment" },
            labels: { app: { value: "nginx" } },
          },
          spec: {
            replicas: { value: 3 },
            selector: {
              matchLabels: { app: { value: "nginx" } },
            },
            template: {
              metadata: {
                labels: {
                  app: { value: "nginx" },
                },
              },
              spec: {
                containers: [
                  {
                    name: { value: "nginx" },
                    image: { value: "nginx:1.7.9" },
                    ports: [{ containerPort: { value: 80 } }],
                  },
                ],
              },
            },
          },
        },
      },
    },
  };

  const response = sg.transformGrpcToGraphql(
    input,
    kubernetesDeploymentCreateEntity,
    kubernetesDeployment,
  );
  // A well known type
  expect(response["entity"]["name"]).toEqual("amonamarth");
  // A well known type - number
  expect(
    response["entity"]["properties"]["kubernetesObject"]["spec"]["replicas"],
  ).toEqual(3);
  // Another, deeply nested well known type
  expect(
    response["entity"]["properties"]["kubernetesObject"]["apiVersion"],
  ).toEqual("apps/v1");
  // A well known type inside of a map
  expect(
    response["entity"]["properties"]["kubernetesObject"]["metadata"]["labels"],
  ).toEqual([{ key: "app", value: "nginx" }]);
  done();
});
