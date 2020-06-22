import { PropLink } from "./prop/link";
import { PropNumber } from "./prop/number";
import {
  PropObject,
  PropMethod,
  PropAction,
  IntegrationService,
} from "./attrList";
import { camelCase } from "change-case";
import { AssociationList } from "./systemObject/associations";
import { SiGraphql } from "./systemObject/graphql";

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

export class BaseObject {
  typeName: string;
  displayTypeName: string;
  siPathName: string;
  serviceName: string;
  mvcc: boolean;

  rootProp: PropObject;
  methodsProp: PropObject;
  associations: AssociationList;

  private internalGraphql: undefined | SiGraphql;

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
    this.internalGraphql = undefined;
    this.mvcc = false;
  }

  get fields(): BaseObject["rootProp"]["properties"] {
    return this.rootProp.properties;
  }

  get methods(): BaseObject["methodsProp"]["properties"] {
    return this.methodsProp.properties;
  }

  get graphql(): SiGraphql {
    if (this.internalGraphql == undefined) {
      this.internalGraphql = new SiGraphql(this);
    }
    return this.internalGraphql;
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

export class EntityObject extends SystemObject {
  baseTypeName: string;
  integrationServices: IntegrationService[];

  constructor(args: BaseObjectConstructor) {
    const typeName = `${args.typeName}Entity`;
    const displayTypeName = `${args.displayTypeName} Entity`;
    super({
      typeName,
      displayTypeName,
      serviceName: args.serviceName,
    });
    this.baseTypeName = args.typeName;
    this.integrationServices = [];
    this.setEntityDefaults();
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
        p.request.properties.addLink({
          name: "properties",
          label: "Properties",
          options(p: PropLink) {
            p.universal = true;
            p.readOnly = true;
            p.required = false;
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

    this.methods.addAction({
      name: "sync",
      label: "Sync State",
      options(p: PropAction) {
        p.mutation = true;
        p.universal = true;
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
