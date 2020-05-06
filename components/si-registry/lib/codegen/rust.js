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

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

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
  } // Generate the 'gen/mod.rs'


  (0, _createClass2["default"])(CodegenRust, [{
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwiQ29tcG9uZW50T2JqZWN0IiwiRW50aXR5T2JqZWN0IiwiRW50aXR5RXZlbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJwcm9wTWV0aG9kIiwicnVzdEZpZWxkTmFtZUZvclByb3AiLCJyZXBsYWNlIiwic2VydmljZU5hbWUiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJraW5kIiwiaXNFbnRpdHlPYmplY3QiLCJpc0VudGl0eUFjdGlvbk1ldGhvZCIsIm5hbWUiLCJlbmRzV2l0aCIsInJlbmRlck9wdGlvbnMiLCJsaXN0IiwicnVzdFR5cGVGb3JQcm9wIiwicmVxdWVzdCIsInJlcGx5Iiwib3B0aW9uIiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicmVzdWx0cyIsInByb3BNZXRob2RzIiwiYXR0cnMiLCJzb3J0IiwiYSIsImIiLCJvdXRwdXQiLCJwdXNoIiwiam9pbiIsInByb3AiLCJQcm9wUHJlbHVkZSIsIlByb3BBY3Rpb24iLCJQcm9wTWV0aG9kIiwicGFyZW50TmFtZSIsIlByb3BOdW1iZXIiLCJudW1iZXJLaW5kIiwiUHJvcEJvb2wiLCJQcm9wT2JqZWN0IiwiUHJvcExpbmsiLCJyZWFsUHJvcCIsImxvb2t1cE15c2VsZiIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcE1hcCIsIlByb3BUZXh0IiwiUHJvcENvZGUiLCJQcm9wU2VsZWN0IiwicmVwZWF0ZWQiLCJyZXN1bHQiLCJjcmVhdGVNZXRob2QiLCJwcm9wZXJ0aWVzIiwibGlzdE1ldGhvZCIsImZpZWxkTmFtZSIsImxpc3RSZXBseVZhbHVlIiwiU3lzdGVtT2JqZWN0IiwibmF0dXJhbEtleSIsIm1pZ3JhdGVhYmxlIiwidmFyaWFibGVOYW1lIiwiUHJvcFBhc3N3b3JkIiwiZmllbGRzIiwicmVxdWlyZWQiLCJwcm9wTmFtZSIsInRvcFByb3AiLCJwcmVmaXgiLCJoaWRkZW4iLCJzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AiLCJyb290UHJvcCIsImZldGNoUHJvcHMiLCJyZWZlcmVuY2VWZWMiLCJzaVByb3BlcnRpZXMiLCJpdGVtTmFtZSIsIkJhc2VPYmplY3QiLCJsZW5ndGgiLCJSdXN0Rm9ybWF0dGVyU2VydmljZSIsInN5c3RlbU9iamVjdHMiLCJyZWdpc3RyeSIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsIm1hcCIsIm8iLCJoYXNFbnRpdGllcyIsImltcGxTZXJ2aWNlVHJhaXROYW1lIiwic3lzdGVtT2JqIiwiZmluZCIsInMiLCJDb2RlZ2VuUnVzdCIsIndyaXRlQ29kZSIsInBhdGhQYXJ0IiwicGF0aCIsImFic29sdXRlUGF0aE5hbWUiLCJyZXNvbHZlIiwiZnMiLCJwcm9taXNlcyIsIm1rZGlyIiwicmVjdXJzaXZlIiwiY29kZSIsInBhdGhuYW1lIiwiZGlybmFtZSIsImJhc2VuYW1lIiwibWFrZVBhdGgiLCJjcmVhdGVkUGF0aCIsImNvZGVGaWxlbmFtZSIsIndyaXRlRmlsZSJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0FBUUE7O0FBQ0E7O0FBR0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7Ozs7Ozs7O0FBRUEsSUFBTUEsT0FBTyxHQUFHQyxpQkFBS0MsU0FBTCxDQUFlQywwQkFBYUMsSUFBNUIsQ0FBaEI7O0lBT2FDLGE7QUFHWCx5QkFBWUMsWUFBWixFQUF5RDtBQUFBO0FBQUE7QUFDdkQsU0FBS0EsWUFBTCxHQUFvQkEsWUFBcEI7QUFDRDs7OztpQ0FFb0I7QUFDbkIsd0NBQTJCLDRCQUFXLEtBQUtBLFlB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QUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHtcbiAgT2JqZWN0VHlwZXMsXG4gIEJhc2VPYmplY3QsXG4gIFN5c3RlbU9iamVjdCxcbiAgQ29tcG9uZW50T2JqZWN0LFxuICBFbnRpdHlPYmplY3QsXG4gIEVudGl0eUV2ZW50T2JqZWN0LFxufSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuaW1wb3J0IHsgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcblxuaW1wb3J0IHsgc25ha2VDYXNlLCBwYXNjYWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5pbXBvcnQgZWpzIGZyb20gXCJlanNcIjtcbmltcG9ydCBmcyBmcm9tIFwiZnNcIjtcbmltcG9ydCBwYXRoIGZyb20gXCJwYXRoXCI7XG5pbXBvcnQgY2hpbGRQcm9jZXNzIGZyb20gXCJjaGlsZF9wcm9jZXNzXCI7XG5pbXBvcnQgdXRpbCBmcm9tIFwidXRpbFwiO1xuXG5jb25zdCBleGVjQ21kID0gdXRpbC5wcm9taXNpZnkoY2hpbGRQcm9jZXNzLmV4ZWMpO1xuXG5pbnRlcmZhY2UgUnVzdFR5cGVBc1Byb3BPcHRpb25zIHtcbiAgcmVmZXJlbmNlPzogYm9vbGVhbjtcbiAgb3B0aW9uPzogYm9vbGVhbjtcbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXIge1xuICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogUnVzdEZvcm1hdHRlcltcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgc3RydWN0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICBtb2RlbE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjptb2RlbDo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX1gO1xuICB9XG5cbiAgY29tcG9uZW50TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1Db21wb25lbnRgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gY29tcG9uZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBjb21wb25lbnRDb25zdHJhaW50c05hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9Q29tcG9uZW50Q29uc3RyYWludHNgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYSBjb21wb25lbnQgY29uc3RyYWludHMgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eUVkaXRNZXRob2ROYW1lKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkge1xuICAgICAgcmV0dXJuIGBlZGl0XyR7dGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wTWV0aG9kKS5yZXBsYWNlKFxuICAgICAgICBcIl9lZGl0XCIsXG4gICAgICAgIFwiXCIsXG4gICAgICApfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlZGl0IG1ldGhvZCBuYW1lIG9uIGEgbm9uLWVudGl0eSBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5RXZlbnROYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eUV2ZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eUV2ZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHkgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eVByb3BlcnRpZXNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eVByb3BlcnRpZXNgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5UHJvcGVydGllcyBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgbW9kZWxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wTWV0aG9kKTtcbiAgfVxuXG4gIHR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSk7XG4gIH1cblxuICBlcnJvclR5cGUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjplcnJvcjo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lKX1FcnJvcmA7XG4gIH1cblxuICBoYXNDcmVhdGVNZXRob2QoKTogYm9vbGVhbiB7XG4gICAgdHJ5IHtcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGNhdGNoIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cblxuICBpc0NvbXBvbmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiY29tcG9uZW50T2JqZWN0XCI7XG4gIH1cblxuICBpc0VudGl0eU9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiZW50aXR5T2JqZWN0XCI7XG4gIH1cblxuICBpc0VudGl0eUV2ZW50T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJlbnRpdHlFdmVudE9iamVjdFwiO1xuICB9XG5cbiAgaXNFbnRpdHlBY3Rpb25NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiBwcm9wTWV0aG9kLmtpbmQoKSA9PSBcImFjdGlvblwiICYmIHRoaXMuaXNFbnRpdHlPYmplY3QoKTtcbiAgfVxuXG4gIGlzRW50aXR5RWRpdE1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHRoaXMuaXNFbnRpdHlBY3Rpb25NZXRob2QocHJvcE1ldGhvZCkgJiYgcHJvcE1ldGhvZC5uYW1lLmVuZHNXaXRoKFwiRWRpdFwiKVxuICAgICk7XG4gIH1cblxuICBpbXBsTGlzdFJlcXVlc3RUeXBlKHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9KTogc3RyaW5nIHtcbiAgICBjb25zdCBsaXN0ID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIFwibGlzdFwiLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZDtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AobGlzdC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxMaXN0UmVwbHlUeXBlKHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9KTogc3RyaW5nIHtcbiAgICBjb25zdCBsaXN0ID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIFwibGlzdFwiLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZDtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AobGlzdC5yZXBseSwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVJlcXVlc3RUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVxdWVzdCwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVJlcGx5VHlwZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKFxuICAgICAgdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZCwge1xuICAgICAgICBvcHRpb246IGZhbHNlLFxuICAgICAgICByZWZlcmVuY2U6IGZhbHNlLFxuICAgICAgfSksXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5QWN0aW9uKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUFjdGlvbi5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlFZGl0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUVkaXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUNvbW1vbkNyZWF0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5Q3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUdldChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VHZXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTGlzdChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VMaXN0LnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUNvbXBvbmVudFBpY2socHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tcG9uZW50UGljay5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDdXN0b21NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ3VzdG9tTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUF1dGgocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgaWYgKHByb3BNZXRob2Quc2tpcEF1dGgpIHtcbiAgICAgIHJldHVybiBgLy8gQXV0aGVudGljYXRpb24gaXMgc2tpcHBlZCBvbiBcXGAke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgICBwcm9wTWV0aG9kLFxuICAgICAgKX1cXGBcXG5gO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gdGhpcy5pbXBsU2VydmljZUF1dGhDYWxsKHByb3BNZXRob2QpO1xuICAgIH1cbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgbGV0IHByZWx1ZGUgPSBcInNpX2FjY291bnQ6OmF1dGhvcml6ZVwiO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSA9PSBcImFjY291bnRcIikge1xuICAgICAgcHJlbHVkZSA9IFwiY3JhdGU6OmF1dGhvcml6ZVwiO1xuICAgIH1cbiAgICByZXR1cm4gYCR7cHJlbHVkZX06OmF1dGhueigmc2VsZi5kYiwgJnJlcXVlc3QsIFwiJHt0aGlzLmltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICAgIHByb3BNZXRob2QsXG4gICAgKX1cIikuYXdhaXQ/O2A7XG4gIH1cblxuICBzZXJ2aWNlTWV0aG9kcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBjb25zdCBwcm9wTWV0aG9kcyA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMuc29ydCgoYSwgYikgPT5cbiAgICAgIGEubmFtZSA+IGIubmFtZSA/IDEgOiAtMSxcbiAgICApO1xuICAgIGZvciAoY29uc3QgcHJvcE1ldGhvZCBvZiBwcm9wTWV0aG9kcykge1xuICAgICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgICAge1xuICAgICAgICAgIGZtdDogdGhpcyxcbiAgICAgICAgICBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kLFxuICAgICAgICB9LFxuICAgICAgICB7XG4gICAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgICB9LFxuICAgICAgKTtcbiAgICAgIHJlc3VsdHMucHVzaChvdXRwdXQpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgcnVzdEZpZWxkTmFtZUZvclByb3AocHJvcDogUHJvcHMpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgfVxuXG4gIHJ1c3RUeXBlRm9yUHJvcChcbiAgICBwcm9wOiBQcm9wcyxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZWZlcmVuY2UgPSByZW5kZXJPcHRpb25zLnJlZmVyZW5jZSB8fCBmYWxzZTtcbiAgICBsZXQgb3B0aW9uID0gdHJ1ZTtcbiAgICBpZiAocmVuZGVyT3B0aW9ucy5vcHRpb24gPT09IGZhbHNlKSB7XG4gICAgICBvcHRpb24gPSBmYWxzZTtcbiAgICB9XG5cbiAgICBsZXQgdHlwZU5hbWU6IHN0cmluZztcblxuICAgIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTnVtYmVyKSB7XG4gICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTMyXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1MzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50NjRcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTY0XCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1NjRcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BCb29sIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgY29uc3QgcmVhbFByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgaWYgKHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBjb25zdCBwcm9wT3duZXIgPSBwcm9wLmxvb2t1cE9iamVjdCgpO1xuICAgICAgICBsZXQgcGF0aE5hbWU6IHN0cmluZztcbiAgICAgICAgaWYgKFxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSAmJlxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSA9PSB0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZVxuICAgICAgICApIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH0gZWxzZSBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBgc2lfJHtwcm9wT3duZXIuc2VydmljZU5hbWV9Ojpwcm90b2J1ZmA7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBcImNyYXRlOjpwcm90b2J1ZlwiO1xuICAgICAgICB9XG4gICAgICAgIHR5cGVOYW1lID0gYCR7cGF0aE5hbWV9Ojoke3Bhc2NhbENhc2UocmVhbFByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICAgIHJlYWxQcm9wLm5hbWUsXG4gICAgICAgICl9YDtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChyZWFsUHJvcCwgcmVuZGVyT3B0aW9ucyk7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1hcCkge1xuICAgICAgdHlwZU5hbWUgPSBgc3RkOjpjb2xsZWN0aW9uczo6SGFzaE1hcDxTdHJpbmcsIFN0cmluZz5gO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFRleHQgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BTZWxlY3RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gXCJTdHJpbmdcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgYENhbm5vdCBnZW5lcmF0ZSB0eXBlIGZvciAke3Byb3AubmFtZX0ga2luZCAke3Byb3Aua2luZCgpfSAtIEJ1ZyFgO1xuICAgIH1cbiAgICBpZiAocmVmZXJlbmNlKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgIGlmICh0eXBlTmFtZSA9PSBcIlN0cmluZ1wiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCImc3RyXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgJiR7dHlwZU5hbWV9YDtcbiAgICAgIH1cbiAgICB9XG4gICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgdHlwZU5hbWUgPSBgVmVjPCR7dHlwZU5hbWV9PmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGlmIChvcHRpb24pIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICAgIHR5cGVOYW1lID0gYE9wdGlvbjwke3R5cGVOYW1lfT5gO1xuICAgICAgfVxuICAgIH1cbiAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICByZXR1cm4gdHlwZU5hbWU7XG4gIH1cblxuICBpbXBsQ3JlYXRlTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKGAke3NuYWtlQ2FzZShwcm9wLm5hbWUpfTogJHt0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wKX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsQ3JlYXRlUGFzc05ld0FyZ3MoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICByZXN1bHQucHVzaChzbmFrZUNhc2UocHJvcC5uYW1lKSk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2RMaXN0UmVzdWx0VG9SZXBseSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGxpc3RNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwibGlzdFwiKTtcbiAgICBpZiAobGlzdE1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBsaXN0TWV0aG9kLnJlcGx5LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGxldCBsaXN0UmVwbHlWYWx1ZSA9IGBTb21lKG91dHB1dC4ke2ZpZWxkTmFtZX0pYDtcbiAgICAgICAgaWYgKGZpZWxkTmFtZSA9PSBcIm5leHRfcGFnZV90b2tlblwiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBcIlNvbWUob3V0cHV0LnBhZ2VfdG9rZW4pXCI7XG4gICAgICAgIH0gZWxzZSBpZiAoZmllbGROYW1lID09IFwiaXRlbXNcIikge1xuICAgICAgICAgIGxpc3RSZXBseVZhbHVlID0gYG91dHB1dC4ke2ZpZWxkTmFtZX1gO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKGAke2ZpZWxkTmFtZX06ICR7bGlzdFJlcGx5VmFsdWV9YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2RDcmVhdGVEZXN0cnVjdHVyZSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IGZpZWxkTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICByZXN1bHQucHVzaChgbGV0ICR7ZmllbGROYW1lfSA9IGlubmVyLiR7ZmllbGROYW1lfTtgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgbmF0dXJhbEtleSgpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCkge1xuICAgICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5uYXR1cmFsS2V5KTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwibmFtZVwiO1xuICAgIH1cbiAgfVxuXG4gIGlzTWlncmF0ZWFibGUoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIiAmJiB0aGlzLnN5c3RlbU9iamVjdC5taWdyYXRlYWJsZVxuICAgICk7XG4gIH1cblxuICBpc1N0b3JhYmxlKCk6IGJvb2xlYW4ge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCkge1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cblxuICBpbXBsQ3JlYXRlU2V0UHJvcGVydGllcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IHZhcmlhYmxlTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BQYXNzd29yZCkge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSBTb21lKHNpX2RhdGE6OnBhc3N3b3JkOjplbmNyeXB0X3Bhc3N3b3JkKCR7dmFyaWFibGVOYW1lfSk/KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSAke3ZhcmlhYmxlTmFtZX07YCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaW1wbENyZWF0ZUFkZFRvVGVuYW5jeSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiYmlsbGluZ0FjY291bnRcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblwiXG4gICAgKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uU2VydmljZVwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiY29tcG9uZW50T2JqZWN0XCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX3NlcnZpY2VfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX3NlcnZpY2VfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25TZXJ2aWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25fc2VydmljZV9pZCk7YCk7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwidXNlclwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImdyb3VwXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwib3JnYW5pemF0aW9uXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25JbnN0YW5jZVwiXG4gICAgKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ3b3Jrc3BhY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgb3JnYW5pemF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5vcmdhbml6YXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMub3JnYW5pemF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMob3JnYW5pemF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgb3JnYW5pemF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5vcmdhbml6YXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMub3JnYW5pemF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMob3JnYW5pemF0aW9uX2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgd29ya3NwYWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS53b3Jrc3BhY2VfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMud29ya3NwYWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMod29ya3NwYWNlX2lkKTtgKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVWYWxpZGF0ZUZ1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5hdHRycykge1xuICAgICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbiAgICAgICAgY29uc3QgcHJvcE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5sZW4oKSA9PSAwIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0uaXNfbm9uZSgpIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgIHRvcFByb3A6IFByb3BQcmVsdWRlLlByb3BPYmplY3QsXG4gICAgcHJlZml4OiBzdHJpbmcsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFsnXCJzaVN0b3JhYmxlLm5hdHVyYWxLZXlcIiddO1xuICAgIGZvciAobGV0IHByb3Agb2YgdG9wUHJvcC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4pIHtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBwcm9wLm5hbWUpKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBgJHtwcmVmaXh9LiR7cHJvcC5uYW1lfWApLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3ByZWZpeH0uJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiLCBcIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnJvb3RQcm9wLFxuICAgICAgXCJcIixcbiAgICApO1xuICAgIHJldHVybiBgdmVjIVske3Jlc3VsdHN9XVxcbmA7XG4gIH1cblxuICBzdG9yYWJsZVJlZmVyZW50aWFsRmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCBmZXRjaFByb3BzID0gW107XG4gICAgY29uc3QgcmVmZXJlbmNlVmVjID0gW107XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCkge1xuICAgICAgbGV0IHNpUHJvcGVydGllcyA9IHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5nZXRFbnRyeShcInNpUHJvcGVydGllc1wiKTtcbiAgICAgIGlmIChzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBzaVByb3BlcnRpZXMgPSBzaVByb3BlcnRpZXMubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAoIShzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSkge1xuICAgICAgICB0aHJvdyBcIkNhbm5vdCBnZXQgcHJvcGVydGllcyBvZiBhIG5vbiBvYmplY3QgaW4gcmVmIGNoZWNrXCI7XG4gICAgICB9XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2Ygc2lQcm9wZXJ0aWVzLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgaWYgKHByb3AucmVmZXJlbmNlKSB7XG4gICAgICAgICAgY29uc3QgaXRlbU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNNYW55KFwiJHtpdGVtTmFtZX1cIiwgJHtpdGVtTmFtZX0pYCxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIGZldGNoUHJvcHMucHVzaChgbGV0ICR7aXRlbU5hbWV9ID0gbWF0Y2ggJnNlbGYuc2lfcHJvcGVydGllcyB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICBTb21lKGNpcCkgPT4gY2lwXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuJHtpdGVtTmFtZX1cbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5hc19yZWYoKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLm1hcChTdHJpbmc6OmFzX3JlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC51bndyYXBfb3IoXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICBOb25lID0+IFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiLFxuICAgICAgICAgICAgICAgICAgICAgICAgIH07YCk7XG4gICAgICAgICAgICByZWZlcmVuY2VWZWMucHVzaChcbiAgICAgICAgICAgICAgYHNpX2RhdGE6OlJlZmVyZW5jZTo6SGFzT25lKFwiJHtpdGVtTmFtZX1cIiwgJHtpdGVtTmFtZX0pYCxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBCYXNlT2JqZWN0KSB7XG4gICAgfVxuXG4gICAgaWYgKGZldGNoUHJvcHMubGVuZ3RoICYmIHJlZmVyZW5jZVZlYy5sZW5ndGgpIHtcbiAgICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICAgIHJlc3VsdHMucHVzaChmZXRjaFByb3BzLmpvaW4oXCJcXG5cIikpO1xuICAgICAgcmVzdWx0cy5wdXNoKGB2ZWMhWyR7cmVmZXJlbmNlVmVjLmpvaW4oXCIsXCIpfV1gKTtcbiAgICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlZlYzo6bmV3KClcIjtcbiAgICB9XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXJTZXJ2aWNlIHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0cyA9IHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSk7XG4gIH1cblxuICBzeXN0ZW1PYmplY3RzQXNGb3JtYXR0ZXJzKCk6IFJ1c3RGb3JtYXR0ZXJbXSB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0c1xuICAgICAgLnNvcnQoKGEsIGIpID0+IChhLnR5cGVOYW1lID4gYi50eXBlTmFtZSA/IDEgOiAtMSkpXG4gICAgICAubWFwKG8gPT4gbmV3IFJ1c3RGb3JtYXR0ZXIobykpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VTdHJ1Y3RCb2R5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wiZGI6IHNpX2RhdGE6OkRiLFwiXTtcbiAgICBpZiAodGhpcy5oYXNFbnRpdGllcygpKSB7XG4gICAgICByZXN1bHQucHVzaChcImFnZW50OiBzaV9jZWE6OkFnZW50Q2xpZW50LFwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VOZXdDb25zdHJ1Y3RvckFyZ3MoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5oYXNFbnRpdGllcygpKSB7XG4gICAgICByZXR1cm4gXCJkYjogc2lfZGF0YTo6RGIsIGFnZW50OiBzaV9jZWE6OkFnZW50Q2xpZW50XCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcImRiOiBzaV9kYXRhOjpEYlwiO1xuICAgIH1cbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Q29uc3RydWN0b3JSZXR1cm4oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXCJkYlwiXTtcbiAgICBpZiAodGhpcy5oYXNFbnRpdGllcygpKSB7XG4gICAgICByZXN1bHQucHVzaChcImFnZW50XCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VUcmFpdE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtzbmFrZUNhc2UoXG4gICAgICB0aGlzLnNlcnZpY2VOYW1lLFxuICAgICl9X3NlcnZlcjo6JHtwYXNjYWxDYXNlKHRoaXMuc2VydmljZU5hbWUpfWA7XG4gIH1cblxuICBpbXBsU2VydmVyTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHt0aGlzLmltcGxTZXJ2aWNlVHJhaXROYW1lKCl9U2VydmVyYDtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWlncmF0ZSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqIG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgaWYgKHN5c3RlbU9iai5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIgJiYgc3lzdGVtT2JqLm1pZ3JhdGVhYmxlID09IHRydWUpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgc3lzdGVtT2JqLnR5cGVOYW1lLFxuICAgICAgICAgICl9OjptaWdyYXRlKCZzZWxmLmRiKS5hd2FpdD87YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaGFzRW50aXRpZXMoKTogYm9vbGVhbiB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0cy5maW5kKHMgPT4gcy5raW5kKCkgPT0gXCJlbnRpdHlPYmplY3RcIikpIHtcbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaGFzTWlncmF0YWJsZXMoKTogYm9vbGVhbiB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3RzLmZpbmQoXG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgcyA9PiBzLmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIiAmJiBzLm1pZ3JhdGVhYmxlID09IHRydWUsXG4gICAgICApXG4gICAgKSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXG4gICAgICBcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsXG4gICAgICBcIi8vIE5vIHRvdWNoeSFcIixcbiAgICAgIFwiXCIsXG4gICAgICBcInB1YiBtb2QgbW9kZWw7XCIsXG4gICAgICBcInB1YiBtb2Qgc2VydmljZTtcIixcbiAgICBdO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vbW9kZWwvbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgZm9yIChjb25zdCBzeXN0ZW1PYmplY3Qgb2YgcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApKSB7XG4gICAgICBpZiAoc3lzdGVtT2JqZWN0LmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIikge1xuICAgICAgICByZXN1bHRzLnB1c2goYHB1YiBtb2QgJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX07YCk7XG4gICAgICB9XG4gICAgfVxuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZGVsL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5TZXJ2aWNlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2UucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyU2VydmljZSh0aGlzLnNlcnZpY2VOYW1lKSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShgZ2VuL3NlcnZpY2UucnNgLCBvdXRwdXQpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2RlbChzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvbW9kZWwucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyKHN5c3RlbU9iamVjdCksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXG4gICAgICBgZ2VuL21vZGVsLyR7c25ha2VDYXNlKHN5c3RlbU9iamVjdC50eXBlTmFtZSl9LnJzYCxcbiAgICAgIG91dHB1dCxcbiAgICApO1xuICB9XG5cbiAgYXN5bmMgbWFrZVBhdGgocGF0aFBhcnQ6IHN0cmluZyk6IFByb21pc2U8c3RyaW5nPiB7XG4gICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXCIuLlwiLCBgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWAsIFwic3JjXCIsIHBhdGhQYXJ0KTtcbiAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbiAgICBhd2FpdCBmcy5wcm9taXNlcy5ta2RpcihwYXRoLnJlc29sdmUocGF0aE5hbWUpLCB7IHJlY3Vyc2l2ZTogdHJ1ZSB9KTtcbiAgICByZXR1cm4gYWJzb2x1dGVQYXRoTmFtZTtcbiAgfVxuXG4gIGFzeW5jIGZvcm1hdENvZGUoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgYXdhaXQgZXhlY0NtZChgY2FyZ28gZm10IC1wIHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gKTtcbiAgfVxuXG4gIGFzeW5jIHdyaXRlQ29kZShmaWxlbmFtZTogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBwYXRobmFtZSA9IHBhdGguZGlybmFtZShmaWxlbmFtZSk7XG4gICAgY29uc3QgYmFzZW5hbWUgPSBwYXRoLmJhc2VuYW1lKGZpbGVuYW1lKTtcbiAgICBjb25zdCBjcmVhdGVkUGF0aCA9IGF3YWl0IHRoaXMubWFrZVBhdGgocGF0aG5hbWUpO1xuICAgIGNvbnN0IGNvZGVGaWxlbmFtZSA9IHBhdGguam9pbihjcmVhdGVkUGF0aCwgYmFzZW5hbWUpO1xuICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGUpO1xuICB9XG59XG5cbi8vIGV4cG9ydCBjbGFzcyBDb2RlZ2VuUnVzdCB7XG4vLyAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG4vLyAgIGZvcm1hdHRlcjogUnVzdEZvcm1hdHRlcjtcbi8vXG4vLyAgIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpIHtcbi8vICAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbi8vICAgICB0aGlzLmZvcm1hdHRlciA9IG5ldyBSdXN0Rm9ybWF0dGVyKHN5c3RlbU9iamVjdCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIHdyaXRlQ29kZShwYXJ0OiBzdHJpbmcsIGNvZGU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IGNyZWF0ZWRQYXRoID0gYXdhaXQgdGhpcy5tYWtlUGF0aCgpO1xuLy8gICAgIGNvbnN0IGNvZGVGaWxlbmFtZSA9IHBhdGguam9pbihjcmVhdGVkUGF0aCwgYCR7c25ha2VDYXNlKHBhcnQpfS5yc2ApO1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGUpO1xuLy8gICAgIGF3YWl0IGV4ZWNDbWQoYHJ1c3RmbXQgJHtjb2RlRmlsZW5hbWV9YCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIG1ha2VQYXRoKCk6IFByb21pc2U8c3RyaW5nPiB7XG4vLyAgICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5zaVBhdGhOYW1lLFxuLy8gICAgICAgXCJzcmNcIixcbi8vICAgICAgIFwiZ2VuXCIsXG4vLyAgICAgICBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpLFxuLy8gICAgICk7XG4vLyAgICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgICAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIocGF0aC5yZXNvbHZlKHBhdGhOYW1lKSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4vLyAgICAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIGdlbmVyYXRlQ29tcG9uZW50SW1wbHMoKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbi8vICAgICAgIFwiPCUtIGluY2x1ZGUoJ3J1c3QvY29tcG9uZW50LnJzLmVqcycsIHsgY29tcG9uZW50OiBjb21wb25lbnQgfSkgJT5cIixcbi8vICAgICAgIHtcbi8vICAgICAgICAgc3lzdGVtT2JqZWN0OiB0aGlzLnN5c3RlbU9iamVjdCxcbi8vICAgICAgICAgZm10OiB0aGlzLmZvcm1hdHRlcixcbi8vICAgICAgIH0sXG4vLyAgICAgICB7XG4vLyAgICAgICAgIGZpbGVuYW1lOiBfX2ZpbGVuYW1lLFxuLy8gICAgICAgfSxcbi8vICAgICApO1xuLy8gICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiY29tcG9uZW50XCIsIG91dHB1dCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIGdlbmVyYXRlQ29tcG9uZW50TW9kKCk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IG1vZHMgPSBbXCJjb21wb25lbnRcIl07XG4vLyAgICAgY29uc3QgbGluZXMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIFRvdWNoeSFcXG5cIl07XG4vLyAgICAgZm9yIChjb25zdCBtb2Qgb2YgbW9kcykge1xuLy8gICAgICAgbGluZXMucHVzaChgcHViIG1vZCAke21vZH07YCk7XG4vLyAgICAgfVxuLy8gICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwibW9kXCIsIGxpbmVzLmpvaW4oXCJcXG5cIikpO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXIge1xuLy8gICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuLy9cbi8vICAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4vLyAgICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudFR5cGVOYW1lKCk6IHN0cmluZyB7XG4vLyAgICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSk7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudE9yZGVyQnlGaWVsZHMoKTogc3RyaW5nIHtcbi8vICAgICBjb25zdCBvcmRlckJ5RmllbGRzID0gW107XG4vLyAgICAgY29uc3QgY29tcG9uZW50T2JqZWN0ID0gdGhpcy5jb21wb25lbnQuYXNDb21wb25lbnQoKTtcbi8vICAgICBmb3IgKGNvbnN0IHAgb2YgY29tcG9uZW50T2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgIGlmIChwLmhpZGRlbikge1xuLy8gICAgICAgICBjb250aW51ZTtcbi8vICAgICAgIH1cbi8vICAgICAgIGlmIChwLm5hbWUgPT0gXCJzdG9yYWJsZVwiKSB7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaCgnXCJzdG9yYWJsZS5uYXR1cmFsS2V5XCInKTtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKCdcInN0b3JhYmxlLnR5cGVOYW1lXCInKTtcbi8vICAgICAgIH0gZWxzZSBpZiAocC5uYW1lID09IFwic2lQcm9wZXJ0aWVzXCIpIHtcbi8vICAgICAgICAgY29udGludWU7XG4vLyAgICAgICB9IGVsc2UgaWYgKHAubmFtZSA9PSBcImNvbnN0cmFpbnRzXCIgJiYgcC5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuLy8gICAgICAgICAvLyBAdHMtaWdub3JlIHRydXN0IHVzIC0gd2UgY2hlY2tlZFxuLy8gICAgICAgICBmb3IgKGNvbnN0IHBjIG9mIHAucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgICAgIGlmIChwYy5raW5kKCkgIT0gXCJvYmplY3RcIikge1xuLy8gICAgICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKGBcImNvbnN0cmFpbnRzLiR7cGMubmFtZX1cImApO1xuLy8gICAgICAgICAgIH1cbi8vICAgICAgICAgfVxuLy8gICAgICAgfSBlbHNlIHtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKGBcIiR7cC5uYW1lfVwiYCk7XG4vLyAgICAgICB9XG4vLyAgICAgfVxuLy8gICAgIHJldHVybiBgdmVjIVske29yZGVyQnlGaWVsZHMuam9pbihcIixcIil9XVxcbmA7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudEltcG9ydHMoKTogc3RyaW5nIHtcbi8vICAgICBjb25zdCByZXN1bHQgPSBbXTtcbi8vICAgICByZXN1bHQucHVzaChcbi8vICAgICAgIGBwdWIgdXNlIGNyYXRlOjpwcm90b2J1Zjo6JHtzbmFrZUNhc2UodGhpcy5jb21wb25lbnQudHlwZU5hbWUpfTo6e2AsXG4vLyAgICAgICBgICBDb25zdHJhaW50cyxgLFxuLy8gICAgICAgYCAgTGlzdENvbXBvbmVudHNSZXBseSxgLFxuLy8gICAgICAgYCAgTGlzdENvbXBvbmVudHNSZXF1ZXN0LGAsXG4vLyAgICAgICBgICBQaWNrQ29tcG9uZW50UmVxdWVzdCxgLFxuLy8gICAgICAgYCAgQ29tcG9uZW50LGAsXG4vLyAgICAgICBgfTtgLFxuLy8gICAgICk7XG4vLyAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRWYWxpZGF0aW9uKCk6IHN0cmluZyB7XG4vLyAgICAgcmV0dXJuIHRoaXMuZ2VuVmFsaWRhdGlvbih0aGlzLmNvbXBvbmVudC5hc0NvbXBvbmVudCgpKTtcbi8vICAgfVxuLy9cbi8vICAgZ2VuVmFsaWRhdGlvbihwcm9wT2JqZWN0OiBQcm9wT2JqZWN0KTogc3RyaW5nIHtcbi8vICAgICBjb25zdCByZXN1bHQgPSBbXTtcbi8vICAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuLy8gICAgICAgICBjb25zdCBwcm9wTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuLy8gICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5pc19ub25lKCkge1xuLy8gICAgICAgICAgIHJldHVybiBFcnIoRGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4vLyAgICAgICAgIH1gKTtcbi8vICAgICAgIH1cbi8vICAgICB9XG4vLyAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdlbmVyYXRlR2VuTW9kKHdyaXR0ZW5Db21wb25lbnRzOiB7XG4vLyAgIFtrZXk6IHN0cmluZ106IHN0cmluZ1tdO1xuLy8gfSk6IFByb21pc2U8dm9pZD4ge1xuLy8gICBmb3IgKGNvbnN0IGNvbXBvbmVudCBpbiB3cml0dGVuQ29tcG9uZW50cykge1xuLy8gICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgICAgX19kaXJuYW1lLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgY29tcG9uZW50LFxuLy8gICAgICAgXCJzcmNcIixcbi8vICAgICAgIFwiZ2VuXCIsXG4vLyAgICAgKTtcbi8vICAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbi8vICAgICBjb25zdCBjb2RlID0gW1xuLy8gICAgICAgXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLFxuLy8gICAgICAgXCIvLyBObyB0b3VjaHkhXCIsXG4vLyAgICAgICBcIlwiLFxuLy8gICAgICAgXCJwdWIgbW9kIG1vZGVsO1wiLFxuLy8gICAgIF07XG4vL1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShcbi8vICAgICAgIHBhdGguam9pbihhYnNvbHV0ZVBhdGhOYW1lLCBcIm1vZC5yc1wiKSxcbi8vICAgICAgIGNvZGUuam9pbihcIlxcblwiKSxcbi8vICAgICApO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdlbmVyYXRlR2VuTW9kTW9kZWwoc2VydmljZU5hbWU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuLy8gICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgc2VydmljZU5hbWUsXG4vLyAgICAgXCJzcmNcIixcbi8vICAgICBcImdlblwiLFxuLy8gICAgIFwibW9kZWxcIixcbi8vICAgKTtcbi8vICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgIGNvbnN0IGNvZGUgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcXG5cIl07XG4vLyAgIGZvciAoY29uc3QgdHlwZU5hbWUgb2Ygd3JpdHRlbkNvbXBvbmVudHNbY29tcG9uZW50XSkge1xuLy8gICAgIGNvZGUucHVzaChgcHViIG1vZCAke3NuYWtlQ2FzZSh0eXBlTmFtZSl9O2ApO1xuLy8gICB9XG4vL1xuLy8gICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoXG4vLyAgICAgcGF0aC5qb2luKGFic29sdXRlUGF0aE5hbWUsIFwibW9kLnJzXCIpLFxuLy8gICAgIGNvZGUuam9pbihcIlxcblwiKSxcbi8vICAgKTtcbi8vIH1cbiJdfQ==