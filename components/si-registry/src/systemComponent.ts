import { PropLink } from "./prop/link";
import { PropNumber } from "./prop/number";
import { PropObject, PropMethod, IntegrationService } from "./attrList";
import { camelCase } from "change-case";
import { AssociationList } from "./systemObject/associations";
import {
  Entity,
  CalculateConfiguresReply,
  CalculatePropertiesResult,
  CalculatePropertiesFullResult,
  CalculatePropertiesRequest,
  System,
  ActionRequest,
  ActionReply,
  ResourceHealth,
  ResourceStatus,
  SyncResourceRequest,
  SyncResourceReply,
} from "./veritech/intelligence";
import _ from "lodash";
import YAML from "yaml";
import { registry } from "./registry";
import { Event, EventLogLevel } from "./veritech/eventLog";

export type ObjectTypes =
  | BaseObject
  | SystemObject
  | ComponentObject
  | EntityObject
  | EntityEventObject;

export interface BaseObjectConstructor {
  typeName: BaseObject["typeName"];
  displayTypeName: BaseObject["displayTypeName"];
  serviceName: string;
  siPathName?: string;
  options?(c: BaseObject): void;
}

export interface AddMethodConstructor {
  isPrivate?: PropMethod["isPrivate"];
}

export interface IEntity {
  uiVisible: boolean;
  uiMenuCategory?: UiMenuCategory | undefined;
  uiMenuSubCategory?: UiAwsMenuSubCategory;
  uiMenuDisplayName?: string;
}

export type UiMenuCategory =
  | "application"
  | "kubernetes"
  | "docker"
  | "aws"
  | "none";

export type UiAwsMenuSubCategory = "iam" | "eks";

export class BaseObject {
  typeName: string;
  displayTypeName: string;
  siPathName: string;
  serviceName: string;
  mvcc: boolean;

  rootProp: PropObject;
  methodsProp: PropObject;
  associations: AssociationList;

  constructor({
    typeName,
    displayTypeName,
    serviceName,
    siPathName = "",
  }: BaseObjectConstructor) {
    this.typeName = camelCase(typeName);
    this.displayTypeName = displayTypeName;
    this.siPathName = siPathName;
    this.serviceName = serviceName || typeName;
    this.rootProp = new PropObject({
      name: typeName,
      label: displayTypeName,
      componentTypeName: typeName,
      parentName: "",
    });
    this.methodsProp = new PropObject({
      name: `${typeName}`,
      label: `${displayTypeName} Methods`,
      componentTypeName: typeName,
      parentName: "",
    });
    this.associations = new AssociationList();
    this.mvcc = false;
  }

  get fields(): BaseObject["rootProp"]["properties"] {
    return this.rootProp.properties;
  }

  get methods(): BaseObject["methodsProp"]["properties"] {
    return this.methodsProp.properties;
  }

  kind(): string {
    return "baseObject";
  }
}

export class SystemObject extends BaseObject {
  naturalKey = "name";
  migrateable = false;

  constructor(args: BaseObjectConstructor) {
    super(args);
    this.setSystemObjectDefaults();
  }

  setSystemObjectDefaults(): void {
    this.fields.addText({
      name: "id",
      label: `${this.displayTypeName} ID`,
      options(p) {
        p.universal = true;
        p.readOnly = true;
        p.required = true;
      },
    });
    if (!this.typeName.endsWith("EntityEvent")) {
      this.fields.addText({
        name: "name",
        label: `${this.displayTypeName} Name`,
        options(p) {
          p.universal = true;
          p.readOnly = true;
          p.required = true;
        },
      });
      this.fields.addText({
        name: "displayName",
        label: `${this.displayTypeName} Display Name`,
        options(p) {
          p.universal = true;
          p.readOnly = true;
          p.required = true;
        },
      });
    }
    this.fields.addLink({
      name: "siStorable",
      label: "SI Storable",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = false;
        p.lookup = {
          typeName: "dataStorable",
        };
        p.required = true;
      },
    });
  }

  kind(): string {
    return "systemObject";
  }

  addGetMethod(args: AddMethodConstructor = {}): void {
    // eslint-disable-next-line
    const systemObject = this;

    systemObject.methods.addMethod({
      name: "get",
      label: `Get a ${systemObject.displayTypeName}`,
      options(p: PropMethod) {
        p.isPrivate = args.isPrivate || false;
        p.request.properties.addText({
          name: "id",
          label: `${systemObject.displayTypeName} ID`,
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${systemObject.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: systemObject.typeName,
            };
          },
        });
      },
    });
  }

  addCreateMethod(args: AddMethodConstructor = {}): void {
    // eslint-disable-next-line
    const systemObject = this;

    systemObject.methods.addMethod({
      name: "create",
      label: `Create a ${systemObject.displayTypeName}`,
      options(p: PropMethod) {
        p.isPrivate = args.isPrivate || false;
        p.mutation = true;

        for (const prop of systemObject.fields.attrs) {
          if (prop.name == "id" || prop.name == "siStorable") {
            continue;
          }
          p.request.properties.addLink({
            name: prop.name,
            label: prop.label,
            options(l: PropLink) {
              l.lookup = {
                typeName: systemObject.typeName,
                names: [prop.name],
              };
              if (prop.repeated) {
                l.repeated = true;
              }
            },
          });
        }
        p.reply.properties.addLink({
          name: "item",
          label: `${systemObject.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: systemObject.typeName,
            };
          },
        });
      },
    });
  }

  addUpdateMethod(args: AddMethodConstructor = {}): void {
    // eslint-disable-next-line
    const systemObject = this;

    systemObject.methods.addMethod({
      name: "update",
      label: `Update a ${systemObject.displayTypeName}`,
      options(p: PropMethod) {
        p.isPrivate = args.isPrivate || false;
        p.mutation = true;

        for (const prop of systemObject.fields.attrs) {
          if (prop.name == "siStorable" || prop.name == "siProperties") {
            continue;
          }
          p.request.properties.addLink({
            name: prop.name,
            label: prop.label,
            options(l: PropLink) {
              l.lookup = {
                typeName: systemObject.typeName,
                names: [prop.name],
              };
              if (prop.repeated) {
                l.repeated = true;
              }
            },
          });
        }
        p.reply.properties.addLink({
          name: "item",
          label: `${systemObject.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: systemObject.typeName,
            };
          },
        });
      },
    });
  }

  addListMethod(args: AddMethodConstructor = {}): void {
    // eslint-disable-next-line
    const systemObject = this;
    systemObject.methods.addMethod({
      name: "list",
      label: `List ${systemObject.displayTypeName}`,
      options(p: PropMethod) {
        p.universal = true;
        p.isPrivate = args.isPrivate || false;
        p.request.properties.addLink({
          name: "query",
          label: "Query",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              typeName: "dataQuery",
            };
          },
        });
        p.request.properties.addNumber({
          name: "pageSize",
          label: "Page Size",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.request.properties.addText({
          name: "orderBy",
          label: "Order By",
          options(p) {
            p.universal = true;
          },
        });
        p.request.properties.addLink({
          name: "orderByDirection",
          label: "Order By Direction",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              typeName: "dataPageToken",
              names: ["orderByDirection"],
            };
          },
        });
        p.request.properties.addText({
          name: "pageToken",
          label: "Page Token",
          options(p) {
            p.universal = true;
          },
        });
        p.request.properties.addText({
          name: "scopeByTenantId",
          label: "Scope By Tenant ID",
          options(p) {
            p.universal = true;
          },
        });
        p.reply.properties.addLink({
          name: "items",
          label: "Items",
          options(p: PropLink) {
            p.universal = true;
            p.required = true;
            p.repeated = true;
            p.lookup = {
              typeName: systemObject.typeName,
            };
          },
        });
        p.reply.properties.addNumber({
          name: "totalCount",
          label: "Total Count",
          options(p: PropNumber) {
            p.universal = true;
            p.numberKind = "uint32";
          },
        });
        p.reply.properties.addText({
          name: "nextPageToken",
          label: "Next Page Token",
          options(p) {
            p.universal = true;
          },
        });
      },
    });
  }
}

export class ComponentObject extends SystemObject {
  baseTypeName: string;

  constructor(args: BaseObjectConstructor) {
    const typeName = `${args.typeName}Component`;
    const displayTypeName = `${args.displayTypeName} Component`;
    super({
      typeName,
      displayTypeName,
      serviceName: args.serviceName,
    });
    this.baseTypeName = args.typeName;
    this.setComponentDefaults();
  }

  setComponentDefaults(): void {
    const baseTypeName = this.baseTypeName;

    this.migrateable = true;

    this.addGetMethod();
    this.addListMethod();

    this.fields.addText({
      name: "description",
      label: "Component Description",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    this.fields.addObject({
      name: "constraints",
      label: "Component Constraints",
      options(p: PropObject) {
        p.universal = true;
        p.required = true;
        p.properties.addText({
          name: "componentName",
          label: "Component Name",
          options(p) {
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "componentDisplayName",
          label: "Component Display Name",
          options(p) {
            p.universal = true;
          },
        });
      },
    });
    this.fields.addLink({
      name: "siProperties",
      label: "SI Properties",
      options(p: PropLink) {
        p.universal = true;
        p.lookup = {
          typeName: "componentSiProperties",
        };
        p.required = true;
      },
    });

    this.methods.addMethod({
      name: "create",
      label: "Create a Component",
      options(p: PropMethod) {
        p.mutation = true;
        p.hidden = true;
        p.isPrivate = true;
        p.request.properties.addText({
          name: "name",
          label: "Integration Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Integration Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "description",
          label: "Integration Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addLink({
          name: "constraints",
          label: "Constraints",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              typeName: `${baseTypeName}Component`,
              names: ["constraints"],
            };
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "Si Properties",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "componentSiProperties",
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${baseTypeName}Component Item`,
          options(p: PropLink) {
            p.universal = true;
            p.readOnly = true;
            p.lookup = {
              typeName: `${baseTypeName}Component`,
            };
          },
        });
      },
    });
    this.methods.addMethod({
      name: "pick",
      label: "Pick Component",
      options(p: PropMethod) {
        p.request.properties.addLink({
          name: "constraints",
          label: "Constraints",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              typeName: `${baseTypeName}Component`,
              names: ["constraints"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "implicitConstraints",
          label: "Implicit Constraints",
          options(p: PropLink) {
            p.universal = true;
            p.required = true;
            p.lookup = {
              typeName: `${baseTypeName}Component`,
              names: ["constraints"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "component",
          label: "Chosen Component",
          options(p: PropLink) {
            p.universal = true;
            p.lookup = {
              typeName: `${baseTypeName}Component`,
            };
          },
        });
      },
    });
  }

  get constraints(): ComponentObject["rootProp"]["properties"] {
    const constraintProp = this.fields.getEntry("constraints") as PropObject;
    return constraintProp.properties;
  }

  kind(): string {
    return "componentObject";
  }
}

interface EntityObjectIntelligence {
  // prettier-ignore
  //calculateProperties: (setProperties: Record<string, any>,) => Record<string, any>; // eslint-disable-line
  calculateProperties?: (req: CalculatePropertiesRequest) => CalculatePropertiesResult;
  calculateConfigures?: (
    entity: Entity,
    configures: Entity[],
    systems: System[],
  ) => CalculateConfiguresReply;
  actions?: {
    [key: string]: (
      request: ActionRequest,
      event: Event,
    ) => Promise<ActionReply>;
  };
  syncResource?: (
    request: SyncResourceRequest,
    event: Event,
  ) => Promise<SyncResourceReply>;
}

export class EntityObject extends SystemObject {
  baseTypeName: string;
  integrationServices: IntegrationService[];
  intelligence: EntityObjectIntelligence;
  inputTypes: EntityObject[];
  secretObjectType: string | undefined;
  secretKind: string | undefined;
  iEntity?: IEntity | undefined;

  constructor(args: BaseObjectConstructor) {
    const typeName = `${args.typeName}`;
    const displayTypeName = `${args.displayTypeName}`;
    super({
      typeName,
      displayTypeName,
      serviceName: args.serviceName,
    });
    this.baseTypeName = args.typeName;
    this.integrationServices = [];
    this.setEntityDefaults();
    this.intelligence = {};
    this.inputTypes = [];
    this.secretObjectType = undefined;
    this.secretKind = undefined;
  }

  inputType(typeName: string): void {
    const entityObj = registry.get(typeName) as EntityObject;
    this.inputTypes.push(entityObj);
  }

  secretType(
    secretObjectType: EntityObject["secretObjectType"],
    secretKind: EntityObject["secretKind"],
  ): void {
    this.secretObjectType = secretObjectType;
    this.secretKind = secretKind;
  }

  display(entity: Entity, action?: string, hypothetical?: boolean): string {
    let s = `${entity.objectType}${JSON.stringify(
      { name: entity.name, id: entity.id },
      undefined,
      0,
    )}`;
    if (action) {
      s += `.${action}()`;
    }
    if (hypothetical) {
      s = `Hypothetical(${s})`;
    }
    return s;
  }

  async syncResource(
    request: SyncResourceRequest,
    event: Event,
  ): Promise<SyncResourceReply> {
    const syncResourceFunc = this.intelligence.syncResource;

    if (syncResourceFunc) {
      event.log(EventLogLevel.Info, "resource sync", {
        default: false,
      });
      return await syncResourceFunc(request, event);
    } else {
      event.log(EventLogLevel.Info, "resource sync", {
        default: true,
      });
      return {
        resource: {
          health: ResourceHealth.Ok,
          status: ResourceStatus.Created,
          state: {
            siDefaultSync: true,
          },
        },
      };
    }
  }

  async action(request: ActionRequest, event: Event): Promise<ActionReply> {
    const actions = this.intelligence.actions;

    if (actions && actions[request.action]) {
      event.log(
        EventLogLevel.Info,
        `invoking action(${this.display(
          request.entity,
          request.action,
          request.hypothetical,
        )})`,
        {
          default: false,
        },
      );
      console.log(
        `invoking action(${this.display(
          request.entity,
          request.action,
          request.hypothetical,
        )})`,
      );
      return actions[request.action](request, event);
    } else if (request.action == "delete") {
      event.log(
        EventLogLevel.Info,
        `invoking delete(${this.display(
          request.entity,
          undefined,
          request.hypothetical,
        )}); marking resource state as deleted`,
        {
          default: false,
        },
      );
      console.log(
        `invoking delete(${this.display(
          request.entity,
          undefined,
          request.hypothetical,
        )}); marking resource state as deleted`,
      );
      return {
        resource: {
          health: ResourceHealth.Unknown,
          status: ResourceStatus.Deleted,
          state: {
            siDefaultDelete: true,
          },
        },
        actions: [],
      };
    } else {
      event.log(
        EventLogLevel.Fatal,
        `cannot find action '${request.action}' for ${this.display(
          request.entity,
          undefined,
          request.hypothetical,
        )}]`,
        {},
      );
      throw new Error(
        `cannot find action '${request.action}' for ${this.display(
          request.entity,
          undefined,
          request.hypothetical,
        )}]`,
      );
    }
  }

  // Based on the manual properties and expression properties, pass the results to the inference
  calculateProperties(
    req: CalculatePropertiesRequest,
  ): CalculatePropertiesFullResult {
    //entity.properties = entity.manualProperties;
    // First calculate the entire baseline
    // Then calculate manual properties
    // Then calculate expression properties, preffering existing manual
    // - For each system an
    // Get the manual properties
    // Calculte the expression properties, and merge with manual, preferring manual properties
    // Finally pass the results to the inferProperties function, get the results
    //
    // @ts-ignore
    //const propArray = this.rootProp.properties.getEntry("properties").properties
    //  .attrs;
    //for (const prop of propArray) {
    //  if (prop.kind() == "code") {
    //    for (const rel of prop.relationships.all()) {
    //      if (rel.kind() == "updates") {
    //        const otherProp = rel.partner.names[1];
    //        properties["__baseline"][prop.name] = YAML.stringify(
    //          properties["__baseline"][otherProp],
    //        );
    //      }
    //    }
    //  }
    //}

    const entity = req.entity;
    let inferredProperties = entity.inferredProperties;
    const manualProperties = entity.manualProperties;

    const calculatePropertiesFunc = this.intelligence.calculateProperties;

    if (calculatePropertiesFunc) {
      console.log(`invoking calculateProperties(${this.display(entity)})`);
      const response: CalculatePropertiesResult = calculatePropertiesFunc(req);
      inferredProperties = response.inferredProperties;
    }

    const properties = _.merge({}, inferredProperties, manualProperties);
    if (properties.__baseline.kubernetesObject) {
      properties.__baseline.kubernetesObjectYaml = YAML.stringify(
        properties.__baseline.kubernetesObject,
      );
    }

    // Check if anything is a code property, and if it is, calculate it.
    return { properties, inferredProperties };
  }

  calculateConfigures(
    entity: Entity,
    configures: Entity[],
    systems: System[],
  ): CalculateConfiguresReply {
    const calculateConfiguresFunc = this.intelligence.calculateConfigures;

    if (calculateConfiguresFunc) {
      console.log(`invoking calculateConfigures(${this.display(entity)})`);
      return calculateConfiguresFunc(entity, configures, systems);
    } else {
      console.log(
        `returning default calculateConfigures() for ${this.display(entity)}`,
      );
      return {
        keep: _.map(configures, entity => {
          return { id: entity.id, systems: _.map(systems, s => s.id) };
        }),
      };
    }
  }

  setEntityDefaults(): void {
    const baseTypeName = this.baseTypeName;

    this.mvcc = true;

    this.addGetMethod();
    this.addListMethod();

    this.fields.addText({
      name: "description",
      label: "Entity Description",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    this.fields.addLink({
      name: "siProperties",
      label: "SI Properties",
      options(p: PropLink) {
        p.universal = true;
        p.lookup = {
          typeName: "entitySiProperties",
        };
        p.required = true;
      },
    });
    this.fields.addObject({
      name: "properties",
      label: "Properties",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    this.fields.addLink({
      name: "constraints",
      label: "Constraints",
      options(p: PropLink) {
        p.universal = true;
        p.readOnly = true;
        p.lookup = {
          typeName: `${baseTypeName}Component`,
          names: ["constraints"],
        };
      },
    });
    this.fields.addLink({
      name: "implicitConstraints",
      label: "Implicit Constraints",
      options(p: PropLink) {
        p.universal = true;
        p.readOnly = true;
        p.lookup = {
          typeName: `${baseTypeName}Component`,
          names: ["constraints"],
        };
      },
    });

    this.methods.addMethod({
      name: "create",
      label: "Create Entity",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "name",
          label: "Name",
          options(p) {
            p.required = true;
            p.universal = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Display Name",
          options(p) {
            p.required = true;
            p.universal = true;
          },
        });
        p.request.properties.addText({
          name: "description",
          label: "Description",
          options(p) {
            p.required = true;
            p.universal = true;
          },
        });
        p.request.properties.addText({
          name: "workspaceId",
          label: `Workspace ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
          },
        });
        p.request.properties.addText({
          name: "changeSetId",
          label: `Change Set ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
          },
        });
        p.request.properties.addText({
          name: "editSessionId",
          label: `Edit Session ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
          },
        });
        p.request.properties.addLink({
          name: "properties",
          label: "Properties",
          options(p: PropLink) {
            p.universal = true;
            p.readOnly = true;
            p.lookup = {
              typeName: `${baseTypeName}Entity`,
              names: ["properties"],
            };
          },
        });
        p.request.properties.addLink({
          name: "constraints",
          label: "Constraints",
          options(p: PropLink) {
            p.universal = true;
            p.readOnly = true;
            p.lookup = {
              typeName: `${baseTypeName}Component`,
              names: ["constraints"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${baseTypeName}Entity Item`,
          options(p: PropLink) {
            p.universal = true;
            p.readOnly = true;
            p.lookup = {
              typeName: `${baseTypeName}Entity`,
            };
          },
        });
      },
    });

    this.methods.addMethod({
      name: "delete",
      label: "Delete Entity",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "id",
          label: `${baseTypeName}Entity ID`,
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "changeSetId",
          label: `Change Set ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
          },
        });
        p.request.properties.addText({
          name: "editSessionId",
          label: `Edit Session ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${baseTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: `${baseTypeName}Entity`,
            };
          },
        });
      },
    });

    this.methods.addMethod({
      name: "update",
      label: "Update an Entity",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "id",
          label: `${baseTypeName}Entity ID`,
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "changeSetId",
          label: `Change Set ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
          },
        });
        p.request.properties.addText({
          name: "editSessionId",
          label: `Edit Session ID`,
          options(p) {
            p.required = true;
            p.hidden = true;
          },
        });
        p.request.properties.addObject({
          name: "update",
          label: `${baseTypeName} Item Update`,
          options(p: PropObject) {
            p.properties.addLink({
              name: "name",
              label: "name",
              options(p: PropLink) {
                p.required = false;
                p.lookup = {
                  typeName: `${baseTypeName}Entity`,
                  names: ["name"],
                };
              },
            });
            p.properties.addLink({
              name: "displayName",
              label: "displayName",
              options(p: PropLink) {
                p.required = false;
                p.lookup = {
                  typeName: `${baseTypeName}Entity`,
                  names: ["displayName"],
                };
              },
            });
            p.properties.addLink({
              name: "description",
              label: "description",
              options(p: PropLink) {
                p.required = false;
                p.lookup = {
                  typeName: `${baseTypeName}Entity`,
                  names: ["description"],
                };
              },
            });
            p.properties.addLink({
              name: "properties",
              label: "properties",
              options(p: PropLink) {
                p.required = false;
                p.lookup = {
                  typeName: `${baseTypeName}Entity`,
                  names: ["properties"],
                };
              },
            });
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${baseTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: `${baseTypeName}Entity`,
            };
          },
        });
      },
    });
  }

  get properties(): EntityObject["rootProp"]["properties"] {
    const prop = this.fields.getEntry("properties") as PropObject;
    return prop.properties;
  }

  kind(): string {
    return "entityObject";
  }
}

export class EntityEventObject extends SystemObject {
  baseTypeName: string;

  constructor(args: BaseObjectConstructor) {
    const typeName = `${args.typeName}EntityEvent`;
    const displayTypeName = `${args.displayTypeName} EntityEvent`;
    super({
      typeName,
      displayTypeName,
      serviceName: args.serviceName,
    });
    this.baseTypeName = args.typeName;
    this.setEntityEventDefaults();
  }

  setEntityEventDefaults(): void {
    const baseTypeName = this.baseTypeName;
    this.fields.addText({
      name: "actionName",
      label: "Action Name",
      options(p) {
        p.universal = true;
        p.required = true;
        p.readOnly = true;
      },
    });
    this.fields.addText({
      name: "createTime",
      label: "Creation Time",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.fields.addText({
      name: "updatedTime",
      label: "Updated Time",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.fields.addText({
      name: "finalTime",
      label: "Final Time",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.fields.addBool({
      name: "success",
      label: "success",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.fields.addBool({
      name: "finalized",
      label: "Finalized",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.fields.addText({
      name: "userId",
      label: "User ID",
      options(p) {
        p.universal = true;
        p.readOnly = true;
      },
    });
    this.fields.addText({
      name: "outputLines",
      label: "Output Lines",
      options(p) {
        p.repeated = true;
        p.universal = true;
      },
    });
    this.fields.addText({
      name: "errorLines",
      label: "Error Lines",
      options(p) {
        p.repeated = true;
        p.universal = true;
      },
    });
    this.fields.addText({
      name: "errorMessage",
      label: "Error Message",
      options(p) {
        p.universal = true;
      },
    });
    this.fields.addLink({
      name: "previousEntity",
      label: "Previous Entity",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          typeName: `${baseTypeName}Entity`,
        };
      },
    });
    this.fields.addLink({
      name: "inputEntity",
      label: "Input Entity",
      options(p: PropLink) {
        p.universal = true;
        p.required = true;
        p.hidden = true;
        p.lookup = {
          typeName: `${baseTypeName}Entity`,
        };
      },
    });
    this.fields.addLink({
      name: "outputEntity",
      label: "Output Entity",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          typeName: `${baseTypeName}Entity`,
        };
      },
    });
    this.fields.addLink({
      name: "siProperties",
      label: "SI Properties",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          typeName: "entityEventSiProperties",
        };
      },
    });

    this.addListMethod();
    this.addGetMethod();
  }

  kind(): string {
    return "entityEventObject";
  }
}

export interface ComponentAndEntityObjectConstructor {
  typeName: BaseObject["typeName"];
  displayTypeName: BaseObject["displayTypeName"];
  siPathName?: string;
  serviceName: string;
  options?(c: ComponentAndEntityObject): void;
}

export class ComponentAndEntityObject {
  component: ComponentObject;
  entity: EntityObject;
  entityEvent: EntityEventObject;

  constructor(args: ComponentAndEntityObjectConstructor) {
    this.component = new ComponentObject({
      typeName: args.typeName,
      displayTypeName: args.displayTypeName,
      siPathName: args.siPathName,
      serviceName: args.serviceName,
    });
    this.entity = new EntityObject({
      typeName: args.typeName,
      displayTypeName: args.displayTypeName,
      siPathName: args.siPathName,
      serviceName: args.serviceName,
    });

    this.entityEvent = new EntityEventObject({
      typeName: args.typeName,
      displayTypeName: args.displayTypeName,
      siPathName: args.siPathName,
      serviceName: args.serviceName,
    });
  }

  get associations(): AssociationList {
    return this.entity.associations;
  }

  set associations(value: AssociationList) {
    this.entity.associations = value;
  }

  get properties(): EntityObject["rootProp"]["properties"] {
    const prop = this.entity.fields.getEntry("properties") as PropObject;
    prop.properties.autoCreateEdits = true;
    return prop.properties;
  }

  get constraints(): ComponentObject["rootProp"]["properties"] {
    const prop = this.component.fields.getEntry("constraints") as PropObject;
    return prop.properties;
  }
}
