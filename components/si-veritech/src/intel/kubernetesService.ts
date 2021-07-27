import Debug from "debug";
import { OpSource, OpType, ResourceInternalHealth, SiEntity } from "si-entity";
import {
  SyncResourceRequest,
  CommandProtocolFinish,
} from "../controllers/syncResource";
import { SiCtx } from "../siCtx";
import WebSocket from "ws";
import _ from "lodash";
import {
  awsAccessKeysEnvironment,
  awsKubeConfigPath,
  findEntityByType,
  k8sDiscoverEntity,
} from "../support";
import {
  InferPropertiesReply,
  InferPropertiesRequest,
} from "../controllers/inferProperties";
import {
  SetArrayEntryFromAllEntities,
  setArrayEntryFromAllEntities,
} from "./inferShared";
import {
  DiscoveryProtocolFinish,
  DiscoveryRequest,
} from "../controllers/discover";
const debug = Debug("veritech:controllers:discover:kubernetesService");

export function inferProperties(
  request: InferPropertiesRequest,
): InferPropertiesReply {
  const context = request.context;
  const entity = request.entity;

  setArrayEntryFromAllEntities({
    entity,
    context,
    entityType: "k8sService",
    toPath: ["healthChecks"],
    valuesCallback(
      fromEntity,
    ): ReturnType<SetArrayEntryFromAllEntities["valuesCallback"]> {
      const toSet: { path: string[]; value: any; system: string }[] = [];

      const portsBySystem: Record<
        string,
        Record<string, any>[]
      > = fromEntity.getPropertyForAllSystems({
        path: ["spec", "ports"],
      });
      for (const system in portsBySystem) {
        const ports = portsBySystem[system];
        for (const port of ports) {
          if (port["port"] == "80" || port["port"] == "8080") {
            toSet.push({
              path: ["protocol"],
              value: "HTTP",
              system,
            });
            toSet.push({
              path: ["port"],
              value: port["port"],
              system,
            });
          }
        }
      }
      return toSet;
    },
  });

  return { entity };
}

export async function syncResource(
  ctx: typeof SiCtx,
  req: SyncResourceRequest,
  ws: WebSocket,
): Promise<CommandProtocolFinish["finish"]> {
  const system = req.system.id;
  const response: CommandProtocolFinish["finish"] = {
    data: {},
    state: req.resource.state,
    health: req.resource.health,
    internalStatus: req.resource.internalStatus,
    internalHealth: req.resource.internalHealth,
    subResources: req.resource.subResources,
  };

  const service = findEntityByType(req, "k8sService");

  if (!service) {
    response.error = "No service connected";
    response.state = "error";
    response.health = "error";
    response.internalHealth = ResourceInternalHealth.Error;
    return response;
  }
  const clusterResource = _.find(
    req.resourceContext,
    (r) => r.entityId == service.id,
  );
  if (clusterResource) {
    response.data = clusterResource.data;
    response.error = clusterResource.error;
    response.state = clusterResource.state;
    response.health = clusterResource.health;
    response.internalHealth = clusterResource.internalHealth;
    response.internalStatus = clusterResource.internalStatus;
    response.subResources = clusterResource.subResources;
  } else {
    response.state = "unknown";
    response.health = "unknown";
    response.internalHealth = ResourceInternalHealth.Unknown;
    return response;
  }

  const healthChecks = req.entity.getProperty({
    system,
    path: ["healthChecks"],
  });
  if (healthChecks) {
    // @ts-ignore
    for (const healthCheck of healthChecks) {
      if (healthCheck["protocol"] == "HTTP") {
        const host = healthCheck["host"];
        const port = healthCheck["port"];
        const path = healthCheck["path"];
        if (!host) {
          for (const kubeClusterId of Object.keys(response.subResources)) {
            if (
              response.data[kubeClusterId] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"]["status"] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"]["status"]["loadBalancer"] &&
              // @ts-ignore
              response.data[kubeClusterId]["data"]["status"]["loadBalancer"][
                "ingress"
              ]
            ) {
              // @ts-ignore
              for (const ingress of response.data[kubeClusterId]["data"][
                "status"
              ]["loadBalancer"]["ingress"]) {
                let hostName;
                // @ts-ignore
                if (ingress["hostname"]) {
                  hostName = ingress["hostname"];
                } else {
                  hostName = ingress["ip"];
                }
                if (hostName) {
                  let url = `http://${hostName}:${port}`;
                  if (path) {
                    url += path;
                  }
                  response.subResources[kubeClusterId].data[
                    "healthCheckUrl"
                  ] = url;
                  const result = await ctx.exec(
                    "curl",
                    ["-v", "-i", "-m", "10", url],
                    {
                      reject: false,
                    },
                  );
                  if (result.exitCode != 0) {
                    response.health = "error";
                    response.state = "error";
                    response.internalHealth = ResourceInternalHealth.Error;
                    response.error = result.all;
                    if (response.subResources[kubeClusterId]) {
                      response.subResources[kubeClusterId].health = "error";
                      response.subResources[kubeClusterId].state = "error";
                      response.subResources[kubeClusterId].internalHealth =
                        ResourceInternalHealth.Error;
                      response.subResources[kubeClusterId].error = result.all;
                    }
                  }
                  response.data[kubeClusterId] =
                    response.subResources[kubeClusterId];
                }
              }
            }
          }
        } else {
          let url = `http://${host}:${port}`;
          if (path) {
            url += path;
          }
          // @ts-ignore
          response.subResources[kubeClusterId]["healthCheckUrl"] = url;
          response.data["healthCheckUrl"] = url;
          const result = await ctx.exec("curl", ["-v", "-i", "-m", "10", url], {
            reject: false,
          });
          if (result.exitCode != 0) {
            response.health = "error";
            response.state = "error";
            response.internalHealth = ResourceInternalHealth.Error;
            response.error = result.all;
          }
        }
      }
    }
  }
  return response;
}

export async function discover(
  ctx: typeof SiCtx,
  req: DiscoveryRequest,
  _ws: WebSocket,
): Promise<DiscoveryProtocolFinish["finish"]> {
  debug("discovery starting");
  const system = req.system.id;
  const response: DiscoveryProtocolFinish["finish"] = {
    discovered: [],
  };
  const cluster = findEntityByType(req, "awsEksCluster");
  debug({ cluster, req });
  const awsEnv = awsAccessKeysEnvironment(req);
  const kubeConfigDir = await awsKubeConfigPath(
    req,
    cluster.getProperty({ system, path: ["name"] }),
  );

  const servicesResult = await ctx.exec(
    "kubectl",
    [
      "get",
      "services",
      "-A",
      "-o",
      "json",
      "--kubeconfig",
      `${kubeConfigDir.path}/config`,
    ],
    { env: awsEnv },
  );
  const listServices: Record<string, any> = JSON.parse(servicesResult.stdout);
  if (listServices["items"]) {
    for (const serviceDataFull of listServices["items"]) {
      const k8sServiceConfigures: any = [];

      if (
        !serviceDataFull["metadata"] ||
        !serviceDataFull["metadata"]["annotations"] ||
        !serviceDataFull["metadata"]["annotations"][
          "kubectl.kubernetes.io/last-applied-configuration"
        ]
      ) {
        debug(serviceDataFull);
        continue;
      }

      const serviceData = JSON.parse(
        serviceDataFull["metadata"]["annotations"][
          "kubectl.kubernetes.io/last-applied-configuration"
        ],
      );
      const k8sService = new SiEntity({ entityType: "k8sService" });
      k8sService.addOpSet({
        op: OpType.Set,
        source: OpSource.Manual,
        path: ["spec", "type"],
        value: "LoadBalancer",
        system: "baseline",
      });

      //k8sService.name = serviceData["metadata"]["name"].replace("-service", "");
      k8sDiscoverEntity(k8sService, serviceData);
      //k8sService.computeProperties();
      //k8sService.computeCode();
      const serviceNamespace = _.get(serviceData, ["metadata", "namespace"]);
      if (serviceNamespace) {
        const namespaceResult = await ctx.exec(
          "kubectl",
          [
            "get",
            "namespace",
            "-o",
            "json",
            "--kubeconfig",
            `${kubeConfigDir.path}/config`,
            serviceNamespace,
          ],
          { env: awsEnv },
        );
        const namespaceDataFull: Record<string, any> = JSON.parse(
          namespaceResult.stdout,
        );
        if (
          namespaceDataFull["metadata"] &&
          namespaceDataFull["metadata"]["name"] != "default"
        ) {
          const namespaceData = JSON.parse(
            namespaceDataFull["metadata"]["annotations"][
              "kubectl.kubernetes.io/last-applied-configuration"
            ],
          );

          const k8sNamespace = new SiEntity({
            entityType: "k8sNamespace",
          });
          k8sDiscoverEntity(k8sNamespace, namespaceData);
          k8sNamespace.computeProperties();
          k8sNamespace.computeCode();

          k8sServiceConfigures.push({
            entity: k8sNamespace,
            configures: [],
          });
        }
      }

      const kubernetesService = new SiEntity({
        entityType: "kubernetesService",
      });
      kubernetesService.name = k8sService.name;

      const deploymentSelectorKey = [];
      if (serviceData["spec"]["selector"]) {
        for (const key of Object.keys(serviceData["spec"]["selector"])) {
          deploymentSelectorKey.push(
            `${key}=${serviceData["spec"]["selector"][key]}`,
          );
        }
      }
      //if (serviceData["spec"]["ports"]) {
      //  for (let x = 0; x < serviceData["spec"]["ports"]; x++) {
      //    const portData = serviceData["spec"]["ports"][x];
      //    kubernetesService.addOpSet({
      //      op: OpType.Set,
      //      source: OpSource.Inferred,
      //      path: ["healthChecks", `${x}`],
      //      // @ts-ignore
      //      value: {},
      //      system: "baseline",
      //    });
      //    kubernetesService.addOpSet({
      //      op: OpType.Set,
      //      source: OpSource.Inferred,
      //      path: ["healthChecks", `${x}`, "port"],
      //      value: portData["port"],
      //      system: "baseline",
      //    });
      //    if (portData["port"] == 80) {
      //      kubernetesService.addOpSet({
      //        op: OpType.Set,
      //        source: OpSource.Inferred,
      //        path: ["healthChecks", `${x}`, "protocol"],
      //        value: "HTTP",
      //        system: "baseline",
      //      });
      //    } else if (portData["port"] == 443) {
      //      kubernetesService.addOpSet({
      //        op: OpType.Set,
      //        source: OpSource.Inferred,
      //        path: ["healthChecks", `${x}`, "protocol"],
      //        value: "HTTPS",
      //        system: "baseline",
      //      });
      //    } else {
      //      kubernetesService.addOpSet({
      //        op: OpType.Set,
      //        source: OpSource.Inferred,
      //        path: ["healthChecks", `${x}`, "protocol"],
      //        value: "TCP",
      //        system: "baseline",
      //      });
      //    }
      //  }
      //}

      // Now, discover the kubernetes deployment via the selector
      const deploymentResult = await ctx.exec(
        "kubectl",
        [
          "get",
          "deployments",
          "-A",
          "-o",
          "json",
          "--kubeconfig",
          `${kubeConfigDir.path}/config`,
          "-l",
          deploymentSelectorKey.join(","),
        ],
        { env: awsEnv },
      );
      const deploymentsList: Record<string, any> = JSON.parse(
        deploymentResult.stdout,
      );
      if (deploymentsList["items"]) {
        for (const deploymentDataFull of deploymentsList["items"]) {
          const deploymentData = JSON.parse(
            deploymentDataFull["metadata"]["annotations"][
              "kubectl.kubernetes.io/last-applied-configuration"
            ],
          );
          const k8sDeployment = new SiEntity({
            entityType: "k8sDeployment",
          });
          k8sDiscoverEntity(k8sDeployment, deploymentData);
          const k8sDeploymentConfigures: any[] = [];

          const containerList = _.get(deploymentData, [
            "spec",
            "template",
            "spec",
            "containers",
          ]);
          if (containerList && containerList.length > 0) {
            for (const containerData of containerList) {
              const dockerImage = new SiEntity({
                entityType: "dockerImage",
              });
              dockerImage.name = containerData["name"];
              dockerImage.addOpSet({
                op: OpType.Set,
                source: OpSource.Manual,
                path: ["image"],
                value: containerData["image"],
                system: "baseline",
              });
              if (containerData["ports"]) {
                for (let x = 0; x < containerData["ports"].length; x++) {
                  const portData = containerData["ports"][x];
                  dockerImage.addOpSet({
                    op: OpType.Set,
                    source: OpSource.Manual,
                    path: ["ExposedPorts", `${x}`],
                    value: `${portData["containerPort"]}/${_.toLower(
                      portData["protocol"],
                    )}`,
                    system: "baseline",
                  });
                }
              }
              //dockerImage.computeProperties();
              //dockerImage.computeCode();

              k8sDeploymentConfigures.push({
                entity: dockerImage,
                configures: [],
              });
            }
          }

          //const deploymentNamespace = _.get(deploymentData, [
          //  "metadata",
          //  "namespace",
          //]);
          //if (deploymentNamespace) {
          //  const namespaceResult = await ctx.exec("kubectl", [
          //    "get",
          //    "namespace",
          //    "-o",
          //    "json",
          //    "--kubeconfig",
          //    `${kubeConfigDir.path}/config`,
          //    deploymentNamespace,
          //  ], { env: awsEnv });
          //  const namespaceDataFull: Record<string, any> = JSON.parse(
          //    namespaceResult.stdout,
          //  );
          //  const namespaceData = JSON.parse(
          //    namespaceDataFull["metadata"]["annotations"][
          //      "kubectl.kubernetes.io/last-applied-configuration"
          //    ],
          //  );

          //  const k8sNamespace = new SiEntity({
          //    entityType: "k8sNamespace",
          //  });
          //  k8sDiscoverEntity(k8sNamespace, namespaceData);
          //  k8sNamespace.computeProperties();
          //  k8sNamespace.computeCode();

          //  k8sDeploymentConfigures.push({
          //    entity: k8sNamespace,
          //    configures: [],
          //  });
          //}
          //k8sDeployment.computeProperties();
          //k8sDeployment.computeCode();

          k8sServiceConfigures.push({
            entity: k8sDeployment,
            configures: k8sDeploymentConfigures,
          });
        }
      }

      const service = new SiEntity({ entityType: "service" });
      service.name = k8sService.name;
      //service.computeCode();
      //service.computeProperties();

      response.discovered.push({
        entity: service,
        configures: [
          {
            entity: kubernetesService,
            configures: [
              { entity: k8sService, configures: k8sServiceConfigures },
            ],
          },
        ],
      });
    }
  }

  debug("***** maybe tomorrow *****");
  debug(response);
  return response;
}

export default { inferProperties, syncResource, discover };
