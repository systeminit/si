import { Props, PropMethod, AttrList } from "@/attrList";
import {
  PropAction,
  PropLink,
  PropNumber,
  PropObject,
} from "@/components/prelude";
import { snakeCase, pascalCase } from "change-case";

export interface ComponentConstructor {
  typeName: Component["typeName"];
  displayTypeName: Component["displayTypeName"];
  noStd?: boolean;
  options?(c: Component): void;
  siPathName?: string;
}

export type ComponentAttrListName =
  | "constraints"
  | "properties"
  | "data"
  | "entityMethods"
  | "entityActions"
  | "componentMethods"
  | "entity"
  | "component"
  | "internalOnly";

export class Component {
  typeName: string;
  displayTypeName: string;
  siPathName: string;

  constraints: AttrList;
  properties: AttrList;
  data: AttrList;

  componentMethods: AttrList;
  entityMethods: AttrList;
  entityActions: AttrList;
  internalOnly: AttrList;
  entity: AttrList;
  component: AttrList;
  entityEvent: AttrList;

  noStd: boolean;

  constructor({
    typeName,
    displayTypeName,
    noStd = false,
    siPathName = "",
  }: ComponentConstructor) {
    this.typeName = typeName;
    this.displayTypeName = displayTypeName;
    this.noStd = noStd;
    this.siPathName = siPathName;
    this.constraints = new AttrList({ component: this });
    this.properties = new AttrList({ component: this, autoCreateEdits: true });
    this.data = new AttrList({ component: this });
    this.componentMethods = new AttrList({ component: this });
    this.entityMethods = new AttrList({ component: this });
    this.entityActions = new AttrList({ component: this });
    this.internalOnly = new AttrList({ component: this });
    this.entity = new AttrList({ component: this });
    this.component = new AttrList({ component: this });
    this.entityEvent = new AttrList({ component: this });
  }

  setDefaultValues(): void {
    this.universalConstraints();
    this.universalComponentMethods();
    this.universalEntityMethods();
    this.universalEntityActions();
    this.universalEntity();
    this.universalEntityEvent();
    this.universalComponent();
  }

  universalEntityActions(): void {
    this.entityActions.addAction({
      name: "syncAction",
      label: "Sync State",
      options(p: PropAction) {
        p.universal = true;
        p.request.addText({
          name: "entityId",
          label: "Entity ID",
          options(p) {
            p.universal = true;
            p.required = true;
          },
        });
      },
    });
  }

  universalEntityMethods(): void {
    this.entityMethods.addMethod({
      name: "createEntity",
      label: "Create Entity",
      options(p: PropMethod) {
        p.request.addConstraints({
          name: "constraints",
          label: "Constraints",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addProperties({
          name: "properties",
          label: "Properties",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addText({
          name: "name",
          label: "Name",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addText({
          name: "displayName",
          label: "Display Name",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addText({
          name: "description",
          label: "Description",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addText({
          name: "workplaceId",
          label: "Workplace ID",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.addEntity({
          name: "entity",
          label: "Entity",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.addEntityEvent({
          name: "entityEvent",
          label: "EntityEvent",
          options(p) {
            p.universal = true;
          },
        });
      },
    });

    this.entityMethods.addMethod({
      name: "listEntities",
      label: "List Entities",
      options(p: PropMethod) {
        p.universal = true;
        p.request.addLink({
          name: "query",
          label: "Query",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              component: "data",
              propType: "internalOnly",
              names: ["query"],
            };
          },
        });
        p.request.addNumber({
          name: "pageSize",
          label: "Page Size",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.request.addText({
          name: "orderBy",
          label: "Order By",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addLink({
          name: "orderByDirection",
          label: "Order By Direction",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              component: "data",
              propType: "internalOnly",
              names: ["pageToken", "orderByDirection"],
            };
          },
        });
        p.request.addText({
          name: "pageToken",
          label: "Page Token",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addText({
          name: "scopeByTenantId",
          label: "Scope By Tenant ID",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.addEntity({
          name: "items",
          label: "Items",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.addNumber({
          name: "totalCount",
          label: "Total Count",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.reply.addText({
          name: "nextPageToken",
          label: "Next Page Token",
          options(p) {
            p.universal = true;
          },
        });
      },
    });

    this.entityMethods.addMethod({
      name: "getEntity",
      label: "Get Entity",
      options(p: PropMethod) {
        p.universal = true;
        p.request.addText({
          name: "entityId",
          label: "Entity ID",
          options(p) {
            p.universal = true;
            p.required = true;
          },
        });
        p.reply.addEntity({
          name: "entity",
          label: `${this.displayName}`,
          options(p) {
            p.universal = true;
          },
        });
      },
    });
  }

  universalConstraints(): void {
    // Built in Constraints on every component! Hurray!
    this.constraints.addText({
      name: "componentName",
      label: "Component Name",
      options(p) {
        p.universal = true;
      },
    });
    this.constraints.addText({
      name: "componentDisplayName",
      label: "Component Display Name",
      options(p) {
        p.universal = true;
      },
    });
  }

  universalComponentMethods(): void {
    // Universal Component Methods
    this.componentMethods = new AttrList({ component: this });
    this.componentMethods.addMethod({
      name: "getComponent",
      label: "Get Component",
      options(p: PropMethod) {
        p.request.addText({
          name: "componentId",
          label: `${this.displayTypeName} ID`,
          options(p) {
            p.universal = true;
            p.required = true;
          },
        });
        p.reply.addComponent({
          name: "component",
          label: this.displayTypeName,
          options(p) {
            p.universal = true;
            p.required = true;
          },
        });
      },
    });
    this.entityMethods.addMethod({
      name: "listComponents",
      label: "List Components",
      options(p: PropMethod) {
        p.universal = true;
        p.request.addLink({
          name: "query",
          label: "Query",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              component: "data",
              propType: "internalOnly",
              names: ["query"],
            };
          },
        });
        p.request.addNumber({
          name: "pageSize",
          label: "Page Size",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.request.addText({
          name: "orderBy",
          label: "Order By",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addLink({
          name: "orderByDirection",
          label: "Order By Direction",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              component: "data",
              propType: "internalOnly",
              names: ["pageToken", "orderByDirection"],
            };
          },
        });
        p.request.addText({
          name: "pageToken",
          label: "Page Token",
          options(p) {
            p.universal = true;
          },
        });
        p.request.addText({
          name: "scopeByTenantId",
          label: "Scope By Tenant ID",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.addEntity({
          name: "items",
          label: "Items",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.addNumber({
          name: "totalCount",
          label: "Total Count",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.reply.addText({
          name: "nextPageToken",
          label: "Next Page Token",
          options(p) {
            p.universal = true;
          },
        });
      },
    });

    this.componentMethods.addMethod({
      name: "pickComponent",
      label: "Pick Component",
      options(p: PropMethod) {
        p.request.addConstraints({
          name: "constraints",
          label: "Constraints",
          options(p) {
            p.universal = true;
            p.required = true;
          },
        });
        p.reply.addConstraints({
          name: "implicitConstraints",
          label: "Implicit Constraints",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.addComponent({
          name: "component",
          label: "Chosen Component",
          options(p) {
            p.universal = true;
          },
        });
      },
    });
  }

  universalComponent(): void {
    this.component.addText({
      name: "id",
      label: "Component ID",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.component.addText({
      name: "name",
      label: "Component Name",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.component.addText({
      name: "displayName",
      label: "Component Display Name",
      options(p) {
        p.universal = true;
      },
    });
    this.component.addLink({
      name: "siStorable",
      label: "SI Storable",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          component: "data",
          propType: "internalOnly",
          names: ["storable"],
        };
      },
    });
    this.component.addLink({
      name: "siProperties",
      label: "SI Properties",
      options(p: PropLink) {
        p.universal = true;
        p.lookup = {
          component: "component",
          propType: "internalOnly",
          names: ["componentSiProperties"],
        };
      },
    });
  }

  universalEntityEvent(): void {
    this.entityEvent.addText({
      name: "id",
      label: "Entity Event ID",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entityEvent.addLink({
      name: "siStorable",
      label: "SI Storable",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          component: "data",
          propType: "internalOnly",
          names: ["storable"],
        };
      },
    });
    this.entityEvent.addText({
      name: "actionName",
      label: "Action Name",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entityEvent.addText({
      name: "createTime",
      label: "Creation Time",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entityEvent.addText({
      name: "updatedTime",
      label: "Updated Time",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entityEvent.addText({
      name: "finalTime",
      label: "Final Time",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entityEvent.addBool({
      name: "finalized",
      label: "Finalized",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entityEvent.addBool({
      name: "success",
      label: "success",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entityEvent.addLink({
      name: "siProperties",
      label: "SI Properties",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          component: "entityEvent",
          propType: "internalOnly",
          names: ["entityEventSiProperties"],
        };
      },
    });
    this.entityEvent.addEntity({
      name: "inputEntity",
      label: "Input Entity",
      options(p) {
        p.universal = true;
      },
    });
    this.entityEvent.addEntity({
      name: "outputEntity",
      label: "Output Entity",
      options(p) {
        p.universal = true;
      },
    });
    this.entityEvent.addEntity({
      name: "previousEntity",
      label: "Previous Entity",
      options(p) {
        p.universal = true;
      },
    });
    this.entityEvent.addText({
      name: "errorMessage",
      label: "Error Message",
      options(p) {
        p.universal = true;
      },
    });
    this.entityEvent.addText({
      name: "errorLines",
      label: "Error Lines",
      options(p) {
        p.repeated = true;
        p.universal = true;
      },
    });
  }

  universalEntity(): void {
    this.entity.addText({
      name: "id",
      label: "Entity ID",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entity.addText({
      name: "name",
      label: "Entity Name",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entity.addText({
      name: "displayName",
      label: "Entity Display Name",
      options(p) {
        p.universal = true;
      },
    });
    this.entity.addLink({
      name: "siStorable",
      label: "SI Storable",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          component: "data",
          propType: "internalOnly",
          names: ["storable"],
        };
      },
    });
    this.entity.addLink({
      name: "siProperties",
      label: "SI Properties",
      options(p: PropLink) {
        p.universal = true;
        p.lookup = {
          component: "entity",
          propType: "internalOnly",
          names: ["entitySiProperties"],
        };
      },
    });
    this.entity.addConstraints({
      name: "constraints",
      label: "Constraints",
      options(p: PropLink) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.entity.addConstraints({
      name: "implicitConstraints",
      label: "Implicit Constraints",
      options(p: PropLink) {
        p.universal = true;
        p.readOnly = true;
      },
    });
  }

  renderProtobufImports(): string {
    let result = "";
    const resultSet = this.renderProtobufImportWalk(this);
    let resultSetSorted = [];
    for (const importValue of resultSet.values()) {
      resultSetSorted.push(importValue);
    }
    resultSetSorted = resultSetSorted.sort();
    for (const importPart of resultSetSorted) {
      // TODO: Figure this out. It's going to be dirty for now.
      //       The issue is EntityEvent doesn't exist when we
      //       walk the imports, so it doesn't get added. We
      //       could either call the importWalk again, but only
      //       if this component requires it. Or, we can use the
      //       fact that if we import entity we also want EntityEvent,
      //       and get it done that way. So... thats what I did.
      result = result + `\nimport "${importPart}";`;
      if (importPart == "si-registry/proto/si.entity.proto") {
        result = result + `\nimport "si-registry/proto/si.entity_event.proto";`;
      }
    }
    return result;
  }

  attrLists(): string[] {
    const attrLists = [
      "constraints",
      "properties",
      "data",
      "entityMethods",
      "entityActions",
      "componentMethods",
      "internalOnly",
      "entity",
      "component",
    ];
    return attrLists;
  }

  renderProtobufImportWalk(topProp: Props | Component): Set<string> {
    const result: Set<string> = new Set();
    for (const attrList of this.attrLists()) {
      if (topProp[attrList] === undefined) {
        continue;
      }
      for (const p of topProp[attrList].attrs) {
        if (p.protobufImportPath(this.typeName)) {
          result.add(p.protobufImportPath(this.typeName));
        }
        if (this.recurseKinds().includes(p.kind())) {
          const newSet = this.renderProtobufImportWalk(p);
          for (const item of newSet.values()) {
            result.add(item);
          }
        }
      }
    }
    return result;
  }

  asEntityEvent(): PropObject {
    // eslint-disable-next-line
    const component = this;
    const p = new PropObject({
      name: "entityEvent",
      label: `${this.displayTypeName} Entity Event`,
      parentName: "",
      componentTypeName: this.typeName,
    });
    p.universal = true;
    p.properties.attrs = component.entityEvent.attrs;

    return p;
  }

  asComponent(): PropObject {
    // eslint-disable-next-line
    const component = this;
    const p = new PropObject({
      name: "component",
      label: `${this.displayTypeName} Component`,
      parentName: "",
      componentTypeName: this.typeName,
    });
    p.universal = true;
    p.properties.attrs = component.component.attrs;
    if (
      this.constraints.length &&
      !p.properties.attrs.find(pv => pv.name == "constraints")
    ) {
      p.properties.addObject({
        name: "constraints",
        label: "Constraints",
        options(p: PropObject) {
          p.universal = true;
          p.parentName = "";
          p.properties.attrs = component.constraints.attrs;
        },
      });
    }

    return p;
  }

  asEntity(): PropObject {
    // eslint-disable-next-line
    const component = this;
    const p = new PropObject({
      name: "entity",
      label: `${this.displayTypeName} Entity`,
      parentName: "",
      componentTypeName: this.typeName,
    });
    p.universal = true;
    p.properties.attrs = component.entity.attrs;
    if (
      this.properties.length &&
      !p.properties.attrs.find(pv => pv.name == "properties")
    ) {
      p.properties.addObject({
        name: "properties",
        label: "Properties",
        options(p: PropObject) {
          p.universal = true;
          p.parentName = "";
          for (const cp of component.properties.attrs) {
            //if (p.kind() == "object") {
            p.properties.addLink({
              name: cp.name,
              label: cp.label,
              options(np: PropLink) {
                np.lookup = {
                  component: cp.componentTypeName,
                  propType: "properties",
                  names: [cp.name],
                };
              },
            });
          }
        },
      });
    }
    if (this.data.length && !p.properties.attrs.find(pv => pv.name == "data")) {
      p.properties.addObject({
        name: "data",
        label: "Data",
        options(p: PropObject) {
          p.universal = true;
          p.parentName = "";
          p.properties.attrs = component.data.attrs;
        },
      });
    }
    return p;
  }

  renderProtobufEntity(): string {
    const p = this.asEntity();
    const messageContents = this.renderProtobufFromAttrList(p.properties);
    const result = `\nmessage ${p.protobufType()} {
${messageContents}
}\n`;
    return result;
  }

  renderProtobufComponent(): string {
    const p = this.asComponent();
    const messageContents = this.renderProtobufFromAttrList(p.properties);
    const result = `\nmessage ${p.protobufType()} {
${messageContents}
}\n`;
    return result;
  }

  renderProtobufEntityEvent(): string {
    const p = this.asEntityEvent();
    const messageContents = this.renderProtobufFromAttrList(p.properties);
    const result = `\nmessage ${p.protobufType()} {
${messageContents}
}\n`;
    return result;
  }

  renderProtobufServices(): string {
    const resultParts = [];
    const methodTypeSet = [
      this.componentMethods,
      this.entityMethods,
      this.entityActions,
    ];
    for (const methodType of methodTypeSet) {
      for (const p of methodType.attrs) {
        if (p.kind() == "method" || p.kind() == "action") {
          const methodName = pascalCase(p.name);
          resultParts.push(
            `  rpc ${methodName}(${methodName}Request) returns (${methodName}Reply);`,
          );
        }
      }
    }
    return resultParts.join("\n");
  }

  renderProtobufMethodMessages(): string {
    let result = "";
    //result = this.renderProtobufMessagesForAttrList(this.componentMethods);
    //result =
    //  result + this.renderProtobufMessagesForAttrList(this.entityMethods);
    //result =
    //  result + this.renderProtobufMessagesForAttrList(this.entityActions);
    return result;
  }

  recurseKinds(): string[] {
    return ["object", "action", "method"];
  }

  renderProtobufMessagesForAttrList(attrList: AttrList): string {
    let result = "";

    // Render myself
    for (const p of attrList.attrs) {
      if (this.recurseKinds().includes(p.kind())) {
        for (const bag of p.bagNames()) {
          const messageContents = this.renderProtobufFromAttrList(p[bag]);
          if (bag == "request") {
            result = result.concat(`\nmessage ${p.protobufType("request")} {
${messageContents}
}\n`);
          } else if (bag == "reply") {
            result = result.concat(`\nmessage ${p.protobufType("reply")} {
${messageContents}
}\n`);
          } else {
            result = result.concat(`\nmessage ${p.protobufType()} {
${messageContents}
}\n`);
          }
        }
      } else if (p.kind() == "enum") {
        // @ts-ignore
        const definition = p.protobufEnumDefinition(0);
        result = result.concat(`\nenum ${p.protobufType()} {
${definition}
}\n`);
      }
    }

    // Render my descendents
    for (const p of attrList.attrs) {
      if (this.recurseKinds().includes(p.kind())) {
        for (const bag of p.bagNames()) {
          const message = this.renderProtobufMessagesForAttrList(p[bag]);
          result = result.concat(message);
        }
      }
    }
    return result;
  }

  renderProtobufEnumMessages(): string {
    const result = "";
    // Walk all the attrLists searching for Enums
    // Find any enums and print them, auto-namespacing-variants
    // For any recurseKinds, recurse and do them same
    return result;
  }

  // For each Prop in an AttrList
  //  * 1) Write our own PropObject message
  //  * 2) Write the message type for any object properties
  // Returns the full string of all protobuf object messages
  renderProtobufObjectMessages(): string {
    let result = "";
    for (const attrList of this.attrLists()) {
      result = result + `\n// ${attrList}\n`;
      if (this[attrList].length > 0) {
        result =
          result +
          "\n" +
          this.renderProtobufMessagesForAttrList(this[attrList]);
      }
    }
    return result;
  }

  renderProtobufFromAttrList(attrList: AttrList): string {
    const result = [];
    let universalBase = 0;
    let customBase = 1000;

    for (const index in attrList.attrs) {
      const p = attrList.attrs[index];

      if (p.universal) {
        universalBase = universalBase + 1;
        result.push("  " + p.protobufDefinition(universalBase));
      } else {
        customBase = customBase + 1;
        result.push("  " + p.protobufDefinition(customBase));
      }
    }
    return result.join("\n");
  }

  renderProtobufSection(
    section: "constraints" | "properties" | "data",
  ): string {
    return this.renderProtobufFromAttrList(this[section]);
  }

  protobufPackageName(): string {
    return snakeCase(this.typeName);
  }

  protobufServiceName(): string {
    return pascalCase(this.typeName);
  }
}
