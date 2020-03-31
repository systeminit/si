import { registry } from "@/componentRegistry";
import { Component } from "@/component";
import { PropObject, PropAction, PropMethod } from "@/attrList";
import { PropNumber } from "@/prop/number";
import { PropText } from "@/prop/text";
import { CodegenProtobuf } from "@/codegen/protobuf";
import "@/loader";

test("registryLoad", done => {
  const c = registry.get("data");
  const cp = new CodegenProtobuf(c);
  console.log(cp.generateString());

  expect(c.typeName).toBe("data");
  done();
});

// test("actions", done => {
//   const datac = registry.component({
//     typeName: "data",
//     displayTypeName: "SI Data",
//     options(c) {
//       c.internalOnly.addObject({
//         name: "query",
//         label: "Query",
//         options(p: PropObject) {
//           p.properties.addText({
//             name: "queryString",
//             label: "Query String",
//           });
//         },
//       });
//     },
//   });
//   const c = registry.component({
//     typeName: "lambOfGod",
//     displayTypeName: "Lamb of God",
//     options(c) {
//       c.properties.addText({
//         name: "isGood",
//         label: "Is Good",
//       });
//       c.componentMethods.addMethod({
//         name: "getComponent",
//         label: "Get Component",
//         options(p: PropMethod) {
//           p.request.addText({
//             name: "componentId",
//             label: "Component ID",
//             options(p: PropText) {
//               p.required = true;
//             },
//           });
//           p.reply.addComponent({
//             name: "component",
//             label: c.displayTypeName,
//             options(p: PropText) {
//               p.required = true;
//             },
//           });
//         },
//       });
//       c.componentMethods.addMethod({
//         name: "listComponents",
//         label: "List Components",
//         options(p: PropMethod) {
//           p.request.addExisting(
//             registry.lookupProp({
//               component: "data",
//               propType: "internalOnly",
//               names: ["query"],
//             }),
//           );
//           p.reply.addComponent({
//             name: c.typeName,
//             label: c.displayTypeName,
//             options(p: PropText) {
//               p.required = true;
//             },
//           });
//         },
//       });
//     },
//   });
//
//   let result = c.renderProtobufMethodMessages();
//   console.log(result);
//
//   result = c.renderProtobufServices();
//   console.log(result);
//
//   done();
// });
//
// test("recursiveGenerator", done => {
//   const c = registry.component({
//     typeName: "kubernetesDeployment",
//     displayTypeName: "Kubernetes Deployment",
//     options(c) {
//       c.properties.addText({
//         name: "kubernetesVersion",
//         label: "Kubernetes Version",
//         options(p) {
//           p.required = true;
//         },
//       });
//       c.properties.addObject({
//         name: "kubernetesObject",
//         label: "Kubernetes Object",
//         options(p: PropObject) {
//           p.properties.addText({
//             name: "apiVersion",
//             label: "API Version",
//             options(p) {
//               p.required = true;
//             },
//           });
//           p.properties.addText({
//             name: "kind",
//             label: "Kind",
//             options(p) {
//               p.required = true;
//             },
//           });
//           p.properties.addObject({
//             name: "metadata",
//             label: "Meta Data",
//             options(p: PropObject) {
//               p.lookupTag = "kubernetes.metadata";
//               p.required = true;
//               p.properties.addText({
//                 name: "name",
//                 label: "Name",
//               });
//               p.properties.addMap({
//                 name: "labels",
//                 label: "Labels",
//               });
//               p.properties.addObject({
//                 name: "glory",
//                 label: "Glorious",
//                 options(p: PropObject) {
//                   p.properties.addText({ name: "sleep", label: "sleepy" });
//                 },
//               });
//             },
//           });
//         },
//       });
//     },
//   });
//
//   const result = c.renderProtobufObjectMessages();
//
//   const prop = registry.lookupProp({
//     component: "kubernetesDeployment",
//     propType: "properties",
//     names: ["kubernetesObject", "metadata", "glory"],
//   });
//
//   expect(prop.name).toBe("glory");
//
//   done();
// });
//
// test("createComponent", done => {
//   const c = new Component({ typeName: "poop", displayTypeName: "pants" });
//   expect(c).toBeInstanceOf(Component);
//   done();
// });
//
// test("createDsl", done => {
//   const c = registry.component({
//     typeName: "kubernetesDeployment",
//     displayTypeName: "Kubernetes Deployment",
//     options(c) {
//       // c.siActions.addQuery({
//       //   name: "GetEntity",
//       //   label: "Get Entity",
//       //   options(p) {
//       //     p.required = true;
//       //     p.parameters.addText({
//       //       name: "entityId",
//       //       label: "Entity ID",
//       //       options(p) {
//       //         p.required = true;
//       //       },
//       //     });
//       //     p.reply.addText({
//       //       name: "entity",
//       //       label: "Entity",
//       //       options(p) {
//       //         p.typeHint = "Entity";
//       //         p.required = true;
//       //       },
//       //     });
//       //   },
//       // });
//       c.constraints.addText({
//         name: "kubernetesVersion",
//         label: "Kubernetes Version",
//         options(p) {
//           p.required = true;
//         },
//       });
//       c.properties.addObject({
//         name: "kubernetesObject",
//         label: "Kubernetes Object",
//         options(p: PropObject) {
//           p.properties.addText({
//             name: "apiVersion",
//             label: "API Version",
//             options(p) {
//               p.required = true;
//             },
//           });
//           p.properties.addText({
//             name: "kind",
//             label: "Kind",
//             options(p) {
//               p.required = true;
//             },
//           });
//           p.properties.addObject({
//             name: "metadata",
//             label: "Meta Data",
//             options(p: PropObject) {
//               p.lookupTag = "kubernetes.metadata";
//               p.required = true;
//               p.properties.addText({
//                 name: "name",
//                 label: "Name",
//               });
//               p.properties.addMap({
//                 name: "labels",
//                 label: "Labels",
//               });
//             },
//           });
//           p.properties.addObject({
//             name: "spec",
//             label: "Deployment Spec",
//             options(p: PropObject) {
//               p.properties.addNumber({
//                 name: "replicas",
//                 label: "Replicas",
//                 options(p: PropNumber) {
//                   p.numberKind = "uint32";
//                 },
//               });
//               p.properties.addObject({
//                 name: "selector",
//                 label: "Label Selector",
//                 options(p: PropObject) {
//                   p.properties.addMap({
//                     name: "matchLabels",
//                     label: "Match Labels",
//                   });
//                 },
//               });
//               p.properties.addObject({
//                 name: "template",
//                 label: "Pod Template Spec",
//                 options(p: PropObject) {
//                   p.properties.addFromRegistry("kubernetes.metadata", {
//                     name: "metadata",
//                     label: "Metadata",
//                   });
//                   p.properties.addObject({
//                     name: "spec",
//                     label: "Pod Spec",
//                     options(p: PropObject) {
//                       p.properties.addObject({
//                         name: "containers",
//                         label: "Containers",
//                         options(p: PropObject) {
//                           p.repeated = true;
//                           p.properties.addText({ name: "name", label: "Name" });
//                           p.properties.addText({
//                             name: "image",
//                             label: "Image",
//                           });
//                           p.properties.addObject({
//                             name: "ports",
//                             label: "Container Ports",
//                             options(p: PropObject) {
//                               p.repeated = true;
//                               p.properties.addNumber({
//                                 name: "containerPort",
//                                 label: "Container Port",
//                                 options(p: PropNumber) {
//                                   p.numberKind = "uint32";
//                                 },
//                               });
//                             },
//                           });
//                         },
//                       });
//                     },
//                   });
//                 },
//               });
//             },
//           });
//         },
//       });
//     },
//   });
//
//   // Render the madness
//   const cp = new CodegenProtobuf(c);
//   const result = cp.generateString();
//   //console.log(result);
//
//   // We should be able to get back things we add
//   const kvc = c.constraints.getEntry("kubernetesVersion");
//   expect(kvc).not.toBeUndefined();
//   expect(kvc).toMatchObject({ name: "kubernetesVersion" });
//
//   // Component should have default constraints
//   const defaultConstraints = ["componentName", "componentDisplayName"];
//   for (const constraintName of defaultConstraints) {
//     const constraint = c.constraints.getEntry(constraintName);
//     expect(constraint).not.toBeUndefined();
//     if (constraint !== undefined) {
//       expect(constraint).toMatchObject({
//         name: constraintName,
//       });
//     }
//   }
//
//   // Properties
//   done();
// });
