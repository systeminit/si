// import {
//   PropObject,
// } from "../../../components/prelude";

// import { registry } from "../../../registry";

// registry.base({
//   typeName: "kubernetesLoadBalancerIngress",
//   displayTypeName: "Kubernetes Load Balancer Ingress",
//   serviceName: "kubernetes",
//   options(c) {
//     c.fields.addObject({
//       name: "ingress",
//       label: "Load Balancer Ingress",
//       options(p: PropObject) {
//         p.repeated = true;
//         p.properties.addText({
//           name: "hostname",
//           label: "Hostname",
//         });
//         p.properties.addText({
//           name: "ip",
//           label: "IP",
//         });
//       },
//     });
//   },
// });
