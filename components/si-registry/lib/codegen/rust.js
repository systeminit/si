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

      var _iterator7 = _createForOfIteratorHelper(this.systemObject.fields.attrs),
          _step7;

      try {
        for (_iterator7.s(); !(_step7 = _iterator7.n()).done;) {
          var _prop = _step7.value;

          var _variableName = (0, _changeCase.snakeCase)(_prop.name);

          var defaultValue = _prop.defaultValue();

          if (defaultValue) {
            if (_prop.kind() == "text") {
              result.push("result.".concat(_variableName, " = \"").concat(defaultValue, "\".to_string();"));
            } else if (_prop.kind() == "enum") {
              var enumName = "".concat((0, _changeCase.pascalCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(_prop.name));
              result.push("result.set_".concat(_variableName, "(crate::protobuf::").concat(enumName, "::").concat((0, _changeCase.pascalCase)(defaultValue), ");"));
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

      var _iterator8 = _createForOfIteratorHelper(this.systemObject.fields.attrs),
          _step8;

      try {
        for (_iterator8.s(); !(_step8 = _iterator8.n()).done;) {
          var prop = _step8.value;

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
        _iterator8.e(err);
      } finally {
        _iterator8.f();
      }

      return result.join("\n");
    }
  }, {
    key: "storableOrderByFieldsByProp",
    value: function storableOrderByFieldsByProp(topProp, prefix) {
      var results = ['"siStorable.naturalKey"'];

      var _iterator9 = _createForOfIteratorHelper(topProp.properties.attrs),
          _step9;

      try {
        for (_iterator9.s(); !(_step9 = _iterator9.n()).done;) {
          var prop = _step9.value;

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
        _iterator9.e(err);
      } finally {
        _iterator9.f();
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

        var _iterator10 = _createForOfIteratorHelper(siProperties.properties.attrs),
            _step10;

        try {
          for (_iterator10.s(); !(_step10 = _iterator10.n()).done;) {
            var prop = _step10.value;

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
          _iterator10.e(err);
        } finally {
          _iterator10.f();
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

      var _iterator11 = _createForOfIteratorHelper(this.systemObjects),
          _step11;

      try {
        for (_iterator11.s(); !(_step11 = _iterator11.n()).done;) {
          var systemObj = _step11.value;

          // @ts-ignore
          if (systemObj.kind() != "baseObject" && systemObj.migrateable == true) {
            result.push("crate::protobuf::".concat((0, _changeCase.pascalCase)(systemObj.typeName), "::migrate(&self.db).await?;"));
          }
        }
      } catch (err) {
        _iterator11.e(err);
      } finally {
        _iterator11.f();
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
        var results, _iterator12, _step12, systemObject;

        return _regenerator["default"].wrap(function _callee2$(_context2) {
          while (1) {
            switch (_context2.prev = _context2.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator12 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(this.serviceName));

                try {
                  for (_iterator12.s(); !(_step12 = _iterator12.n()).done;) {
                    systemObject = _step12.value;

                    if (systemObject.kind() != "baseObject") {
                      results.push("pub mod ".concat((0, _changeCase.snakeCase)(systemObject.typeName), ";"));
                    }
                  }
                } catch (err) {
                  _iterator12.e(err);
                } finally {
                  _iterator12.f();
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwiQ29tcG9uZW50T2JqZWN0IiwiRW50aXR5T2JqZWN0IiwiRW50aXR5RXZlbnRPYmplY3QiLCJiYXNlVHlwZU5hbWUiLCJwcm9wTWV0aG9kIiwicnVzdEZpZWxkTmFtZUZvclByb3AiLCJyZXBsYWNlIiwic2VydmljZU5hbWUiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJraW5kIiwiaXNFbnRpdHlPYmplY3QiLCJpc0VudGl0eUFjdGlvbk1ldGhvZCIsIm5hbWUiLCJlbmRzV2l0aCIsInJlbmRlck9wdGlvbnMiLCJsaXN0IiwicnVzdFR5cGVGb3JQcm9wIiwicmVxdWVzdCIsInJlcGx5Iiwib3B0aW9uIiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicmVzdWx0cyIsInByb3BNZXRob2RzIiwiYXR0cnMiLCJzb3J0IiwiYSIsImIiLCJvdXRwdXQiLCJwdXNoIiwiam9pbiIsInByb3AiLCJQcm9wUHJlbHVkZSIsIlByb3BBY3Rpb24iLCJQcm9wTWV0aG9kIiwicGFyZW50TmFtZSIsIlByb3BOdW1iZXIiLCJudW1iZXJLaW5kIiwiUHJvcEJvb2wiLCJQcm9wT2JqZWN0IiwiUHJvcExpbmsiLCJyZWFsUHJvcCIsImxvb2t1cE15c2VsZiIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcE1hcCIsIlByb3BUZXh0IiwiUHJvcENvZGUiLCJQcm9wU2VsZWN0IiwicmVwZWF0ZWQiLCJyZXN1bHQiLCJjcmVhdGVNZXRob2QiLCJwcm9wZXJ0aWVzIiwibGlzdE1ldGhvZCIsImZpZWxkTmFtZSIsImxpc3RSZXBseVZhbHVlIiwiU3lzdGVtT2JqZWN0IiwibmF0dXJhbEtleSIsIm1pZ3JhdGVhYmxlIiwidmFyaWFibGVOYW1lIiwiUHJvcFBhc3N3b3JkIiwiZmllbGRzIiwiZGVmYXVsdFZhbHVlIiwiZW51bU5hbWUiLCJtdmNjIiwicmVxdWlyZWQiLCJwcm9wTmFtZSIsInRvcFByb3AiLCJwcmVmaXgiLCJoaWRkZW4iLCJzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AiLCJyb290UHJvcCIsImZldGNoUHJvcHMiLCJyZWZlcmVuY2VWZWMiLCJzaVByb3BlcnRpZXMiLCJpdGVtTmFtZSIsIkJhc2VPYmplY3QiLCJsZW5ndGgiLCJSdXN0Rm9ybWF0dGVyU2VydmljZSIsInN5c3RlbU9iamVjdHMiLCJyZWdpc3RyeSIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsIm1hcCIsIm8iLCJoYXNFbnRpdGllcyIsImltcGxTZXJ2aWNlVHJhaXROYW1lIiwic3lzdGVtT2JqIiwiZmluZCIsInMiLCJDb2RlZ2VuUnVzdCIsImZsYXRNYXAiLCJ3cml0ZUNvZGUiLCJwYXRoUGFydCIsInBhdGgiLCJhYnNvbHV0ZVBhdGhOYW1lIiwicmVzb2x2ZSIsImZzIiwicHJvbWlzZXMiLCJta2RpciIsInJlY3Vyc2l2ZSIsImNvZGUiLCJwYXRobmFtZSIsImRpcm5hbWUiLCJiYXNlbmFtZSIsIm1ha2VQYXRoIiwiY3JlYXRlZFBhdGgiLCJjb2RlRmlsZW5hbWUiLCJ3cml0ZUZpbGUiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFBOztBQVFBOztBQUNBOztBQUdBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOzs7Ozs7OztBQUVBLElBQU1BLE9BQU8sR0FBR0MsaUJBQUtDLFNBQUwsQ0FBZUMsMEJBQWFDLElBQTVCLENBQWhCOztJQU9hQyxhO0FBR1gseUJBQVlDLFlBQVosRUFBeUQ7QUFBQTtBQUFBO0FBQ3ZELFNBQUtBLFlBQUwsR0FBb0JBLFlBQXBC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Q0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIE9iamVjdFR5cGVzLFxuICBCYXNlT2JqZWN0LFxuICBTeXN0ZW1PYmplY3QsXG4gIENvbXBvbmVudE9iamVjdCxcbiAgRW50aXR5T2JqZWN0LFxuICBFbnRpdHlFdmVudE9iamVjdCxcbn0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0ICogYXMgUHJvcFByZWx1ZGUgZnJvbSBcIi4uL2NvbXBvbmVudHMvcHJlbHVkZVwiO1xuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3BzIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5cbmltcG9ydCB7IHNuYWtlQ2FzZSwgcGFzY2FsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IGVqcyBmcm9tIFwiZWpzXCI7XG5pbXBvcnQgZnMgZnJvbSBcImZzXCI7XG5pbXBvcnQgcGF0aCBmcm9tIFwicGF0aFwiO1xuaW1wb3J0IGNoaWxkUHJvY2VzcyBmcm9tIFwiY2hpbGRfcHJvY2Vzc1wiO1xuaW1wb3J0IHV0aWwgZnJvbSBcInV0aWxcIjtcblxuY29uc3QgZXhlY0NtZCA9IHV0aWwucHJvbWlzaWZ5KGNoaWxkUHJvY2Vzcy5leGVjKTtcblxuaW50ZXJmYWNlIFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyB7XG4gIHJlZmVyZW5jZT86IGJvb2xlYW47XG4gIG9wdGlvbj86IGJvb2xlYW47XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyIHtcbiAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcblxuICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFJ1c3RGb3JtYXR0ZXJbXCJzeXN0ZW1PYmplY3RcIl0pIHtcbiAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbiAgfVxuXG4gIHN0cnVjdE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX1gO1xuICB9XG5cbiAgbW9kZWxOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6bW9kZWw6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIGNvbXBvbmVudE5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9Q29tcG9uZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGNvbXBvbmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgY29tcG9uZW50Q29uc3RyYWludHNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUNvbXBvbmVudENvbnN0cmFpbnRzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGEgY29tcG9uZW50IGNvbnN0cmFpbnRzIG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlFZGl0TWV0aG9kTmFtZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICAgIHJldHVybiBgZWRpdF8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcE1ldGhvZCkucmVwbGFjZShcbiAgICAgICAgXCJfZWRpdFwiLFxuICAgICAgICBcIlwiLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZWRpdCBtZXRob2QgbmFtZSBvbiBhIG5vbi1lbnRpdHkgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eUV2ZW50TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlFdmVudGA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHlFdmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlQcm9wZXJ0aWVzTmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlQcm9wZXJ0aWVzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eVByb3BlcnRpZXMgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIG1vZGVsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcE1ldGhvZCk7XG4gIH1cblxuICB0eXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuICB9XG5cbiAgZXJyb3JUeXBlKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6ZXJyb3I6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSl9RXJyb3JgO1xuICB9XG5cbiAgaGFzQ3JlYXRlTWV0aG9kKCk6IGJvb2xlYW4ge1xuICAgIHRyeSB7XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBjYXRjaCB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaXNDb21wb25lbnRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImNvbXBvbmVudE9iamVjdFwiO1xuICB9XG5cbiAgaXNFbnRpdHlPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImVudGl0eU9iamVjdFwiO1xuICB9XG5cbiAgaXNFbnRpdHlFdmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiZW50aXR5RXZlbnRPYmplY3RcIjtcbiAgfVxuXG4gIGlzRW50aXR5QWN0aW9uTWV0aG9kKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBib29sZWFuIHtcbiAgICByZXR1cm4gcHJvcE1ldGhvZC5raW5kKCkgPT0gXCJhY3Rpb25cIiAmJiB0aGlzLmlzRW50aXR5T2JqZWN0KCk7XG4gIH1cblxuICBpc0VudGl0eUVkaXRNZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLmlzRW50aXR5QWN0aW9uTWV0aG9kKHByb3BNZXRob2QpICYmIHByb3BNZXRob2QubmFtZS5lbmRzV2l0aChcIkVkaXRcIilcbiAgICApO1xuICB9XG5cbiAgaW1wbExpc3RSZXF1ZXN0VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVxdWVzdCwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsTGlzdFJlcGx5VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXF1ZXN0VHlwZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXBseVR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXBseSwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShcbiAgICAgIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QsIHtcbiAgICAgICAgb3B0aW9uOiBmYWxzZSxcbiAgICAgICAgcmVmZXJlbmNlOiBmYWxzZSxcbiAgICAgIH0pLFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUFjdGlvbihwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlBY3Rpb24ucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5RWRpdChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlFZGl0LnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUNvbW1vbkNyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDb21tb25DcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5Q3JlYXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUNyZWF0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VHZXQocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlR2V0LnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUxpc3QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlTGlzdC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDb21wb25lbnRQaWNrKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUNvbXBvbmVudFBpY2sucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ3VzdG9tTWV0aG9kKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUN1c3RvbU1ldGhvZC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGlmIChwcm9wTWV0aG9kLnNraXBBdXRoKSB7XG4gICAgICByZXR1cm4gYC8vIEF1dGhlbnRpY2F0aW9uIGlzIHNraXBwZWQgb24gXFxgJHt0aGlzLmltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICAgICAgcHJvcE1ldGhvZCxcbiAgICAgICl9XFxgXFxuYDtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHRoaXMuaW1wbFNlcnZpY2VBdXRoQ2FsbChwcm9wTWV0aG9kKTtcbiAgICB9XG4gIH1cblxuICBpbXBsU2VydmljZUF1dGhDYWxsKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGxldCBwcmVsdWRlID0gXCJzaV9hY2NvdW50OjphdXRob3JpemVcIjtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWUgPT0gXCJhY2NvdW50XCIpIHtcbiAgICAgIHByZWx1ZGUgPSBcImNyYXRlOjphdXRob3JpemVcIjtcbiAgICB9XG4gICAgcmV0dXJuIGAke3ByZWx1ZGV9OjphdXRobnooJnNlbGYuZGIsICZyZXF1ZXN0LCBcIiR7dGhpcy5pbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgICBwcm9wTWV0aG9kLFxuICAgICl9XCIpLmF3YWl0PztgO1xuICB9XG5cbiAgc2VydmljZU1ldGhvZHMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgY29uc3QgcHJvcE1ldGhvZHMgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmF0dHJzLnNvcnQoKGEsIGIpID0+XG4gICAgICBhLm5hbWUgPiBiLm5hbWUgPyAxIDogLTEsXG4gICAgKTtcbiAgICBmb3IgKGNvbnN0IHByb3BNZXRob2Qgb2YgcHJvcE1ldGhvZHMpIHtcbiAgICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3Qvc2VydmljZU1ldGhvZC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICAgIHtcbiAgICAgICAgICBmbXQ6IHRoaXMsXG4gICAgICAgICAgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCxcbiAgICAgICAgfSxcbiAgICAgICAge1xuICAgICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgICAgfSxcbiAgICAgICk7XG4gICAgICByZXN1bHRzLnB1c2gob3V0cHV0KTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3A6IFByb3BzKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gIH1cblxuICBydXN0VHlwZUZvclByb3AoXG4gICAgcHJvcDogUHJvcHMsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVmZXJlbmNlID0gcmVuZGVyT3B0aW9ucy5yZWZlcmVuY2UgfHwgZmFsc2U7XG4gICAgbGV0IG9wdGlvbiA9IHRydWU7XG4gICAgaWYgKHJlbmRlck9wdGlvbnMub3B0aW9uID09PSBmYWxzZSkge1xuICAgICAgb3B0aW9uID0gZmFsc2U7XG4gICAgfVxuXG4gICAgbGV0IHR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgICBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbiB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gYCR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE51bWJlcikge1xuICAgICAgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcImludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcImkzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50MzJcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwidTMyXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcImludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcImk2NFwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50NjRcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwidTY0XCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInUxMjhcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwidTEyOFwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEJvb2wgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAocmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICAgIGxldCBwYXRoTmFtZTogc3RyaW5nO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lICYmXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lID09IHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lXG4gICAgICAgICkge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IGBzaV8ke3Byb3BPd25lci5zZXJ2aWNlTmFtZX06OnByb3RvYnVmYDtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH1cbiAgICAgICAgdHlwZU5hbWUgPSBgJHtwYXRoTmFtZX06OiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHJlYWxQcm9wLCByZW5kZXJPcHRpb25zKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWFwKSB7XG4gICAgICB0eXBlTmFtZSA9IGBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoTWFwPFN0cmluZywgU3RyaW5nPmA7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgQ2Fubm90IGdlbmVyYXRlIHR5cGUgZm9yICR7cHJvcC5uYW1lfSBraW5kICR7cHJvcC5raW5kKCl9IC0gQnVnIWA7XG4gICAgfVxuICAgIGlmIChyZWZlcmVuY2UpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgaWYgKHR5cGVOYW1lID09IFwiU3RyaW5nXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcIiZzdHJcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGAmJHt0eXBlTmFtZX1gO1xuICAgICAgfVxuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICB0eXBlTmFtZSA9IGBWZWM8JHt0eXBlTmFtZX0+YDtcbiAgICB9IGVsc2Uge1xuICAgICAgaWYgKG9wdGlvbikge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgT3B0aW9uPCR7dHlwZU5hbWV9PmA7XG4gICAgICB9XG4gICAgfVxuICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgIHJldHVybiB0eXBlTmFtZTtcbiAgfVxuXG4gIGltcGxDcmVhdGVOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goYCR7c25ha2VDYXNlKHByb3AubmFtZSl9OiAke3RoaXMucnVzdFR5cGVGb3JQcm9wKHByb3ApfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVQYXNzTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKHNuYWtlQ2FzZShwcm9wLm5hbWUpKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZExpc3RSZXN1bHRUb1JlcGx5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgbGlzdE1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJsaXN0XCIpO1xuICAgIGlmIChsaXN0TWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGxpc3RNZXRob2QucmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgbGV0IGxpc3RSZXBseVZhbHVlID0gYFNvbWUob3V0cHV0LiR7ZmllbGROYW1lfSlgO1xuICAgICAgICBpZiAoZmllbGROYW1lID09IFwibmV4dF9wYWdlX3Rva2VuXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IFwiU29tZShvdXRwdXQucGFnZV90b2tlbilcIjtcbiAgICAgICAgfSBlbHNlIGlmIChmaWVsZE5hbWUgPT0gXCJpdGVtc1wiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBgb3V0cHV0LiR7ZmllbGROYW1lfWA7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfTogJHtsaXN0UmVwbHlWYWx1ZX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZENyZWF0ZURlc3RydWN0dXJlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKGBsZXQgJHtmaWVsZE5hbWV9ID0gaW5uZXIuJHtmaWVsZE5hbWV9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBuYXR1cmFsS2V5KCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0Lm5hdHVyYWxLZXkpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJuYW1lXCI7XG4gICAgfVxuICB9XG5cbiAgaXNNaWdyYXRlYWJsZSgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiICYmIHRoaXMuc3lzdGVtT2JqZWN0Lm1pZ3JhdGVhYmxlXG4gICAgKTtcbiAgfVxuXG4gIGlzU3RvcmFibGUoKTogYm9vbGVhbiB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGltcGxDcmVhdGVTZXRQcm9wZXJ0aWVzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFBhc3N3b3JkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9IFNvbWUoc2lfZGF0YTo6cGFzc3dvcmQ6OmVuY3J5cHRfcGFzc3dvcmQoJHt2YXJpYWJsZU5hbWV9KT8pO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9ICR7dmFyaWFibGVOYW1lfTtgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmF0dHJzKSB7XG4gICAgICBjb25zdCB2YXJpYWJsZU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgIGNvbnN0IGRlZmF1bHRWYWx1ZSA9IHByb3AuZGVmYXVsdFZhbHVlKCk7XG4gICAgICBpZiAoZGVmYXVsdFZhbHVlKSB7XG4gICAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcInRleHRcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSBcIiR7ZGVmYXVsdFZhbHVlfVwiLnRvX3N0cmluZygpO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImVudW1cIikge1xuICAgICAgICAgIGNvbnN0IGVudW1OYW1lID0gYCR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lLFxuICAgICAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuc2V0XyR7dmFyaWFibGVOYW1lfShjcmF0ZTo6cHJvdG9idWY6OiR7ZW51bU5hbWV9Ojoke3Bhc2NhbENhc2UoXG4gICAgICAgICAgICAgIGRlZmF1bHRWYWx1ZSBhcyBzdHJpbmcsXG4gICAgICAgICAgICApfSk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVBZGRUb1RlbmFuY3koKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImJpbGxpbmdBY2NvdW50XCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25cIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblNlcnZpY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImNvbXBvbmVudE9iamVjdFwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uU2VydmljZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX3NlcnZpY2VfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcInVzZXJcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJncm91cFwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIm9yZ2FuaXphdGlvblwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uSW5zdGFuY2VcIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwid29ya3NwYWNlXCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IHdvcmtzcGFjZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkud29ya3NwYWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLndvcmtzcGFjZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKHdvcmtzcGFjZV9pZCk7YCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlSXNNdmNjKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0Lm12Y2MgPT0gdHJ1ZSkge1xuICAgICAgcmV0dXJuIFwidHJ1ZVwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJmYWxzZVwiO1xuICAgIH1cbiAgfVxuXG4gIHN0b3JhYmxlVmFsaWRhdGVGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICAgIGNvbnN0IHByb3BOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0ubGVuKCkgPT0gMCB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmlzX25vbmUoKSB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICB0b3BQcm9wOiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0LFxuICAgIHByZWZpeDogc3RyaW5nLFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbJ1wic2lTdG9yYWJsZS5uYXR1cmFsS2V5XCInXTtcbiAgICBmb3IgKGxldCBwcm9wIG9mIHRvcFByb3AucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKHByb3AuaGlkZGVuKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBwcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgcHJvcC5uYW1lKSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgICAgdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgYCR7cHJlZml4fS4ke3Byb3AubmFtZX1gKSxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcmVmaXh9LiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5yb290UHJvcCxcbiAgICAgIFwiXCIsXG4gICAgKTtcbiAgICByZXR1cm4gYHZlYyFbJHtyZXN1bHRzfV1cXG5gO1xuICB9XG5cbiAgc3RvcmFibGVSZWZlcmVudGlhbEZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgZmV0Y2hQcm9wcyA9IFtdO1xuICAgIGNvbnN0IHJlZmVyZW5jZVZlYyA9IFtdO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QpIHtcbiAgICAgIGxldCBzaVByb3BlcnRpZXMgPSB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuZ2V0RW50cnkoXCJzaVByb3BlcnRpZXNcIik7XG4gICAgICBpZiAoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgc2lQcm9wZXJ0aWVzID0gc2lQcm9wZXJ0aWVzLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKCEoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkpIHtcbiAgICAgICAgdGhyb3cgXCJDYW5ub3QgZ2V0IHByb3BlcnRpZXMgb2YgYSBub24gb2JqZWN0IGluIHJlZiBjaGVja1wiO1xuICAgICAgfVxuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIHNpUHJvcGVydGllcy5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGlmIChwcm9wLnJlZmVyZW5jZSkge1xuICAgICAgICAgIGNvbnN0IGl0ZW1OYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICAgIGZldGNoUHJvcHMucHVzaChgbGV0ICR7aXRlbU5hbWV9ID0gbWF0Y2ggJnNlbGYuc2lfcHJvcGVydGllcyB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICBTb21lKGNpcCkgPT4gY2lwXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuJHtpdGVtTmFtZX1cbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5hc19yZWYoKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLm1hcChTdHJpbmc6OmFzX3JlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC51bndyYXBfb3IoXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICBOb25lID0+IFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiLFxuICAgICAgICAgICAgICAgICAgICAgICAgIH07YCk7XG4gICAgICAgICAgICByZWZlcmVuY2VWZWMucHVzaChcbiAgICAgICAgICAgICAgYHNpX2RhdGE6OlJlZmVyZW5jZTo6SGFzTWFueShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc09uZShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgfVxuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQmFzZU9iamVjdCkge1xuICAgIH1cblxuICAgIGlmIChmZXRjaFByb3BzLmxlbmd0aCAmJiByZWZlcmVuY2VWZWMubGVuZ3RoKSB7XG4gICAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgICByZXN1bHRzLnB1c2goZmV0Y2hQcm9wcy5qb2luKFwiXFxuXCIpKTtcbiAgICAgIHJlc3VsdHMucHVzaChgdmVjIVske3JlZmVyZW5jZVZlYy5qb2luKFwiLFwiKX1dYCk7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJWZWM6Om5ldygpXCI7XG4gICAgfVxuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyU2VydmljZSB7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZykge1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWUpO1xuICB9XG5cbiAgc3lzdGVtT2JqZWN0c0FzRm9ybWF0dGVycygpOiBSdXN0Rm9ybWF0dGVyW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNcbiAgICAgIC5zb3J0KChhLCBiKSA9PiAoYS50eXBlTmFtZSA+IGIudHlwZU5hbWUgPyAxIDogLTEpKVxuICAgICAgLm1hcChvID0+IG5ldyBSdXN0Rm9ybWF0dGVyKG8pKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Qm9keSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiOiBzaV9kYXRhOjpEYixcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudCxcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTmV3Q29uc3RydWN0b3JBcmdzKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiLCBhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudFwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJkYjogc2lfZGF0YTo6RGJcIjtcbiAgICB9XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdENvbnN0cnVjdG9yUmV0dXJuKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wiZGJcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudFwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlVHJhaXROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7c25ha2VDYXNlKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApfV9zZXJ2ZXI6OiR7cGFzY2FsQ2FzZSh0aGlzLnNlcnZpY2VOYW1lKX1gO1xuICB9XG5cbiAgaW1wbFNlcnZlck5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5pbXBsU2VydmljZVRyYWl0TmFtZSgpfVNlcnZlcmA7XG4gIH1cblxuICBpbXBsU2VydmljZU1pZ3JhdGUoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iaiBvZiB0aGlzLnN5c3RlbU9iamVjdHMpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIGlmIChzeXN0ZW1PYmoua2luZCgpICE9IFwiYmFzZU9iamVjdFwiICYmIHN5c3RlbU9iai5taWdyYXRlYWJsZSA9PSB0cnVlKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgIHN5c3RlbU9iai50eXBlTmFtZSxcbiAgICAgICAgICApfTo6bWlncmF0ZSgmc2VsZi5kYikuYXdhaXQ/O2AsXG4gICAgICAgICk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGhhc0VudGl0aWVzKCk6IGJvb2xlYW4ge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdHMuZmluZChzID0+IHMua2luZCgpID09IFwiZW50aXR5T2JqZWN0XCIpKSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGhhc01pZ3JhdGFibGVzKCk6IGJvb2xlYW4ge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0cy5maW5kKFxuICAgICAgICAvLyBAdHMtaWdub3JlXG4gICAgICAgIHMgPT4gcy5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIgJiYgcy5taWdyYXRlYWJsZSA9PSB0cnVlLFxuICAgICAgKVxuICAgICkge1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIENvZGVnZW5SdXN0IHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICB9XG5cbiAgaGFzU2VydmljZU1ldGhvZHMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHJlZ2lzdHJ5XG4gICAgICAgIC5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUodGhpcy5zZXJ2aWNlTmFtZSlcbiAgICAgICAgLmZsYXRNYXAobyA9PiBvLm1ldGhvZHMuYXR0cnMpLmxlbmd0aCA+IDBcbiAgICApO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbk1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1xuICAgICAgXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLFxuICAgICAgXCIvLyBObyB0b3VjaHkhXCIsXG4gICAgICBcIlwiLFxuICAgICAgXCJwdWIgbW9kIG1vZGVsO1wiLFxuICAgICAgXCJwdWIgbW9kIHNlcnZpY2U7XCIsXG4gICAgXTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL21vZGVsL21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2RlbE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXCIsIFwiXCJdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqZWN0IG9mIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKSkge1xuICAgICAgaWYgKHN5c3RlbU9iamVjdC5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKGBwdWIgbW9kICR7c25ha2VDYXNlKHN5c3RlbU9iamVjdC50eXBlTmFtZSl9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2RlbC9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuU2VydmljZSgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlclNlcnZpY2UodGhpcy5zZXJ2aWNlTmFtZSksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoYGdlbi9zZXJ2aWNlLnJzYCwgb3V0cHV0KTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWwoc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcyk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L21vZGVsLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlcihzeXN0ZW1PYmplY3QpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFxuICAgICAgYGdlbi9tb2RlbC8ke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfS5yc2AsXG4gICAgICBvdXRwdXQsXG4gICAgKTtcbiAgfVxuXG4gIGFzeW5jIG1ha2VQYXRoKHBhdGhQYXJ0OiBzdHJpbmcpOiBQcm9taXNlPHN0cmluZz4ge1xuICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFwiLi5cIiwgYHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gLCBcInNyY1wiLCBwYXRoUGFydCk7XG4gICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4gICAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIocGF0aC5yZXNvbHZlKHBhdGhOYW1lKSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4gICAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4gIH1cblxuICBhc3luYyBmb3JtYXRDb2RlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGF3YWl0IGV4ZWNDbWQoYGNhcmdvIGZtdCAtcCBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCk7XG4gIH1cblxuICBhc3luYyB3cml0ZUNvZGUoZmlsZW5hbWU6IHN0cmluZywgY29kZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcGF0aG5hbWUgPSBwYXRoLmRpcm5hbWUoZmlsZW5hbWUpO1xuICAgIGNvbnN0IGJhc2VuYW1lID0gcGF0aC5iYXNlbmFtZShmaWxlbmFtZSk7XG4gICAgY29uc3QgY3JlYXRlZFBhdGggPSBhd2FpdCB0aGlzLm1ha2VQYXRoKHBhdGhuYW1lKTtcbiAgICBjb25zdCBjb2RlRmlsZW5hbWUgPSBwYXRoLmpvaW4oY3JlYXRlZFBhdGgsIGJhc2VuYW1lKTtcbiAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoY29kZUZpbGVuYW1lLCBjb2RlKTtcbiAgfVxufVxuXG4vLyBleHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuLy8gICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuLy8gICBmb3JtYXR0ZXI6IFJ1c3RGb3JtYXR0ZXI7XG4vL1xuLy8gICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzKSB7XG4vLyAgICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4vLyAgICAgdGhpcy5mb3JtYXR0ZXIgPSBuZXcgUnVzdEZvcm1hdHRlcihzeXN0ZW1PYmplY3QpO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyB3cml0ZUNvZGUocGFydDogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBjcmVhdGVkUGF0aCA9IGF3YWl0IHRoaXMubWFrZVBhdGgoKTtcbi8vICAgICBjb25zdCBjb2RlRmlsZW5hbWUgPSBwYXRoLmpvaW4oY3JlYXRlZFBhdGgsIGAke3NuYWtlQ2FzZShwYXJ0KX0ucnNgKTtcbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoY29kZUZpbGVuYW1lLCBjb2RlKTtcbi8vICAgICBhd2FpdCBleGVjQ21kKGBydXN0Zm10ICR7Y29kZUZpbGVuYW1lfWApO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBtYWtlUGF0aCgpOiBQcm9taXNlPHN0cmluZz4ge1xuLy8gICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgICAgX19kaXJuYW1lLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgdGhpcy5zeXN0ZW1PYmplY3Quc2lQYXRoTmFtZSxcbi8vICAgICAgIFwic3JjXCIsXG4vLyAgICAgICBcImdlblwiLFxuLy8gICAgICAgc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKSxcbi8vICAgICApO1xuLy8gICAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKHBhdGgucmVzb2x2ZShwYXRoTmFtZSksIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuLy8gICAgIHJldHVybiBhYnNvbHV0ZVBhdGhOYW1lO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBnZW5lcmF0ZUNvbXBvbmVudEltcGxzKCk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4vLyAgICAgICBcIjwlLSBpbmNsdWRlKCdydXN0L2NvbXBvbmVudC5ycy5lanMnLCB7IGNvbXBvbmVudDogY29tcG9uZW50IH0pICU+XCIsXG4vLyAgICAgICB7XG4vLyAgICAgICAgIHN5c3RlbU9iamVjdDogdGhpcy5zeXN0ZW1PYmplY3QsXG4vLyAgICAgICAgIGZtdDogdGhpcy5mb3JtYXR0ZXIsXG4vLyAgICAgICB9LFxuLy8gICAgICAge1xuLy8gICAgICAgICBmaWxlbmFtZTogX19maWxlbmFtZSxcbi8vICAgICAgIH0sXG4vLyAgICAgKTtcbi8vICAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImNvbXBvbmVudFwiLCBvdXRwdXQpO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBnZW5lcmF0ZUNvbXBvbmVudE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBtb2RzID0gW1wiY29tcG9uZW50XCJdO1xuLy8gICAgIGNvbnN0IGxpbmVzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyBUb3VjaHkhXFxuXCJdO1xuLy8gICAgIGZvciAoY29uc3QgbW9kIG9mIG1vZHMpIHtcbi8vICAgICAgIGxpbmVzLnB1c2goYHB1YiBtb2QgJHttb2R9O2ApO1xuLy8gICAgIH1cbi8vICAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcIm1vZFwiLCBsaW5lcy5qb2luKFwiXFxuXCIpKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyIHtcbi8vICAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcbi8vXG4vLyAgIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogUnVzdEZvcm1hdHRlcltcInN5c3RlbU9iamVjdFwiXSkge1xuLy8gICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRUeXBlTmFtZSgpOiBzdHJpbmcge1xuLy8gICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRPcmRlckJ5RmllbGRzKCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3Qgb3JkZXJCeUZpZWxkcyA9IFtdO1xuLy8gICAgIGNvbnN0IGNvbXBvbmVudE9iamVjdCA9IHRoaXMuY29tcG9uZW50LmFzQ29tcG9uZW50KCk7XG4vLyAgICAgZm9yIChjb25zdCBwIG9mIGNvbXBvbmVudE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICBpZiAocC5oaWRkZW4pIHtcbi8vICAgICAgICAgY29udGludWU7XG4vLyAgICAgICB9XG4vLyAgICAgICBpZiAocC5uYW1lID09IFwic3RvcmFibGVcIikge1xuLy8gICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goJ1wic3RvcmFibGUubmF0dXJhbEtleVwiJyk7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaCgnXCJzdG9yYWJsZS50eXBlTmFtZVwiJyk7XG4vLyAgICAgICB9IGVsc2UgaWYgKHAubmFtZSA9PSBcInNpUHJvcGVydGllc1wiKSB7XG4vLyAgICAgICAgIGNvbnRpbnVlO1xuLy8gICAgICAgfSBlbHNlIGlmIChwLm5hbWUgPT0gXCJjb25zdHJhaW50c1wiICYmIHAua2luZCgpID09IFwib2JqZWN0XCIpIHtcbi8vICAgICAgICAgLy8gQHRzLWlnbm9yZSB0cnVzdCB1cyAtIHdlIGNoZWNrZWRcbi8vICAgICAgICAgZm9yIChjb25zdCBwYyBvZiBwLnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgICAgICBpZiAocGMua2luZCgpICE9IFwib2JqZWN0XCIpIHtcbi8vICAgICAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaChgXCJjb25zdHJhaW50cy4ke3BjLm5hbWV9XCJgKTtcbi8vICAgICAgICAgICB9XG4vLyAgICAgICAgIH1cbi8vICAgICAgIH0gZWxzZSB7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaChgXCIke3AubmFtZX1cImApO1xuLy8gICAgICAgfVxuLy8gICAgIH1cbi8vICAgICByZXR1cm4gYHZlYyFbJHtvcmRlckJ5RmllbGRzLmpvaW4oXCIsXCIpfV1cXG5gO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRJbXBvcnRzKCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3QgcmVzdWx0ID0gW107XG4vLyAgICAgcmVzdWx0LnB1c2goXG4vLyAgICAgICBgcHViIHVzZSBjcmF0ZTo6cHJvdG9idWY6OiR7c25ha2VDYXNlKHRoaXMuY29tcG9uZW50LnR5cGVOYW1lKX06OntgLFxuLy8gICAgICAgYCAgQ29uc3RyYWludHMsYCxcbi8vICAgICAgIGAgIExpc3RDb21wb25lbnRzUmVwbHksYCxcbi8vICAgICAgIGAgIExpc3RDb21wb25lbnRzUmVxdWVzdCxgLFxuLy8gICAgICAgYCAgUGlja0NvbXBvbmVudFJlcXVlc3QsYCxcbi8vICAgICAgIGAgIENvbXBvbmVudCxgLFxuLy8gICAgICAgYH07YCxcbi8vICAgICApO1xuLy8gICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50VmFsaWRhdGlvbigpOiBzdHJpbmcge1xuLy8gICAgIHJldHVybiB0aGlzLmdlblZhbGlkYXRpb24odGhpcy5jb21wb25lbnQuYXNDb21wb25lbnQoKSk7XG4vLyAgIH1cbi8vXG4vLyAgIGdlblZhbGlkYXRpb24ocHJvcE9iamVjdDogUHJvcE9iamVjdCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3QgcmVzdWx0ID0gW107XG4vLyAgICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbi8vICAgICAgICAgY29uc3QgcHJvcE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbi8vICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0uaXNfbm9uZSgpIHtcbi8vICAgICAgICAgICByZXR1cm4gRXJyKERhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuLy8gICAgICAgICB9YCk7XG4vLyAgICAgICB9XG4vLyAgICAgfVxuLy8gICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZW5lcmF0ZUdlbk1vZCh3cml0dGVuQ29tcG9uZW50czoge1xuLy8gICBba2V5OiBzdHJpbmddOiBzdHJpbmdbXTtcbi8vIH0pOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgZm9yIChjb25zdCBjb21wb25lbnQgaW4gd3JpdHRlbkNvbXBvbmVudHMpIHtcbi8vICAgICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICAgIF9fZGlybmFtZSxcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIGNvbXBvbmVudCxcbi8vICAgICAgIFwic3JjXCIsXG4vLyAgICAgICBcImdlblwiLFxuLy8gICAgICk7XG4vLyAgICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgICAgY29uc3QgY29kZSA9IFtcbi8vICAgICAgIFwiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIixcbi8vICAgICAgIFwiLy8gTm8gdG91Y2h5IVwiLFxuLy8gICAgICAgXCJcIixcbi8vICAgICAgIFwicHViIG1vZCBtb2RlbDtcIixcbi8vICAgICBdO1xuLy9cbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoXG4vLyAgICAgICBwYXRoLmpvaW4oYWJzb2x1dGVQYXRoTmFtZSwgXCJtb2QucnNcIiksXG4vLyAgICAgICBjb2RlLmpvaW4oXCJcXG5cIiksXG4vLyAgICAgKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZW5lcmF0ZUdlbk1vZE1vZGVsKHNlcnZpY2VOYW1lOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgX19kaXJuYW1lLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIHNlcnZpY2VOYW1lLFxuLy8gICAgIFwic3JjXCIsXG4vLyAgICAgXCJnZW5cIixcbi8vICAgICBcIm1vZGVsXCIsXG4vLyAgICk7XG4vLyAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICBjb25zdCBjb2RlID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXFxuXCJdO1xuLy8gICBmb3IgKGNvbnN0IHR5cGVOYW1lIG9mIHdyaXR0ZW5Db21wb25lbnRzW2NvbXBvbmVudF0pIHtcbi8vICAgICBjb2RlLnB1c2goYHB1YiBtb2QgJHtzbmFrZUNhc2UodHlwZU5hbWUpfTtgKTtcbi8vICAgfVxuLy9cbi8vICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKFxuLy8gICAgIHBhdGguam9pbihhYnNvbHV0ZVBhdGhOYW1lLCBcIm1vZC5yc1wiKSxcbi8vICAgICBjb2RlLmpvaW4oXCJcXG5cIiksXG4vLyAgICk7XG4vLyB9XG4iXX0=