import {
  PropBool,
  PropCode,
  PropEnum,
  PropLink,
  PropNumber,
  PropObject,
  PropText,
} from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "kubernetesService",
  displayTypeName: "Kubernetes Service Object",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    // Constraints
    //
    // TODO(fnicho): this should be a common enum across all AWS/EKS related
    // objects
    c.constraints.addEnum({
      name: "kubernetesVersion",
      label: "Kubernetes Version",
      options(p: PropEnum) {
        p.variants = ["v1.12", "v1.13", "v1.14", "v1.15"];
      },
    });

    // Properties
    c.properties.addObject({
      name: "kubernetesObject",
      label: "Kubernetes Object",
      options(p: PropObject) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesServiceEntity",
            names: ["properties", "kubernetesObjectYaml"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesServiceEntity",
            names: ["properties", "kubernetesObjectYaml"],
          },
        });
        p.properties.addText({
          name: "apiVersion",
          label: "API Version",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "kind",
          label: "Kind",
          options(p: PropText) {
            p.required = true;
            p.baseDefaultValue = "Service";
          },
        });
        p.properties.addLink({
          name: "metadata",
          label: "Metadat",
          options(p: PropLink) {
            p.lookup = {
              typeName: "kubernetesMetadata",
            };
          },
        });
        p.properties.addObject({
          name: "spec",
          label: "Service Spec",
          options(p: PropObject) {
            // TODO(fnichol): The specification records this field in YAML as
            // `clusterIP`
            p.properties.addText({
              name: "clusterIp",
              label: "Host IP",
            });
            // TODO(fnichol): The specification records this field in YAML as
            // `externalIPs`
            p.properties.addText({
              name: "externalIps",
              label: "External IPs",
              options(p: PropText) {
                p.repeated = true;
              },
            });
            p.properties.addText({
              name: "externalName",
              label: "External Name",
            });
            p.properties.addEnum({
              name: "externalTrafficPolicy",
              label: "External Traffic Policy",
              options(p: PropEnum) {
                p.variants = ["Local", "Cluster"];
              },
            });
            p.properties.addNumber({
              name: "healthCheckNodePort",
              label: "Health Check Node Port",
              options(p: PropNumber) {
                p.numberKind = "uint32";
              },
            });
            p.properties.addEnum({
              name: "ipFamily",
              label: "IP Family",
              options(p: PropEnum) {
                p.variants = ["IPv4", "IPv6"];
              },
            });
            // TODO(fnichol); The specification records this field in YAML as
            // `LoadBalancerIP`
            p.properties.addText({
              name: "loadBalancerIp",
              label: "Load Balancer IP",
            });
            p.properties.addText({
              name: "loadBalancerSourceRanges",
              label: "Load Balancer Source Ranges",
              options(p: PropText) {
                p.repeated = true;
              },
            });
            p.properties.addLink({
              name: "ports",
              label: "Ports",
              options(p: PropLink) {
                p.repeated = true;
                p.lookup = {
                  typeName: "kubernetesServicePort",
                };
              },
            });
            p.properties.addBool({
              name: "publishNotReadyAddress",
              label: "Publish Not Ready Address",
              options(p: PropBool) {
                p.baseDefaultValue = false;
              },
            });
            p.properties.addLink({
              name: "selector",
              label: "Selector",
              options(p: PropLink) {
                p.lookup = {
                  typeName: "kubernetesSelector",
                };
              },
            });
            p.properties.addEnum({
              name: "sessionAffinity",
              label: "Session Affinity",
              options(p: PropEnum) {
                p.variants = ["ClientIP", "None"];
                p.baseDefaultValue = "None";
              },
            });
            p.properties.addObject({
              name: "sessionAffinityConfig",
              label: "Session Affinity Config",
              options(p: PropObject) {
                // TODO(fnichol): The specification records this field in YAML
                // as `clientIP`
                p.properties.addObject({
                  name: "clientIp",
                  label: "Client IP Config",
                  options(p: PropObject) {
                    p.properties.addNumber({
                      name: "timeoutSeconds",
                      label: "Timeout Seconds",
                      options(p: PropNumber) {
                        p.numberKind = "uint32";
                        // Default is 3 hours
                        p.baseDefaultValue = "10800";
                      },
                    });
                  },
                });
              },
            });
            p.properties.addText({
              name: "topologyKeys",
              label: "Topology Keys",
              options(p: PropText) {
                p.repeated = true;
              },
            });
            p.properties.addEnum({
              name: "type",
              label: "Type",
              options(p: PropEnum) {
                p.variants = [
                  "ExternalName",
                  "ClusterIP",
                  "NodePort",
                  "LoadBalancer",
                ];
                p.baseDefaultValue = "ClusterIP";
              },
            });
          },
        });
        p.properties.addObject({
          name: "status",
          label: "Service Status",
          options(p: PropObject) {
            p.properties.addLink({
              name: "loadBalancer",
              label: "Load Balancer Status",
              options(p: PropLink) {
                p.lookup = {
                  typeName: "kubernetesLoadBalancerStatus",
                };
              },
            });
          },
        });
      },
    });
    c.properties.addCode({
      name: "kubernetesObjectYaml",
      label: "Kubernetes Object YAML",
      options(p: PropCode) {
        p.relationships.updates({
          partner: {
            typeName: "kubernetesServiceEntity",
            names: ["properties", "kubernetesObject"],
          },
        });
        p.relationships.either({
          partner: {
            typeName: "kubernetesServiceEntity",
            names: ["properties", "kubernetesObject"],
          },
        });
        p.language = "yaml";
      },
    });
  },
});

registry.base({
  typeName: "kubernetesServicePort",
  displayTypeName: "Kubernetes Service Port",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addText({
      name: "appProtocol",
      label: "App Protocol",
    });
    c.fields.addText({
      name: "name",
      label: "Name",
    });
    c.fields.addNumber({
      name: "nodePort",
      label: "Node Port",
      options(p: PropNumber) {
        p.numberKind = "uint32";
      },
    });
    c.fields.addNumber({
      name: "port",
      label: "Port",
      options(p: PropNumber) {
        p.numberKind = "uint32";
      },
    });
    c.fields.addEnum({
      name: "protocol",
      label: "Protocol",
      options(p: PropEnum) {
        p.variants = ["TCP", "UDP", "SCTP"];
        p.baseDefaultValue = "TCP";
      },
    });
    // NOTE: "Number or name of the port...", implying either an integer or
    // string name. ugh.
    c.fields.addText({
      name: "targetPort",
      label: "Target Port",
    });
  },
});
