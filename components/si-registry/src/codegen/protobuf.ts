import { ObjectTypes } from "../systemComponent";
import ejs from "ejs";
import { Props, PropObject } from "../attrList";
import { PropEnum } from "../prop/enum";
import * as PropPrelude from "../components/prelude";

import { snakeCase, pascalCase, constantCase } from "change-case";

export class ProtobufFormatter {
  systemObjects: ObjectTypes[];

  recurseKinds = ["object"];

  constructor(systemObject: ObjectTypes[]) {
    if (systemObject.length == 0) {
      throw "You must provide objects to generate";
    }
    this.systemObjects = systemObject;
  }

  first(): ObjectTypes {
    return this.systemObjects[0];
  }

  protobufPackageName(): string {
    return `si.${snakeCase(this.first().serviceName)}`;
  }

  protobufServices(): string {
    const results = [];
    if (
      this.systemObjects.filter(obj => obj.methodsProp.properties.length > 0)
        .length > 0
    ) {
      results.push(`service ${pascalCase(this.first().serviceName)} {`);
      for (const object of this.systemObjects) {
        for (const method of object.methods.attrs) {
          const methodName =
            pascalCase(method.parentName) + pascalCase(method.name);
          results.push(
            `  rpc ${methodName}(${methodName}Request) returns (${methodName}Reply);`,
          );
        }
      }
      results.push(`}`);
      return results.join("\n");
    }
    return "// No Services";
  }

  protobufMessages(): string {
    const results = [];
    for (const object of this.systemObjects) {
      results.push(this.protobufMessageForPropObject(object.rootProp));
      if (object.methodsProp.properties.length) {
        for (const methodHolder of object.methodsProp.properties.attrs) {
          if (
            methodHolder instanceof PropPrelude.PropMethod ||
            methodHolder instanceof PropPrelude.PropAction
          ) {
            results.push(
              this.protobufMessageForPropObject(methodHolder.request),
            );
            results.push(this.protobufMessageForPropObject(methodHolder.reply));
          } else {
            throw `Error generating protobuf - non method/action prop found on ${object.typeName}`;
          }
        }
      }
    }
    return results.join("\n");
  }

  protobufImportForProp(prop: Props): string {
    if (prop instanceof PropPrelude.PropLink) {
      const propOwner = prop.lookupObject();
      let pathName = "si-registry/proto/si.";
      if (propOwner.serviceName) {
        pathName = pathName + snakeCase(propOwner.serviceName) + ".proto";
      } else {
        pathName = pathName + snakeCase(propOwner.typeName) + ".proto";
      }
      return pathName;
    } else {
      return "";
    }
  }

  protobufTypeForProp(prop: Props): string {
    if (prop instanceof PropPrelude.PropBool) {
      return "google.protobuf.BoolValue";
    } else if (prop instanceof PropPrelude.PropCode) {
      return "google.protobuf.StringValue";
    } else if (prop instanceof PropPrelude.PropEnum) {
      return `${pascalCase(prop.parentName)}${pascalCase(prop.name)}`;
    } else if (prop instanceof PropPrelude.PropLink) {
      const realProp = prop.lookupMyself();
      if (realProp instanceof PropPrelude.PropObject) {
        const propOwner = prop.lookupObject();
        let pathName = "si.";
        if (propOwner.serviceName) {
          pathName = pathName + snakeCase(propOwner.serviceName);
        } else {
          pathName = pathName + snakeCase(propOwner.typeName);
        }
        return `${pathName}.${pascalCase(realProp.parentName)}${pascalCase(
          realProp.name,
        )}`;
      } else {
        return this.protobufTypeForProp(realProp);
      }
    } else if (prop instanceof PropPrelude.PropMap) {
      return "map<string, string>";
    } else if (prop instanceof PropPrelude.PropNumber) {
      if (prop.numberKind == "int32") {
        return "google.protobuf.Int32Value";
      } else if (prop.numberKind == "uint32") {
        return "google.protobuf.UInt32Value";
      } else if (prop.numberKind == "int64") {
        return "google.protobuf.Int64Value";
      } else if (prop.numberKind == "uint64") {
        return "google.protobuf.UInt64Value";
      }
    } else if (prop instanceof PropPrelude.PropObject) {
      return `${this.protobufPackageName()}.${pascalCase(
        prop.parentName,
      )}${pascalCase(prop.name)}`;
    } else if (prop instanceof PropPrelude.PropMethod) {
      return `${this.protobufPackageName()}.${pascalCase(
        prop.parentName,
      )}${pascalCase(prop.name)}`;
    } else if (prop instanceof PropPrelude.PropAction) {
      return `${this.protobufPackageName()}.${pascalCase(
        prop.parentName,
      )}${pascalCase(prop.name)}`;
    } else if (
      prop instanceof PropPrelude.PropSelect ||
      prop instanceof PropPrelude.PropText
    ) {
      return "google.protobuf.StringValue";
    } else {
      // @ts-ignore
      throw `Unknown property type for rendering protobuf! Fix me: ${prop.kind()}`;
    }
  }

  protobufDefinitionForProp(prop: Props, inputNumber: number): string {
    let repeated: string;
    if (prop.repeated) {
      repeated = "repeated ";
    } else {
      repeated = "";
    }
    return `${repeated}${this.protobufTypeForProp(prop)} ${snakeCase(
      prop.name,
    )} = ${inputNumber};`;
  }

  protobufMessageForPropObject(prop: PropObject | PropEnum): string {
    const results = [];

    if (prop instanceof PropEnum) {
      let enumCount = 0;
      results.push(
        `enum ${pascalCase(prop.parentName)}${pascalCase(prop.name)} {`,
      );
      results.push(
        `  ${constantCase(
          this.protobufTypeForProp(prop),
        )}_UNKNOWN = ${enumCount};`,
      );
      for (const variant of prop.variants) {
        enumCount = enumCount + 1;
        results.push(
          `  ${constantCase(this.protobufTypeForProp(prop))}_${constantCase(
            variant,
          )} = ${enumCount};`,
        );
      }
      results.push("}");
      return results.join("\n");
    }

    for (const bag of prop.bagNames()) {
      for (const childProp of prop[bag].attrs) {
        if (childProp instanceof PropObject || childProp instanceof PropEnum) {
          results.push(this.protobufMessageForPropObject(childProp));
        }
      }

      const messageName = `${pascalCase(prop.parentName)}${pascalCase(
        prop.name,
      )}`;
      results.push(`message ${messageName} {`);

      let universalBase = 0;
      let customBase = 1000;
      for (const index in prop[bag].attrs) {
        const p = prop[bag].attrs[index];

        if (p.universal) {
          universalBase = universalBase + 1;
          results.push("  " + this.protobufDefinitionForProp(p, universalBase));
        } else {
          customBase = customBase + 1;
          results.push("  " + this.protobufDefinitionForProp(p, customBase));
        }
      }
      results.push("}");
    }
    return results.join("\n");
  }

  protobufImports(): string {
    const results = []; // This creates a newline!
    const resultSet = this.protobufImportWalk(
      this.systemObjects.map(v => v.rootProp),
    );
    for (const line of resultSet.values()) {
      results.push(`import "${line}";`);
    }
    if (results.length > 0) {
      return results.join("\n");
    } else {
      return "// No Imports";
    }
  }

  protobufImportWalk(props: Props[]): Set<string> {
    const result: Set<string> = new Set();

    for (const prop of props) {
      if (prop.kind() == "link") {
        const importPath = this.protobufImportForProp(prop);
        if (
          importPath &&
          !importPath.startsWith(
            `si-registry/proto/${this.protobufPackageName()}`,
          )
        ) {
          result.add(importPath);
        }
      } else {
        result.add("google/protobuf/wrappers.proto");
      }

      if (this.recurseKinds.includes(prop.kind())) {
        for (const bag of prop.bagNames()) {
          for (const internalProp of prop[bag].attrs) {
            const newSet = this.protobufImportWalk([internalProp]);
            for (const item of newSet.values()) {
              result.add(item);
            }
          }
        }
      }
    }
    return result;
  }

  generateString(): string {
    return ejs.render(
      "<%- include('protobuf/proto', { fmt }) %>",
      {
        fmt: this,
      },
      {
        filename: __filename,
      },
    );
  }
}

//export class CodegenProtobuf {
//  component: Component;
//
//  constructor(component: Component) {
//    this.component = component;
//  }
//
//  generateString(): string {
//    return ejs.render(
//      "<%- include('protobuf/full', { component: component }) %>",
//      {
//        component: this.component,
//      },
//      {
//        filename: __filename,
//      },
//    );
//  }
//}
