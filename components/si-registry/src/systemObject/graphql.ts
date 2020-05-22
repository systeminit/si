import { PropMethod, PropObject, Props } from "../attrList";
import { PropLink } from "../prop/link";
import { ObjectTypes } from "../systemComponent";
import { registry } from "../registry";

import { pascalCase, camelCase } from "change-case";
import gql from "graphql-tag";
import { DocumentNode } from "graphql";
import { Association } from "./associations";

interface QueryArgs {
  methodName: string;
  overrideName?: string;
  overrideFields?: string;
  associations?: {
    [key: string]: string[];
  };
}

interface VariablesObjectArgs {
  methodName: string;
}

interface ValidateResultArgs {
  methodName: string;
  data: Record<string, any>;
  overrideName?: string;
}

// Second argument is if you want a repeated field
export function variablesObjectForProperty(prop: Props, repeated = false): any {
  if (
    prop.kind() == "text" ||
    prop.kind() == "number" ||
    prop.kind() == "code" ||
    prop.kind() == "enum"
  ) {
    if (prop.repeated && repeated) {
      return [];
    } else {
      return "";
    }
  } else if (prop.kind() == "map") {
    if (prop.repeated && repeated) {
      return [];
    } else {
      return {};
    }
  } else if (prop.kind() == "link") {
    const propLink = prop as PropLink;
    if (prop.repeated && repeated) {
      return [];
    } else {
      // TODO: There might be a bug here, where the name of the prop itself
      // and the name of the linked prop don't match, and so we get the
      // wrong field name if the prop is an object.
      return variablesObjectForProperty(
        propLink.lookupMyself(),
        repeated,
      );
    }
  } else if (prop.kind() == "object" || prop.kind() == "method") {
    const propObject = prop as PropObject;
    const result: Record<string, unknown> = {};
    for (const field of propObject.properties.attrs) {
      const fieldVariables = variablesObjectForProperty(field, repeated);
      result[`${field.name}`] = fieldVariables;
    }
    if (prop.repeated && repeated) {
      return [];
    } else {
      return result;
    }
  }
}

export class SiGraphql {
  systemObject: ObjectTypes;

  constructor(systemObject: SiGraphql["systemObject"]) {
    this.systemObject = systemObject;
  }

  validateResult(args: ValidateResultArgs): Record<string, any> {
    const method = this.systemObject.methods.getEntry(
      args.methodName,
    ) as PropMethod;
    const reply = method.reply;
    const lookupName =
      args.overrideName ||
      `${camelCase(this.systemObject.typeName)}${pascalCase(args.methodName)}`;
    const result = args.data.data[lookupName];
    for (const field of reply.properties.attrs) {
      if (field.required && result[field.name] == undefined) {
        throw `response incomplete; missing required field ${field}`;
      }
    }
    return result;
  }

  variablesObject(args: VariablesObjectArgs): Record<string, any> {
    const method = this.systemObject.methods.getEntry(
      args.methodName,
    ) as PropMethod;
    const request = method.request;
    return variablesObjectForProperty(request, true);
  }

  graphqlTypeName(prop: Props, inputType?: boolean): string {
    let result = "";
    if (prop.kind() == "object" || prop.kind() == "enum") {
      let request = "";
      if (inputType && prop.kind() != "enum") {
        request = "Request";
      }
      result = `${pascalCase(prop.parentName)}${pascalCase(
        prop.name,
      )}${request}`;
    } else if (prop.kind() == "text" || prop.kind() == "password") {
      if (prop.name == "id") {
        result = "ID";
      } else {
        result = "String";
      }
    } else if (prop.kind() == "number") {
      result = "String";
    } else if (prop.kind() == "link") {
      const linkProp = prop as PropLink;
      const realProp = linkProp.lookupMyself();
      return this.graphqlTypeName(realProp, inputType);
    }
    if (prop.required) {
      return `${result}!`;
    } else {
      return result;
    }
  }

  associationFieldList(
    associations: QueryArgs["associations"],
    systemObject: ObjectTypes,
  ): string {
    const associationList = associations && associations[systemObject.typeName];
    if (associationList) {
      const result: string[] = [];
      result.push("associations {");
      for (const fieldName of associationList) {
        const assocObj = systemObject.associations.getByFieldName(fieldName);
        const assocSystem = registry.get(assocObj.typeName);
        const assocMethod = assocSystem.methods.getEntry(
          assocObj.methodName,
        ) as PropMethod;

        result.push(`${fieldName} {`);
        result.push(
          this.fieldList(assocMethod.reply, associations, assocSystem),
        );
        result.push(`}`);
      }
      result.push("}");
      return result.join(" ");
    } else {
      return "";
    }
  }

  fieldList(
    propObject: PropObject,
    associations: QueryArgs["associations"],
    systemObjectMemo: ObjectTypes,
  ): string {
    let systemObject;
    if (systemObjectMemo) {
      systemObject = systemObjectMemo;
    } else {
      systemObject = this.systemObject;
    }
    const result: string[] = [];
    for (const prop of propObject.properties.attrs) {
      if (prop.hidden || prop.skip) {
        continue;
      }
      result.push(`${prop.name}`);
      if (prop.kind() == "object") {
        result.push("{");
        result.push(
          this.fieldList(prop as PropObject, undefined, systemObject),
        );
        result.push(this.associationFieldList(associations, systemObject));
        result.push("}");
      }
      if (prop.kind() == "map") {
        result.push("{ key value }");
      } else if (prop.kind() == "link") {
        // @ts-ignore
        const realObj = prop.lookupMyself();
        if (realObj.kind() == "object") {
          result.push("{");
        }
        result.push(
          this.fieldList(realObj as PropObject, undefined, systemObject),
        );
        if (realObj.kind() == "object") {
          result.push(this.associationFieldList(associations, systemObject));
          result.push("}");
        }
      }
    }
    return `${result.join(" ")}`;
  }

  query(args: QueryArgs): DocumentNode {
    const method = this.systemObject.methods.getEntry(
      args.methodName,
    ) as PropMethod;
    const methodName =
      args.overrideName ||
      `${camelCase(this.systemObject.typeName)}${pascalCase(args.methodName)}`;

    const request = method.request;
    const requestVariables = [];
    const inputVariables = [];
    for (const prop of request.properties.attrs) {
      requestVariables.push(
        `$${prop.name}: ${this.graphqlTypeName(prop, true)}`,
      );
      inputVariables.push(`${prop.name}: $${prop.name}`);
    }

    const reply = method.reply;
    let fieldList: string;
    if (args.overrideFields) {
      fieldList = `${args.overrideFields}`;
    } else {
      fieldList = this.fieldList(reply, args.associations, this.systemObject);
    }

    const resultString = `query ${methodName}(${requestVariables.join(
      ", ",
    )}) { ${methodName}(input: { ${inputVariables.join(
      ", ",
    )} }) { ${fieldList} } }`;
    return gql`
      ${resultString}
    `;
  }

  mutation(args: QueryArgs): DocumentNode {
    const method = this.systemObject.methods.getEntry(
      args.methodName,
    ) as PropMethod;
    const methodName =
      args.overrideName ||
      `${camelCase(this.systemObject.typeName)}${pascalCase(args.methodName)}`;

    const request = method.request;
    const requestVariables = [];
    const inputVariables = [];
    for (const prop of request.properties.attrs) {
      requestVariables.push(
        `$${prop.name}: ${this.graphqlTypeName(prop, true)}`,
      );
      inputVariables.push(`${prop.name}: $${prop.name}`);
    }

    const reply = method.reply;
    let fieldList: string;
    if (args.overrideFields) {
      fieldList = `${args.overrideFields}`;
    } else {
      fieldList = this.fieldList(reply, args.associations, this.systemObject);
    }

    const resultString = `mutation ${methodName}(${requestVariables.join(
      ", ",
    )}) { ${methodName}(input: { ${inputVariables.join(
      ", ",
    )} }) { ${fieldList} } }`;
    console.log(resultString);
    return gql`
      ${resultString}
    `;
  }
}
