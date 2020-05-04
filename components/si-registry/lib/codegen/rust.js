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
    key: "componentConstraintsName",
    value: function componentConstraintsName() {
      return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.typeName), "Constraints");
    }
  }, {
    key: "entityName",
    value: function entityName() {
      if (this.systemObject.kind() == "baseObject") {
        throw "You asked for an entity name on a baseObject; this is a bug!";
      }

      return "crate::protobuf::".concat((0, _changeCase.pascalCase)( // @ts-ignore
      this.systemObject.baseTypeName), "Entity");
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
              result.push("result_obj.".concat(variableName, " = Some(si_data::password::encrypt_password(").concat(variableName, ")?);"));
            } else {
              result.push("result_obj.".concat(variableName, " = ").concat(variableName, ";"));
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
        result.push("si_properties.as_ref().ok_or(si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let integration_id = si_properties.as_ref().unwrap().integration_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.integrationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(integration_id);");
      } else if (this.systemObject.kind() == "componentObject") {
        result.push("si_storable.add_to_tenant_ids(\"global\");");
        result.push("si_properties.as_ref().ok_or(si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let integration_id = si_properties.as_ref().unwrap().integration_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.integrationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(integration_id);");
        result.push("let integration_service_id = si_properties.as_ref().unwrap().integration_service_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.integrationServiceId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(integration_service_id);");
      } else if (this.systemObject.typeName == "user" || this.systemObject.typeName == "group" || this.systemObject.typeName == "organization" || this.systemObject.typeName == "integrationInstance") {
        result.push("si_properties.as_ref().ok_or(si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.billingAccountId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(billing_account_id);");
      } else if (this.systemObject.typeName == "workspace") {
        result.push("si_properties.as_ref().ok_or(si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.billingAccountId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(billing_account_id);");
        result.push("let organization_id = si_properties.as_ref().unwrap().organization_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.organizationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(organization_id);");
      } else {
        result.push("si_properties.as_ref().ok_or(si_data::DataError::ValidationError(\"siProperties\".into()))?;");
        result.push("let billing_account_id = si_properties.as_ref().unwrap().billing_account_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.billingAccountId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(billing_account_id);");
        result.push("let organization_id = si_properties.as_ref().unwrap().organization_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.organizationId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(organization_id);");
        result.push("let workspace_id = si_properties.as_ref().unwrap().workspace_id.as_ref().ok_or(\n            si_data::DataError::ValidationError(\"siProperties.workspaceId\".into()),\n        )?;\n        si_storable.add_to_tenant_ids(workspace_id);");
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
      var result = ["pub db: si_data::Db,"];

      if (this.hasComponents()) {
        result.push("pub agent: si_cea::AgentClient,");
      }

      return result.join("\n");
    }
  }, {
    key: "implServiceStructConstructorReturn",
    value: function implServiceStructConstructorReturn() {
      var result = ["db"];

      if (this.hasComponents()) {
        result.push("agent");
      }

      return result.join(",");
    }
  }, {
    key: "implServiceNewConstructorArgs",
    value: function implServiceNewConstructorArgs() {
      if (this.hasComponents()) {
        return "db: si_data::Db, agent: si_cea::AgentClient";
      } else {
        return "db: si_data::Db";
      }
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
    key: "hasComponents",
    value: function hasComponents() {
      if (this.systemObjects.find(function (s) {
        return s.kind() == "component";
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwia2luZCIsImJhc2VUeXBlTmFtZSIsInNlcnZpY2VOYW1lIiwibWV0aG9kcyIsImdldEVudHJ5IiwicmVuZGVyT3B0aW9ucyIsImxpc3QiLCJydXN0VHlwZUZvclByb3AiLCJyZXF1ZXN0IiwicmVwbHkiLCJwcm9wTWV0aG9kIiwib3B0aW9uIiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicmVzdWx0cyIsInByb3BNZXRob2RzIiwiYXR0cnMiLCJzb3J0IiwiYSIsImIiLCJuYW1lIiwib3V0cHV0IiwicHVzaCIsImpvaW4iLCJwcm9wIiwiUHJvcFByZWx1ZGUiLCJQcm9wQWN0aW9uIiwiUHJvcE1ldGhvZCIsInBhcmVudE5hbWUiLCJQcm9wTnVtYmVyIiwibnVtYmVyS2luZCIsIlByb3BCb29sIiwiUHJvcE9iamVjdCIsIlByb3BMaW5rIiwicmVhbFByb3AiLCJsb29rdXBNeXNlbGYiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BNYXAiLCJQcm9wVGV4dCIsIlByb3BDb2RlIiwiUHJvcFNlbGVjdCIsInJlcGVhdGVkIiwicmVzdWx0IiwiY3JlYXRlTWV0aG9kIiwicHJvcGVydGllcyIsImxpc3RNZXRob2QiLCJmaWVsZE5hbWUiLCJsaXN0UmVwbHlWYWx1ZSIsIlN5c3RlbU9iamVjdCIsIm5hdHVyYWxLZXkiLCJtaWdyYXRlYWJsZSIsInZhcmlhYmxlTmFtZSIsIlByb3BQYXNzd29yZCIsImZpZWxkcyIsInJlcXVpcmVkIiwicHJvcE5hbWUiLCJ0b3BQcm9wIiwicHJlZml4IiwiaGlkZGVuIiwic3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wIiwicm9vdFByb3AiLCJmZXRjaFByb3BzIiwicmVmZXJlbmNlVmVjIiwiRW50aXR5RXZlbnRPYmplY3QiLCJFbnRpdHlPYmplY3QiLCJDb21wb25lbnRPYmplY3QiLCJzaVByb3BlcnRpZXMiLCJpdGVtTmFtZSIsIkJhc2VPYmplY3QiLCJsZW5ndGgiLCJSdXN0Rm9ybWF0dGVyU2VydmljZSIsInN5c3RlbU9iamVjdHMiLCJyZWdpc3RyeSIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsIm1hcCIsIm8iLCJoYXNDb21wb25lbnRzIiwic3lzdGVtT2JqIiwiZmluZCIsInMiLCJDb2RlZ2VuUnVzdCIsIndyaXRlQ29kZSIsInBhdGhQYXJ0IiwicGF0aCIsImFic29sdXRlUGF0aE5hbWUiLCJyZXNvbHZlIiwiZnMiLCJwcm9taXNlcyIsIm1rZGlyIiwicmVjdXJzaXZlIiwiY29kZSIsInBhdGhuYW1lIiwiZGlybmFtZSIsImJhc2VuYW1lIiwibWFrZVBhdGgiLCJjcmVhdGVkUGF0aCIsImNvZGVGaWxlbmFtZSIsIndyaXRlRmlsZSJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0FBUUE7O0FBQ0E7O0FBR0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7Ozs7Ozs7O0FBRUEsSUFBTUEsT0FBTyxHQUFHQyxpQkFBS0MsU0FBTCxDQUFlQywwQkFBYUMsSUFBNUIsQ0FBaEI7O0lBT2FDLGE7QUFHWCx5QkFBWUMsWUFBWixFQUF5RDtBQUFBO0FBQUE7QUFDdkQsU0FBS0EsWUFBTCxHQUFvQkEsWUFBcEI7QUFDRDs7OztpQ0FFb0I7QUFDbkIsd0NBQTJCLDRCQUFXLEtBQUtBLFlBQUwsQ0FBa0JDLFFBQTdCLENBQTNCO0FBQ0Q7OztnQ0FFbUI7QUFDbEIscUNBQXdCLDRCQUFXLEtBQUtELFlBQUwsQ0FBa0JDLFFBQTdCLENBQXhCO0FBQ0Q7OzsrQ0FFa0M7QUFDakMs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QUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0EiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQge1xuICBPYmplY3RUeXBlcyxcbiAgQmFzZU9iamVjdCxcbiAgU3lzdGVtT2JqZWN0LFxuICBDb21wb25lbnRPYmplY3QsXG4gIEVudGl0eU9iamVjdCxcbiAgRW50aXR5RXZlbnRPYmplY3QsXG59IGZyb20gXCIuLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCAqIGFzIFByb3BQcmVsdWRlIGZyb20gXCIuLi9jb21wb25lbnRzL3ByZWx1ZGVcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5pbXBvcnQgeyBQcm9wcyB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuXG5pbXBvcnQgeyBzbmFrZUNhc2UsIHBhc2NhbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBlanMgZnJvbSBcImVqc1wiO1xuaW1wb3J0IGZzIGZyb20gXCJmc1wiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG5cbmNvbnN0IGV4ZWNDbWQgPSB1dGlsLnByb21pc2lmeShjaGlsZFByb2Nlc3MuZXhlYyk7XG5cbmludGVyZmFjZSBSdXN0VHlwZUFzUHJvcE9wdGlvbnMge1xuICByZWZlcmVuY2U/OiBib29sZWFuO1xuICBvcHRpb24/OiBib29sZWFuO1xufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBzdHJ1Y3ROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIG1vZGVsTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6Om1vZGVsOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICBjb21wb25lbnRDb25zdHJhaW50c05hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUsXG4gICAgKX1Db25zdHJhaW50c2A7XG4gIH1cblxuICBlbnRpdHlOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImJhc2VPYmplY3RcIikge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eSBuYW1lIG9uIGEgYmFzZU9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICApfUVudGl0eWA7XG4gIH1cblxuICB0eXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuICB9XG5cbiAgZXJyb3JUeXBlKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6ZXJyb3I6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSl9RXJyb3JgO1xuICB9XG5cbiAgaGFzQ3JlYXRlTWV0aG9kKCk6IGJvb2xlYW4ge1xuICAgIHRyeSB7XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBjYXRjaCB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaXNDb21wb25lbnRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImNvbXBvbmVudE9iamVjdFwiO1xuICB9XG5cbiAgaXNFbnRpdHlPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImVudGl0eU9iamVjdFwiO1xuICB9XG5cbiAgaXNFbnRpdHlFdmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiZW50aXR5RXZlbnRPYmplY3RcIjtcbiAgfVxuXG4gIGltcGxMaXN0UmVxdWVzdFR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbExpc3RSZXBseVR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVxdWVzdFR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVwbHlUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UoXG4gICAgICB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLCB7XG4gICAgICAgIG9wdGlvbjogZmFsc2UsXG4gICAgICAgIHJlZmVyZW5jZTogZmFsc2UsXG4gICAgICB9KSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDb21tb25DcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUNyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlR2V0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUdldC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VMaXN0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUxpc3QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tcG9uZW50UGljayhwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDb21wb25lbnRQaWNrLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUN1c3RvbU1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDdXN0b21NZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcE1ldGhvZC5za2lwQXV0aCkge1xuICAgICAgcmV0dXJuIGAvLyBBdXRoZW50aWNhdGlvbiBpcyBza2lwcGVkIG9uIFxcYCR7dGhpcy5pbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgICAgIHByb3BNZXRob2QsXG4gICAgICApfVxcYFxcbmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiB0aGlzLmltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZCk7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoQ2FsbChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBsZXQgcHJlbHVkZSA9IFwic2lfYWNjb3VudDo6YXV0aG9yaXplXCI7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lID09IFwiYWNjb3VudFwiKSB7XG4gICAgICBwcmVsdWRlID0gXCJjcmF0ZTo6YXV0aG9yaXplXCI7XG4gICAgfVxuICAgIHJldHVybiBgJHtwcmVsdWRlfTo6YXV0aG56KCZzZWxmLmRiLCAmcmVxdWVzdCwgXCIke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgcHJvcE1ldGhvZCxcbiAgICApfVwiKS5hd2FpdD87YDtcbiAgfVxuXG4gIHNlcnZpY2VNZXRob2RzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGNvbnN0IHByb3BNZXRob2RzID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5hdHRycy5zb3J0KChhLCBiKSA9PlxuICAgICAgYS5uYW1lID4gYi5uYW1lID8gMSA6IC0xLFxuICAgICk7XG4gICAgZm9yIChjb25zdCBwcm9wTWV0aG9kIG9mIHByb3BNZXRob2RzKSB7XG4gICAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2VNZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgICB7XG4gICAgICAgICAgZm10OiB0aGlzLFxuICAgICAgICAgIHByb3BNZXRob2Q6IHByb3BNZXRob2QsXG4gICAgICAgIH0sXG4gICAgICAgIHtcbiAgICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICAgIH0sXG4gICAgICApO1xuICAgICAgcmVzdWx0cy5wdXNoKG91dHB1dCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBydXN0RmllbGROYW1lRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICB9XG5cbiAgcnVzdFR5cGVGb3JQcm9wKFxuICAgIHByb3A6IFByb3BzLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlZmVyZW5jZSA9IHJlbmRlck9wdGlvbnMucmVmZXJlbmNlIHx8IGZhbHNlO1xuICAgIGxldCBvcHRpb24gPSB0cnVlO1xuICAgIGlmIChyZW5kZXJPcHRpb25zLm9wdGlvbiA9PT0gZmFsc2UpIHtcbiAgICAgIG9wdGlvbiA9IGZhbHNlO1xuICAgIH1cblxuICAgIGxldCB0eXBlTmFtZTogc3RyaW5nO1xuXG4gICAgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24gfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kXG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpMzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpNjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInU2NFwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEJvb2wgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAocmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICAgIGxldCBwYXRoTmFtZTogc3RyaW5nO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lICYmXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lID09IHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lXG4gICAgICAgICkge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IGBzaV8ke3Byb3BPd25lci5zZXJ2aWNlTmFtZX06OnByb3RvYnVmYDtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH1cbiAgICAgICAgdHlwZU5hbWUgPSBgJHtwYXRoTmFtZX06OiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHJlYWxQcm9wLCByZW5kZXJPcHRpb25zKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWFwKSB7XG4gICAgICB0eXBlTmFtZSA9IGBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoTWFwPFN0cmluZywgU3RyaW5nPmA7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgQ2Fubm90IGdlbmVyYXRlIHR5cGUgZm9yICR7cHJvcC5uYW1lfSBraW5kICR7cHJvcC5raW5kKCl9IC0gQnVnIWA7XG4gICAgfVxuICAgIGlmIChyZWZlcmVuY2UpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgaWYgKHR5cGVOYW1lID09IFwiU3RyaW5nXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcIiZzdHJcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGAmJHt0eXBlTmFtZX1gO1xuICAgICAgfVxuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICB0eXBlTmFtZSA9IGBWZWM8JHt0eXBlTmFtZX0+YDtcbiAgICB9IGVsc2Uge1xuICAgICAgaWYgKG9wdGlvbikge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgT3B0aW9uPCR7dHlwZU5hbWV9PmA7XG4gICAgICB9XG4gICAgfVxuICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgIHJldHVybiB0eXBlTmFtZTtcbiAgfVxuXG4gIGltcGxDcmVhdGVOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goYCR7c25ha2VDYXNlKHByb3AubmFtZSl9OiAke3RoaXMucnVzdFR5cGVGb3JQcm9wKHByb3ApfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVQYXNzTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKHNuYWtlQ2FzZShwcm9wLm5hbWUpKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZExpc3RSZXN1bHRUb1JlcGx5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgbGlzdE1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJsaXN0XCIpO1xuICAgIGlmIChsaXN0TWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGxpc3RNZXRob2QucmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgbGV0IGxpc3RSZXBseVZhbHVlID0gYFNvbWUob3V0cHV0LiR7ZmllbGROYW1lfSlgO1xuICAgICAgICBpZiAoZmllbGROYW1lID09IFwibmV4dF9wYWdlX3Rva2VuXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IFwiU29tZShvdXRwdXQucGFnZV90b2tlbilcIjtcbiAgICAgICAgfSBlbHNlIGlmIChmaWVsZE5hbWUgPT0gXCJpdGVtc1wiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBgb3V0cHV0LiR7ZmllbGROYW1lfWA7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfTogJHtsaXN0UmVwbHlWYWx1ZX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZENyZWF0ZURlc3RydWN0dXJlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKGBsZXQgJHtmaWVsZE5hbWV9ID0gaW5uZXIuJHtmaWVsZE5hbWV9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBuYXR1cmFsS2V5KCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0Lm5hdHVyYWxLZXkpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJuYW1lXCI7XG4gICAgfVxuICB9XG5cbiAgaXNNaWdyYXRlYWJsZSgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiICYmIHRoaXMuc3lzdGVtT2JqZWN0Lm1pZ3JhdGVhYmxlXG4gICAgKTtcbiAgfVxuXG4gIGlzU3RvcmFibGUoKTogYm9vbGVhbiB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGltcGxDcmVhdGVTZXRQcm9wZXJ0aWVzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFBhc3N3b3JkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0X29iai4ke3ZhcmlhYmxlTmFtZX0gPSBTb21lKHNpX2RhdGE6OnBhc3N3b3JkOjplbmNyeXB0X3Bhc3N3b3JkKCR7dmFyaWFibGVOYW1lfSk/KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYHJlc3VsdF9vYmouJHt2YXJpYWJsZU5hbWV9ID0gJHt2YXJpYWJsZU5hbWV9O2ApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVBZGRUb1RlbmFuY3koKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImJpbGxpbmdBY2NvdW50XCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25cIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblNlcnZpY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3IoXG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImNvbXBvbmVudE9iamVjdFwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3Ioc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcihcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkLmFzX3JlZigpLm9rX29yKFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25TZXJ2aWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25fc2VydmljZV9pZCk7YCk7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwidXNlclwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImdyb3VwXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwib3JnYW5pemF0aW9uXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25JbnN0YW5jZVwiXG4gICAgKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3Ioc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yKFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ3b3Jrc3BhY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcihcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgb3JnYW5pemF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5vcmdhbml6YXRpb25faWQuYXNfcmVmKCkub2tfb3IoXG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5vcmdhbml6YXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhvcmdhbml6YXRpb25faWQpO2ApO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3Ioc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yKFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcihcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IHdvcmtzcGFjZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkud29ya3NwYWNlX2lkLmFzX3JlZigpLm9rX29yKFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMud29ya3NwYWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMod29ya3NwYWNlX2lkKTtgKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVWYWxpZGF0ZUZ1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5hdHRycykge1xuICAgICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbiAgICAgICAgY29uc3QgcHJvcE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5sZW4oKSA9PSAwIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0uaXNfbm9uZSgpIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgIHRvcFByb3A6IFByb3BQcmVsdWRlLlByb3BPYmplY3QsXG4gICAgcHJlZml4OiBzdHJpbmcsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFsnXCJzaVN0b3JhYmxlLm5hdHVyYWxLZXlcIiddO1xuICAgIGZvciAobGV0IHByb3Agb2YgdG9wUHJvcC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4pIHtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBwcm9wLm5hbWUpKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBgJHtwcmVmaXh9LiR7cHJvcC5uYW1lfWApLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3ByZWZpeH0uJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiLCBcIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnJvb3RQcm9wLFxuICAgICAgXCJcIixcbiAgICApO1xuICAgIHJldHVybiBgdmVjIVske3Jlc3VsdHN9XVxcbmA7XG4gIH1cblxuICBzdG9yYWJsZVJlZmVyZW50aWFsRmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCBmZXRjaFByb3BzID0gW107XG4gICAgY29uc3QgcmVmZXJlbmNlVmVjID0gW107XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCkge1xuICAgICAgbGV0IHNpUHJvcGVydGllcyA9IHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5nZXRFbnRyeShcInNpUHJvcGVydGllc1wiKTtcbiAgICAgIGlmIChzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBzaVByb3BlcnRpZXMgPSBzaVByb3BlcnRpZXMubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAoIShzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSkge1xuICAgICAgICB0aHJvdyBcIkNhbm5vdCBnZXQgcHJvcGVydGllcyBvZiBhIG5vbiBvYmplY3QgaW4gcmVmIGNoZWNrXCI7XG4gICAgICB9XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2Ygc2lQcm9wZXJ0aWVzLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgaWYgKHByb3AucmVmZXJlbmNlKSB7XG4gICAgICAgICAgY29uc3QgaXRlbU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNNYW55KFwiJHtpdGVtTmFtZX1cIiwgJHtpdGVtTmFtZX0pYCxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIGZldGNoUHJvcHMucHVzaChgbGV0ICR7aXRlbU5hbWV9ID0gbWF0Y2ggJnNlbGYuc2lfcHJvcGVydGllcyB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICBTb21lKGNpcCkgPT4gY2lwXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuJHtpdGVtTmFtZX1cbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5hc19yZWYoKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLm1hcChTdHJpbmc6OmFzX3JlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC51bndyYXBfb3IoXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICBOb25lID0+IFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiLFxuICAgICAgICAgICAgICAgICAgICAgICAgIH07YCk7XG4gICAgICAgICAgICByZWZlcmVuY2VWZWMucHVzaChcbiAgICAgICAgICAgICAgYHNpX2RhdGE6OlJlZmVyZW5jZTo6SGFzT25lKFwiJHtpdGVtTmFtZX1cIiwgJHtpdGVtTmFtZX0pYCxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBCYXNlT2JqZWN0KSB7XG4gICAgfVxuXG4gICAgaWYgKGZldGNoUHJvcHMubGVuZ3RoICYmIHJlZmVyZW5jZVZlYy5sZW5ndGgpIHtcbiAgICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICAgIHJlc3VsdHMucHVzaChmZXRjaFByb3BzLmpvaW4oXCJcXG5cIikpO1xuICAgICAgcmVzdWx0cy5wdXNoKGB2ZWMhWyR7cmVmZXJlbmNlVmVjLmpvaW4oXCIsXCIpfV1gKTtcbiAgICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlZlYzo6bmV3KClcIjtcbiAgICB9XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXJTZXJ2aWNlIHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0cyA9IHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSk7XG4gIH1cblxuICBzeXN0ZW1PYmplY3RzQXNGb3JtYXR0ZXJzKCk6IFJ1c3RGb3JtYXR0ZXJbXSB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0c1xuICAgICAgLnNvcnQoKGEsIGIpID0+IChhLnR5cGVOYW1lID4gYi50eXBlTmFtZSA/IDEgOiAtMSkpXG4gICAgICAubWFwKG8gPT4gbmV3IFJ1c3RGb3JtYXR0ZXIobykpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VTdHJ1Y3RCb2R5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wicHViIGRiOiBzaV9kYXRhOjpEYixcIl07XG4gICAgaWYgKHRoaXMuaGFzQ29tcG9uZW50cygpKSB7XG4gICAgICByZXN1bHQucHVzaChcInB1YiBhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudCxcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Q29uc3RydWN0b3JSZXR1cm4oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXCJkYlwiXTtcbiAgICBpZiAodGhpcy5oYXNDb21wb25lbnRzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnRcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIixcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU5ld0NvbnN0cnVjdG9yQXJncygpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLmhhc0NvbXBvbmVudHMoKSkge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiLCBhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudFwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJkYjogc2lfZGF0YTo6RGJcIjtcbiAgICB9XG4gIH1cblxuICBpbXBsU2VydmljZVRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3NuYWtlQ2FzZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKX1fc2VydmVyOjoke3Bhc2NhbENhc2UodGhpcy5zZXJ2aWNlTmFtZSl9YDtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWlncmF0ZSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqIG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgaWYgKHN5c3RlbU9iai5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIgJiYgc3lzdGVtT2JqLm1pZ3JhdGVhYmxlID09IHRydWUpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgc3lzdGVtT2JqLnR5cGVOYW1lLFxuICAgICAgICAgICl9OjptaWdyYXRlKCZzZWxmLmRiKS5hd2FpdD87YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaGFzQ29tcG9uZW50cygpOiBib29sZWFuIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3RzLmZpbmQocyA9PiBzLmtpbmQoKSA9PSBcImNvbXBvbmVudFwiKSkge1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cblxuICBoYXNNaWdyYXRhYmxlcygpOiBib29sZWFuIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdHMuZmluZChcbiAgICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgICBzID0+IHMua2luZCgpICE9IFwiYmFzZU9iamVjdFwiICYmIHMubWlncmF0ZWFibGUgPT0gdHJ1ZSxcbiAgICAgIClcbiAgICApIHtcbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBDb2RlZ2VuUnVzdCB7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZykge1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcbiAgICAgIFwiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIixcbiAgICAgIFwiLy8gTm8gdG91Y2h5IVwiLFxuICAgICAgXCJcIixcbiAgICAgIFwicHViIG1vZCBtb2RlbDtcIixcbiAgICAgIFwicHViIG1vZCBzZXJ2aWNlO1wiLFxuICAgIF07XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2RlbC9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWxNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iamVjdCBvZiByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoXG4gICAgICB0aGlzLnNlcnZpY2VOYW1lLFxuICAgICkpIHtcbiAgICAgIGlmIChzeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSB7XG4gICAgICAgIHJlc3VsdHMucHVzaChgcHViIG1vZCAke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfTtgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kZWwvbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlblNlcnZpY2UoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3Qvc2VydmljZS5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXJTZXJ2aWNlKHRoaXMuc2VydmljZU5hbWUpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vc2VydmljZS5yc2AsIG91dHB1dCk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9tb2RlbC5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXIoc3lzdGVtT2JqZWN0KSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcbiAgICAgIGBnZW4vbW9kZWwvJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ucnNgLFxuICAgICAgb3V0cHV0LFxuICAgICk7XG4gIH1cblxuICBhc3luYyBtYWtlUGF0aChwYXRoUGFydDogc3RyaW5nKTogUHJvbWlzZTxzdHJpbmc+IHtcbiAgICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcIi4uXCIsIGBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCwgXCJzcmNcIiwgcGF0aFBhcnQpO1xuICAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuICAgIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKHBhdGgucmVzb2x2ZShwYXRoTmFtZSksIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuICAgIHJldHVybiBhYnNvbHV0ZVBhdGhOYW1lO1xuICB9XG5cbiAgYXN5bmMgZm9ybWF0Q29kZSgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBhd2FpdCBleGVjQ21kKGBjYXJnbyBmbXQgLXAgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWApO1xuICB9XG5cbiAgYXN5bmMgd3JpdGVDb2RlKGZpbGVuYW1lOiBzdHJpbmcsIGNvZGU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHBhdGhuYW1lID0gcGF0aC5kaXJuYW1lKGZpbGVuYW1lKTtcbiAgICBjb25zdCBiYXNlbmFtZSA9IHBhdGguYmFzZW5hbWUoZmlsZW5hbWUpO1xuICAgIGNvbnN0IGNyZWF0ZWRQYXRoID0gYXdhaXQgdGhpcy5tYWtlUGF0aChwYXRobmFtZSk7XG4gICAgY29uc3QgY29kZUZpbGVuYW1lID0gcGF0aC5qb2luKGNyZWF0ZWRQYXRoLCBiYXNlbmFtZSk7XG4gICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKGNvZGVGaWxlbmFtZSwgY29kZSk7XG4gIH1cbn1cblxuLy8gZXhwb3J0IGNsYXNzIENvZGVnZW5SdXN0IHtcbi8vICAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcbi8vICAgZm9ybWF0dGVyOiBSdXN0Rm9ybWF0dGVyO1xuLy9cbi8vICAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcykge1xuLy8gICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuLy8gICAgIHRoaXMuZm9ybWF0dGVyID0gbmV3IFJ1c3RGb3JtYXR0ZXIoc3lzdGVtT2JqZWN0KTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgd3JpdGVDb2RlKHBhcnQ6IHN0cmluZywgY29kZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgICAgY29uc3QgY3JlYXRlZFBhdGggPSBhd2FpdCB0aGlzLm1ha2VQYXRoKCk7XG4vLyAgICAgY29uc3QgY29kZUZpbGVuYW1lID0gcGF0aC5qb2luKGNyZWF0ZWRQYXRoLCBgJHtzbmFrZUNhc2UocGFydCl9LnJzYCk7XG4vLyAgICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKGNvZGVGaWxlbmFtZSwgY29kZSk7XG4vLyAgICAgYXdhaXQgZXhlY0NtZChgcnVzdGZtdCAke2NvZGVGaWxlbmFtZX1gKTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgbWFrZVBhdGgoKTogUHJvbWlzZTxzdHJpbmc+IHtcbi8vICAgICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICAgIF9fZGlybmFtZSxcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnNpUGF0aE5hbWUsXG4vLyAgICAgICBcInNyY1wiLFxuLy8gICAgICAgXCJnZW5cIixcbi8vICAgICAgIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSksXG4vLyAgICAgKTtcbi8vICAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy5ta2RpcihwYXRoLnJlc29sdmUocGF0aE5hbWUpLCB7IHJlY3Vyc2l2ZTogdHJ1ZSB9KTtcbi8vICAgICByZXR1cm4gYWJzb2x1dGVQYXRoTmFtZTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgZ2VuZXJhdGVDb21wb25lbnRJbXBscygpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuLy8gICAgICAgXCI8JS0gaW5jbHVkZSgncnVzdC9jb21wb25lbnQucnMuZWpzJywgeyBjb21wb25lbnQ6IGNvbXBvbmVudCB9KSAlPlwiLFxuLy8gICAgICAge1xuLy8gICAgICAgICBzeXN0ZW1PYmplY3Q6IHRoaXMuc3lzdGVtT2JqZWN0LFxuLy8gICAgICAgICBmbXQ6IHRoaXMuZm9ybWF0dGVyLFxuLy8gICAgICAgfSxcbi8vICAgICAgIHtcbi8vICAgICAgICAgZmlsZW5hbWU6IF9fZmlsZW5hbWUsXG4vLyAgICAgICB9LFxuLy8gICAgICk7XG4vLyAgICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJjb21wb25lbnRcIiwgb3V0cHV0KTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgZ2VuZXJhdGVDb21wb25lbnRNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgICAgY29uc3QgbW9kcyA9IFtcImNvbXBvbmVudFwiXTtcbi8vICAgICBjb25zdCBsaW5lcyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gVG91Y2h5IVxcblwiXTtcbi8vICAgICBmb3IgKGNvbnN0IG1vZCBvZiBtb2RzKSB7XG4vLyAgICAgICBsaW5lcy5wdXNoKGBwdWIgbW9kICR7bW9kfTtgKTtcbi8vICAgICB9XG4vLyAgICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJtb2RcIiwgbGluZXMuam9pbihcIlxcblwiKSk7XG4vLyAgIH1cbi8vIH1cbi8vXG4vLyBleHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4vLyAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG4vL1xuLy8gICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFJ1c3RGb3JtYXR0ZXJbXCJzeXN0ZW1PYmplY3RcIl0pIHtcbi8vICAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50VHlwZU5hbWUoKTogc3RyaW5nIHtcbi8vICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKTtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50T3JkZXJCeUZpZWxkcygpOiBzdHJpbmcge1xuLy8gICAgIGNvbnN0IG9yZGVyQnlGaWVsZHMgPSBbXTtcbi8vICAgICBjb25zdCBjb21wb25lbnRPYmplY3QgPSB0aGlzLmNvbXBvbmVudC5hc0NvbXBvbmVudCgpO1xuLy8gICAgIGZvciAoY29uc3QgcCBvZiBjb21wb25lbnRPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgaWYgKHAuaGlkZGVuKSB7XG4vLyAgICAgICAgIGNvbnRpbnVlO1xuLy8gICAgICAgfVxuLy8gICAgICAgaWYgKHAubmFtZSA9PSBcInN0b3JhYmxlXCIpIHtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKCdcInN0b3JhYmxlLm5hdHVyYWxLZXlcIicpO1xuLy8gICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goJ1wic3RvcmFibGUudHlwZU5hbWVcIicpO1xuLy8gICAgICAgfSBlbHNlIGlmIChwLm5hbWUgPT0gXCJzaVByb3BlcnRpZXNcIikge1xuLy8gICAgICAgICBjb250aW51ZTtcbi8vICAgICAgIH0gZWxzZSBpZiAocC5uYW1lID09IFwiY29uc3RyYWludHNcIiAmJiBwLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4vLyAgICAgICAgIC8vIEB0cy1pZ25vcmUgdHJ1c3QgdXMgLSB3ZSBjaGVja2VkXG4vLyAgICAgICAgIGZvciAoY29uc3QgcGMgb2YgcC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICAgICAgaWYgKHBjLmtpbmQoKSAhPSBcIm9iamVjdFwiKSB7XG4vLyAgICAgICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goYFwiY29uc3RyYWludHMuJHtwYy5uYW1lfVwiYCk7XG4vLyAgICAgICAgICAgfVxuLy8gICAgICAgICB9XG4vLyAgICAgICB9IGVsc2Uge1xuLy8gICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goYFwiJHtwLm5hbWV9XCJgKTtcbi8vICAgICAgIH1cbi8vICAgICB9XG4vLyAgICAgcmV0dXJuIGB2ZWMhWyR7b3JkZXJCeUZpZWxkcy5qb2luKFwiLFwiKX1dXFxuYDtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50SW1wb3J0cygpOiBzdHJpbmcge1xuLy8gICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuLy8gICAgIHJlc3VsdC5wdXNoKFxuLy8gICAgICAgYHB1YiB1c2UgY3JhdGU6OnByb3RvYnVmOjoke3NuYWtlQ2FzZSh0aGlzLmNvbXBvbmVudC50eXBlTmFtZSl9Ojp7YCxcbi8vICAgICAgIGAgIENvbnN0cmFpbnRzLGAsXG4vLyAgICAgICBgICBMaXN0Q29tcG9uZW50c1JlcGx5LGAsXG4vLyAgICAgICBgICBMaXN0Q29tcG9uZW50c1JlcXVlc3QsYCxcbi8vICAgICAgIGAgIFBpY2tDb21wb25lbnRSZXF1ZXN0LGAsXG4vLyAgICAgICBgICBDb21wb25lbnQsYCxcbi8vICAgICAgIGB9O2AsXG4vLyAgICAgKTtcbi8vICAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudFZhbGlkYXRpb24oKTogc3RyaW5nIHtcbi8vICAgICByZXR1cm4gdGhpcy5nZW5WYWxpZGF0aW9uKHRoaXMuY29tcG9uZW50LmFzQ29tcG9uZW50KCkpO1xuLy8gICB9XG4vL1xuLy8gICBnZW5WYWxpZGF0aW9uKHByb3BPYmplY3Q6IFByb3BPYmplY3QpOiBzdHJpbmcge1xuLy8gICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuLy8gICAgIGZvciAoY29uc3QgcHJvcCBvZiBwcm9wT2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4vLyAgICAgICAgIGNvbnN0IHByb3BOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4vLyAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmlzX25vbmUoKSB7XG4vLyAgICAgICAgICAgcmV0dXJuIEVycihEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbi8vICAgICAgICAgfWApO1xuLy8gICAgICAgfVxuLy8gICAgIH1cbi8vICAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4vLyAgIH1cbi8vIH1cbi8vXG4vLyBleHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2VuZXJhdGVHZW5Nb2Qod3JpdHRlbkNvbXBvbmVudHM6IHtcbi8vICAgW2tleTogc3RyaW5nXTogc3RyaW5nW107XG4vLyB9KTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgIGZvciAoY29uc3QgY29tcG9uZW50IGluIHdyaXR0ZW5Db21wb25lbnRzKSB7XG4vLyAgICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBjb21wb25lbnQsXG4vLyAgICAgICBcInNyY1wiLFxuLy8gICAgICAgXCJnZW5cIixcbi8vICAgICApO1xuLy8gICAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICAgIGNvbnN0IGNvZGUgPSBbXG4vLyAgICAgICBcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsXG4vLyAgICAgICBcIi8vIE5vIHRvdWNoeSFcIixcbi8vICAgICAgIFwiXCIsXG4vLyAgICAgICBcInB1YiBtb2QgbW9kZWw7XCIsXG4vLyAgICAgXTtcbi8vXG4vLyAgICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKFxuLy8gICAgICAgcGF0aC5qb2luKGFic29sdXRlUGF0aE5hbWUsIFwibW9kLnJzXCIpLFxuLy8gICAgICAgY29kZS5qb2luKFwiXFxuXCIpLFxuLy8gICAgICk7XG4vLyAgIH1cbi8vIH1cbi8vXG4vLyBleHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2VuZXJhdGVHZW5Nb2RNb2RlbChzZXJ2aWNlTmFtZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgIF9fZGlybmFtZSxcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBzZXJ2aWNlTmFtZSxcbi8vICAgICBcInNyY1wiLFxuLy8gICAgIFwiZ2VuXCIsXG4vLyAgICAgXCJtb2RlbFwiLFxuLy8gICApO1xuLy8gICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbi8vICAgY29uc3QgY29kZSA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVxcblwiXTtcbi8vICAgZm9yIChjb25zdCB0eXBlTmFtZSBvZiB3cml0dGVuQ29tcG9uZW50c1tjb21wb25lbnRdKSB7XG4vLyAgICAgY29kZS5wdXNoKGBwdWIgbW9kICR7c25ha2VDYXNlKHR5cGVOYW1lKX07YCk7XG4vLyAgIH1cbi8vXG4vLyAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShcbi8vICAgICBwYXRoLmpvaW4oYWJzb2x1dGVQYXRoTmFtZSwgXCJtb2QucnNcIiksXG4vLyAgICAgY29kZS5qb2luKFwiXFxuXCIpLFxuLy8gICApO1xuLy8gfVxuIl19