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
import { Props } from "../attrList";

import { snakeCase, pascalCase } from "change-case";
import ejs from "ejs";
import fs from "fs";
import path from "path";
import childProcess from "child_process";
import util from "util";

const execCmd = util.promisify(childProcess.exec);

interface RustTypeAsPropOptions {
  reference?: boolean;
  option?: boolean;
}

export class RustFormatter {
  systemObject: ObjectTypes;

  constructor(systemObject: RustFormatter["systemObject"]) {
    this.systemObject = systemObject;
  }

  structName(): string {
    return `crate::protobuf::${pascalCase(this.systemObject.typeName)}`;
  }

  modelName(): string {
    return `crate::model::${pascalCase(this.systemObject.typeName)}`;
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
      throw "You asked for an component name on a non-component object; this is a bug!";
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
      throw "You asked for a component constraints name on a non-component object; this is a bug!";
    }
  }

  entityEditMethodName(propMethod: PropPrelude.PropMethod): string {
    if (this.systemObject instanceof EntityObject) {
      return `edit_${this.rustFieldNameForProp(propMethod).replace(
        "_edit",
        "",
      )}`;
    } else {
      throw "You asked for an edit method name on a non-entity object; this is a bug!";
    }
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
      throw "You asked for an entityEvent name on a non-component object; this is a bug!";
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
      throw "You asked for an entity name on a non-component object; this is a bug!";
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
      throw "You asked for an entityProperties name on a non-component object; this is a bug!";
    }
  }

  modelServiceMethodName(
    propMethod: PropPrelude.PropMethod | PropPrelude.PropAction,
  ): string {
    return this.rustFieldNameForProp(propMethod);
  }

  typeName(): string {
    return snakeCase(this.systemObject.typeName);
  }

  errorType(): string {
    return `crate::error::${pascalCase(this.systemObject.serviceName)}Error`;
  }

  hasCreateMethod(): boolean {
    try {
      this.systemObject.methods.getEntry("create");
      return true;
    } catch {
      return false;
    }
  }

  isComponentObject(): boolean {
    return this.systemObject.kind() == "componentObject";
  }

  isEntityObject(): boolean {
    return this.systemObject.kind() == "entityObject";
  }

  isEntityEventObject(): boolean {
    return this.systemObject.kind() == "entityEventObject";
  }

  isEntityActionMethod(propMethod: PropPrelude.PropMethod): boolean {
    return propMethod.kind() == "action" && this.isEntityObject();
  }

  isEntityEditMethod(propMethod: PropPrelude.PropMethod): boolean {
    return (
      this.isEntityActionMethod(propMethod) && propMethod.name.endsWith("Edit")
    );
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

  implServiceEntityCreate(propMethod: PropPrelude.PropMethod): string {
    return ejs.render(
      "<%- include('src/codegen/rust/implServiceEntityCreate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>",
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
      }
    } else if (
      prop instanceof PropPrelude.PropBool ||
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
      throw `Cannot generate type for ${prop.name} kind ${prop.kind()} - Bug!`;
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

  isMigrateable(): boolean {
    return (
      // @ts-ignore
      this.systemObject.kind() != "baseObject" && this.systemObject.migrateable
    );
  }

  isStorable(): boolean {
    if (this.systemObject instanceof SystemObject) {
      return true;
    } else {
      return false;
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
        throw "Cannot get properties of a non object in ref check";
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

  implServiceMigrate(): string {
    const result = [];
    for (const systemObj of this.systemObjects) {
      // @ts-ignore
      if (systemObj.kind() != "baseObject" && systemObj.migrateable == true) {
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
    if (this.systemObjects.find(s => s.kind() == "entityObject")) {
      return true;
    } else {
      return false;
    }
  }

  hasMigratables(): boolean {
    if (
      this.systemObjects.find(
        // @ts-ignore
        s => s.kind() != "baseObject" && s.migrateable == true,
      )
    ) {
      return true;
    } else {
      return false;
    }
  }
}

export class CodegenRust {
  serviceName: string;

  constructor(serviceName: string) {
    this.serviceName = serviceName;
  }

  // Generate the 'gen/mod.rs'
  async generateGenMod(): Promise<void> {
    const results = [
      "// Auto-generated code!",
      "// No touchy!",
      "",
      "pub mod model;",
      "pub mod service;",
    ];
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

  async makePath(pathPart: string): Promise<string> {
    const pathName = path.join("..", `si-${this.serviceName}`, "src", pathPart);
    const absolutePathName = path.resolve(pathName);
    await fs.promises.mkdir(path.resolve(pathName), { recursive: true });
    return absolutePathName;
  }

  async formatCode(): Promise<void> {
    await execCmd(`cargo fmt -p si-${this.serviceName}`);
  }

  async writeCode(filename: string, code: string): Promise<void> {
    const pathname = path.dirname(filename);
    const basename = path.basename(filename);
    const createdPath = await this.makePath(pathname);
    const codeFilename = path.join(createdPath, basename);
    await fs.promises.writeFile(codeFilename, code);
  }
}

// export class CodegenRust {
//   systemObject: ObjectTypes;
//   formatter: RustFormatter;
//
//   constructor(systemObject: ObjectTypes) {
//     this.systemObject = systemObject;
//     this.formatter = new RustFormatter(systemObject);
//   }
//
//   async writeCode(part: string, code: string): Promise<void> {
//     const createdPath = await this.makePath();
//     const codeFilename = path.join(createdPath, `${snakeCase(part)}.rs`);
//     await fs.promises.writeFile(codeFilename, code);
//     await execCmd(`rustfmt ${codeFilename}`);
//   }
//
//   async makePath(): Promise<string> {
//     const pathName = path.join(
//       __dirname,
//       "..",
//       "..",
//       "..",
//       this.systemObject.siPathName,
//       "src",
//       "gen",
//       snakeCase(this.systemObject.typeName),
//     );
//     const absolutePathName = path.resolve(pathName);
//     await fs.promises.mkdir(path.resolve(pathName), { recursive: true });
//     return absolutePathName;
//   }
//
//   async generateComponentImpls(): Promise<void> {
//     const output = ejs.render(
//       "<%- include('rust/component.rs.ejs', { component: component }) %>",
//       {
//         systemObject: this.systemObject,
//         fmt: this.formatter,
//       },
//       {
//         filename: __filename,
//       },
//     );
//     await this.writeCode("component", output);
//   }
//
//   async generateComponentMod(): Promise<void> {
//     const mods = ["component"];
//     const lines = ["// Auto-generated code!", "// No Touchy!\n"];
//     for (const mod of mods) {
//       lines.push(`pub mod ${mod};`);
//     }
//     await this.writeCode("mod", lines.join("\n"));
//   }
// }
//
// export class RustFormatter {
//   systemObject: ObjectTypes;
//
//   constructor(systemObject: RustFormatter["systemObject"]) {
//     this.systemObject = systemObject;
//   }
//
//   componentTypeName(): string {
//     return snakeCase(this.systemObject.typeName);
//   }
//
//   componentOrderByFields(): string {
//     const orderByFields = [];
//     const componentObject = this.component.asComponent();
//     for (const p of componentObject.properties.attrs) {
//       if (p.hidden) {
//         continue;
//       }
//       if (p.name == "storable") {
//         orderByFields.push('"storable.naturalKey"');
//         orderByFields.push('"storable.typeName"');
//       } else if (p.name == "siProperties") {
//         continue;
//       } else if (p.name == "constraints" && p.kind() == "object") {
//         // @ts-ignore trust us - we checked
//         for (const pc of p.properties.attrs) {
//           if (pc.kind() != "object") {
//             orderByFields.push(`"constraints.${pc.name}"`);
//           }
//         }
//       } else {
//         orderByFields.push(`"${p.name}"`);
//       }
//     }
//     return `vec![${orderByFields.join(",")}]\n`;
//   }
//
//   componentImports(): string {
//     const result = [];
//     result.push(
//       `pub use crate::protobuf::${snakeCase(this.component.typeName)}::{`,
//       `  Constraints,`,
//       `  ListComponentsReply,`,
//       `  ListComponentsRequest,`,
//       `  PickComponentRequest,`,
//       `  Component,`,
//       `};`,
//     );
//     return result.join("\n");
//   }
//
//   componentValidation(): string {
//     return this.genValidation(this.component.asComponent());
//   }
//
//   genValidation(propObject: PropObject): string {
//     const result = [];
//     for (const prop of propObject.properties.attrs) {
//       if (prop.required) {
//         const propName = snakeCase(prop.name);
//         result.push(`if self.${propName}.is_none() {
//           return Err(DataError::ValidationError("missing required ${propName} value".into()));
//         }`);
//       }
//     }
//     return result.join("\n");
//   }
// }
//
// export async function generateGenMod(writtenComponents: {
//   [key: string]: string[];
// }): Promise<void> {
//   for (const component in writtenComponents) {
//     const pathName = path.join(
//       __dirname,
//       "..",
//       "..",
//       "..",
//       component,
//       "src",
//       "gen",
//     );
//     const absolutePathName = path.resolve(pathName);
//     const code = [
//       "// Auto-generated code!",
//       "// No touchy!",
//       "",
//       "pub mod model;",
//     ];
//
//     await fs.promises.writeFile(
//       path.join(absolutePathName, "mod.rs"),
//       code.join("\n"),
//     );
//   }
// }
//
// export async function generateGenModModel(serviceName: string): Promise<void> {
//   const pathName = path.join(
//     __dirname,
//     "..",
//     "..",
//     "..",
//     serviceName,
//     "src",
//     "gen",
//     "model",
//   );
//   const absolutePathName = path.resolve(pathName);
//   const code = ["// Auto-generated code!", "// No touchy!\n"];
//   for (const typeName of writtenComponents[component]) {
//     code.push(`pub mod ${snakeCase(typeName)};`);
//   }
//
//   await fs.promises.writeFile(
//     path.join(absolutePathName, "mod.rs"),
//     code.join("\n"),
//   );
// }
