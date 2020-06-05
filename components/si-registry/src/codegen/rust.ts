import {
  ObjectTypes,
  BaseObject,
  SystemObject,
  ComponentObject,
  EntityObject,
  EntityEventObject,
} from "../systemComponent";
import * as PropPrelude from "../components/prelude";
import { registry } from "../registry";
import { Props, IntegrationService } from "../attrList";

import { snakeCase, pascalCase } from "change-case";
import ejs from "ejs";
import path from "path";
import childProcess from "child_process";
import util from "util";
import * as codeFs from "./fs";

const execCmd = util.promisify(childProcess.exec);

interface RustTypeAsPropOptions {
  reference?: boolean;
  option?: boolean;
}

interface AgentIntegrationService {
  agentName: string;
  entity: EntityObject;
  integrationName: string;
  integrationServiceName: string;
}

interface PropertyUpdate {
  from: PropPrelude.Props;
  to: PropPrelude.Props;
}

interface PropertyEitherSet {
  entries: PropPrelude.Props[];
}

export class RustFormatter {
  systemObject: ObjectTypes;

  constructor(systemObject: RustFormatter["systemObject"]) {
    this.systemObject = systemObject;
  }

  entityActionMethodNames(): string[] {
    const results = ["create"];

    if (this.systemObject.kind() == "entityEventObject") {
      // @ts-ignore
      const entity = registry.get(`${this.systemObject.baseTypeName}Entity`);
      const fmt = new RustFormatter(entity);
      for (const prop of fmt.actionProps()) {
        if (fmt.isEntityEditMethod(prop)) {
          results.push(fmt.entityEditMethodName(prop));
        } else {
          results.push(prop.name);
        }
      }
    } else {
      for (const prop of this.actionProps()) {
        if (this.isEntityEditMethod(prop)) {
          results.push(this.entityEditMethodName(prop));
        } else {
          results.push(prop.name);
        }
      }
    }

    return results;
  }

  hasCreateMethod(): boolean {
    try {
      this.systemObject.methods.getEntry("create");
      return true;
    } catch {
      return false;
    }
  }

  hasEditEithersForAction(propAction: PropPrelude.PropAction): boolean {
    return this.entityEditProperty(propAction)
      .relationships.all()
      .some(rel => rel instanceof PropPrelude.Either);
  }

  hasEditUpdatesForAction(propAction: PropPrelude.PropAction): boolean {
    return this.entityEditProperty(propAction)
      .relationships.all()
      .some(rel => rel instanceof PropPrelude.Updates);
  }

  hasEditUpdatesAndEithers(): boolean {
    if (this.isEntityObject()) {
      return this.entityEditMethods().some(
        propAction =>
          this.hasEditUpdatesForAction(propAction) &&
          this.hasEditUpdatesForAction(propAction),
      );
    } else {
      throw new Error(
        "You ran 'hasEditUpdatesAndEithers()' on a non-entity object; this is a bug!",
      );
    }
  }

  isComponentObject(): boolean {
    return this.systemObject instanceof ComponentObject;
  }

  isEntityActionMethod(propMethod: PropPrelude.PropMethod): boolean {
    return (
      this.isEntityObject() && propMethod instanceof PropPrelude.PropAction
    );
  }

  isEntityEditMethod(propMethod: PropPrelude.PropMethod): boolean {
    return (
      this.isEntityActionMethod(propMethod) && propMethod.name.endsWith("Edit")
    );
  }

  isEntityEventObject(): boolean {
    return this.systemObject instanceof EntityEventObject;
  }

  isEntityObject(): boolean {
    return this.systemObject instanceof EntityObject;
  }

  isChangeSetObject(): boolean {
    return this.systemObject.typeName == "changeSet";
  }

  isMigrateable(): boolean {
    return (
      this.systemObject instanceof SystemObject && this.systemObject.migrateable
    );
  }

  isStorable(): boolean {
    return this.systemObject instanceof SystemObject;
  }

  actionProps(): PropPrelude.PropAction[] {
    return this.systemObject.methods.attrs.filter(
      m => m instanceof PropPrelude.PropAction,
    ) as PropPrelude.PropAction[];
  }

  componentName(): string {
    if (
      this.systemObject instanceof ComponentObject ||
      this.systemObject instanceof EntityObject ||
      this.systemObject instanceof EntityEventObject
    ) {
      return `crate::protobuf::${pascalCase(
        this.systemObject.baseTypeName,
      )}Component`;
    } else {
      throw new Error(
        "You asked for an component name on a non-component object; this is a bug!",
      );
    }
  }

  componentConstraintsName(): string {
    if (
      this.systemObject instanceof ComponentObject ||
      this.systemObject instanceof EntityObject ||
      this.systemObject instanceof EntityEventObject
    ) {
      return `crate::protobuf::${pascalCase(
        this.systemObject.baseTypeName,
      )}ComponentConstraints`;
    } else {
      throw new Error(
        "You asked for a component constraints name on a non-component object; this is a bug!",
      );
    }
  }

  componentContraintsEnums(): PropPrelude.PropEnum[] {
    if (this.systemObject instanceof ComponentObject) {
      return this.systemObject.constraints.attrs
        .filter(c => c instanceof PropPrelude.PropEnum)
        .map(c => c as PropPrelude.PropEnum);
    } else {
      throw new Error(
        "You asked for component contraints on a non-component object; this is a bug!",
      );
    }
  }

  entityEditMethodName(propMethod: PropPrelude.PropMethod): string {
    if (this.systemObject instanceof EntityObject) {
      return `edit_${this.rustFieldNameForProp(propMethod).replace(
        "_edit",
        "",
      )}`;
    } else {
      throw new Error(
        "You asked for an edit method name on a non-entity object; this is a bug!",
      );
    }
  }

  entityEditMethods(): PropPrelude.PropAction[] {
    return this.actionProps().filter(p => this.isEntityEditMethod(p));
  }

  entityEditProperty(propAction: PropPrelude.PropAction): Props {
    let property = propAction.request.properties.getEntry("property");
    if (property instanceof PropPrelude.PropLink) {
      property = property.lookupMyself();
    }
    return property;
  }

  entityEditPropertyField(propAction: PropPrelude.PropAction): string {
    return this.rustFieldNameForProp(this.entityEditProperty(propAction));
  }

  entityEditPropertyType(propAction: PropPrelude.PropAction): string {
    return this.rustTypeForProp(this.entityEditProperty(propAction), {
      option: false,
    });
  }

  entityEditPropertyUpdates(
    propAction: PropPrelude.PropAction,
  ): PropertyUpdate[] {
    return this.entityEditProperty(propAction)
      .relationships.all()
      .filter(r => r instanceof PropPrelude.Updates)
      .map(update => ({
        from: this.entityEditProperty(propAction),
        to: update.partnerProp(),
      }));
  }

  allEntityEditPropertyUpdates(): PropertyUpdate[] {
    const results = this.entityEditMethods().flatMap(method =>
      this.entityEditPropertyUpdates(method),
    );

    return Array.from(new Set(results)).sort((a, b) =>
      `${a.from.name},${a.to.name}` > `${b.from.name},${b.to.name}` ? 1 : -1,
    );
  }

  entityEditPropertyEithers(): PropertyEitherSet[] {
    const results = new Map();
    const properties = (this.systemObject.fields.getEntry(
      "properties",
    ) as PropPrelude.PropObject).properties.attrs;

    for (const property of properties) {
      const propEithers = property.relationships
        .all()
        .filter(rel => rel instanceof PropPrelude.Either);

      if (propEithers.length > 0) {
        const eithers = new Set<PropPrelude.Props>();
        eithers.add(property);
        for (const property of propEithers) {
          eithers.add(property.partnerProp());
        }

        const eithersArray = Array.from(eithers).sort((a, b) =>
          a.name > b.name ? 1 : -1,
        );
        results.set(eithersArray.map(e => e.name).join(","), {
          entries: eithersArray,
        });
      }
    }

    return Array.from(results.values()).sort();
  }

  entityEditPropertyUpdateMethodName(propertyUpdate: PropertyUpdate): string {
    return `update_${this.rustFieldNameForProp(
      propertyUpdate.to,
    )}_from_${this.rustFieldNameForProp(propertyUpdate.from)}`;
  }

  entityEventName(): string {
    if (
      this.systemObject instanceof ComponentObject ||
      this.systemObject instanceof EntityObject ||
      this.systemObject instanceof EntityEventObject
    ) {
      return `crate::protobuf::${pascalCase(
        this.systemObject.baseTypeName,
      )}EntityEvent`;
    } else {
      throw new Error(
        "You asked for an entityEvent name on a non-component object; this is a bug!",
      );
    }
  }

  entityName(): string {
    if (
      this.systemObject instanceof ComponentObject ||
      this.systemObject instanceof EntityObject ||
      this.systemObject instanceof EntityEventObject
    ) {
      return `crate::protobuf::${pascalCase(
        this.systemObject.baseTypeName,
      )}Entity`;
    } else {
      throw new Error(
        "You asked for an entity name on a non-component object; this is a bug!",
      );
    }
  }

  entityPropertiesName(): string {
    if (
      this.systemObject instanceof ComponentObject ||
      this.systemObject instanceof EntityObject ||
      this.systemObject instanceof EntityEventObject
    ) {
      return `crate::protobuf::${pascalCase(
        this.systemObject.baseTypeName,
      )}EntityProperties`;
    } else {
      throw new Error(
        "You asked for an entityProperties name on a non-component object; this is a bug!",
      );
    }
  }

  errorType(): string {
    return `crate::error::${pascalCase(this.systemObject.serviceName)}Error`;
  }

  modelName(): string {
    return `crate::model::${pascalCase(this.systemObject.typeName)}`;
  }

  modelServiceMethodName(
    propMethod: PropPrelude.PropMethod | PropPrelude.PropAction,
  ): string {
    return this.rustFieldNameForProp(propMethod);
  }

  structName(): string {
    return `crate::protobuf::${pascalCase(this.systemObject.typeName)}`;
  }

  typeName(): string {
    return snakeCase(this.systemObject.typeName);
  }

  implTryFromForPropertyUpdate(propertyUpdate: PropertyUpdate): string {
    const from = propertyUpdate.from;
    const to = propertyUpdate.to;

    // Every fallthrough/default/else needs a `throw` clause to loudly proclaim
    // that a specific conversion is not supported. This allows us to add
    // conversions as we go without rogue and unexplained errors. In short,
    // treat this like Rust code with fully satisfied match arms. Thank you,
    // love, us.
    if (from instanceof PropPrelude.PropCode) {
      switch (from.language) {
        case "yaml":
          if (to instanceof PropPrelude.PropObject) {
            return `Ok(serde_yaml::from_str(value)?)`;
          } else {
            throw new Error(
              `conversion from language '${
                from.language
              }' to type '${to.kind()}' is not supported`,
            );
          }
        default:
          throw new Error(
            `conversion from language '${from.language}' is not supported`,
          );
      }
    } else if (from instanceof PropPrelude.PropObject) {
      if (to instanceof PropPrelude.PropCode) {
        switch (to.language) {
          case "yaml":
            return `Ok(serde_yaml::to_string(value)?)`;
          default:
            throw new Error(
              `conversion from PropObject to language '${to.language}' is not supported`,
            );
        }
      } else {
        throw new Error(
          `conversion from PropObject to type '${to.kind()}' is not supported`,
        );
      }
    } else {
      throw new Error(
        `conversion from type '${from.kind()}' to type '${to.kind()}' is not supported`,
      );
    }
  }

  implUpdateRequestType(renderOptions: RustTypeAsPropOptions = {}): string {
    const list = this.systemObject.methods.getEntry(
      "update",
    ) as PropPrelude.PropMethod;
    const updateProp = list.request.properties.getEntry("update");
    return this.rustTypeForProp(updateProp, renderOptions);
  }

  implListRequestType(renderOptions: RustTypeAsPropOptions = {}): string {
    const list = this.systemObject.methods.getEntry(
      "list",
    ) as PropPrelude.PropMethod;
    return this.rustTypeForProp(list.request, renderOptions);
  }

  implListReplyType(renderOptions: RustTypeAsPropOptions = {}): string {
    const list = this.systemObject.methods.getEntry(
      "list",
    ) as PropPrelude.PropMethod;
    return this.rustTypeForProp(list.reply, renderOptions);
  }

  implServiceRequestType(
    propMethod: PropPrelude.PropMethod,
    renderOptions: RustTypeAsPropOptions = {},
  ): string {
    return this.rustTypeForProp(propMethod.request, renderOptions);
  }

  implServiceReplyType(
    propMethod: PropPrelude.PropMethod,
    renderOptions: RustTypeAsPropOptions = {},
  ): string {
    return this.rustTypeForProp(propMethod.reply, renderOptions);
  }

  implServiceTraceName(
    propMethod: PropPrelude.PropMethod | PropPrelude.PropAction,
  ): string {
    return `${this.systemObject.serviceName}.${snakeCase(
      this.rustTypeForProp(propMethod, {
        option: false,
        reference: false,
      }),
    )}`;
  }

  implServiceMethodName(
    propMethod: PropPrelude.PropMethod | PropPrelude.PropAction,
  ): string {
    return snakeCase(
      this.rustTypeForProp(propMethod, {
        option: false,
        reference: false,
      }),
    );
  }

  implProtobufEnum(propEnum: PropPrelude.PropEnum): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implProtobufEnum.rs.ejs', { fmt: fmt, propEnum: propEnum }) %>",
      { fmt: this, propEnum: propEnum },
      { filename: "." },
    );
  }

  implServiceEntityAction(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceEntityAction.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceEntityEdit(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceEntityEdit.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceCommonCreate(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceCommonCreate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceChangeSetCreate(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceChangeSetCreate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceEntityCreate(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceEntityCreate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceEntityDelete(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceEntityDelete.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceEntityUpdate(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceEntityUpdate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceGet(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceGet.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceList(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceList.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceComponentPick(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceComponentPick.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceCustomMethod(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceCustomMethod.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
      { fmt: this, propMethod: propMethod },
      { filename: "." },
    );
  }

  implServiceAuth(propMethod: PropPrelude.PropMethod): string {
    if (propMethod.skipAuth) {
      return `// Authentication is skipped on \`${this.implServiceMethodName(
        propMethod,
      )}\`\n`;
    } else {
      return this.implServiceAuthCall(propMethod);
    }
  }

  implServiceAuthCall(propMethod: PropPrelude.PropMethod): string {
    let prelude = "si_account::authorize";
    if (this.systemObject.serviceName == "account") {
      prelude = "crate::authorize";
    }
    return `${prelude}::authnz(&self.db, &request, "${this.implServiceMethodName(
      propMethod,
    )}").await?;`;
  }

  serviceMethods(): string {
    const results = [];
    const propMethods = this.systemObject.methods.attrs.sort((a, b) =>
      a.name > b.name ? 1 : -1,
    );
    for (const propMethod of propMethods) {
      const output = ejs.render(
        "<%- include('src/codegen/rust/serviceMethod.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
        {
          fmt: this,
          propMethod: propMethod,
        },
        {
          filename: ".",
        },
      );
      results.push(output);
    }
    return results.join("\n");
  }

  rustFieldNameForProp(prop: Props): string {
    return snakeCase(prop.name);
  }

  rustTypeForProp(
    prop: Props,
    renderOptions: RustTypeAsPropOptions = {},
  ): string {
    const reference = renderOptions.reference || false;
    let option = true;
    if (renderOptions.option === false) {
      option = false;
    }

    let typeName: string;

    if (
      prop instanceof PropPrelude.PropAction ||
      prop instanceof PropPrelude.PropMethod
    ) {
      typeName = `${pascalCase(prop.parentName)}${pascalCase(prop.name)}`;
    } else if (prop instanceof PropPrelude.PropNumber) {
      if (prop.numberKind == "int32") {
        typeName = "i32";
      } else if (prop.numberKind == "uint32") {
        typeName = "u32";
      } else if (prop.numberKind == "int64") {
        typeName = "i64";
      } else if (prop.numberKind == "uint64") {
        typeName = "u64";
      } else if (prop.numberKind == "u128") {
        typeName = "u128";
      }
    } else if (
      prop instanceof PropPrelude.PropBool ||
      prop instanceof PropPrelude.PropEnum ||
      prop instanceof PropPrelude.PropObject
    ) {
      typeName = `crate::protobuf::${pascalCase(prop.parentName)}${pascalCase(
        prop.name,
      )}`;
    } else if (prop instanceof PropPrelude.PropLink) {
      const realProp = prop.lookupMyself();
      if (realProp instanceof PropPrelude.PropObject) {
        const propOwner = prop.lookupObject();
        let pathName: string;
        if (
          propOwner.serviceName &&
          propOwner.serviceName == this.systemObject.serviceName
        ) {
          pathName = "crate::protobuf";
        } else if (propOwner.serviceName) {
          pathName = `si_${propOwner.serviceName}::protobuf`;
        } else {
          pathName = "crate::protobuf";
        }
        typeName = `${pathName}::${pascalCase(realProp.parentName)}${pascalCase(
          realProp.name,
        )}`;
      } else {
        return this.rustTypeForProp(realProp, renderOptions);
      }
    } else if (prop instanceof PropPrelude.PropMap) {
      typeName = `std::collections::HashMap<String, String>`;
    } else if (
      prop instanceof PropPrelude.PropText ||
      prop instanceof PropPrelude.PropCode ||
      prop instanceof PropPrelude.PropSelect
    ) {
      typeName = "String";
    } else {
      throw new Error("All Props types covered; this code is unreachable!");
    }
    if (reference) {
      // @ts-ignore - we do assign it, you just cant tell
      if (typeName == "String") {
        typeName = "&str";
      } else {
        // @ts-ignore - we do assign it, you just cant tell
        typeName = `&${typeName}`;
      }
    }
    if (prop.repeated) {
      // @ts-ignore - we do assign it, you just cant tell
      typeName = `Vec<${typeName}>`;
    } else {
      if (option) {
        // @ts-ignore - we do assign it, you just cant tell
        typeName = `Option<${typeName}>`;
      }
    }
    // @ts-ignore - we do assign it, you just cant tell
    return typeName;
  }

  rustNameForEnumVariant(variant: string): string {
    return pascalCase(variant.replace(".", ""));
  }

  implCreateNewArgs(): string {
    const result = [];
    const createMethod = this.systemObject.methods.getEntry("create");
    if (createMethod instanceof PropPrelude.PropMethod) {
      for (const prop of createMethod.request.properties.attrs) {
        result.push(`${snakeCase(prop.name)}: ${this.rustTypeForProp(prop)}`);
      }
    }
    return result.join(", ");
  }

  implCreatePassNewArgs(): string {
    const result = [];
    const createMethod = this.systemObject.methods.getEntry("create");
    if (createMethod instanceof PropPrelude.PropMethod) {
      for (const prop of createMethod.request.properties.attrs) {
        result.push(snakeCase(prop.name));
      }
    }
    return result.join(", ");
  }

  implServiceMethodListResultToReply(): string {
    const result = [];
    const listMethod = this.systemObject.methods.getEntry("list");
    if (listMethod instanceof PropPrelude.PropMethod) {
      for (const prop of listMethod.reply.properties.attrs) {
        const fieldName = snakeCase(prop.name);
        let listReplyValue = `Some(output.${fieldName})`;
        if (fieldName == "next_page_token") {
          listReplyValue = "Some(output.page_token)";
        } else if (fieldName == "items") {
          listReplyValue = `output.${fieldName}`;
        }
        result.push(`${fieldName}: ${listReplyValue}`);
      }
    }
    return result.join(", ");
  }

  implServiceMethodCreateDestructure(): string {
    const result = [];
    const createMethod = this.systemObject.methods.getEntry("create");
    if (createMethod instanceof PropPrelude.PropMethod) {
      for (const prop of createMethod.request.properties.attrs) {
        const fieldName = snakeCase(prop.name);
        result.push(`let ${fieldName} = inner.${fieldName};`);
      }
    }
    return result.join("\n");
  }

  naturalKey(): string {
    if (this.systemObject instanceof SystemObject) {
      return snakeCase(this.systemObject.naturalKey);
    } else {
      return "name";
    }
  }

  implCreateSetProperties(): string {
    const result = [];
    const createMethod = this.systemObject.methods.getEntry("create");
    if (createMethod instanceof PropPrelude.PropMethod) {
      for (const prop of createMethod.request.properties.attrs) {
        const variableName = snakeCase(prop.name);
        if (prop instanceof PropPrelude.PropPassword) {
          result.push(
            `result.${variableName} = Some(si_data::password::encrypt_password(${variableName})?);`,
          );
        } else {
          result.push(`result.${variableName} = ${variableName};`);
        }
      }
    }
    for (const prop of this.systemObject.fields.attrs) {
      const variableName = snakeCase(prop.name);
      const defaultValue = prop.defaultValue();
      if (defaultValue) {
        if (prop.kind() == "text") {
          result.push(
            `result.${variableName} = "${defaultValue}".to_string();`,
          );
        } else if (prop.kind() == "enum") {
          const enumName = `${pascalCase(
            this.systemObject.typeName,
          )}${pascalCase(prop.name)}`;
          result.push(
            `result.set_${variableName}(crate::protobuf::${enumName}::${pascalCase(
              defaultValue as string,
            )});`,
          );
        }
      }
    }
    return result.join("\n");
  }

  implCreateAddToTenancy(): string {
    const result = [];
    if (
      this.systemObject.typeName == "billingAccount" ||
      this.systemObject.typeName == "integration"
    ) {
      result.push(`si_storable.add_to_tenant_ids("global");`);
    } else if (this.systemObject.typeName == "integrationService") {
      result.push(`si_storable.add_to_tenant_ids("global");`);
      result.push(
        `si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;`,
      );
      result.push(`let integration_id = si_properties.as_ref().unwrap().integration_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.integrationId".into()),
        )?;
        si_storable.add_to_tenant_ids(integration_id);`);
    } else if (this.systemObject.kind() == "componentObject") {
      result.push(`si_storable.add_to_tenant_ids("global");`);
      result.push(
        `si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;`,
      );
      result.push(`let integration_id = si_properties.as_ref().unwrap().integration_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.integrationId".into()),
        )?;
        si_storable.add_to_tenant_ids(integration_id);`);
      result.push(`let integration_service_id = si_properties.as_ref().unwrap().integration_service_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.integrationServiceId".into()),
        )?;
        si_storable.add_to_tenant_ids(integration_service_id);`);
    } else if (
      this.systemObject.typeName == "user" ||
      this.systemObject.typeName == "group" ||
      this.systemObject.typeName == "organization" ||
      this.systemObject.typeName == "integrationInstance"
    ) {
      result.push(
        `si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;`,
      );
      result.push(`let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.billingAccountId".into()),
        )?;
        si_storable.add_to_tenant_ids(billing_account_id);`);
    } else if (this.systemObject.typeName == "workspace") {
      result.push(
        `si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;`,
      );
      result.push(`let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.billingAccountId".into()),
        )?;
        si_storable.add_to_tenant_ids(billing_account_id);`);
      result.push(`let organization_id = si_properties.as_ref().unwrap().organization_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.organizationId".into()),
        )?;
        si_storable.add_to_tenant_ids(organization_id);`);
    } else {
      result.push(
        `si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError("siProperties".into()))?;`,
      );
      result.push(`let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.billingAccountId".into()),
        )?;
        si_storable.add_to_tenant_ids(billing_account_id);`);
      result.push(`let organization_id = si_properties.as_ref().unwrap().organization_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.organizationId".into()),
        )?;
        si_storable.add_to_tenant_ids(organization_id);`);
      result.push(`let workspace_id = si_properties.as_ref().unwrap().workspace_id.as_ref().ok_or_else(||
            si_data::DataError::ValidationError("siProperties.workspaceId".into()),
        )?;
        si_storable.add_to_tenant_ids(workspace_id);`);
    }
    return result.join("\n");
  }

  storableIsMvcc(): string {
    if (this.systemObject.mvcc == true) {
      return "true";
    } else {
      return "false";
    }
  }

  storableValidateFunction(): string {
    const result = [];
    for (const prop of this.systemObject.fields.attrs) {
      if (prop.required) {
        const propName = snakeCase(prop.name);
        if (prop.repeated) {
          result.push(`if self.${propName}.len() == 0 {
             return Err(si_data::DataError::ValidationError("missing required ${propName} value".into()));
           }`);
        } else {
          result.push(`if self.${propName}.is_none() {
             return Err(si_data::DataError::ValidationError("missing required ${propName} value".into()));
           }`);
        }
      }
    }
    return result.join("\n");
  }

  storableOrderByFieldsByProp(
    topProp: PropPrelude.PropObject,
    prefix: string,
  ): string {
    const results = ['"siStorable.naturalKey"'];
    for (let prop of topProp.properties.attrs) {
      if (prop.hidden) {
        continue;
      }
      if (prop instanceof PropPrelude.PropLink) {
        prop = prop.lookupMyself();
      }
      if (prop instanceof PropPrelude.PropObject) {
        if (prefix == "") {
          results.push(this.storableOrderByFieldsByProp(prop, prop.name));
        } else {
          results.push(
            this.storableOrderByFieldsByProp(prop, `${prefix}.${prop.name}`),
          );
        }
      } else {
        if (prefix == "") {
          results.push(`"${prop.name}"`);
        } else {
          results.push(`"${prefix}.${prop.name}"`);
        }
      }
    }
    return results.join(", ");
  }

  storableOrderByFieldsFunction(): string {
    const results = this.storableOrderByFieldsByProp(
      this.systemObject.rootProp,
      "",
    );
    return `vec![${results}]\n`;
  }

  storableReferentialFieldsFunction(): string {
    const fetchProps = [];
    const referenceVec = [];
    if (this.systemObject instanceof EntityEventObject) {
    } else if (this.systemObject instanceof EntityObject) {
    } else if (this.systemObject instanceof ComponentObject) {
      let siProperties = this.systemObject.fields.getEntry("siProperties");
      if (siProperties instanceof PropPrelude.PropLink) {
        siProperties = siProperties.lookupMyself();
      }
      if (!(siProperties instanceof PropPrelude.PropObject)) {
        throw new Error("Cannot get properties of a non object in ref check");
      }
      for (const prop of siProperties.properties.attrs) {
        if (prop.reference) {
          const itemName = snakeCase(prop.name);
          if (prop.repeated) {
            fetchProps.push(`let ${itemName} = match &self.si_properties {
                           Some(cip) => cip
                           .${itemName}
                           .as_ref()
                           .map(String::as_ref)
                           .unwrap_or("No ${itemName} found for referential integrity check"),
                             None => "No ${itemName} found for referential integrity check",
                         };`);
            referenceVec.push(
              `si_data::Reference::HasMany("${itemName}", ${itemName})`,
            );
          } else {
            fetchProps.push(`let ${itemName} = match &self.si_properties {
                           Some(cip) => cip
                           .${itemName}
                           .as_ref()
                           .map(String::as_ref)
                           .unwrap_or("No ${itemName} found for referential integrity check"),
                             None => "No ${itemName} found for referential integrity check",
                         };`);
            referenceVec.push(
              `si_data::Reference::HasOne("${itemName}", ${itemName})`,
            );
          }
        }
      }
    } else if (this.systemObject instanceof SystemObject) {
    } else if (this.systemObject instanceof BaseObject) {
    }

    if (fetchProps.length && referenceVec.length) {
      const results = [];
      results.push(fetchProps.join("\n"));
      results.push(`vec![${referenceVec.join(",")}]`);
      return results.join("\n");
    } else {
      return "Vec::new()";
    }
  }
}

export class RustFormatterService {
  serviceName: string;
  systemObjects: ObjectTypes[];

  constructor(serviceName: string) {
    this.serviceName = serviceName;
    this.systemObjects = registry.getObjectsForServiceName(serviceName);
  }

  systemObjectsAsFormatters(): RustFormatter[] {
    return this.systemObjects
      .sort((a, b) => (a.typeName > b.typeName ? 1 : -1))
      .map(o => new RustFormatter(o));
  }

  implServiceStructBody(): string {
    const result = ["db: si_data::Db,"];
    if (this.hasEntities()) {
      result.push("agent: si_cea::AgentClient,");
    }
    return result.join("\n");
  }

  implServiceNewConstructorArgs(): string {
    if (this.hasEntities()) {
      return "db: si_data::Db, agent: si_cea::AgentClient";
    } else {
      return "db: si_data::Db";
    }
  }

  implServiceStructConstructorReturn(): string {
    const result = ["db"];
    if (this.hasEntities()) {
      result.push("agent");
    }
    return result.join(",");
  }

  implServiceTraitName(): string {
    return `crate::protobuf::${snakeCase(
      this.serviceName,
    )}_server::${pascalCase(this.serviceName)}`;
  }

  implServerName(): string {
    return `${this.implServiceTraitName()}Server`;
  }

  implServiceMigrate(): string {
    const result = [];
    for (const systemObj of this.systemObjects) {
      if (this.isMigrateable(systemObj)) {
        result.push(
          `crate::protobuf::${pascalCase(
            systemObj.typeName,
          )}::migrate(&self.db).await?;`,
        );
      }
    }
    return result.join("\n");
  }

  hasEntities(): boolean {
    return this.systemObjects.some(obj => obj instanceof EntityObject);
  }

  isMigrateable(prop: ObjectTypes): boolean {
    return prop instanceof SystemObject && prop.migrateable;
  }

  hasMigratables(): boolean {
    return this.systemObjects.some(obj => this.isMigrateable(obj));
  }
}

export class RustFormatterAgent {
  agentName: string;
  entity: EntityObject;
  entityFormatter: RustFormatter;
  integrationName: string;
  integrationServiceName: string;
  serviceName: string;
  systemObjects: ObjectTypes[];

  constructor(serviceName: string, agent: AgentIntegrationService) {
    this.agentName = agent.agentName;
    this.entity = agent.entity;
    this.entityFormatter = new RustFormatter(this.entity);
    this.integrationName = agent.integrationName;
    this.integrationServiceName = agent.integrationServiceName;
    this.serviceName = serviceName;
    this.systemObjects = registry.getObjectsForServiceName(serviceName);
  }

  systemObjectsAsFormatters(): RustFormatter[] {
    return this.systemObjects
      .sort((a, b) => (a.typeName > b.typeName ? 1 : -1))
      .map(o => new RustFormatter(o));
  }

  actionProps(): PropPrelude.PropAction[] {
    return this.entity.methods.attrs.filter(
      m => m instanceof PropPrelude.PropAction,
    ) as PropPrelude.PropAction[];
  }

  entityActionMethodNames(): string[] {
    const results = ["create"];

    for (const prop of this.actionProps()) {
      if (this.entityFormatter.isEntityEditMethod(prop)) {
        results.push(this.entityFormatter.entityEditMethodName(prop));
      } else {
        results.push(prop.name);
      }
    }

    return results;
  }

  dispatcherBaseTypeName(): string {
    return `${pascalCase(this.integrationName)}${pascalCase(
      this.integrationServiceName,
    )}${pascalCase(this.entity.baseTypeName)}`;
  }

  dispatcherTypeName(): string {
    return `${this.dispatcherBaseTypeName()}Dispatcher`;
  }

  dispatchFunctionTraitName(): string {
    return `${this.dispatcherBaseTypeName()}DispatchFunctions`;
  }
}

export class CodegenRust {
  serviceName: string;

  constructor(serviceName: string) {
    this.serviceName = serviceName;
  }

  hasModels(): boolean {
    return registry
      .getObjectsForServiceName(this.serviceName)
      .some(o => o.kind() != "baseObject");
  }

  hasServiceMethods(): boolean {
    return (
      registry
        .getObjectsForServiceName(this.serviceName)
        .flatMap(o => o.methods.attrs).length > 0
    );
  }

  hasEntityIntegrationServcices(): boolean {
    const integrationServices = new Set(
      this.entities().flatMap(entity =>
        this.entityintegrationServicesFor(entity),
      ),
    );
    return integrationServices.size > 0;
  }

  entities(): EntityObject[] {
    return registry
      .getObjectsForServiceName(this.serviceName)
      .filter(o => o instanceof EntityObject) as EntityObject[];
  }

  entityActions(entity: EntityObject): PropPrelude.PropAction[] {
    return entity.methods.attrs.filter(
      m => m instanceof PropPrelude.PropAction,
    ) as PropPrelude.PropAction[];
  }

  entityintegrationServicesFor(entity: EntityObject): IntegrationService[] {
    const result: Set<IntegrationService> = new Set();
    for (const integrationService of entity.integrationServices) {
      result.add(integrationService);
    }
    for (const action of this.entityActions(entity)) {
      for (const integrationService of action.integrationServices) {
        result.add(integrationService);
      }
    }
    return Array.from(result);
  }

  entityIntegrationServices(): AgentIntegrationService[] {
    return this.entities().flatMap(entity =>
      this.entityintegrationServicesFor(entity).map(integrationService => ({
        integrationName: integrationService.integrationName,
        integrationServiceName: integrationService.integrationServiceName,
        entity: entity,
        agentName: `${snakeCase(
          integrationService.integrationName,
        )}_${snakeCase(integrationService.integrationServiceName)}_${snakeCase(
          entity.baseTypeName,
        )}`,
      })),
    );
  }

  // Generate the 'gen/mod.rs'
  async generateGenMod(): Promise<void> {
    const results = ["// Auto-generated code!", "// No touchy!", ""];
    if (this.hasEntityIntegrationServcices()) {
      results.push("pub mod agent;");
    }
    if (this.hasModels()) {
      results.push("pub mod model;");
    }
    if (this.hasServiceMethods()) {
      results.push("pub mod service;");
    }
    await this.writeCode("gen/mod.rs", results.join("\n"));
  }

  // Generate the 'gen/model/mod.rs'
  async generateGenModelMod(): Promise<void> {
    const results = ["// Auto-generated code!", "// No touchy!", ""];
    for (const systemObject of registry.getObjectsForServiceName(
      this.serviceName,
    )) {
      if (systemObject.kind() != "baseObject") {
        results.push(`pub mod ${snakeCase(systemObject.typeName)};`);
      }
    }
    await this.writeCode("gen/model/mod.rs", results.join("\n"));
  }

  async generateGenService(): Promise<void> {
    const output = ejs.render(
      "<%- include('src/codegen/rust/service.rs.ejs', { fmt: fmt }) %>",
      {
        fmt: new RustFormatterService(this.serviceName),
      },
      {
        filename: ".",
      },
    );
    await this.writeCode(`gen/service.rs`, output);
  }

  async generateGenModel(systemObject: ObjectTypes): Promise<void> {
    const output = ejs.render(
      "<%- include('src/codegen/rust/model.rs.ejs', { fmt: fmt }) %>",
      {
        fmt: new RustFormatter(systemObject),
      },
      {
        filename: ".",
      },
    );
    await this.writeCode(
      `gen/model/${snakeCase(systemObject.typeName)}.rs`,
      output,
    );
  }

  // Generate the 'gen/agent/mod.rs'
  async generateGenAgentMod(): Promise<void> {
    const results = ["// Auto-generated code!", "// No touchy!", ""];
    for (const agent of this.entityIntegrationServices()) {
      results.push(`pub mod ${agent.agentName};`);
    }
    results.push("");
    for (const agent of this.entityIntegrationServices()) {
      const fmt = new RustFormatterAgent(this.serviceName, agent);
      results.push(
        `pub use ${
          agent.agentName
        }::{${fmt.dispatchFunctionTraitName()}, ${fmt.dispatcherTypeName()}};`,
      );
    }
    await this.writeCode("gen/agent/mod.rs", results.join("\n"));
  }

  async generateGenAgent(agent: AgentIntegrationService): Promise<void> {
    const output = ejs.render(
      "<%- include('src/codegen/rust/agent.rs.ejs', { fmt: fmt }) %>",
      {
        fmt: new RustFormatterAgent(this.serviceName, agent),
      },
      {
        filename: ".",
      },
    );
    await this.writeCode(`gen/agent/${snakeCase(agent.agentName)}.rs`, output);
  }

  //async makePath(pathPart: string): Promise<string> {
  //  const pathName = path.join("..", `si-${this.serviceName}`, "src", pathPart);
  //  const absolutePathName = path.resolve(pathName);
  //  await fs.promises.mkdir(path.resolve(pathName), { recursive: true });
  //  return absolutePathName;
  //}

  async formatCode(): Promise<void> {
    await execCmd(`cargo fmt -p si-${this.serviceName}`);
  }

  async writeCode(filename: string, code: string): Promise<void> {
    const fullPathName = path.join(
      "..",
      `si-${this.serviceName}`,
      "src",
      filename,
    );
    await codeFs.writeCode(fullPathName, code);
  }
}
