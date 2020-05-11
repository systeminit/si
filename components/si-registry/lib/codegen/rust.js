"use strict";

var _interopRequireWildcard = require("@babel/runtime/helpers/interopRequireWildcard");

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.CodegenRust = exports.RustFormatterService = exports.RustFormatter = void 0;

var _regenerator = _interopRequireDefault(require("@babel/runtime/regenerator"));

var _asyncToGenerator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncToGenerator"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _systemComponent = require("../systemComponent");

var PropPrelude = _interopRequireWildcard(require("../components/prelude"));

var _registry = require("../registry");

var _changeCase = require("change-case");

var _ejs = _interopRequireDefault(require("ejs"));

var _fs = _interopRequireDefault(require("fs"));

var _path = _interopRequireDefault(require("path"));

var _child_process = _interopRequireDefault(require("child_process"));

var _util = _interopRequireDefault(require("util"));

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(n); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

var execCmd = _util["default"].promisify(_child_process["default"].exec);

var RustFormatter = /*#__PURE__*/function () {
  function RustFormatter(systemObject) {
    (0, _classCallCheck2["default"])(this, RustFormatter);
    (0, _defineProperty2["default"])(this, "systemObject", void 0);
    this.systemObject = systemObject;
  }

  (0, _createClass2["default"])(RustFormatter, [{
    key: "structName",
    value: function structName() {
      return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.typeName));
    }
  }, {
    key: "modelName",
    value: function modelName() {
      return "crate::model::".concat((0, _changeCase.pascalCase)(this.systemObject.typeName));
    }
  }, {
    key: "componentName",
    value: function componentName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "Component");
      } else {
        throw "You asked for an component name on a non-component object; this is a bug!";
      }
    }
  }, {
    key: "componentConstraintsName",
    value: function componentConstraintsName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "ComponentConstraints");
      } else {
        throw "You asked for a component constraints name on a non-component object; this is a bug!";
      }
    }
  }, {
    key: "entityEditMethodName",
    value: function entityEditMethodName(propMethod) {
      if (this.systemObject instanceof _systemComponent.EntityObject) {
        return "edit_".concat(this.rustFieldNameForProp(propMethod).replace("_edit", ""));
      } else {
        throw "You asked for an edit method name on a non-entity object; this is a bug!";
      }
    }
  }, {
    key: "entityEventName",
    value: function entityEventName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "EntityEvent");
      } else {
        throw "You asked for an entityEvent name on a non-component object; this is a bug!";
      }
    }
  }, {
    key: "entityName",
    value: function entityName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "Entity");
      } else {
        throw "You asked for an entity name on a non-component object; this is a bug!";
      }
    }
  }, {
    key: "entityPropertiesName",
    value: function entityPropertiesName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "EntityProperties");
      } else {
        throw "You asked for an entityProperties name on a non-component object; this is a bug!";
      }
    }
  }, {
    key: "modelServiceMethodName",
    value: function modelServiceMethodName(propMethod) {
      return this.rustFieldNameForProp(propMethod);
    }
  }, {
    key: "typeName",
    value: function typeName() {
      return (0, _changeCase.snakeCase)(this.systemObject.typeName);
    }
  }, {
    key: "errorType",
    value: function errorType() {
      return "crate::error::".concat((0, _changeCase.pascalCase)(this.systemObject.serviceName), "Error");
    }
  }, {
    key: "hasCreateMethod",
    value: function hasCreateMethod() {
      try {
        this.systemObject.methods.getEntry("create");
        return true;
      } catch (_unused) {
        return false;
      }
    }
  }, {
    key: "isComponentObject",
    value: function isComponentObject() {
      return this.systemObject.kind() == "componentObject";
    }
  }, {
    key: "isEntityObject",
    value: function isEntityObject() {
      return this.systemObject.kind() == "entityObject";
    }
  }, {
    key: "isEntityEventObject",
    value: function isEntityEventObject() {
      return this.systemObject.kind() == "entityEventObject";
    }
  }, {
    key: "isEntityActionMethod",
    value: function isEntityActionMethod(propMethod) {
      return propMethod.kind() == "action" && this.isEntityObject();
    }
  }, {
    key: "isEntityEditMethod",
    value: function isEntityEditMethod(propMethod) {
      return this.isEntityActionMethod(propMethod) && propMethod.name.endsWith("Edit");
    }
  }, {
    key: "implListRequestType",
    value: function implListRequestType() {
      var renderOptions = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : {};
      var list = this.systemObject.methods.getEntry("list");
      return this.rustTypeForProp(list.request, renderOptions);
    }
  }, {
    key: "implListReplyType",
    value: function implListReplyType() {
      var renderOptions = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : {};
      var list = this.systemObject.methods.getEntry("list");
      return this.rustTypeForProp(list.reply, renderOptions);
    }
  }, {
    key: "implServiceRequestType",
    value: function implServiceRequestType(propMethod) {
      var renderOptions = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
      return this.rustTypeForProp(propMethod.request, renderOptions);
    }
  }, {
    key: "implServiceReplyType",
    value: function implServiceReplyType(propMethod) {
      var renderOptions = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
      return this.rustTypeForProp(propMethod.reply, renderOptions);
    }
  }, {
    key: "implServiceMethodName",
    value: function implServiceMethodName(propMethod) {
      return (0, _changeCase.snakeCase)(this.rustTypeForProp(propMethod, {
        option: false,
        reference: false
      }));
    }
  }, {
    key: "implServiceEntityAction",
    value: function implServiceEntityAction(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceEntityAction.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceEntityEdit",
    value: function implServiceEntityEdit(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceEntityEdit.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceCommonCreate",
    value: function implServiceCommonCreate(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceCommonCreate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceEntityCreate",
    value: function implServiceEntityCreate(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceEntityCreate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceGet",
    value: function implServiceGet(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceGet.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceList",
    value: function implServiceList(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceList.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceComponentPick",
    value: function implServiceComponentPick(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceComponentPick.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceCustomMethod",
    value: function implServiceCustomMethod(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceCustomMethod.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceAuth",
    value: function implServiceAuth(propMethod) {
      if (propMethod.skipAuth) {
        return "// Authentication is skipped on `".concat(this.implServiceMethodName(propMethod), "`\n");
      } else {
        return this.implServiceAuthCall(propMethod);
      }
    }
  }, {
    key: "implServiceAuthCall",
    value: function implServiceAuthCall(propMethod) {
      var prelude = "si_account::authorize";

      if (this.systemObject.serviceName == "account") {
        prelude = "crate::authorize";
      }

      return "".concat(prelude, "::authnz(&self.db, &request, \"").concat(this.implServiceMethodName(propMethod), "\").await?;");
    }
  }, {
    key: "serviceMethods",
    value: function serviceMethods() {
      var results = [];
      var propMethods = this.systemObject.methods.attrs.sort(function (a, b) {
        return a.name > b.name ? 1 : -1;
      });

      var _iterator = _createForOfIteratorHelper(propMethods),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var propMethod = _step.value;

          var output = _ejs["default"].render("<%- include('src/codegen/rust/serviceMethod.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
            fmt: this,
            propMethod: propMethod
          }, {
            filename: "."
          });

          results.push(output);
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      return results.join("\n");
    }
  }, {
    key: "rustFieldNameForProp",
    value: function rustFieldNameForProp(prop) {
      return (0, _changeCase.snakeCase)(prop.name);
    }
  }, {
    key: "rustTypeForProp",
    value: function rustTypeForProp(prop) {
      var renderOptions = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
      var reference = renderOptions.reference || false;
      var option = true;

      if (renderOptions.option === false) {
        option = false;
      }

      var typeName;

      if (prop instanceof PropPrelude.PropAction || prop instanceof PropPrelude.PropMethod) {
        typeName = "".concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name));
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
      } else if (prop instanceof PropPrelude.PropBool || prop instanceof PropPrelude.PropObject) {
        typeName = "crate::protobuf::".concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name));
      } else if (prop instanceof PropPrelude.PropLink) {
        var realProp = prop.lookupMyself();

        if (realProp instanceof PropPrelude.PropObject) {
          var propOwner = prop.lookupObject();
          var pathName;

          if (propOwner.serviceName && propOwner.serviceName == this.systemObject.serviceName) {
            pathName = "crate::protobuf";
          } else if (propOwner.serviceName) {
            pathName = "si_".concat(propOwner.serviceName, "::protobuf");
          } else {
            pathName = "crate::protobuf";
          }

          typeName = "".concat(pathName, "::").concat((0, _changeCase.pascalCase)(realProp.parentName)).concat((0, _changeCase.pascalCase)(realProp.name));
        } else {
          return this.rustTypeForProp(realProp, renderOptions);
        }
      } else if (prop instanceof PropPrelude.PropMap) {
        typeName = "std::collections::HashMap<String, String>";
      } else if (prop instanceof PropPrelude.PropText || prop instanceof PropPrelude.PropCode || prop instanceof PropPrelude.PropSelect) {
        typeName = "String";
      } else {
        throw "Cannot generate type for ".concat(prop.name, " kind ").concat(prop.kind(), " - Bug!");
      }

      if (reference) {
        // @ts-ignore - we do assign it, you just cant tell
        if (typeName == "String") {
          typeName = "&str";
        } else {
          // @ts-ignore - we do assign it, you just cant tell
          typeName = "&".concat(typeName);
        }
      }

      if (prop.repeated) {
        // @ts-ignore - we do assign it, you just cant tell
        typeName = "Vec<".concat(typeName, ">");
      } else {
        if (option) {
          // @ts-ignore - we do assign it, you just cant tell
          typeName = "Option<".concat(typeName, ">");
        }
      } // @ts-ignore - we do assign it, you just cant tell


      return typeName;
    }
  }, {
    key: "implCreateNewArgs",
    value: function implCreateNewArgs() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator2 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step2;

        try {
          for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
            var prop = _step2.value;
            result.push("".concat((0, _changeCase.snakeCase)(prop.name), ": ").concat(this.rustTypeForProp(prop)));
          }
        } catch (err) {
          _iterator2.e(err);
        } finally {
          _iterator2.f();
        }
      }

      return result.join(", ");
    }
  }, {
    key: "implCreatePassNewArgs",
    value: function implCreatePassNewArgs() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator3 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step3;

        try {
          for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
            var prop = _step3.value;
            result.push((0, _changeCase.snakeCase)(prop.name));
          }
        } catch (err) {
          _iterator3.e(err);
        } finally {
          _iterator3.f();
        }
      }

      return result.join(", ");
    }
  }, {
    key: "implServiceMethodListResultToReply",
    value: function implServiceMethodListResultToReply() {
      var result = [];
      var listMethod = this.systemObject.methods.getEntry("list");

      if (listMethod instanceof PropPrelude.PropMethod) {
        var _iterator4 = _createForOfIteratorHelper(listMethod.reply.properties.attrs),
            _step4;

        try {
          for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
            var prop = _step4.value;
            var fieldName = (0, _changeCase.snakeCase)(prop.name);
            var listReplyValue = "Some(output.".concat(fieldName, ")");

            if (fieldName == "next_page_token") {
              listReplyValue = "Some(output.page_token)";
            } else if (fieldName == "items") {
              listReplyValue = "output.".concat(fieldName);
            }

            result.push("".concat(fieldName, ": ").concat(listReplyValue));
          }
        } catch (err) {
          _iterator4.e(err);
        } finally {
          _iterator4.f();
        }
      }

      return result.join(", ");
    }
  }, {
    key: "implServiceMethodCreateDestructure",
    value: function implServiceMethodCreateDestructure() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator5 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step5;

        try {
          for (_iterator5.s(); !(_step5 = _iterator5.n()).done;) {
            var prop = _step5.value;
            var fieldName = (0, _changeCase.snakeCase)(prop.name);
            result.push("let ".concat(fieldName, " = inner.").concat(fieldName, ";"));
          }
        } catch (err) {
          _iterator5.e(err);
        } finally {
          _iterator5.f();
        }
      }

      return result.join("\n");
    }
  }, {
    key: "naturalKey",
    value: function naturalKey() {
      if (this.systemObject instanceof _systemComponent.SystemObject) {
        return (0, _changeCase.snakeCase)(this.systemObject.naturalKey);
      } else {
        return "name";
      }
    }
  }, {
    key: "isMigrateable",
    value: function isMigrateable() {
      return (// @ts-ignore
        this.systemObject.kind() != "baseObject" && this.systemObject.migrateable
      );
    }
  }, {
    key: "isStorable",
    value: function isStorable() {
      if (this.systemObject instanceof _systemComponent.SystemObject) {
        return true;
      } else {
        return false;
      }
    }
  }, {
    key: "implCreateSetProperties",
    value: function implCreateSetProperties() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator6 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step6;

        try {
          for (_iterator6.s(); !(_step6 = _iterator6.n()).done;) {
            var prop = _step6.value;
            var variableName = (0, _changeCase.snakeCase)(prop.name);

            if (prop instanceof PropPrelude.PropPassword) {
              result.push("result.".concat(variableName, " = Some(si_data::password::encrypt_password(").concat(variableName, ")?);"));
            } else {
              result.push("result.".concat(variableName, " = ").concat(variableName, ";"));
            }
          }
        } catch (err) {
          _iterator6.e(err);
        } finally {
          _iterator6.f();
        }
      }

      return result.join("\n");
    }
  }, {
    key: "implCreateAddToTenancy",
    value: function implCreateAddToTenancy() {
      var result = [];

      if (this.systemObject.typeName == "billingAccount" || this.systemObject.typeName == "integration") {
        result.push("si_storable.add_to_tenant_ids(\"global\");");
      } else if (this.systemObject.typeName == "integrationService") {
        result.push("si_storable.add_to_tenant_ids(\"global\");");
        result.push("si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let integration_id = si_properties.as_ref().unwrap().integration_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.integrationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(integration_id);");
      } else if (this.systemObject.kind() == "componentObject") {
        result.push("si_storable.add_to_tenant_ids(\"global\");");
        result.push("si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let integration_id = si_properties.as_ref().unwrap().integration_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.integrationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(integration_id);");
        result.push("let integration_service_id = si_properties.as_ref().unwrap().integration_service_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.integrationServiceId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(integration_service_id);");
      } else if (this.systemObject.typeName == "user" || this.systemObject.typeName == "group" || this.systemObject.typeName == "organization" || this.systemObject.typeName == "integrationInstance") {
        result.push("si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.billingAccountId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(billing_account_id);");
      } else if (this.systemObject.typeName == "workspace") {
        result.push("si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.billingAccountId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(billing_account_id);");
        result.push("let organization_id = si_properties.as_ref().unwrap().organization_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.organizationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(organization_id);");
      } else {
        result.push("si_properties.as_ref().ok_or_else(|| si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.billingAccountId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(billing_account_id);");
        result.push("let organization_id = si_properties.as_ref().unwrap().organization_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.organizationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(organization_id);");
        result.push("let workspace_id = si_properties.as_ref().unwrap().workspace_id.as_ref().ok_or_else(||\n            si_data::DataError::ValidationError(\"siProperties.workspaceId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(workspace_id);");
      }

      return result.join("\n");
    }
  }, {
    key: "storableIsMvcc",
    value: function storableIsMvcc() {
      if (this.systemObject.mvcc == true) {
        return "true";
      } else {
        return "false";
      }
    }
  }, {
    key: "storableValidateFunction",
    value: function storableValidateFunction() {
      var result = [];

      var _iterator7 = _createForOfIteratorHelper(this.systemObject.fields.attrs),
          _step7;

      try {
        for (_iterator7.s(); !(_step7 = _iterator7.n()).done;) {
          var prop = _step7.value;

          if (prop.required) {
            var propName = (0, _changeCase.snakeCase)(prop.name);

            if (prop.repeated) {
              result.push("if self.".concat(propName, ".len() == 0 {\n             return Err(si_data::DataError::ValidationError(\"missing required ").concat(propName, " value\".into()));\n           }"));
            } else {
              result.push("if self.".concat(propName, ".is_none() {\n             return Err(si_data::DataError::ValidationError(\"missing required ").concat(propName, " value\".into()));\n           }"));
            }
          }
        }
      } catch (err) {
        _iterator7.e(err);
      } finally {
        _iterator7.f();
      }

      return result.join("\n");
    }
  }, {
    key: "storableOrderByFieldsByProp",
    value: function storableOrderByFieldsByProp(topProp, prefix) {
      var results = ['"siStorable.naturalKey"'];

      var _iterator8 = _createForOfIteratorHelper(topProp.properties.attrs),
          _step8;

      try {
        for (_iterator8.s(); !(_step8 = _iterator8.n()).done;) {
          var prop = _step8.value;

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
              results.push(this.storableOrderByFieldsByProp(prop, "".concat(prefix, ".").concat(prop.name)));
            }
          } else {
            if (prefix == "") {
              results.push("\"".concat(prop.name, "\""));
            } else {
              results.push("\"".concat(prefix, ".").concat(prop.name, "\""));
            }
          }
        }
      } catch (err) {
        _iterator8.e(err);
      } finally {
        _iterator8.f();
      }

      return results.join(", ");
    }
  }, {
    key: "storableOrderByFieldsFunction",
    value: function storableOrderByFieldsFunction() {
      var results = this.storableOrderByFieldsByProp(this.systemObject.rootProp, "");
      return "vec![".concat(results, "]\n");
    }
  }, {
    key: "storableReferentialFieldsFunction",
    value: function storableReferentialFieldsFunction() {
      var fetchProps = [];
      var referenceVec = [];

      if (this.systemObject instanceof _systemComponent.EntityEventObject) {} else if (this.systemObject instanceof _systemComponent.EntityObject) {} else if (this.systemObject instanceof _systemComponent.ComponentObject) {
        var siProperties = this.systemObject.fields.getEntry("siProperties");

        if (siProperties instanceof PropPrelude.PropLink) {
          siProperties = siProperties.lookupMyself();
        }

        if (!(siProperties instanceof PropPrelude.PropObject)) {
          throw "Cannot get properties of a non object in ref check";
        }

        var _iterator9 = _createForOfIteratorHelper(siProperties.properties.attrs),
            _step9;

        try {
          for (_iterator9.s(); !(_step9 = _iterator9.n()).done;) {
            var prop = _step9.value;

            if (prop.reference) {
              var itemName = (0, _changeCase.snakeCase)(prop.name);

              if (prop.repeated) {
                fetchProps.push("let ".concat(itemName, " = match &self.si_properties {\n                           Some(cip) => cip\n                           .").concat(itemName, "\n                           .as_ref()\n                           .map(String::as_ref)\n                           .unwrap_or(\"No ").concat(itemName, " found for referential integrity check\"),\n                             None => \"No ").concat(itemName, " found for referential integrity check\",\n                         };"));
                referenceVec.push("si_data::Reference::HasMany(\"".concat(itemName, "\", ").concat(itemName, ")"));
              } else {
                fetchProps.push("let ".concat(itemName, " = match &self.si_properties {\n                           Some(cip) => cip\n                           .").concat(itemName, "\n                           .as_ref()\n                           .map(String::as_ref)\n                           .unwrap_or(\"No ").concat(itemName, " found for referential integrity check\"),\n                             None => \"No ").concat(itemName, " found for referential integrity check\",\n                         };"));
                referenceVec.push("si_data::Reference::HasOne(\"".concat(itemName, "\", ").concat(itemName, ")"));
              }
            }
          }
        } catch (err) {
          _iterator9.e(err);
        } finally {
          _iterator9.f();
        }
      } else if (this.systemObject instanceof _systemComponent.SystemObject) {} else if (this.systemObject instanceof _systemComponent.BaseObject) {}

      if (fetchProps.length && referenceVec.length) {
        var results = [];
        results.push(fetchProps.join("\n"));
        results.push("vec![".concat(referenceVec.join(","), "]"));
        return results.join("\n");
      } else {
        return "Vec::new()";
      }
    }
  }]);
  return RustFormatter;
}();

exports.RustFormatter = RustFormatter;

var RustFormatterService = /*#__PURE__*/function () {
  function RustFormatterService(serviceName) {
    (0, _classCallCheck2["default"])(this, RustFormatterService);
    (0, _defineProperty2["default"])(this, "serviceName", void 0);
    (0, _defineProperty2["default"])(this, "systemObjects", void 0);
    this.serviceName = serviceName;
    this.systemObjects = _registry.registry.getObjectsForServiceName(serviceName);
  }

  (0, _createClass2["default"])(RustFormatterService, [{
    key: "systemObjectsAsFormatters",
    value: function systemObjectsAsFormatters() {
      return this.systemObjects.sort(function (a, b) {
        return a.typeName > b.typeName ? 1 : -1;
      }).map(function (o) {
        return new RustFormatter(o);
      });
    }
  }, {
    key: "implServiceStructBody",
    value: function implServiceStructBody() {
      var result = ["db: si_data::Db,"];

      if (this.hasEntities()) {
        result.push("agent: si_cea::AgentClient,");
      }

      return result.join("\n");
    }
  }, {
    key: "implServiceNewConstructorArgs",
    value: function implServiceNewConstructorArgs() {
      if (this.hasEntities()) {
        return "db: si_data::Db, agent: si_cea::AgentClient";
      } else {
        return "db: si_data::Db";
      }
    }
  }, {
    key: "implServiceStructConstructorReturn",
    value: function implServiceStructConstructorReturn() {
      var result = ["db"];

      if (this.hasEntities()) {
        result.push("agent");
      }

      return result.join(",");
    }
  }, {
    key: "implServiceTraitName",
    value: function implServiceTraitName() {
      return "crate::protobuf::".concat((0, _changeCase.snakeCase)(this.serviceName), "_server::").concat((0, _changeCase.pascalCase)(this.serviceName));
    }
  }, {
    key: "implServerName",
    value: function implServerName() {
      return "".concat(this.implServiceTraitName(), "Server");
    }
  }, {
    key: "implServiceMigrate",
    value: function implServiceMigrate() {
      var result = [];

      var _iterator10 = _createForOfIteratorHelper(this.systemObjects),
          _step10;

      try {
        for (_iterator10.s(); !(_step10 = _iterator10.n()).done;) {
          var systemObj = _step10.value;

          // @ts-ignore
          if (systemObj.kind() != "baseObject" && systemObj.migrateable == true) {
            result.push("crate::protobuf::".concat((0, _changeCase.pascalCase)(systemObj.typeName), "::migrate(&self.db).await?;"));
          }
        }
      } catch (err) {
        _iterator10.e(err);
      } finally {
        _iterator10.f();
      }

      return result.join("\n");
    }
  }, {
    key: "hasEntities",
    value: function hasEntities() {
      if (this.systemObjects.find(function (s) {
        return s.kind() == "entityObject";
      })) {
        return true;
      } else {
        return false;
      }
    }
  }, {
    key: "hasMigratables",
    value: function hasMigratables() {
      if (this.systemObjects.find( // @ts-ignore
      function (s) {
        return s.kind() != "baseObject" && s.migrateable == true;
      })) {
        return true;
      } else {
        return false;
      }
    }
  }]);
  return RustFormatterService;
}();

exports.RustFormatterService = RustFormatterService;

var CodegenRust = /*#__PURE__*/function () {
  function CodegenRust(serviceName) {
    (0, _classCallCheck2["default"])(this, CodegenRust);
    (0, _defineProperty2["default"])(this, "serviceName", void 0);
    this.serviceName = serviceName;
  }

  (0, _createClass2["default"])(CodegenRust, [{
    key: "hasServiceMethods",
    value: function hasServiceMethods() {
      return _registry.registry.getObjectsForServiceName(this.serviceName).flatMap(function (o) {
        return o.methods.attrs;
      }).length > 0;
    } // Generate the 'gen/mod.rs'

  }, {
    key: "generateGenMod",
    value: function () {
      var _generateGenMod = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee() {
        var results;
        return _regenerator["default"].wrap(function _callee$(_context) {
          while (1) {
            switch (_context.prev = _context.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", "", "pub mod model;", "pub mod service;"];
                _context.next = 3;
                return this.writeCode("gen/mod.rs", results.join("\n"));

              case 3:
              case "end":
                return _context.stop();
            }
          }
        }, _callee, this);
      }));

      function generateGenMod() {
        return _generateGenMod.apply(this, arguments);
      }

      return generateGenMod;
    }() // Generate the 'gen/model/mod.rs'

  }, {
    key: "generateGenModelMod",
    value: function () {
      var _generateGenModelMod = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee2() {
        var results, _iterator11, _step11, systemObject;

        return _regenerator["default"].wrap(function _callee2$(_context2) {
          while (1) {
            switch (_context2.prev = _context2.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator11 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(this.serviceName));

                try {
                  for (_iterator11.s(); !(_step11 = _iterator11.n()).done;) {
                    systemObject = _step11.value;

                    if (systemObject.kind() != "baseObject") {
                      results.push("pub mod ".concat((0, _changeCase.snakeCase)(systemObject.typeName), ";"));
                    }
                  }
                } catch (err) {
                  _iterator11.e(err);
                } finally {
                  _iterator11.f();
                }

                _context2.next = 5;
                return this.writeCode("gen/model/mod.rs", results.join("\n"));

              case 5:
              case "end":
                return _context2.stop();
            }
          }
        }, _callee2, this);
      }));

      function generateGenModelMod() {
        return _generateGenModelMod.apply(this, arguments);
      }

      return generateGenModelMod;
    }()
  }, {
    key: "generateGenService",
    value: function () {
      var _generateGenService = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee3() {
        var output;
        return _regenerator["default"].wrap(function _callee3$(_context3) {
          while (1) {
            switch (_context3.prev = _context3.next) {
              case 0:
                output = _ejs["default"].render("<%- include('src/codegen/rust/service.rs.ejs', { fmt: fmt }) %>", {
                  fmt: new RustFormatterService(this.serviceName)
                }, {
                  filename: "."
                });
                _context3.next = 3;
                return this.writeCode("gen/service.rs", output);

              case 3:
              case "end":
                return _context3.stop();
            }
          }
        }, _callee3, this);
      }));

      function generateGenService() {
        return _generateGenService.apply(this, arguments);
      }

      return generateGenService;
    }()
  }, {
    key: "generateGenModel",
    value: function () {
      var _generateGenModel = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee4(systemObject) {
        var output;
        return _regenerator["default"].wrap(function _callee4$(_context4) {
          while (1) {
            switch (_context4.prev = _context4.next) {
              case 0:
                output = _ejs["default"].render("<%- include('src/codegen/rust/model.rs.ejs', { fmt: fmt }) %>", {
                  fmt: new RustFormatter(systemObject)
                }, {
                  filename: "."
                });
                _context4.next = 3;
                return this.writeCode("gen/model/".concat((0, _changeCase.snakeCase)(systemObject.typeName), ".rs"), output);

              case 3:
              case "end":
                return _context4.stop();
            }
          }
        }, _callee4, this);
      }));

      function generateGenModel(_x) {
        return _generateGenModel.apply(this, arguments);
      }

      return generateGenModel;
    }()
  }, {
    key: "makePath",
    value: function () {
      var _makePath = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee5(pathPart) {
        var pathName, absolutePathName;
        return _regenerator["default"].wrap(function _callee5$(_context5) {
          while (1) {
            switch (_context5.prev = _context5.next) {
              case 0:
                pathName = _path["default"].join("..", "si-".concat(this.serviceName), "src", pathPart);
                absolutePathName = _path["default"].resolve(pathName);
                _context5.next = 4;
                return _fs["default"].promises.mkdir(_path["default"].resolve(pathName), {
                  recursive: true
                });

              case 4:
                return _context5.abrupt("return", absolutePathName);

              case 5:
              case "end":
                return _context5.stop();
            }
          }
        }, _callee5, this);
      }));

      function makePath(_x2) {
        return _makePath.apply(this, arguments);
      }

      return makePath;
    }()
  }, {
    key: "formatCode",
    value: function () {
      var _formatCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee6() {
        return _regenerator["default"].wrap(function _callee6$(_context6) {
          while (1) {
            switch (_context6.prev = _context6.next) {
              case 0:
                _context6.next = 2;
                return execCmd("cargo fmt -p si-".concat(this.serviceName));

              case 2:
              case "end":
                return _context6.stop();
            }
          }
        }, _callee6, this);
      }));

      function formatCode() {
        return _formatCode.apply(this, arguments);
      }

      return formatCode;
    }()
  }, {
    key: "writeCode",
    value: function () {
      var _writeCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee7(filename, code) {
        var pathname, basename, createdPath, codeFilename;
        return _regenerator["default"].wrap(function _callee7$(_context7) {
          while (1) {
            switch (_context7.prev = _context7.next) {
              case 0:
                pathname = _path["default"].dirname(filename);
                basename = _path["default"].basename(filename);
                _context7.next = 4;
                return this.makePath(pathname);

              case 4:
                createdPath = _context7.sent;
                codeFilename = _path["default"].join(createdPath, basename);
                _context7.next = 8;
                return _fs["default"].promises.writeFile(codeFilename, code);

              case 8:
              case "end":
                return _context7.stop();
            }
          }
        }, _callee7, this);
      }));

      function writeCode(_x3, _x4) {
        return _writeCode.apply(this, arguments);
      }

      return writeCode;
    }()
  }]);
  return CodegenRust;
}(); // export class CodegenRust {
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


exports.CodegenRust = CodegenRust;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwiQ29tcG9uZW50T2JqZWN0IiwiRW50aXR5T2JqZWN0IiwiRW50aXR5RXZlbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJwcm9wTWV0aG9kIiwicnVzdEZpZWxkTmFtZUZvclByb3AiLCJyZXBsYWNlIiwic2VydmljZU5hbWUiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJraW5kIiwiaXNFbnRpdHlPYmplY3QiLCJpc0VudGl0eUFjdGlvbk1ldGhvZCIsIm5hbWUiLCJlbmRzV2l0aCIsInJlbmRlck9wdGlvbnMiLCJsaXN0IiwicnVzdFR5cGVGb3JQcm9wIiwicmVxdWVzdCIsInJlcGx5Iiwib3B0aW9uIiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicmVzdWx0cyIsInByb3BNZXRob2RzIiwiYXR0cnMiLCJzb3J0IiwiYSIsImIiLCJvdXRwdXQiLCJwdXNoIiwiam9pbiIsInByb3AiLCJQcm9wUHJlbHVkZSIsIlByb3BBY3Rpb24iLCJQcm9wTWV0aG9kIiwicGFyZW50TmFtZSIsIlByb3BOdW1iZXIiLCJudW1iZXJLaW5kIiwiUHJvcEJvb2wiLCJQcm9wT2JqZWN0IiwiUHJvcExpbmsiLCJyZWFsUHJvcCIsImxvb2t1cE15c2VsZiIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcE1hcCIsIlByb3BUZXh0IiwiUHJvcENvZGUiLCJQcm9wU2VsZWN0IiwicmVwZWF0ZWQiLCJyZXN1bHQiLCJjcmVhdGVNZXRob2QiLCJwcm9wZXJ0aWVzIiwibGlzdE1ldGhvZCIsImZpZWxkTmFtZSIsImxpc3RSZXBseVZhbHVlIiwiU3lzdGVtT2JqZWN0IiwibmF0dXJhbEtleSIsIm1pZ3JhdGVhYmxlIiwidmFyaWFibGVOYW1lIiwiUHJvcFBhc3N3b3JkIiwibXZjYyIsImZpZWxkcyIsInJlcXVpcmVkIiwicHJvcE5hbWUiLCJ0b3BQcm9wIiwicHJlZml4IiwiaGlkZGVuIiwic3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wIiwicm9vdFByb3AiLCJmZXRjaFByb3BzIiwicmVmZXJlbmNlVmVjIiwic2lQcm9wZXJ0aWVzIiwiaXRlbU5hbWUiLCJCYXNlT2JqZWN0IiwibGVuZ3RoIiwiUnVzdEZvcm1hdHRlclNlcnZpY2UiLCJzeXN0ZW1PYmplY3RzIiwicmVnaXN0cnkiLCJnZXRPYmplY3RzRm9yU2VydmljZU5hbWUiLCJtYXAiLCJvIiwiaGFzRW50aXRpZXMiLCJpbXBsU2VydmljZVRyYWl0TmFtZSIsInN5c3RlbU9iaiIsImZpbmQiLCJzIiwiQ29kZWdlblJ1c3QiLCJmbGF0TWFwIiwid3JpdGVDb2RlIiwicGF0aFBhcnQiLCJwYXRoIiwiYWJzb2x1dGVQYXRoTmFtZSIsInJlc29sdmUiLCJmcyIsInByb21pc2VzIiwibWtkaXIiLCJyZWN1cnNpdmUiLCJjb2RlIiwicGF0aG5hbWUiLCJkaXJuYW1lIiwiYmFzZW5hbWUiLCJtYWtlUGF0aCIsImNyZWF0ZWRQYXRoIiwiY29kZUZpbGVuYW1lIiwid3JpdGVGaWxlIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7QUFRQTs7QUFDQTs7QUFHQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7Ozs7Ozs7QUFFQSxJQUFNQSxPQUFPLEdBQUdDLGlCQUFLQyxTQUFMLENBQWVDLDBCQUFhQyxJQUE1QixDQUFoQjs7SUFPYUMsYTtBQUdYLHlCQUFZQyxZQUFaLEVBQXlEO0FBQUE7QUFBQTtBQUN2RCxTQUFLQSxZQUFMLEdBQW9CQSxZQUFwQjtBQUNEOzs7O2lDQUVvQjtBQUNuQix3Q0FB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LFFBQUwsQ0FBY0gsUUFBZCxDOzs7QUFBcEJJLGdCQUFBQSxXO0FBQ0FDLGdCQUFBQSxZLEdBQWViLGlCQUFLOUQsSUFBTCxDQUFVMEUsV0FBVixFQUF1QkYsUUFBdkIsQzs7dUJBQ2ZQLGVBQUdDLFFBQUgsQ0FBWVUsU0FBWixDQUFzQkQsWUFBdEIsRUFBb0NOLElBQXBDLEM7Ozs7Ozs7Ozs7Ozs7Ozs7OztLQUlWO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHtcbiAgT2JqZWN0VHlwZXMsXG4gIEJhc2VPYmplY3QsXG4gIFN5c3RlbU9iamVjdCxcbiAgQ29tcG9uZW50T2JqZWN0LFxuICBFbnRpdHlPYmplY3QsXG4gIEVudGl0eUV2ZW50T2JqZWN0LFxufSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuaW1wb3J0IHsgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcblxuaW1wb3J0IHsgc25ha2VDYXNlLCBwYXNjYWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5pbXBvcnQgZWpzIGZyb20gXCJlanNcIjtcbmltcG9ydCBmcyBmcm9tIFwiZnNcIjtcbmltcG9ydCBwYXRoIGZyb20gXCJwYXRoXCI7XG5pbXBvcnQgY2hpbGRQcm9jZXNzIGZyb20gXCJjaGlsZF9wcm9jZXNzXCI7XG5pbXBvcnQgdXRpbCBmcm9tIFwidXRpbFwiO1xuXG5jb25zdCBleGVjQ21kID0gdXRpbC5wcm9taXNpZnkoY2hpbGRQcm9jZXNzLmV4ZWMpO1xuXG5pbnRlcmZhY2UgUnVzdFR5cGVBc1Byb3BPcHRpb25zIHtcbiAgcmVmZXJlbmNlPzogYm9vbGVhbjtcbiAgb3B0aW9uPzogYm9vbGVhbjtcbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXIge1xuICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogUnVzdEZvcm1hdHRlcltcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgc3RydWN0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICBtb2RlbE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjptb2RlbDo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX1gO1xuICB9XG5cbiAgY29tcG9uZW50TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1Db21wb25lbnRgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gY29tcG9uZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBjb21wb25lbnRDb25zdHJhaW50c05hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9Q29tcG9uZW50Q29uc3RyYWludHNgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYSBjb21wb25lbnQgY29uc3RyYWludHMgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eUVkaXRNZXRob2ROYW1lKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkge1xuICAgICAgcmV0dXJuIGBlZGl0XyR7dGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wTWV0aG9kKS5yZXBsYWNlKFxuICAgICAgICBcIl9lZGl0XCIsXG4gICAgICAgIFwiXCIsXG4gICAgICApfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlZGl0IG1ldGhvZCBuYW1lIG9uIGEgbm9uLWVudGl0eSBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5RXZlbnROYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eUV2ZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eUV2ZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHkgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eVByb3BlcnRpZXNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eVByb3BlcnRpZXNgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5UHJvcGVydGllcyBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgbW9kZWxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wTWV0aG9kKTtcbiAgfVxuXG4gIHR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSk7XG4gIH1cblxuICBlcnJvclR5cGUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjplcnJvcjo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lKX1FcnJvcmA7XG4gIH1cblxuICBoYXNDcmVhdGVNZXRob2QoKTogYm9vbGVhbiB7XG4gICAgdHJ5IHtcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGNhdGNoIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cblxuICBpc0NvbXBvbmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiY29tcG9uZW50T2JqZWN0XCI7XG4gIH1cblxuICBpc0VudGl0eU9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiZW50aXR5T2JqZWN0XCI7XG4gIH1cblxuICBpc0VudGl0eUV2ZW50T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJlbnRpdHlFdmVudE9iamVjdFwiO1xuICB9XG5cbiAgaXNFbnRpdHlBY3Rpb25NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiBwcm9wTWV0aG9kLmtpbmQoKSA9PSBcImFjdGlvblwiICYmIHRoaXMuaXNFbnRpdHlPYmplY3QoKTtcbiAgfVxuXG4gIGlzRW50aXR5RWRpdE1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHRoaXMuaXNFbnRpdHlBY3Rpb25NZXRob2QocHJvcE1ldGhvZCkgJiYgcHJvcE1ldGhvZC5uYW1lLmVuZHNXaXRoKFwiRWRpdFwiKVxuICAgICk7XG4gIH1cblxuICBpbXBsTGlzdFJlcXVlc3RUeXBlKHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9KTogc3RyaW5nIHtcbiAgICBjb25zdCBsaXN0ID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIFwibGlzdFwiLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZDtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AobGlzdC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxMaXN0UmVwbHlUeXBlKHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9KTogc3RyaW5nIHtcbiAgICBjb25zdCBsaXN0ID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIFwibGlzdFwiLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZDtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AobGlzdC5yZXBseSwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVJlcXVlc3RUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVxdWVzdCwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVJlcGx5VHlwZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKFxuICAgICAgdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZCwge1xuICAgICAgICBvcHRpb246IGZhbHNlLFxuICAgICAgICByZWZlcmVuY2U6IGZhbHNlLFxuICAgICAgfSksXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5QWN0aW9uKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUFjdGlvbi5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlFZGl0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUVkaXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUNvbW1vbkNyZWF0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5Q3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUdldChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VHZXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTGlzdChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VMaXN0LnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUNvbXBvbmVudFBpY2socHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tcG9uZW50UGljay5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDdXN0b21NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ3VzdG9tTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUF1dGgocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgaWYgKHByb3BNZXRob2Quc2tpcEF1dGgpIHtcbiAgICAgIHJldHVybiBgLy8gQXV0aGVudGljYXRpb24gaXMgc2tpcHBlZCBvbiBcXGAke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgICBwcm9wTWV0aG9kLFxuICAgICAgKX1cXGBcXG5gO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gdGhpcy5pbXBsU2VydmljZUF1dGhDYWxsKHByb3BNZXRob2QpO1xuICAgIH1cbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgbGV0IHByZWx1ZGUgPSBcInNpX2FjY291bnQ6OmF1dGhvcml6ZVwiO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSA9PSBcImFjY291bnRcIikge1xuICAgICAgcHJlbHVkZSA9IFwiY3JhdGU6OmF1dGhvcml6ZVwiO1xuICAgIH1cbiAgICByZXR1cm4gYCR7cHJlbHVkZX06OmF1dGhueigmc2VsZi5kYiwgJnJlcXVlc3QsIFwiJHt0aGlzLmltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICAgIHByb3BNZXRob2QsXG4gICAgKX1cIikuYXdhaXQ/O2A7XG4gIH1cblxuICBzZXJ2aWNlTWV0aG9kcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBjb25zdCBwcm9wTWV0aG9kcyA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMuc29ydCgoYSwgYikgPT5cbiAgICAgIGEubmFtZSA+IGIubmFtZSA/IDEgOiAtMSxcbiAgICApO1xuICAgIGZvciAoY29uc3QgcHJvcE1ldGhvZCBvZiBwcm9wTWV0aG9kcykge1xuICAgICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgICAge1xuICAgICAgICAgIGZtdDogdGhpcyxcbiAgICAgICAgICBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kLFxuICAgICAgICB9LFxuICAgICAgICB7XG4gICAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgICB9LFxuICAgICAgKTtcbiAgICAgIHJlc3VsdHMucHVzaChvdXRwdXQpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgcnVzdEZpZWxkTmFtZUZvclByb3AocHJvcDogUHJvcHMpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgfVxuXG4gIHJ1c3RUeXBlRm9yUHJvcChcbiAgICBwcm9wOiBQcm9wcyxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZWZlcmVuY2UgPSByZW5kZXJPcHRpb25zLnJlZmVyZW5jZSB8fCBmYWxzZTtcbiAgICBsZXQgb3B0aW9uID0gdHJ1ZTtcbiAgICBpZiAocmVuZGVyT3B0aW9ucy5vcHRpb24gPT09IGZhbHNlKSB7XG4gICAgICBvcHRpb24gPSBmYWxzZTtcbiAgICB9XG5cbiAgICBsZXQgdHlwZU5hbWU6IHN0cmluZztcblxuICAgIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTnVtYmVyKSB7XG4gICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTMyXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1MzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50NjRcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTY0XCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1NjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidTEyOFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1MTI4XCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQm9vbCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLm5hbWUsXG4gICAgICApfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgIGNvbnN0IHJlYWxQcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIGlmIChyZWFsUHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgY29uc3QgcHJvcE93bmVyID0gcHJvcC5sb29rdXBPYmplY3QoKTtcbiAgICAgICAgbGV0IHBhdGhOYW1lOiBzdHJpbmc7XG4gICAgICAgIGlmIChcbiAgICAgICAgICBwcm9wT3duZXIuc2VydmljZU5hbWUgJiZcbiAgICAgICAgICBwcm9wT3duZXIuc2VydmljZU5hbWUgPT0gdGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWVcbiAgICAgICAgKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBcImNyYXRlOjpwcm90b2J1ZlwiO1xuICAgICAgICB9IGVsc2UgaWYgKHByb3BPd25lci5zZXJ2aWNlTmFtZSkge1xuICAgICAgICAgIHBhdGhOYW1lID0gYHNpXyR7cHJvcE93bmVyLnNlcnZpY2VOYW1lfTo6cHJvdG9idWZgO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfVxuICAgICAgICB0eXBlTmFtZSA9IGAke3BhdGhOYW1lfTo6JHtwYXNjYWxDYXNlKHJlYWxQcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgICByZWFsUHJvcC5uYW1lLFxuICAgICAgICApfWA7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocmVhbFByb3AsIHJlbmRlck9wdGlvbnMpO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNYXApIHtcbiAgICAgIHR5cGVOYW1lID0gYHN0ZDo6Y29sbGVjdGlvbnM6Okhhc2hNYXA8U3RyaW5nLCBTdHJpbmc+YDtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BUZXh0IHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wU2VsZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IFwiU3RyaW5nXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IGBDYW5ub3QgZ2VuZXJhdGUgdHlwZSBmb3IgJHtwcm9wLm5hbWV9IGtpbmQgJHtwcm9wLmtpbmQoKX0gLSBCdWchYDtcbiAgICB9XG4gICAgaWYgKHJlZmVyZW5jZSkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICBpZiAodHlwZU5hbWUgPT0gXCJTdHJpbmdcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiJnN0clwiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICAgIHR5cGVOYW1lID0gYCYke3R5cGVOYW1lfWA7XG4gICAgICB9XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgIHR5cGVOYW1lID0gYFZlYzwke3R5cGVOYW1lfT5gO1xuICAgIH0gZWxzZSB7XG4gICAgICBpZiAob3B0aW9uKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGBPcHRpb248JHt0eXBlTmFtZX0+YDtcbiAgICAgIH1cbiAgICB9XG4gICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgcmV0dXJuIHR5cGVOYW1lO1xuICB9XG5cbiAgaW1wbENyZWF0ZU5ld0FyZ3MoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICByZXN1bHQucHVzaChgJHtzbmFrZUNhc2UocHJvcC5uYW1lKX06ICR7dGhpcy5ydXN0VHlwZUZvclByb3AocHJvcCl9YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbENyZWF0ZVBhc3NOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goc25ha2VDYXNlKHByb3AubmFtZSkpO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTGlzdFJlc3VsdFRvUmVwbHkoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBsaXN0TWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImxpc3RcIik7XG4gICAgaWYgKGxpc3RNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgbGlzdE1ldGhvZC5yZXBseS5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IGZpZWxkTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBsZXQgbGlzdFJlcGx5VmFsdWUgPSBgU29tZShvdXRwdXQuJHtmaWVsZE5hbWV9KWA7XG4gICAgICAgIGlmIChmaWVsZE5hbWUgPT0gXCJuZXh0X3BhZ2VfdG9rZW5cIikge1xuICAgICAgICAgIGxpc3RSZXBseVZhbHVlID0gXCJTb21lKG91dHB1dC5wYWdlX3Rva2VuKVwiO1xuICAgICAgICB9IGVsc2UgaWYgKGZpZWxkTmFtZSA9PSBcIml0ZW1zXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IGBvdXRwdXQuJHtmaWVsZE5hbWV9YDtcbiAgICAgICAgfVxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9OiAke2xpc3RSZXBseVZhbHVlfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kQ3JlYXRlRGVzdHJ1Y3R1cmUoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgcmVzdWx0LnB1c2goYGxldCAke2ZpZWxkTmFtZX0gPSBpbm5lci4ke2ZpZWxkTmFtZX07YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIG5hdHVyYWxLZXkoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QubmF0dXJhbEtleSk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIm5hbWVcIjtcbiAgICB9XG4gIH1cblxuICBpc01pZ3JhdGVhYmxlKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICAvLyBAdHMtaWdub3JlXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIgJiYgdGhpcy5zeXN0ZW1PYmplY3QubWlncmF0ZWFibGVcbiAgICApO1xuICB9XG5cbiAgaXNTdG9yYWJsZSgpOiBib29sZWFuIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaW1wbENyZWF0ZVNldFByb3BlcnRpZXMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCB2YXJpYWJsZU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wUGFzc3dvcmQpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gU29tZShzaV9kYXRhOjpwYXNzd29yZDo6ZW5jcnlwdF9wYXNzd29yZCgke3ZhcmlhYmxlTmFtZX0pPyk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gJHt2YXJpYWJsZU5hbWV9O2ApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVBZGRUb1RlbmFuY3koKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImJpbGxpbmdBY2NvdW50XCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25cIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblNlcnZpY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImNvbXBvbmVudE9iamVjdFwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uU2VydmljZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX3NlcnZpY2VfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcInVzZXJcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJncm91cFwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIm9yZ2FuaXphdGlvblwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uSW5zdGFuY2VcIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwid29ya3NwYWNlXCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IHdvcmtzcGFjZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkud29ya3NwYWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLndvcmtzcGFjZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKHdvcmtzcGFjZV9pZCk7YCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlSXNNdmNjKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0Lm12Y2MgPT0gdHJ1ZSkge1xuICAgICAgcmV0dXJuIFwidHJ1ZVwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJmYWxzZVwiO1xuICAgIH1cbiAgfVxuXG4gIHN0b3JhYmxlVmFsaWRhdGVGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICAgIGNvbnN0IHByb3BOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0ubGVuKCkgPT0gMCB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmlzX25vbmUoKSB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICB0b3BQcm9wOiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0LFxuICAgIHByZWZpeDogc3RyaW5nLFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbJ1wic2lTdG9yYWJsZS5uYXR1cmFsS2V5XCInXTtcbiAgICBmb3IgKGxldCBwcm9wIG9mIHRvcFByb3AucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKHByb3AuaGlkZGVuKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBwcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgcHJvcC5uYW1lKSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgICAgdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgYCR7cHJlZml4fS4ke3Byb3AubmFtZX1gKSxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcmVmaXh9LiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5yb290UHJvcCxcbiAgICAgIFwiXCIsXG4gICAgKTtcbiAgICByZXR1cm4gYHZlYyFbJHtyZXN1bHRzfV1cXG5gO1xuICB9XG5cbiAgc3RvcmFibGVSZWZlcmVudGlhbEZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgZmV0Y2hQcm9wcyA9IFtdO1xuICAgIGNvbnN0IHJlZmVyZW5jZVZlYyA9IFtdO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QpIHtcbiAgICAgIGxldCBzaVByb3BlcnRpZXMgPSB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuZ2V0RW50cnkoXCJzaVByb3BlcnRpZXNcIik7XG4gICAgICBpZiAoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgc2lQcm9wZXJ0aWVzID0gc2lQcm9wZXJ0aWVzLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKCEoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkpIHtcbiAgICAgICAgdGhyb3cgXCJDYW5ub3QgZ2V0IHByb3BlcnRpZXMgb2YgYSBub24gb2JqZWN0IGluIHJlZiBjaGVja1wiO1xuICAgICAgfVxuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIHNpUHJvcGVydGllcy5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGlmIChwcm9wLnJlZmVyZW5jZSkge1xuICAgICAgICAgIGNvbnN0IGl0ZW1OYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICAgIGZldGNoUHJvcHMucHVzaChgbGV0ICR7aXRlbU5hbWV9ID0gbWF0Y2ggJnNlbGYuc2lfcHJvcGVydGllcyB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICBTb21lKGNpcCkgPT4gY2lwXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuJHtpdGVtTmFtZX1cbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5hc19yZWYoKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLm1hcChTdHJpbmc6OmFzX3JlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC51bndyYXBfb3IoXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICBOb25lID0+IFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiLFxuICAgICAgICAgICAgICAgICAgICAgICAgIH07YCk7XG4gICAgICAgICAgICByZWZlcmVuY2VWZWMucHVzaChcbiAgICAgICAgICAgICAgYHNpX2RhdGE6OlJlZmVyZW5jZTo6SGFzTWFueShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc09uZShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgfVxuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQmFzZU9iamVjdCkge1xuICAgIH1cblxuICAgIGlmIChmZXRjaFByb3BzLmxlbmd0aCAmJiByZWZlcmVuY2VWZWMubGVuZ3RoKSB7XG4gICAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgICByZXN1bHRzLnB1c2goZmV0Y2hQcm9wcy5qb2luKFwiXFxuXCIpKTtcbiAgICAgIHJlc3VsdHMucHVzaChgdmVjIVske3JlZmVyZW5jZVZlYy5qb2luKFwiLFwiKX1dYCk7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJWZWM6Om5ldygpXCI7XG4gICAgfVxuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyU2VydmljZSB7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZykge1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWUpO1xuICB9XG5cbiAgc3lzdGVtT2JqZWN0c0FzRm9ybWF0dGVycygpOiBSdXN0Rm9ybWF0dGVyW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNcbiAgICAgIC5zb3J0KChhLCBiKSA9PiAoYS50eXBlTmFtZSA+IGIudHlwZU5hbWUgPyAxIDogLTEpKVxuICAgICAgLm1hcChvID0+IG5ldyBSdXN0Rm9ybWF0dGVyKG8pKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Qm9keSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiOiBzaV9kYXRhOjpEYixcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudCxcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTmV3Q29uc3RydWN0b3JBcmdzKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiLCBhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudFwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJkYjogc2lfZGF0YTo6RGJcIjtcbiAgICB9XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdENvbnN0cnVjdG9yUmV0dXJuKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wiZGJcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudFwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlVHJhaXROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7c25ha2VDYXNlKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApfV9zZXJ2ZXI6OiR7cGFzY2FsQ2FzZSh0aGlzLnNlcnZpY2VOYW1lKX1gO1xuICB9XG5cbiAgaW1wbFNlcnZlck5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5pbXBsU2VydmljZVRyYWl0TmFtZSgpfVNlcnZlcmA7XG4gIH1cblxuICBpbXBsU2VydmljZU1pZ3JhdGUoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iaiBvZiB0aGlzLnN5c3RlbU9iamVjdHMpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIGlmIChzeXN0ZW1PYmoua2luZCgpICE9IFwiYmFzZU9iamVjdFwiICYmIHN5c3RlbU9iai5taWdyYXRlYWJsZSA9PSB0cnVlKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgIHN5c3RlbU9iai50eXBlTmFtZSxcbiAgICAgICAgICApfTo6bWlncmF0ZSgmc2VsZi5kYikuYXdhaXQ/O2AsXG4gICAgICAgICk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGhhc0VudGl0aWVzKCk6IGJvb2xlYW4ge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdHMuZmluZChzID0+IHMua2luZCgpID09IFwiZW50aXR5T2JqZWN0XCIpKSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGhhc01pZ3JhdGFibGVzKCk6IGJvb2xlYW4ge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0cy5maW5kKFxuICAgICAgICAvLyBAdHMtaWdub3JlXG4gICAgICAgIHMgPT4gcy5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIgJiYgcy5taWdyYXRlYWJsZSA9PSB0cnVlLFxuICAgICAgKVxuICAgICkge1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIENvZGVnZW5SdXN0IHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICB9XG5cbiAgaGFzU2VydmljZU1ldGhvZHMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHJlZ2lzdHJ5XG4gICAgICAgIC5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUodGhpcy5zZXJ2aWNlTmFtZSlcbiAgICAgICAgLmZsYXRNYXAobyA9PiBvLm1ldGhvZHMuYXR0cnMpLmxlbmd0aCA+IDBcbiAgICApO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbk1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1xuICAgICAgXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLFxuICAgICAgXCIvLyBObyB0b3VjaHkhXCIsXG4gICAgICBcIlwiLFxuICAgICAgXCJwdWIgbW9kIG1vZGVsO1wiLFxuICAgICAgXCJwdWIgbW9kIHNlcnZpY2U7XCIsXG4gICAgXTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL21vZGVsL21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2RlbE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXCIsIFwiXCJdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqZWN0IG9mIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKSkge1xuICAgICAgaWYgKHN5c3RlbU9iamVjdC5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKGBwdWIgbW9kICR7c25ha2VDYXNlKHN5c3RlbU9iamVjdC50eXBlTmFtZSl9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2RlbC9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuU2VydmljZSgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlclNlcnZpY2UodGhpcy5zZXJ2aWNlTmFtZSksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoYGdlbi9zZXJ2aWNlLnJzYCwgb3V0cHV0KTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWwoc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcyk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L21vZGVsLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlcihzeXN0ZW1PYmplY3QpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFxuICAgICAgYGdlbi9tb2RlbC8ke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfS5yc2AsXG4gICAgICBvdXRwdXQsXG4gICAgKTtcbiAgfVxuXG4gIGFzeW5jIG1ha2VQYXRoKHBhdGhQYXJ0OiBzdHJpbmcpOiBQcm9taXNlPHN0cmluZz4ge1xuICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFwiLi5cIiwgYHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gLCBcInNyY1wiLCBwYXRoUGFydCk7XG4gICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4gICAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIocGF0aC5yZXNvbHZlKHBhdGhOYW1lKSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4gICAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4gIH1cblxuICBhc3luYyBmb3JtYXRDb2RlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGF3YWl0IGV4ZWNDbWQoYGNhcmdvIGZtdCAtcCBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCk7XG4gIH1cblxuICBhc3luYyB3cml0ZUNvZGUoZmlsZW5hbWU6IHN0cmluZywgY29kZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcGF0aG5hbWUgPSBwYXRoLmRpcm5hbWUoZmlsZW5hbWUpO1xuICAgIGNvbnN0IGJhc2VuYW1lID0gcGF0aC5iYXNlbmFtZShmaWxlbmFtZSk7XG4gICAgY29uc3QgY3JlYXRlZFBhdGggPSBhd2FpdCB0aGlzLm1ha2VQYXRoKHBhdGhuYW1lKTtcbiAgICBjb25zdCBjb2RlRmlsZW5hbWUgPSBwYXRoLmpvaW4oY3JlYXRlZFBhdGgsIGJhc2VuYW1lKTtcbiAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoY29kZUZpbGVuYW1lLCBjb2RlKTtcbiAgfVxufVxuXG4vLyBleHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuLy8gICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuLy8gICBmb3JtYXR0ZXI6IFJ1c3RGb3JtYXR0ZXI7XG4vL1xuLy8gICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzKSB7XG4vLyAgICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4vLyAgICAgdGhpcy5mb3JtYXR0ZXIgPSBuZXcgUnVzdEZvcm1hdHRlcihzeXN0ZW1PYmplY3QpO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyB3cml0ZUNvZGUocGFydDogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBjcmVhdGVkUGF0aCA9IGF3YWl0IHRoaXMubWFrZVBhdGgoKTtcbi8vICAgICBjb25zdCBjb2RlRmlsZW5hbWUgPSBwYXRoLmpvaW4oY3JlYXRlZFBhdGgsIGAke3NuYWtlQ2FzZShwYXJ0KX0ucnNgKTtcbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoY29kZUZpbGVuYW1lLCBjb2RlKTtcbi8vICAgICBhd2FpdCBleGVjQ21kKGBydXN0Zm10ICR7Y29kZUZpbGVuYW1lfWApO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBtYWtlUGF0aCgpOiBQcm9taXNlPHN0cmluZz4ge1xuLy8gICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgICAgX19kaXJuYW1lLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgdGhpcy5zeXN0ZW1PYmplY3Quc2lQYXRoTmFtZSxcbi8vICAgICAgIFwic3JjXCIsXG4vLyAgICAgICBcImdlblwiLFxuLy8gICAgICAgc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKSxcbi8vICAgICApO1xuLy8gICAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKHBhdGgucmVzb2x2ZShwYXRoTmFtZSksIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuLy8gICAgIHJldHVybiBhYnNvbHV0ZVBhdGhOYW1lO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBnZW5lcmF0ZUNvbXBvbmVudEltcGxzKCk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4vLyAgICAgICBcIjwlLSBpbmNsdWRlKCdydXN0L2NvbXBvbmVudC5ycy5lanMnLCB7IGNvbXBvbmVudDogY29tcG9uZW50IH0pICU+XCIsXG4vLyAgICAgICB7XG4vLyAgICAgICAgIHN5c3RlbU9iamVjdDogdGhpcy5zeXN0ZW1PYmplY3QsXG4vLyAgICAgICAgIGZtdDogdGhpcy5mb3JtYXR0ZXIsXG4vLyAgICAgICB9LFxuLy8gICAgICAge1xuLy8gICAgICAgICBmaWxlbmFtZTogX19maWxlbmFtZSxcbi8vICAgICAgIH0sXG4vLyAgICAgKTtcbi8vICAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImNvbXBvbmVudFwiLCBvdXRwdXQpO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBnZW5lcmF0ZUNvbXBvbmVudE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBtb2RzID0gW1wiY29tcG9uZW50XCJdO1xuLy8gICAgIGNvbnN0IGxpbmVzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyBUb3VjaHkhXFxuXCJdO1xuLy8gICAgIGZvciAoY29uc3QgbW9kIG9mIG1vZHMpIHtcbi8vICAgICAgIGxpbmVzLnB1c2goYHB1YiBtb2QgJHttb2R9O2ApO1xuLy8gICAgIH1cbi8vICAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcIm1vZFwiLCBsaW5lcy5qb2luKFwiXFxuXCIpKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyIHtcbi8vICAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcbi8vXG4vLyAgIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogUnVzdEZvcm1hdHRlcltcInN5c3RlbU9iamVjdFwiXSkge1xuLy8gICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRUeXBlTmFtZSgpOiBzdHJpbmcge1xuLy8gICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRPcmRlckJ5RmllbGRzKCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3Qgb3JkZXJCeUZpZWxkcyA9IFtdO1xuLy8gICAgIGNvbnN0IGNvbXBvbmVudE9iamVjdCA9IHRoaXMuY29tcG9uZW50LmFzQ29tcG9uZW50KCk7XG4vLyAgICAgZm9yIChjb25zdCBwIG9mIGNvbXBvbmVudE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICBpZiAocC5oaWRkZW4pIHtcbi8vICAgICAgICAgY29udGludWU7XG4vLyAgICAgICB9XG4vLyAgICAgICBpZiAocC5uYW1lID09IFwic3RvcmFibGVcIikge1xuLy8gICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goJ1wic3RvcmFibGUubmF0dXJhbEtleVwiJyk7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaCgnXCJzdG9yYWJsZS50eXBlTmFtZVwiJyk7XG4vLyAgICAgICB9IGVsc2UgaWYgKHAubmFtZSA9PSBcInNpUHJvcGVydGllc1wiKSB7XG4vLyAgICAgICAgIGNvbnRpbnVlO1xuLy8gICAgICAgfSBlbHNlIGlmIChwLm5hbWUgPT0gXCJjb25zdHJhaW50c1wiICYmIHAua2luZCgpID09IFwib2JqZWN0XCIpIHtcbi8vICAgICAgICAgLy8gQHRzLWlnbm9yZSB0cnVzdCB1cyAtIHdlIGNoZWNrZWRcbi8vICAgICAgICAgZm9yIChjb25zdCBwYyBvZiBwLnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgICAgICBpZiAocGMua2luZCgpICE9IFwib2JqZWN0XCIpIHtcbi8vICAgICAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaChgXCJjb25zdHJhaW50cy4ke3BjLm5hbWV9XCJgKTtcbi8vICAgICAgICAgICB9XG4vLyAgICAgICAgIH1cbi8vICAgICAgIH0gZWxzZSB7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaChgXCIke3AubmFtZX1cImApO1xuLy8gICAgICAgfVxuLy8gICAgIH1cbi8vICAgICByZXR1cm4gYHZlYyFbJHtvcmRlckJ5RmllbGRzLmpvaW4oXCIsXCIpfV1cXG5gO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRJbXBvcnRzKCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3QgcmVzdWx0ID0gW107XG4vLyAgICAgcmVzdWx0LnB1c2goXG4vLyAgICAgICBgcHViIHVzZSBjcmF0ZTo6cHJvdG9idWY6OiR7c25ha2VDYXNlKHRoaXMuY29tcG9uZW50LnR5cGVOYW1lKX06OntgLFxuLy8gICAgICAgYCAgQ29uc3RyYWludHMsYCxcbi8vICAgICAgIGAgIExpc3RDb21wb25lbnRzUmVwbHksYCxcbi8vICAgICAgIGAgIExpc3RDb21wb25lbnRzUmVxdWVzdCxgLFxuLy8gICAgICAgYCAgUGlja0NvbXBvbmVudFJlcXVlc3QsYCxcbi8vICAgICAgIGAgIENvbXBvbmVudCxgLFxuLy8gICAgICAgYH07YCxcbi8vICAgICApO1xuLy8gICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50VmFsaWRhdGlvbigpOiBzdHJpbmcge1xuLy8gICAgIHJldHVybiB0aGlzLmdlblZhbGlkYXRpb24odGhpcy5jb21wb25lbnQuYXNDb21wb25lbnQoKSk7XG4vLyAgIH1cbi8vXG4vLyAgIGdlblZhbGlkYXRpb24ocHJvcE9iamVjdDogUHJvcE9iamVjdCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3QgcmVzdWx0ID0gW107XG4vLyAgICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbi8vICAgICAgICAgY29uc3QgcHJvcE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbi8vICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0uaXNfbm9uZSgpIHtcbi8vICAgICAgICAgICByZXR1cm4gRXJyKERhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuLy8gICAgICAgICB9YCk7XG4vLyAgICAgICB9XG4vLyAgICAgfVxuLy8gICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZW5lcmF0ZUdlbk1vZCh3cml0dGVuQ29tcG9uZW50czoge1xuLy8gICBba2V5OiBzdHJpbmddOiBzdHJpbmdbXTtcbi8vIH0pOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgZm9yIChjb25zdCBjb21wb25lbnQgaW4gd3JpdHRlbkNvbXBvbmVudHMpIHtcbi8vICAgICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICAgIF9fZGlybmFtZSxcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIGNvbXBvbmVudCxcbi8vICAgICAgIFwic3JjXCIsXG4vLyAgICAgICBcImdlblwiLFxuLy8gICAgICk7XG4vLyAgICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgICAgY29uc3QgY29kZSA9IFtcbi8vICAgICAgIFwiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIixcbi8vICAgICAgIFwiLy8gTm8gdG91Y2h5IVwiLFxuLy8gICAgICAgXCJcIixcbi8vICAgICAgIFwicHViIG1vZCBtb2RlbDtcIixcbi8vICAgICBdO1xuLy9cbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoXG4vLyAgICAgICBwYXRoLmpvaW4oYWJzb2x1dGVQYXRoTmFtZSwgXCJtb2QucnNcIiksXG4vLyAgICAgICBjb2RlLmpvaW4oXCJcXG5cIiksXG4vLyAgICAgKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZW5lcmF0ZUdlbk1vZE1vZGVsKHNlcnZpY2VOYW1lOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgX19kaXJuYW1lLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIHNlcnZpY2VOYW1lLFxuLy8gICAgIFwic3JjXCIsXG4vLyAgICAgXCJnZW5cIixcbi8vICAgICBcIm1vZGVsXCIsXG4vLyAgICk7XG4vLyAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICBjb25zdCBjb2RlID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXFxuXCJdO1xuLy8gICBmb3IgKGNvbnN0IHR5cGVOYW1lIG9mIHdyaXR0ZW5Db21wb25lbnRzW2NvbXBvbmVudF0pIHtcbi8vICAgICBjb2RlLnB1c2goYHB1YiBtb2QgJHtzbmFrZUNhc2UodHlwZU5hbWUpfTtgKTtcbi8vICAgfVxuLy9cbi8vICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKFxuLy8gICAgIHBhdGguam9pbihhYnNvbHV0ZVBhdGhOYW1lLCBcIm1vZC5yc1wiKSxcbi8vICAgICBjb2RlLmpvaW4oXCJcXG5cIiksXG4vLyAgICk7XG4vLyB9XG4iXX0=