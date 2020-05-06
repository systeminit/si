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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwiQ29tcG9uZW50T2JqZWN0IiwiRW50aXR5T2JqZWN0IiwiRW50aXR5RXZlbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJwcm9wTWV0aG9kIiwicnVzdEZpZWxkTmFtZUZvclByb3AiLCJyZXBsYWNlIiwic2VydmljZU5hbWUiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJraW5kIiwiaXNFbnRpdHlPYmplY3QiLCJpc0VudGl0eUFjdGlvbk1ldGhvZCIsIm5hbWUiLCJlbmRzV2l0aCIsInJlbmRlck9wdGlvbnMiLCJsaXN0IiwicnVzdFR5cGVGb3JQcm9wIiwicmVxdWVzdCIsInJlcGx5Iiwib3B0aW9uIiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicmVzdWx0cyIsInByb3BNZXRob2RzIiwiYXR0cnMiLCJzb3J0IiwiYSIsImIiLCJvdXRwdXQiLCJwdXNoIiwiam9pbiIsInByb3AiLCJQcm9wUHJlbHVkZSIsIlByb3BBY3Rpb24iLCJQcm9wTWV0aG9kIiwicGFyZW50TmFtZSIsIlByb3BOdW1iZXIiLCJudW1iZXJLaW5kIiwiUHJvcEJvb2wiLCJQcm9wT2JqZWN0IiwiUHJvcExpbmsiLCJyZWFsUHJvcCIsImxvb2t1cE15c2VsZiIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcE1hcCIsIlByb3BUZXh0IiwiUHJvcENvZGUiLCJQcm9wU2VsZWN0IiwicmVwZWF0ZWQiLCJyZXN1bHQiLCJjcmVhdGVNZXRob2QiLCJwcm9wZXJ0aWVzIiwibGlzdE1ldGhvZCIsImZpZWxkTmFtZSIsImxpc3RSZXBseVZhbHVlIiwiU3lzdGVtT2JqZWN0IiwibmF0dXJhbEtleSIsIm1pZ3JhdGVhYmxlIiwidmFyaWFibGVOYW1lIiwiUHJvcFBhc3N3b3JkIiwiZmllbGRzIiwicmVxdWlyZWQiLCJwcm9wTmFtZSIsInRvcFByb3AiLCJwcmVmaXgiLCJoaWRkZW4iLCJzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AiLCJyb290UHJvcCIsImZldGNoUHJvcHMiLCJyZWZlcmVuY2VWZWMiLCJzaVByb3BlcnRpZXMiLCJpdGVtTmFtZSIsIkJhc2VPYmplY3QiLCJsZW5ndGgiLCJSdXN0Rm9ybWF0dGVyU2VydmljZSIsInN5c3RlbU9iamVjdHMiLCJyZWdpc3RyeSIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsIm1hcCIsIm8iLCJoYXNFbnRpdGllcyIsInN5c3RlbU9iaiIsImZpbmQiLCJzIiwiQ29kZWdlblJ1c3QiLCJ3cml0ZUNvZGUiLCJwYXRoUGFydCIsInBhdGgiLCJhYnNvbHV0ZVBhdGhOYW1lIiwicmVzb2x2ZSIsImZzIiwicHJvbWlzZXMiLCJta2RpciIsInJlY3Vyc2l2ZSIsImNvZGUiLCJwYXRobmFtZSIsImRpcm5hbWUiLCJiYXNlbmFtZSIsIm1ha2VQYXRoIiwiY3JlYXRlZFBhdGgiLCJjb2RlRmlsZW5hbWUiLCJ3cml0ZUZpbGUiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFBOztBQVFBOztBQUNBOztBQUdBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOzs7Ozs7OztBQUVBLElBQU1BLE9BQU8sR0FBR0MsaUJBQUtDLFNBQUwsQ0FBZUMsMEJBQWFDLElBQTVCLENBQWhCOztJQU9hQyxhO0FBR1gseUJBQVlDLFlBQVosRUFBeUQ7QUFBQTtBQUFBO0FBQ3ZELFNBQUtBLFlBQUwsR0FBb0JBLFlBQXBCO0FBQ0Q7Ozs7aUNBRW9CO0FBQ25CLHdDQUEyQiw0QkFBVyxLQUFLQSxZQUFMLENBQWtCQyxRQUE3QixDQUEzQjtB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QTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0EiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQge1xuICBPYmplY3RUeXBlcyxcbiAgQmFzZU9iamVjdCxcbiAgU3lzdGVtT2JqZWN0LFxuICBDb21wb25lbnRPYmplY3QsXG4gIEVudGl0eU9iamVjdCxcbiAgRW50aXR5RXZlbnRPYmplY3QsXG59IGZyb20gXCIuLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCAqIGFzIFByb3BQcmVsdWRlIGZyb20gXCIuLi9jb21wb25lbnRzL3ByZWx1ZGVcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5pbXBvcnQgeyBQcm9wcyB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuXG5pbXBvcnQgeyBzbmFrZUNhc2UsIHBhc2NhbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBlanMgZnJvbSBcImVqc1wiO1xuaW1wb3J0IGZzIGZyb20gXCJmc1wiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG5cbmNvbnN0IGV4ZWNDbWQgPSB1dGlsLnByb21pc2lmeShjaGlsZFByb2Nlc3MuZXhlYyk7XG5cbmludGVyZmFjZSBSdXN0VHlwZUFzUHJvcE9wdGlvbnMge1xuICByZWZlcmVuY2U/OiBib29sZWFuO1xuICBvcHRpb24/OiBib29sZWFuO1xufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBzdHJ1Y3ROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIG1vZGVsTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6Om1vZGVsOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICBjb21wb25lbnROYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUNvbXBvbmVudGA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBjb21wb25lbnQgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGNvbXBvbmVudENvbnN0cmFpbnRzTmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1Db21wb25lbnRDb25zdHJhaW50c2A7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhIGNvbXBvbmVudCBjb25zdHJhaW50cyBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSB7XG4gICAgICByZXR1cm4gYGVkaXRfJHt0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BNZXRob2QpLnJlcGxhY2UoXG4gICAgICAgIFwiX2VkaXRcIixcbiAgICAgICAgXCJcIixcbiAgICAgICl9YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVkaXQgbWV0aG9kIG5hbWUgb24gYSBub24tZW50aXR5IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlFdmVudE5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9RW50aXR5RXZlbnRgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5RXZlbnQgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eU5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9RW50aXR5YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eSBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5UHJvcGVydGllc05hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9RW50aXR5UHJvcGVydGllc2A7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHlQcm9wZXJ0aWVzIG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBtb2RlbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BNZXRob2QpO1xuICB9XG5cbiAgdHlwZU5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKTtcbiAgfVxuXG4gIGVycm9yVHlwZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OmVycm9yOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWUpfUVycm9yYDtcbiAgfVxuXG4gIGhhc0NyZWF0ZU1ldGhvZCgpOiBib29sZWFuIHtcbiAgICB0cnkge1xuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH0gY2F0Y2gge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGlzQ29tcG9uZW50T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJjb21wb25lbnRPYmplY3RcIjtcbiAgfVxuXG4gIGlzRW50aXR5T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJlbnRpdHlPYmplY3RcIjtcbiAgfVxuXG4gIGlzRW50aXR5RXZlbnRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImVudGl0eUV2ZW50T2JqZWN0XCI7XG4gIH1cblxuICBpc0VudGl0eUFjdGlvbk1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHByb3BNZXRob2Qua2luZCgpID09IFwiYWN0aW9uXCIgJiYgdGhpcy5pc0VudGl0eU9iamVjdCgpO1xuICB9XG5cbiAgaXNFbnRpdHlFZGl0TWV0aG9kKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgdGhpcy5pc0VudGl0eUFjdGlvbk1ldGhvZChwcm9wTWV0aG9kKSAmJiBwcm9wTWV0aG9kLm5hbWUuZW5kc1dpdGgoXCJFZGl0XCIpXG4gICAgKTtcbiAgfVxuXG4gIGltcGxMaXN0UmVxdWVzdFR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbExpc3RSZXBseVR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVxdWVzdFR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVwbHlUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UoXG4gICAgICB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLCB7XG4gICAgICAgIG9wdGlvbjogZmFsc2UsXG4gICAgICAgIHJlZmVyZW5jZTogZmFsc2UsXG4gICAgICB9KSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlBY3Rpb24ocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5QWN0aW9uLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUVkaXQocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5RWRpdC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDb21tb25DcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUNyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlR2V0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUdldC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VMaXN0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUxpc3QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tcG9uZW50UGljayhwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDb21wb25lbnRQaWNrLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUN1c3RvbU1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDdXN0b21NZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcE1ldGhvZC5za2lwQXV0aCkge1xuICAgICAgcmV0dXJuIGAvLyBBdXRoZW50aWNhdGlvbiBpcyBza2lwcGVkIG9uIFxcYCR7dGhpcy5pbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgICAgIHByb3BNZXRob2QsXG4gICAgICApfVxcYFxcbmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiB0aGlzLmltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZCk7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoQ2FsbChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBsZXQgcHJlbHVkZSA9IFwic2lfYWNjb3VudDo6YXV0aG9yaXplXCI7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lID09IFwiYWNjb3VudFwiKSB7XG4gICAgICBwcmVsdWRlID0gXCJjcmF0ZTo6YXV0aG9yaXplXCI7XG4gICAgfVxuICAgIHJldHVybiBgJHtwcmVsdWRlfTo6YXV0aG56KCZzZWxmLmRiLCAmcmVxdWVzdCwgXCIke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgcHJvcE1ldGhvZCxcbiAgICApfVwiKS5hd2FpdD87YDtcbiAgfVxuXG4gIHNlcnZpY2VNZXRob2RzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGNvbnN0IHByb3BNZXRob2RzID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5hdHRycy5zb3J0KChhLCBiKSA9PlxuICAgICAgYS5uYW1lID4gYi5uYW1lID8gMSA6IC0xLFxuICAgICk7XG4gICAgZm9yIChjb25zdCBwcm9wTWV0aG9kIG9mIHByb3BNZXRob2RzKSB7XG4gICAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2VNZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgICB7XG4gICAgICAgICAgZm10OiB0aGlzLFxuICAgICAgICAgIHByb3BNZXRob2Q6IHByb3BNZXRob2QsXG4gICAgICAgIH0sXG4gICAgICAgIHtcbiAgICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICAgIH0sXG4gICAgICApO1xuICAgICAgcmVzdWx0cy5wdXNoKG91dHB1dCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBydXN0RmllbGROYW1lRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICB9XG5cbiAgcnVzdFR5cGVGb3JQcm9wKFxuICAgIHByb3A6IFByb3BzLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlZmVyZW5jZSA9IHJlbmRlck9wdGlvbnMucmVmZXJlbmNlIHx8IGZhbHNlO1xuICAgIGxldCBvcHRpb24gPSB0cnVlO1xuICAgIGlmIChyZW5kZXJPcHRpb25zLm9wdGlvbiA9PT0gZmFsc2UpIHtcbiAgICAgIG9wdGlvbiA9IGZhbHNlO1xuICAgIH1cblxuICAgIGxldCB0eXBlTmFtZTogc3RyaW5nO1xuXG4gICAgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24gfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kXG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpMzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpNjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInU2NFwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEJvb2wgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAocmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICAgIGxldCBwYXRoTmFtZTogc3RyaW5nO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lICYmXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lID09IHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lXG4gICAgICAgICkge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IGBzaV8ke3Byb3BPd25lci5zZXJ2aWNlTmFtZX06OnByb3RvYnVmYDtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH1cbiAgICAgICAgdHlwZU5hbWUgPSBgJHtwYXRoTmFtZX06OiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHJlYWxQcm9wLCByZW5kZXJPcHRpb25zKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWFwKSB7XG4gICAgICB0eXBlTmFtZSA9IGBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoTWFwPFN0cmluZywgU3RyaW5nPmA7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgQ2Fubm90IGdlbmVyYXRlIHR5cGUgZm9yICR7cHJvcC5uYW1lfSBraW5kICR7cHJvcC5raW5kKCl9IC0gQnVnIWA7XG4gICAgfVxuICAgIGlmIChyZWZlcmVuY2UpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgaWYgKHR5cGVOYW1lID09IFwiU3RyaW5nXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcIiZzdHJcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGAmJHt0eXBlTmFtZX1gO1xuICAgICAgfVxuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICB0eXBlTmFtZSA9IGBWZWM8JHt0eXBlTmFtZX0+YDtcbiAgICB9IGVsc2Uge1xuICAgICAgaWYgKG9wdGlvbikge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgT3B0aW9uPCR7dHlwZU5hbWV9PmA7XG4gICAgICB9XG4gICAgfVxuICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgIHJldHVybiB0eXBlTmFtZTtcbiAgfVxuXG4gIGltcGxDcmVhdGVOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goYCR7c25ha2VDYXNlKHByb3AubmFtZSl9OiAke3RoaXMucnVzdFR5cGVGb3JQcm9wKHByb3ApfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVQYXNzTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKHNuYWtlQ2FzZShwcm9wLm5hbWUpKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZExpc3RSZXN1bHRUb1JlcGx5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgbGlzdE1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJsaXN0XCIpO1xuICAgIGlmIChsaXN0TWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGxpc3RNZXRob2QucmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgbGV0IGxpc3RSZXBseVZhbHVlID0gYFNvbWUob3V0cHV0LiR7ZmllbGROYW1lfSlgO1xuICAgICAgICBpZiAoZmllbGROYW1lID09IFwibmV4dF9wYWdlX3Rva2VuXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IFwiU29tZShvdXRwdXQucGFnZV90b2tlbilcIjtcbiAgICAgICAgfSBlbHNlIGlmIChmaWVsZE5hbWUgPT0gXCJpdGVtc1wiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBgb3V0cHV0LiR7ZmllbGROYW1lfWA7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfTogJHtsaXN0UmVwbHlWYWx1ZX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZENyZWF0ZURlc3RydWN0dXJlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKGBsZXQgJHtmaWVsZE5hbWV9ID0gaW5uZXIuJHtmaWVsZE5hbWV9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBuYXR1cmFsS2V5KCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0Lm5hdHVyYWxLZXkpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJuYW1lXCI7XG4gICAgfVxuICB9XG5cbiAgaXNNaWdyYXRlYWJsZSgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiICYmIHRoaXMuc3lzdGVtT2JqZWN0Lm1pZ3JhdGVhYmxlXG4gICAgKTtcbiAgfVxuXG4gIGlzU3RvcmFibGUoKTogYm9vbGVhbiB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGltcGxDcmVhdGVTZXRQcm9wZXJ0aWVzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFBhc3N3b3JkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9IFNvbWUoc2lfZGF0YTo6cGFzc3dvcmQ6OmVuY3J5cHRfcGFzc3dvcmQoJHt2YXJpYWJsZU5hbWV9KT8pO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9ICR7dmFyaWFibGVOYW1lfTtgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsQ3JlYXRlQWRkVG9UZW5hbmN5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJiaWxsaW5nQWNjb3VudFwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uXCJcbiAgICApIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25TZXJ2aWNlXCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJjb21wb25lbnRPYmplY3RcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25fc2VydmljZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25fc2VydmljZV9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvblNlcnZpY2VJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ1c2VyXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiZ3JvdXBcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJvcmdhbml6YXRpb25cIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvbkluc3RhbmNlXCJcbiAgICApIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIndvcmtzcGFjZVwiKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5vcmdhbml6YXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhvcmdhbml6YXRpb25faWQpO2ApO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5vcmdhbml6YXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhvcmdhbml6YXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCB3b3Jrc3BhY2VfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLndvcmtzcGFjZV9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy53b3Jrc3BhY2VJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyh3b3Jrc3BhY2VfaWQpO2ApO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZVZhbGlkYXRlRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuICAgICAgICBjb25zdCBwcm9wTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmxlbigpID09IDAge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5pc19ub25lKCkge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgdG9wUHJvcDogUHJvcFByZWx1ZGUuUHJvcE9iamVjdCxcbiAgICBwcmVmaXg6IHN0cmluZyxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gWydcInNpU3RvcmFibGUubmF0dXJhbEtleVwiJ107XG4gICAgZm9yIChsZXQgcHJvcCBvZiB0b3BQcm9wLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLmhpZGRlbikge1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgcHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIHByb3AubmFtZSkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICAgIHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIGAke3ByZWZpeH0uJHtwcm9wLm5hbWV9YCksXG4gICAgICAgICAgKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJlZml4fS4ke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3Qucm9vdFByb3AsXG4gICAgICBcIlwiLFxuICAgICk7XG4gICAgcmV0dXJuIGB2ZWMhWyR7cmVzdWx0c31dXFxuYDtcbiAgfVxuXG4gIHN0b3JhYmxlUmVmZXJlbnRpYWxGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IGZldGNoUHJvcHMgPSBbXTtcbiAgICBjb25zdCByZWZlcmVuY2VWZWMgPSBbXTtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0KSB7XG4gICAgICBsZXQgc2lQcm9wZXJ0aWVzID0gdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmdldEVudHJ5KFwic2lQcm9wZXJ0aWVzXCIpO1xuICAgICAgaWYgKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHNpUHJvcGVydGllcyA9IHNpUHJvcGVydGllcy5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmICghKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpKSB7XG4gICAgICAgIHRocm93IFwiQ2Fubm90IGdldCBwcm9wZXJ0aWVzIG9mIGEgbm9uIG9iamVjdCBpbiByZWYgY2hlY2tcIjtcbiAgICAgIH1cbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBzaVByb3BlcnRpZXMucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBpZiAocHJvcC5yZWZlcmVuY2UpIHtcbiAgICAgICAgICBjb25zdCBpdGVtTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc01hbnkoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNPbmUoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEJhc2VPYmplY3QpIHtcbiAgICB9XG5cbiAgICBpZiAoZmV0Y2hQcm9wcy5sZW5ndGggJiYgcmVmZXJlbmNlVmVjLmxlbmd0aCkge1xuICAgICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgICAgcmVzdWx0cy5wdXNoKGZldGNoUHJvcHMuam9pbihcIlxcblwiKSk7XG4gICAgICByZXN1bHRzLnB1c2goYHZlYyFbJHtyZWZlcmVuY2VWZWMuam9pbihcIixcIil9XWApO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiVmVjOjpuZXcoKVwiO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlclNlcnZpY2Uge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzeXN0ZW1PYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzXG4gICAgICAuc29ydCgoYSwgYikgPT4gKGEudHlwZU5hbWUgPiBiLnR5cGVOYW1lID8gMSA6IC0xKSlcbiAgICAgIC5tYXAobyA9PiBuZXcgUnVzdEZvcm1hdHRlcihvKSk7XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdEJvZHkoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXCJkYjogc2lfZGF0YTo6RGIsXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnQsXCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsU2VydmljZU5ld0NvbnN0cnVjdG9yQXJncygpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJldHVybiBcImRiOiBzaV9kYXRhOjpEYiwgYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnRcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiXCI7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VTdHJ1Y3RDb25zdHJ1Y3RvclJldHVybigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnRcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIixcIik7XG4gIH1cblxuICBpbXBsU2VydmljZVRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3NuYWtlQ2FzZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKX1fc2VydmVyOjoke3Bhc2NhbENhc2UodGhpcy5zZXJ2aWNlTmFtZSl9YDtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWlncmF0ZSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqIG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgaWYgKHN5c3RlbU9iai5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIgJiYgc3lzdGVtT2JqLm1pZ3JhdGVhYmxlID09IHRydWUpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgc3lzdGVtT2JqLnR5cGVOYW1lLFxuICAgICAgICAgICl9OjptaWdyYXRlKCZzZWxmLmRiKS5hd2FpdD87YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaGFzRW50aXRpZXMoKTogYm9vbGVhbiB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0cy5maW5kKHMgPT4gcy5raW5kKCkgPT0gXCJlbnRpdHlPYmplY3RcIikpIHtcbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaGFzTWlncmF0YWJsZXMoKTogYm9vbGVhbiB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3RzLmZpbmQoXG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgcyA9PiBzLmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIiAmJiBzLm1pZ3JhdGVhYmxlID09IHRydWUsXG4gICAgICApXG4gICAgKSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXG4gICAgICBcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsXG4gICAgICBcIi8vIE5vIHRvdWNoeSFcIixcbiAgICAgIFwiXCIsXG4gICAgICBcInB1YiBtb2QgbW9kZWw7XCIsXG4gICAgICBcInB1YiBtb2Qgc2VydmljZTtcIixcbiAgICBdO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vbW9kZWwvbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgZm9yIChjb25zdCBzeXN0ZW1PYmplY3Qgb2YgcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApKSB7XG4gICAgICBpZiAoc3lzdGVtT2JqZWN0LmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIikge1xuICAgICAgICByZXN1bHRzLnB1c2goYHB1YiBtb2QgJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX07YCk7XG4gICAgICB9XG4gICAgfVxuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZGVsL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5TZXJ2aWNlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2UucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyU2VydmljZSh0aGlzLnNlcnZpY2VOYW1lKSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShgZ2VuL3NlcnZpY2UucnNgLCBvdXRwdXQpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2RlbChzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvbW9kZWwucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyKHN5c3RlbU9iamVjdCksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXG4gICAgICBgZ2VuL21vZGVsLyR7c25ha2VDYXNlKHN5c3RlbU9iamVjdC50eXBlTmFtZSl9LnJzYCxcbiAgICAgIG91dHB1dCxcbiAgICApO1xuICB9XG5cbiAgYXN5bmMgbWFrZVBhdGgocGF0aFBhcnQ6IHN0cmluZyk6IFByb21pc2U8c3RyaW5nPiB7XG4gICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXCIuLlwiLCBgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWAsIFwic3JjXCIsIHBhdGhQYXJ0KTtcbiAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbiAgICBhd2FpdCBmcy5wcm9taXNlcy5ta2RpcihwYXRoLnJlc29sdmUocGF0aE5hbWUpLCB7IHJlY3Vyc2l2ZTogdHJ1ZSB9KTtcbiAgICByZXR1cm4gYWJzb2x1dGVQYXRoTmFtZTtcbiAgfVxuXG4gIGFzeW5jIGZvcm1hdENvZGUoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgYXdhaXQgZXhlY0NtZChgY2FyZ28gZm10IC1wIHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gKTtcbiAgfVxuXG4gIGFzeW5jIHdyaXRlQ29kZShmaWxlbmFtZTogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBwYXRobmFtZSA9IHBhdGguZGlybmFtZShmaWxlbmFtZSk7XG4gICAgY29uc3QgYmFzZW5hbWUgPSBwYXRoLmJhc2VuYW1lKGZpbGVuYW1lKTtcbiAgICBjb25zdCBjcmVhdGVkUGF0aCA9IGF3YWl0IHRoaXMubWFrZVBhdGgocGF0aG5hbWUpO1xuICAgIGNvbnN0IGNvZGVGaWxlbmFtZSA9IHBhdGguam9pbihjcmVhdGVkUGF0aCwgYmFzZW5hbWUpO1xuICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGUpO1xuICB9XG59XG5cbi8vIGV4cG9ydCBjbGFzcyBDb2RlZ2VuUnVzdCB7XG4vLyAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG4vLyAgIGZvcm1hdHRlcjogUnVzdEZvcm1hdHRlcjtcbi8vXG4vLyAgIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpIHtcbi8vICAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbi8vICAgICB0aGlzLmZvcm1hdHRlciA9IG5ldyBSdXN0Rm9ybWF0dGVyKHN5c3RlbU9iamVjdCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIHdyaXRlQ29kZShwYXJ0OiBzdHJpbmcsIGNvZGU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IGNyZWF0ZWRQYXRoID0gYXdhaXQgdGhpcy5tYWtlUGF0aCgpO1xuLy8gICAgIGNvbnN0IGNvZGVGaWxlbmFtZSA9IHBhdGguam9pbihjcmVhdGVkUGF0aCwgYCR7c25ha2VDYXNlKHBhcnQpfS5yc2ApO1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGUpO1xuLy8gICAgIGF3YWl0IGV4ZWNDbWQoYHJ1c3RmbXQgJHtjb2RlRmlsZW5hbWV9YCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIG1ha2VQYXRoKCk6IFByb21pc2U8c3RyaW5nPiB7XG4vLyAgICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5zaVBhdGhOYW1lLFxuLy8gICAgICAgXCJzcmNcIixcbi8vICAgICAgIFwiZ2VuXCIsXG4vLyAgICAgICBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpLFxuLy8gICAgICk7XG4vLyAgICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgICAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIocGF0aC5yZXNvbHZlKHBhdGhOYW1lKSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4vLyAgICAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIGdlbmVyYXRlQ29tcG9uZW50SW1wbHMoKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbi8vICAgICAgIFwiPCUtIGluY2x1ZGUoJ3J1c3QvY29tcG9uZW50LnJzLmVqcycsIHsgY29tcG9uZW50OiBjb21wb25lbnQgfSkgJT5cIixcbi8vICAgICAgIHtcbi8vICAgICAgICAgc3lzdGVtT2JqZWN0OiB0aGlzLnN5c3RlbU9iamVjdCxcbi8vICAgICAgICAgZm10OiB0aGlzLmZvcm1hdHRlcixcbi8vICAgICAgIH0sXG4vLyAgICAgICB7XG4vLyAgICAgICAgIGZpbGVuYW1lOiBfX2ZpbGVuYW1lLFxuLy8gICAgICAgfSxcbi8vICAgICApO1xuLy8gICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiY29tcG9uZW50XCIsIG91dHB1dCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIGdlbmVyYXRlQ29tcG9uZW50TW9kKCk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IG1vZHMgPSBbXCJjb21wb25lbnRcIl07XG4vLyAgICAgY29uc3QgbGluZXMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIFRvdWNoeSFcXG5cIl07XG4vLyAgICAgZm9yIChjb25zdCBtb2Qgb2YgbW9kcykge1xuLy8gICAgICAgbGluZXMucHVzaChgcHViIG1vZCAke21vZH07YCk7XG4vLyAgICAgfVxuLy8gICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwibW9kXCIsIGxpbmVzLmpvaW4oXCJcXG5cIikpO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXIge1xuLy8gICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuLy9cbi8vICAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4vLyAgICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudFR5cGVOYW1lKCk6IHN0cmluZyB7XG4vLyAgICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSk7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudE9yZGVyQnlGaWVsZHMoKTogc3RyaW5nIHtcbi8vICAgICBjb25zdCBvcmRlckJ5RmllbGRzID0gW107XG4vLyAgICAgY29uc3QgY29tcG9uZW50T2JqZWN0ID0gdGhpcy5jb21wb25lbnQuYXNDb21wb25lbnQoKTtcbi8vICAgICBmb3IgKGNvbnN0IHAgb2YgY29tcG9uZW50T2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgIGlmIChwLmhpZGRlbikge1xuLy8gICAgICAgICBjb250aW51ZTtcbi8vICAgICAgIH1cbi8vICAgICAgIGlmIChwLm5hbWUgPT0gXCJzdG9yYWJsZVwiKSB7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaCgnXCJzdG9yYWJsZS5uYXR1cmFsS2V5XCInKTtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKCdcInN0b3JhYmxlLnR5cGVOYW1lXCInKTtcbi8vICAgICAgIH0gZWxzZSBpZiAocC5uYW1lID09IFwic2lQcm9wZXJ0aWVzXCIpIHtcbi8vICAgICAgICAgY29udGludWU7XG4vLyAgICAgICB9IGVsc2UgaWYgKHAubmFtZSA9PSBcImNvbnN0cmFpbnRzXCIgJiYgcC5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuLy8gICAgICAgICAvLyBAdHMtaWdub3JlIHRydXN0IHVzIC0gd2UgY2hlY2tlZFxuLy8gICAgICAgICBmb3IgKGNvbnN0IHBjIG9mIHAucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgICAgIGlmIChwYy5raW5kKCkgIT0gXCJvYmplY3RcIikge1xuLy8gICAgICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKGBcImNvbnN0cmFpbnRzLiR7cGMubmFtZX1cImApO1xuLy8gICAgICAgICAgIH1cbi8vICAgICAgICAgfVxuLy8gICAgICAgfSBlbHNlIHtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKGBcIiR7cC5uYW1lfVwiYCk7XG4vLyAgICAgICB9XG4vLyAgICAgfVxuLy8gICAgIHJldHVybiBgdmVjIVske29yZGVyQnlGaWVsZHMuam9pbihcIixcIil9XVxcbmA7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudEltcG9ydHMoKTogc3RyaW5nIHtcbi8vICAgICBjb25zdCByZXN1bHQgPSBbXTtcbi8vICAgICByZXN1bHQucHVzaChcbi8vICAgICAgIGBwdWIgdXNlIGNyYXRlOjpwcm90b2J1Zjo6JHtzbmFrZUNhc2UodGhpcy5jb21wb25lbnQudHlwZU5hbWUpfTo6e2AsXG4vLyAgICAgICBgICBDb25zdHJhaW50cyxgLFxuLy8gICAgICAgYCAgTGlzdENvbXBvbmVudHNSZXBseSxgLFxuLy8gICAgICAgYCAgTGlzdENvbXBvbmVudHNSZXF1ZXN0LGAsXG4vLyAgICAgICBgICBQaWNrQ29tcG9uZW50UmVxdWVzdCxgLFxuLy8gICAgICAgYCAgQ29tcG9uZW50LGAsXG4vLyAgICAgICBgfTtgLFxuLy8gICAgICk7XG4vLyAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRWYWxpZGF0aW9uKCk6IHN0cmluZyB7XG4vLyAgICAgcmV0dXJuIHRoaXMuZ2VuVmFsaWRhdGlvbih0aGlzLmNvbXBvbmVudC5hc0NvbXBvbmVudCgpKTtcbi8vICAgfVxuLy9cbi8vICAgZ2VuVmFsaWRhdGlvbihwcm9wT2JqZWN0OiBQcm9wT2JqZWN0KTogc3RyaW5nIHtcbi8vICAgICBjb25zdCByZXN1bHQgPSBbXTtcbi8vICAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuLy8gICAgICAgICBjb25zdCBwcm9wTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuLy8gICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5pc19ub25lKCkge1xuLy8gICAgICAgICAgIHJldHVybiBFcnIoRGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4vLyAgICAgICAgIH1gKTtcbi8vICAgICAgIH1cbi8vICAgICB9XG4vLyAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdlbmVyYXRlR2VuTW9kKHdyaXR0ZW5Db21wb25lbnRzOiB7XG4vLyAgIFtrZXk6IHN0cmluZ106IHN0cmluZ1tdO1xuLy8gfSk6IFByb21pc2U8dm9pZD4ge1xuLy8gICBmb3IgKGNvbnN0IGNvbXBvbmVudCBpbiB3cml0dGVuQ29tcG9uZW50cykge1xuLy8gICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgICAgX19kaXJuYW1lLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgY29tcG9uZW50LFxuLy8gICAgICAgXCJzcmNcIixcbi8vICAgICAgIFwiZ2VuXCIsXG4vLyAgICAgKTtcbi8vICAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbi8vICAgICBjb25zdCBjb2RlID0gW1xuLy8gICAgICAgXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLFxuLy8gICAgICAgXCIvLyBObyB0b3VjaHkhXCIsXG4vLyAgICAgICBcIlwiLFxuLy8gICAgICAgXCJwdWIgbW9kIG1vZGVsO1wiLFxuLy8gICAgIF07XG4vL1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShcbi8vICAgICAgIHBhdGguam9pbihhYnNvbHV0ZVBhdGhOYW1lLCBcIm1vZC5yc1wiKSxcbi8vICAgICAgIGNvZGUuam9pbihcIlxcblwiKSxcbi8vICAgICApO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdlbmVyYXRlR2VuTW9kTW9kZWwoc2VydmljZU5hbWU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuLy8gICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgc2VydmljZU5hbWUsXG4vLyAgICAgXCJzcmNcIixcbi8vICAgICBcImdlblwiLFxuLy8gICAgIFwibW9kZWxcIixcbi8vICAgKTtcbi8vICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgIGNvbnN0IGNvZGUgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcXG5cIl07XG4vLyAgIGZvciAoY29uc3QgdHlwZU5hbWUgb2Ygd3JpdHRlbkNvbXBvbmVudHNbY29tcG9uZW50XSkge1xuLy8gICAgIGNvZGUucHVzaChgcHViIG1vZCAke3NuYWtlQ2FzZSh0eXBlTmFtZSl9O2ApO1xuLy8gICB9XG4vL1xuLy8gICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoXG4vLyAgICAgcGF0aC5qb2luKGFic29sdXRlUGF0aE5hbWUsIFwibW9kLnJzXCIpLFxuLy8gICAgIGNvZGUuam9pbihcIlxcblwiKSxcbi8vICAgKTtcbi8vIH1cbiJdfQ==