"use strict";

function _typeof(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.CodegenRust = exports.RustFormatterService = exports.RustFormatter = void 0;

var _systemComponent = require("src/systemComponent");

var PropPrelude = _interopRequireWildcard(require("src/components/prelude"));

var _registry = require("src/registry");

var _changeCase = require("change-case");

var _ejs = _interopRequireDefault(require("ejs"));

var _fs = _interopRequireDefault(require("fs"));

var _path = _interopRequireDefault(require("path"));

var _child_process = _interopRequireDefault(require("child_process"));

var _util = _interopRequireDefault(require("util"));

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { "default": obj }; }

function _getRequireWildcardCache() { if (typeof WeakMap !== "function") return null; var cache = new WeakMap(); _getRequireWildcardCache = function _getRequireWildcardCache() { return cache; }; return cache; }

function _interopRequireWildcard(obj) { if (obj && obj.__esModule) { return obj; } if (obj === null || _typeof(obj) !== "object" && typeof obj !== "function") { return { "default": obj }; } var cache = _getRequireWildcardCache(); if (cache && cache.has(obj)) { return cache.get(obj); } var newObj = {}; var hasPropertyDescriptor = Object.defineProperty && Object.getOwnPropertyDescriptor; for (var key in obj) { if (Object.prototype.hasOwnProperty.call(obj, key)) { var desc = hasPropertyDescriptor ? Object.getOwnPropertyDescriptor(obj, key) : null; if (desc && (desc.get || desc.set)) { Object.defineProperty(newObj, key, desc); } else { newObj[key] = obj[key]; } } } newObj["default"] = obj; if (cache) { cache.set(obj, newObj); } return newObj; }

function asyncGeneratorStep(gen, resolve, reject, _next, _throw, key, arg) { try { var info = gen[key](arg); var value = info.value; } catch (error) { reject(error); return; } if (info.done) { resolve(value); } else { Promise.resolve(value).then(_next, _throw); } }

function _asyncToGenerator(fn) { return function () { var self = this, args = arguments; return new Promise(function (resolve, reject) { var gen = fn.apply(self, args); function _next(value) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "next", value); } function _throw(err) { asyncGeneratorStep(gen, resolve, reject, _next, _throw, "throw", err); } _next(undefined); }); }; }

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

var execCmd = _util["default"].promisify(_child_process["default"].exec);

var RustFormatter = /*#__PURE__*/function () {
  function RustFormatter(systemObject) {
    _classCallCheck(this, RustFormatter);

    _defineProperty(this, "systemObject", void 0);

    this.systemObject = systemObject;
  }

  _createClass(RustFormatter, [{
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
    key: "rustFieldNameForProp",
    value: function rustFieldNameForProp(prop) {
      return (0, _changeCase.snakeCase)(prop.name);
    }
  }, {
    key: "implServiceAuth",
    value: function implServiceAuth(propMethod) {
      if (propMethod.skipAuth) {
        return "// Skipping authentication\n";
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
    key: "implServiceGetMethodBody",
    value: function implServiceGetMethodBody(propMethod) {
      var results = [];

      var _iterator = _createForOfIteratorHelper(propMethod.request.properties.attrs),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var field = _step.value;

          if (field.required) {
            var rustVariableName = this.rustFieldNameForProp(field);
          } else {}
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      return results.join("\n");
    }
  }, {
    key: "serviceMethods",
    value: function serviceMethods() {
      var results = [];

      var _iterator2 = _createForOfIteratorHelper(this.systemObject.methods.attrs),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var propMethod = _step2.value;

          var output = _ejs["default"].render("<%- include('rust/serviceMethod.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
            fmt: this,
            propMethod: propMethod
          }, {
            filename: __filename
          });

          results.push(output);
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return results.join("\n");
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
        if (typeName == "String") {
          typeName = "&str";
        } else {
          typeName = "&".concat(typeName);
        }
      }

      if (prop.repeated) {
        typeName = "Vec<".concat(typeName, ">");
      } else {
        if (option) {
          typeName = "Option<".concat(typeName, ">");
        }
      }

      return typeName;
    }
  }, {
    key: "implCreateNewArgs",
    value: function implCreateNewArgs() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator3 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step3;

        try {
          for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
            var prop = _step3.value;
            result.push("".concat((0, _changeCase.snakeCase)(prop.name), ": ").concat(this.rustTypeForProp(prop)));
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
    key: "implCreatePassNewArgs",
    value: function implCreatePassNewArgs() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator4 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step4;

        try {
          for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
            var prop = _step4.value;
            result.push((0, _changeCase.snakeCase)(prop.name));
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
    key: "implServiceMethodListResultToReply",
    value: function implServiceMethodListResultToReply() {
      var result = [];
      var listMethod = this.systemObject.methods.getEntry("list");

      if (listMethod instanceof PropPrelude.PropMethod) {
        var _iterator5 = _createForOfIteratorHelper(listMethod.reply.properties.attrs),
            _step5;

        try {
          for (_iterator5.s(); !(_step5 = _iterator5.n()).done;) {
            var prop = _step5.value;
            var fieldName = (0, _changeCase.snakeCase)(prop.name);
            var listReplyValue = "Some(list_reply.".concat(fieldName, ")");

            if (fieldName == "next_page_token") {
              listReplyValue = "Some(list_reply.page_token)";
            } else if (fieldName == "items") {
              listReplyValue = "list_reply.".concat(fieldName);
            }

            result.push("".concat(fieldName, ": ").concat(listReplyValue));
          }
        } catch (err) {
          _iterator5.e(err);
        } finally {
          _iterator5.f();
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
        var _iterator6 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step6;

        try {
          for (_iterator6.s(); !(_step6 = _iterator6.n()).done;) {
            var prop = _step6.value;
            var fieldName = (0, _changeCase.snakeCase)(prop.name);
            result.push("let ".concat(fieldName, " = inner.").concat(fieldName, ";"));
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
      try {
        this.systemObject.fields.getEntry("version");
        return true;
      } catch (_unused2) {
        return false;
      }
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
        var _iterator7 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step7;

        try {
          for (_iterator7.s(); !(_step7 = _iterator7.n()).done;) {
            var prop = _step7.value;
            var variableName = (0, _changeCase.snakeCase)(prop.name);

            if (prop instanceof PropPrelude.PropPassword) {
              result.push("result_obj.".concat(variableName, " = Some(si_data::password::encrypt_password(").concat(variableName, ")?);"));
            } else {
              result.push("result_obj.".concat(variableName, " = ").concat(variableName, ";"));
            }
          }
        } catch (err) {
          _iterator7.e(err);
        } finally {
          _iterator7.f();
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
    _classCallCheck(this, RustFormatterService);

    _defineProperty(this, "serviceName", void 0);

    _defineProperty(this, "systemObjects", void 0);

    this.serviceName = serviceName;
    this.systemObjects = _registry.registry.getObjectsForServiceName(serviceName);
  }

  _createClass(RustFormatterService, [{
    key: "systemObjectsAsFormatters",
    value: function systemObjectsAsFormatters() {
      return this.systemObjects.map(function (o) {
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
  }]);

  return RustFormatterService;
}();

exports.RustFormatterService = RustFormatterService;

var CodegenRust = /*#__PURE__*/function () {
  function CodegenRust(serviceName) {
    _classCallCheck(this, CodegenRust);

    _defineProperty(this, "serviceName", void 0);

    this.serviceName = serviceName;
  } // Generate the 'gen/mod.rs'


  _createClass(CodegenRust, [{
    key: "generateGenMod",
    value: function () {
      var _generateGenMod = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee() {
        var results;
        return regeneratorRuntime.wrap(function _callee$(_context) {
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
      var _generateGenModelMod = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee2() {
        var results, _iterator11, _step11, systemObject;

        return regeneratorRuntime.wrap(function _callee2$(_context2) {
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
      var _generateGenService = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee3() {
        var output;
        return regeneratorRuntime.wrap(function _callee3$(_context3) {
          while (1) {
            switch (_context3.prev = _context3.next) {
              case 0:
                output = _ejs["default"].render("<%- include('rust/service.rs.ejs', { fmt: fmt }) %>", {
                  fmt: new RustFormatterService(this.serviceName)
                }, {
                  filename: __filename
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
      var _generateGenModel = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee4(systemObject) {
        var output;
        return regeneratorRuntime.wrap(function _callee4$(_context4) {
          while (1) {
            switch (_context4.prev = _context4.next) {
              case 0:
                output = _ejs["default"].render("<%- include('rust/model.rs.ejs', { fmt: fmt }) %>", {
                  fmt: new RustFormatter(systemObject)
                }, {
                  filename: __filename
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
      var _makePath = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee5(pathPart) {
        var pathName, absolutePathName;
        return regeneratorRuntime.wrap(function _callee5$(_context5) {
          while (1) {
            switch (_context5.prev = _context5.next) {
              case 0:
                pathName = _path["default"].join(__dirname, "..", "..", "..", "si-".concat(this.serviceName), "src", pathPart);
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
      var _formatCode = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee6() {
        return regeneratorRuntime.wrap(function _callee6$(_context6) {
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
      var _writeCode = _asyncToGenerator( /*#__PURE__*/regeneratorRuntime.mark(function _callee7(filename, code) {
        var pathname, basename, createdPath, codeFilename;
        return regeneratorRuntime.wrap(function _callee7$(_context7) {
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwic2VydmljZU5hbWUiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJyZW5kZXJPcHRpb25zIiwibGlzdCIsInJ1c3RUeXBlRm9yUHJvcCIsInJlcXVlc3QiLCJyZXBseSIsInByb3BNZXRob2QiLCJvcHRpb24iLCJyZWZlcmVuY2UiLCJwcm9wIiwibmFtZSIsInNraXBBdXRoIiwiaW1wbFNlcnZpY2VBdXRoQ2FsbCIsInByZWx1ZGUiLCJpbXBsU2VydmljZU1ldGhvZE5hbWUiLCJyZXN1bHRzIiwicHJvcGVydGllcyIsImF0dHJzIiwiZmllbGQiLCJyZXF1aXJlZCIsInJ1c3RWYXJpYWJsZU5hbWUiLCJydXN0RmllbGROYW1lRm9yUHJvcCIsImpvaW4iLCJvdXRwdXQiLCJlanMiLCJyZW5kZXIiLCJmbXQiLCJmaWxlbmFtZSIsIl9fZmlsZW5hbWUiLCJwdXNoIiwiUHJvcFByZWx1ZGUiLCJQcm9wQWN0aW9uIiwiUHJvcE1ldGhvZCIsInBhcmVudE5hbWUiLCJQcm9wTnVtYmVyIiwibnVtYmVyS2luZCIsIlByb3BCb29sIiwiUHJvcE9iamVjdCIsIlByb3BMaW5rIiwicmVhbFByb3AiLCJsb29rdXBNeXNlbGYiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BNYXAiLCJQcm9wVGV4dCIsIlByb3BDb2RlIiwiUHJvcFNlbGVjdCIsImtpbmQiLCJyZXBlYXRlZCIsInJlc3VsdCIsImNyZWF0ZU1ldGhvZCIsImxpc3RNZXRob2QiLCJmaWVsZE5hbWUiLCJsaXN0UmVwbHlWYWx1ZSIsIlN5c3RlbU9iamVjdCIsIm5hdHVyYWxLZXkiLCJmaWVsZHMiLCJ2YXJpYWJsZU5hbWUiLCJQcm9wUGFzc3dvcmQiLCJwcm9wTmFtZSIsInRvcFByb3AiLCJwcmVmaXgiLCJoaWRkZW4iLCJzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AiLCJyb290UHJvcCIsImZldGNoUHJvcHMiLCJyZWZlcmVuY2VWZWMiLCJFbnRpdHlFdmVudE9iamVjdCIsIkVudGl0eU9iamVjdCIsIkNvbXBvbmVudE9iamVjdCIsInNpUHJvcGVydGllcyIsIml0ZW1OYW1lIiwiQmFzZU9iamVjdCIsImxlbmd0aCIsIlJ1c3RGb3JtYXR0ZXJTZXJ2aWNlIiwic3lzdGVtT2JqZWN0cyIsInJlZ2lzdHJ5IiwiZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lIiwibWFwIiwibyIsImhhc0NvbXBvbmVudHMiLCJmaW5kIiwicyIsIkNvZGVnZW5SdXN0Iiwid3JpdGVDb2RlIiwicGF0aFBhcnQiLCJwYXRoIiwiX19kaXJuYW1lIiwiYWJzb2x1dGVQYXRoTmFtZSIsInJlc29sdmUiLCJmcyIsInByb21pc2VzIiwibWtkaXIiLCJyZWN1cnNpdmUiLCJjb2RlIiwicGF0aG5hbWUiLCJkaXJuYW1lIiwiYmFzZW5hbWUiLCJtYWtlUGF0aCIsImNyZWF0ZWRQYXRoIiwiY29kZUZpbGVuYW1lIiwid3JpdGVGaWxlIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7QUFBQTs7QUFRQTs7QUFDQTs7QUFHQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFFQSxJQUFNQSxPQUFPLEdBQUdDLGlCQUFLQyxTQUFMLENBQWVDLDBCQUFhQyxJQUE1QixDQUFoQjs7SUFPYUMsYTtBQUdYLHlCQUFZQyxZQUFaLEVBQXlEO0FBQUE7O0FBQUE7O0FBQ3ZELFNBQUtBLFlBQUwsR0FBb0JBLFlBQXBCO0FBQ0Q7Ozs7aUNBRW9CO0FBQ25CLHdDQUEyQiw0QkFBVyxLQUFLQSxZQUFMLENBQWtCQyxRQUE3QixDQUEzQjtBQUNEOzs7Z0NBRW1CO0FBQ2xCLHFDQUF3Qiw0QkFBVyxLQUFLRCxZQUFMLENBQWtCQyxRQUE3QixDQUF4QjtBQUNEOzs7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QUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIE9iamVjdFR5cGVzLFxuICBCYXNlT2JqZWN0LFxuICBTeXN0ZW1PYmplY3QsXG4gIENvbXBvbmVudE9iamVjdCxcbiAgRW50aXR5T2JqZWN0LFxuICBFbnRpdHlFdmVudE9iamVjdCxcbn0gZnJvbSBcInNyYy9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCAqIGFzIFByb3BQcmVsdWRlIGZyb20gXCJzcmMvY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCJzcmMvcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3BzIH0gZnJvbSBcInNyYy9hdHRyTGlzdFwiO1xuXG5pbXBvcnQgeyBzbmFrZUNhc2UsIHBhc2NhbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBlanMgZnJvbSBcImVqc1wiO1xuaW1wb3J0IGZzIGZyb20gXCJmc1wiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG5cbmNvbnN0IGV4ZWNDbWQgPSB1dGlsLnByb21pc2lmeShjaGlsZFByb2Nlc3MuZXhlYyk7XG5cbmludGVyZmFjZSBSdXN0VHlwZUFzUHJvcE9wdGlvbnMge1xuICByZWZlcmVuY2U/OiBib29sZWFuO1xuICBvcHRpb24/OiBib29sZWFuO1xufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBzdHJ1Y3ROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIG1vZGVsTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6Om1vZGVsOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICB0eXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuICB9XG5cbiAgZXJyb3JUeXBlKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6ZXJyb3I6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSl9RXJyb3JgO1xuICB9XG5cbiAgaGFzQ3JlYXRlTWV0aG9kKCk6IGJvb2xlYW4ge1xuICAgIHRyeSB7XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBjYXRjaCB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaW1wbExpc3RSZXF1ZXN0VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVxdWVzdCwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsTGlzdFJlcGx5VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXF1ZXN0VHlwZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXBseVR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXBseSwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShcbiAgICAgIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QsIHtcbiAgICAgICAgb3B0aW9uOiBmYWxzZSxcbiAgICAgICAgcmVmZXJlbmNlOiBmYWxzZSxcbiAgICAgIH0pLFxuICAgICk7XG4gIH1cblxuICBydXN0RmllbGROYW1lRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGlmIChwcm9wTWV0aG9kLnNraXBBdXRoKSB7XG4gICAgICByZXR1cm4gXCIvLyBTa2lwcGluZyBhdXRoZW50aWNhdGlvblxcblwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gdGhpcy5pbXBsU2VydmljZUF1dGhDYWxsKHByb3BNZXRob2QpO1xuICAgIH1cbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgbGV0IHByZWx1ZGUgPSBcInNpX2FjY291bnQ6OmF1dGhvcml6ZVwiO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSA9PSBcImFjY291bnRcIikge1xuICAgICAgcHJlbHVkZSA9IFwiY3JhdGU6OmF1dGhvcml6ZVwiO1xuICAgIH1cbiAgICByZXR1cm4gYCR7cHJlbHVkZX06OmF1dGhueigmc2VsZi5kYiwgJnJlcXVlc3QsIFwiJHt0aGlzLmltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICAgIHByb3BNZXRob2QsXG4gICAgKX1cIikuYXdhaXQ/O2A7XG4gIH1cblxuICBpbXBsU2VydmljZUdldE1ldGhvZEJvZHkocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGZvciAoY29uc3QgZmllbGQgb2YgcHJvcE1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChmaWVsZC5yZXF1aXJlZCkge1xuICAgICAgICBjb25zdCBydXN0VmFyaWFibGVOYW1lID0gdGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChmaWVsZCk7XG4gICAgICB9IGVsc2Uge1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc2VydmljZU1ldGhvZHMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wTWV0aG9kIG9mIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMpIHtcbiAgICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICAgIFwiPCUtIGluY2x1ZGUoJ3J1c3Qvc2VydmljZU1ldGhvZC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICAgIHtcbiAgICAgICAgICBmbXQ6IHRoaXMsXG4gICAgICAgICAgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCxcbiAgICAgICAgfSxcbiAgICAgICAge1xuICAgICAgICAgIGZpbGVuYW1lOiBfX2ZpbGVuYW1lLFxuICAgICAgICB9LFxuICAgICAgKTtcbiAgICAgIHJlc3VsdHMucHVzaChvdXRwdXQpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgcnVzdFR5cGVGb3JQcm9wKFxuICAgIHByb3A6IFByb3BzLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlZmVyZW5jZSA9IHJlbmRlck9wdGlvbnMucmVmZXJlbmNlIHx8IGZhbHNlO1xuICAgIGxldCBvcHRpb24gPSB0cnVlO1xuICAgIGlmIChyZW5kZXJPcHRpb25zLm9wdGlvbiA9PT0gZmFsc2UpIHtcbiAgICAgIG9wdGlvbiA9IGZhbHNlO1xuICAgIH1cblxuICAgIGxldCB0eXBlTmFtZTogc3RyaW5nO1xuXG4gICAgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24gfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kXG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpMzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpNjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInU2NFwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEJvb2wgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAocmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICAgIGxldCBwYXRoTmFtZTogc3RyaW5nO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lICYmXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lID09IHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lXG4gICAgICAgICkge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IGBzaV8ke3Byb3BPd25lci5zZXJ2aWNlTmFtZX06OnByb3RvYnVmYDtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH1cbiAgICAgICAgdHlwZU5hbWUgPSBgJHtwYXRoTmFtZX06OiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHJlYWxQcm9wLCByZW5kZXJPcHRpb25zKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWFwKSB7XG4gICAgICB0eXBlTmFtZSA9IGBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoTWFwPFN0cmluZywgU3RyaW5nPmA7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgQ2Fubm90IGdlbmVyYXRlIHR5cGUgZm9yICR7cHJvcC5uYW1lfSBraW5kICR7cHJvcC5raW5kKCl9IC0gQnVnIWA7XG4gICAgfVxuICAgIGlmIChyZWZlcmVuY2UpIHtcbiAgICAgIGlmICh0eXBlTmFtZSA9PSBcIlN0cmluZ1wiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCImc3RyXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICB0eXBlTmFtZSA9IGAmJHt0eXBlTmFtZX1gO1xuICAgICAgfVxuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgdHlwZU5hbWUgPSBgVmVjPCR7dHlwZU5hbWV9PmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGlmIChvcHRpb24pIHtcbiAgICAgICAgdHlwZU5hbWUgPSBgT3B0aW9uPCR7dHlwZU5hbWV9PmA7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiB0eXBlTmFtZTtcbiAgfVxuXG4gIGltcGxDcmVhdGVOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goYCR7c25ha2VDYXNlKHByb3AubmFtZSl9OiAke3RoaXMucnVzdFR5cGVGb3JQcm9wKHByb3ApfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVQYXNzTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKHNuYWtlQ2FzZShwcm9wLm5hbWUpKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZExpc3RSZXN1bHRUb1JlcGx5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgbGlzdE1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJsaXN0XCIpO1xuICAgIGlmIChsaXN0TWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGxpc3RNZXRob2QucmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgbGV0IGxpc3RSZXBseVZhbHVlID0gYFNvbWUobGlzdF9yZXBseS4ke2ZpZWxkTmFtZX0pYDtcbiAgICAgICAgaWYgKGZpZWxkTmFtZSA9PSBcIm5leHRfcGFnZV90b2tlblwiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBcIlNvbWUobGlzdF9yZXBseS5wYWdlX3Rva2VuKVwiO1xuICAgICAgICB9IGVsc2UgaWYgKGZpZWxkTmFtZSA9PSBcIml0ZW1zXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IGBsaXN0X3JlcGx5LiR7ZmllbGROYW1lfWA7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfTogJHtsaXN0UmVwbHlWYWx1ZX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZENyZWF0ZURlc3RydWN0dXJlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKGBsZXQgJHtmaWVsZE5hbWV9ID0gaW5uZXIuJHtmaWVsZE5hbWV9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBuYXR1cmFsS2V5KCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0Lm5hdHVyYWxLZXkpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJuYW1lXCI7XG4gICAgfVxuICB9XG5cbiAgaXNNaWdyYXRlYWJsZSgpOiBib29sZWFuIHtcbiAgICB0cnkge1xuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmdldEVudHJ5KFwidmVyc2lvblwiKTtcbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH0gY2F0Y2gge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGlzU3RvcmFibGUoKTogYm9vbGVhbiB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGltcGxDcmVhdGVTZXRQcm9wZXJ0aWVzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFBhc3N3b3JkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0X29iai4ke3ZhcmlhYmxlTmFtZX0gPSBTb21lKHNpX2RhdGE6OnBhc3N3b3JkOjplbmNyeXB0X3Bhc3N3b3JkKCR7dmFyaWFibGVOYW1lfSk/KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYHJlc3VsdF9vYmouJHt2YXJpYWJsZU5hbWV9ID0gJHt2YXJpYWJsZU5hbWV9O2ApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVBZGRUb1RlbmFuY3koKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImJpbGxpbmdBY2NvdW50XCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25cIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblNlcnZpY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3IoXG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ1c2VyXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiZ3JvdXBcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJvcmdhbml6YXRpb25cIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvbkluc3RhbmNlXCJcbiAgICApIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3IoXG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIndvcmtzcGFjZVwiKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3Ioc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yKFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcihcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3IoXG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yKFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMub3JnYW5pemF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMob3JnYW5pemF0aW9uX2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgd29ya3NwYWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS53b3Jrc3BhY2VfaWQuYXNfcmVmKCkub2tfb3IoXG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy53b3Jrc3BhY2VJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyh3b3Jrc3BhY2VfaWQpO2ApO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZVZhbGlkYXRlRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuICAgICAgICBjb25zdCBwcm9wTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmxlbigpID09IDAge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5pc19ub25lKCkge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgdG9wUHJvcDogUHJvcFByZWx1ZGUuUHJvcE9iamVjdCxcbiAgICBwcmVmaXg6IHN0cmluZyxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gWydcInNpU3RvcmFibGUubmF0dXJhbEtleVwiJ107XG4gICAgZm9yIChsZXQgcHJvcCBvZiB0b3BQcm9wLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLmhpZGRlbikge1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgcHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIHByb3AubmFtZSkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICAgIHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIGAke3ByZWZpeH0uJHtwcm9wLm5hbWV9YCksXG4gICAgICAgICAgKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJlZml4fS4ke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3Qucm9vdFByb3AsXG4gICAgICBcIlwiLFxuICAgICk7XG4gICAgcmV0dXJuIGB2ZWMhWyR7cmVzdWx0c31dXFxuYDtcbiAgfVxuXG4gIHN0b3JhYmxlUmVmZXJlbnRpYWxGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IGZldGNoUHJvcHMgPSBbXTtcbiAgICBjb25zdCByZWZlcmVuY2VWZWMgPSBbXTtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0KSB7XG4gICAgICBsZXQgc2lQcm9wZXJ0aWVzID0gdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmdldEVudHJ5KFwic2lQcm9wZXJ0aWVzXCIpO1xuICAgICAgaWYgKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHNpUHJvcGVydGllcyA9IHNpUHJvcGVydGllcy5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmICghKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpKSB7XG4gICAgICAgIHRocm93IFwiQ2Fubm90IGdldCBwcm9wZXJ0aWVzIG9mIGEgbm9uIG9iamVjdCBpbiByZWYgY2hlY2tcIjtcbiAgICAgIH1cbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBzaVByb3BlcnRpZXMucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBpZiAocHJvcC5yZWZlcmVuY2UpIHtcbiAgICAgICAgICBjb25zdCBpdGVtTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc01hbnkoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNPbmUoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEJhc2VPYmplY3QpIHtcbiAgICB9XG5cbiAgICBpZiAoZmV0Y2hQcm9wcy5sZW5ndGggJiYgcmVmZXJlbmNlVmVjLmxlbmd0aCkge1xuICAgICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgICAgcmVzdWx0cy5wdXNoKGZldGNoUHJvcHMuam9pbihcIlxcblwiKSk7XG4gICAgICByZXN1bHRzLnB1c2goYHZlYyFbJHtyZWZlcmVuY2VWZWMuam9pbihcIixcIil9XWApO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiVmVjOjpuZXcoKVwiO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlclNlcnZpY2Uge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzeXN0ZW1PYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzLm1hcChvID0+IG5ldyBSdXN0Rm9ybWF0dGVyKG8pKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Qm9keSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcInB1YiBkYjogc2lfZGF0YTo6RGIsXCJdO1xuICAgIGlmICh0aGlzLmhhc0NvbXBvbmVudHMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJwdWIgYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnQsXCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdENvbnN0cnVjdG9yUmV0dXJuKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wiZGJcIl07XG4gICAgaWYgKHRoaXMuaGFzQ29tcG9uZW50cygpKSB7XG4gICAgICByZXN1bHQucHVzaChcImFnZW50XCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VOZXdDb25zdHJ1Y3RvckFyZ3MoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5oYXNDb21wb25lbnRzKCkpIHtcbiAgICAgIHJldHVybiBcImRiOiBzaV9kYXRhOjpEYiwgYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnRcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiXCI7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VUcmFpdE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtzbmFrZUNhc2UoXG4gICAgICB0aGlzLnNlcnZpY2VOYW1lLFxuICAgICl9X3NlcnZlcjo6JHtwYXNjYWxDYXNlKHRoaXMuc2VydmljZU5hbWUpfWA7XG4gIH1cblxuICBoYXNDb21wb25lbnRzKCk6IGJvb2xlYW4ge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdHMuZmluZChzID0+IHMua2luZCgpID09IFwiY29tcG9uZW50XCIpKSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXG4gICAgICBcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsXG4gICAgICBcIi8vIE5vIHRvdWNoeSFcIixcbiAgICAgIFwiXCIsXG4gICAgICBcInB1YiBtb2QgbW9kZWw7XCIsXG4gICAgICBcInB1YiBtb2Qgc2VydmljZTtcIixcbiAgICBdO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vbW9kZWwvbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgZm9yIChjb25zdCBzeXN0ZW1PYmplY3Qgb2YgcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApKSB7XG4gICAgICBpZiAoc3lzdGVtT2JqZWN0LmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIikge1xuICAgICAgICByZXN1bHRzLnB1c2goYHB1YiBtb2QgJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX07YCk7XG4gICAgICB9XG4gICAgfVxuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZGVsL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5TZXJ2aWNlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdydXN0L3NlcnZpY2UucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyU2VydmljZSh0aGlzLnNlcnZpY2VOYW1lKSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBfX2ZpbGVuYW1lLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vc2VydmljZS5yc2AsIG91dHB1dCk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgncnVzdC9tb2RlbC5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXIoc3lzdGVtT2JqZWN0KSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBfX2ZpbGVuYW1lLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFxuICAgICAgYGdlbi9tb2RlbC8ke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfS5yc2AsXG4gICAgICBvdXRwdXQsXG4gICAgKTtcbiAgfVxuXG4gIGFzeW5jIG1ha2VQYXRoKHBhdGhQYXJ0OiBzdHJpbmcpOiBQcm9taXNlPHN0cmluZz4ge1xuICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuICAgICAgX19kaXJuYW1lLFxuICAgICAgXCIuLlwiLFxuICAgICAgXCIuLlwiLFxuICAgICAgXCIuLlwiLFxuICAgICAgYHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gLFxuICAgICAgXCJzcmNcIixcbiAgICAgIHBhdGhQYXJ0LFxuICAgICk7XG4gICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4gICAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIocGF0aC5yZXNvbHZlKHBhdGhOYW1lKSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4gICAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4gIH1cblxuICBhc3luYyBmb3JtYXRDb2RlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGF3YWl0IGV4ZWNDbWQoYGNhcmdvIGZtdCAtcCBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCk7XG4gIH1cblxuICBhc3luYyB3cml0ZUNvZGUoZmlsZW5hbWU6IHN0cmluZywgY29kZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcGF0aG5hbWUgPSBwYXRoLmRpcm5hbWUoZmlsZW5hbWUpO1xuICAgIGNvbnN0IGJhc2VuYW1lID0gcGF0aC5iYXNlbmFtZShmaWxlbmFtZSk7XG4gICAgY29uc3QgY3JlYXRlZFBhdGggPSBhd2FpdCB0aGlzLm1ha2VQYXRoKHBhdGhuYW1lKTtcbiAgICBjb25zdCBjb2RlRmlsZW5hbWUgPSBwYXRoLmpvaW4oY3JlYXRlZFBhdGgsIGJhc2VuYW1lKTtcbiAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoY29kZUZpbGVuYW1lLCBjb2RlKTtcbiAgfVxufVxuXG4vLyBleHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuLy8gICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuLy8gICBmb3JtYXR0ZXI6IFJ1c3RGb3JtYXR0ZXI7XG4vL1xuLy8gICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzKSB7XG4vLyAgICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4vLyAgICAgdGhpcy5mb3JtYXR0ZXIgPSBuZXcgUnVzdEZvcm1hdHRlcihzeXN0ZW1PYmplY3QpO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyB3cml0ZUNvZGUocGFydDogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBjcmVhdGVkUGF0aCA9IGF3YWl0IHRoaXMubWFrZVBhdGgoKTtcbi8vICAgICBjb25zdCBjb2RlRmlsZW5hbWUgPSBwYXRoLmpvaW4oY3JlYXRlZFBhdGgsIGAke3NuYWtlQ2FzZShwYXJ0KX0ucnNgKTtcbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoY29kZUZpbGVuYW1lLCBjb2RlKTtcbi8vICAgICBhd2FpdCBleGVjQ21kKGBydXN0Zm10ICR7Y29kZUZpbGVuYW1lfWApO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBtYWtlUGF0aCgpOiBQcm9taXNlPHN0cmluZz4ge1xuLy8gICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgICAgX19kaXJuYW1lLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgdGhpcy5zeXN0ZW1PYmplY3Quc2lQYXRoTmFtZSxcbi8vICAgICAgIFwic3JjXCIsXG4vLyAgICAgICBcImdlblwiLFxuLy8gICAgICAgc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKSxcbi8vICAgICApO1xuLy8gICAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKHBhdGgucmVzb2x2ZShwYXRoTmFtZSksIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuLy8gICAgIHJldHVybiBhYnNvbHV0ZVBhdGhOYW1lO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBnZW5lcmF0ZUNvbXBvbmVudEltcGxzKCk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4vLyAgICAgICBcIjwlLSBpbmNsdWRlKCdydXN0L2NvbXBvbmVudC5ycy5lanMnLCB7IGNvbXBvbmVudDogY29tcG9uZW50IH0pICU+XCIsXG4vLyAgICAgICB7XG4vLyAgICAgICAgIHN5c3RlbU9iamVjdDogdGhpcy5zeXN0ZW1PYmplY3QsXG4vLyAgICAgICAgIGZtdDogdGhpcy5mb3JtYXR0ZXIsXG4vLyAgICAgICB9LFxuLy8gICAgICAge1xuLy8gICAgICAgICBmaWxlbmFtZTogX19maWxlbmFtZSxcbi8vICAgICAgIH0sXG4vLyAgICAgKTtcbi8vICAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImNvbXBvbmVudFwiLCBvdXRwdXQpO1xuLy8gICB9XG4vL1xuLy8gICBhc3luYyBnZW5lcmF0ZUNvbXBvbmVudE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBtb2RzID0gW1wiY29tcG9uZW50XCJdO1xuLy8gICAgIGNvbnN0IGxpbmVzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyBUb3VjaHkhXFxuXCJdO1xuLy8gICAgIGZvciAoY29uc3QgbW9kIG9mIG1vZHMpIHtcbi8vICAgICAgIGxpbmVzLnB1c2goYHB1YiBtb2QgJHttb2R9O2ApO1xuLy8gICAgIH1cbi8vICAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcIm1vZFwiLCBsaW5lcy5qb2luKFwiXFxuXCIpKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyIHtcbi8vICAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcbi8vXG4vLyAgIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogUnVzdEZvcm1hdHRlcltcInN5c3RlbU9iamVjdFwiXSkge1xuLy8gICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRUeXBlTmFtZSgpOiBzdHJpbmcge1xuLy8gICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRPcmRlckJ5RmllbGRzKCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3Qgb3JkZXJCeUZpZWxkcyA9IFtdO1xuLy8gICAgIGNvbnN0IGNvbXBvbmVudE9iamVjdCA9IHRoaXMuY29tcG9uZW50LmFzQ29tcG9uZW50KCk7XG4vLyAgICAgZm9yIChjb25zdCBwIG9mIGNvbXBvbmVudE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICBpZiAocC5oaWRkZW4pIHtcbi8vICAgICAgICAgY29udGludWU7XG4vLyAgICAgICB9XG4vLyAgICAgICBpZiAocC5uYW1lID09IFwic3RvcmFibGVcIikge1xuLy8gICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goJ1wic3RvcmFibGUubmF0dXJhbEtleVwiJyk7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaCgnXCJzdG9yYWJsZS50eXBlTmFtZVwiJyk7XG4vLyAgICAgICB9IGVsc2UgaWYgKHAubmFtZSA9PSBcInNpUHJvcGVydGllc1wiKSB7XG4vLyAgICAgICAgIGNvbnRpbnVlO1xuLy8gICAgICAgfSBlbHNlIGlmIChwLm5hbWUgPT0gXCJjb25zdHJhaW50c1wiICYmIHAua2luZCgpID09IFwib2JqZWN0XCIpIHtcbi8vICAgICAgICAgLy8gQHRzLWlnbm9yZSB0cnVzdCB1cyAtIHdlIGNoZWNrZWRcbi8vICAgICAgICAgZm9yIChjb25zdCBwYyBvZiBwLnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgICAgICBpZiAocGMua2luZCgpICE9IFwib2JqZWN0XCIpIHtcbi8vICAgICAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaChgXCJjb25zdHJhaW50cy4ke3BjLm5hbWV9XCJgKTtcbi8vICAgICAgICAgICB9XG4vLyAgICAgICAgIH1cbi8vICAgICAgIH0gZWxzZSB7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaChgXCIke3AubmFtZX1cImApO1xuLy8gICAgICAgfVxuLy8gICAgIH1cbi8vICAgICByZXR1cm4gYHZlYyFbJHtvcmRlckJ5RmllbGRzLmpvaW4oXCIsXCIpfV1cXG5gO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRJbXBvcnRzKCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3QgcmVzdWx0ID0gW107XG4vLyAgICAgcmVzdWx0LnB1c2goXG4vLyAgICAgICBgcHViIHVzZSBjcmF0ZTo6cHJvdG9idWY6OiR7c25ha2VDYXNlKHRoaXMuY29tcG9uZW50LnR5cGVOYW1lKX06OntgLFxuLy8gICAgICAgYCAgQ29uc3RyYWludHMsYCxcbi8vICAgICAgIGAgIExpc3RDb21wb25lbnRzUmVwbHksYCxcbi8vICAgICAgIGAgIExpc3RDb21wb25lbnRzUmVxdWVzdCxgLFxuLy8gICAgICAgYCAgUGlja0NvbXBvbmVudFJlcXVlc3QsYCxcbi8vICAgICAgIGAgIENvbXBvbmVudCxgLFxuLy8gICAgICAgYH07YCxcbi8vICAgICApO1xuLy8gICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50VmFsaWRhdGlvbigpOiBzdHJpbmcge1xuLy8gICAgIHJldHVybiB0aGlzLmdlblZhbGlkYXRpb24odGhpcy5jb21wb25lbnQuYXNDb21wb25lbnQoKSk7XG4vLyAgIH1cbi8vXG4vLyAgIGdlblZhbGlkYXRpb24ocHJvcE9iamVjdDogUHJvcE9iamVjdCk6IHN0cmluZyB7XG4vLyAgICAgY29uc3QgcmVzdWx0ID0gW107XG4vLyAgICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbi8vICAgICAgICAgY29uc3QgcHJvcE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbi8vICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0uaXNfbm9uZSgpIHtcbi8vICAgICAgICAgICByZXR1cm4gRXJyKERhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuLy8gICAgICAgICB9YCk7XG4vLyAgICAgICB9XG4vLyAgICAgfVxuLy8gICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZW5lcmF0ZUdlbk1vZCh3cml0dGVuQ29tcG9uZW50czoge1xuLy8gICBba2V5OiBzdHJpbmddOiBzdHJpbmdbXTtcbi8vIH0pOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgZm9yIChjb25zdCBjb21wb25lbnQgaW4gd3JpdHRlbkNvbXBvbmVudHMpIHtcbi8vICAgICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICAgIF9fZGlybmFtZSxcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIGNvbXBvbmVudCxcbi8vICAgICAgIFwic3JjXCIsXG4vLyAgICAgICBcImdlblwiLFxuLy8gICAgICk7XG4vLyAgICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgICAgY29uc3QgY29kZSA9IFtcbi8vICAgICAgIFwiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIixcbi8vICAgICAgIFwiLy8gTm8gdG91Y2h5IVwiLFxuLy8gICAgICAgXCJcIixcbi8vICAgICAgIFwicHViIG1vZCBtb2RlbDtcIixcbi8vICAgICBdO1xuLy9cbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoXG4vLyAgICAgICBwYXRoLmpvaW4oYWJzb2x1dGVQYXRoTmFtZSwgXCJtb2QucnNcIiksXG4vLyAgICAgICBjb2RlLmpvaW4oXCJcXG5cIiksXG4vLyAgICAgKTtcbi8vICAgfVxuLy8gfVxuLy9cbi8vIGV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZW5lcmF0ZUdlbk1vZE1vZGVsKHNlcnZpY2VOYW1lOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgX19kaXJuYW1lLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIHNlcnZpY2VOYW1lLFxuLy8gICAgIFwic3JjXCIsXG4vLyAgICAgXCJnZW5cIixcbi8vICAgICBcIm1vZGVsXCIsXG4vLyAgICk7XG4vLyAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICBjb25zdCBjb2RlID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXFxuXCJdO1xuLy8gICBmb3IgKGNvbnN0IHR5cGVOYW1lIG9mIHdyaXR0ZW5Db21wb25lbnRzW2NvbXBvbmVudF0pIHtcbi8vICAgICBjb2RlLnB1c2goYHB1YiBtb2QgJHtzbmFrZUNhc2UodHlwZU5hbWUpfTtgKTtcbi8vICAgfVxuLy9cbi8vICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKFxuLy8gICAgIHBhdGguam9pbihhYnNvbHV0ZVBhdGhOYW1lLCBcIm1vZC5yc1wiKSxcbi8vICAgICBjb2RlLmpvaW4oXCJcXG5cIiksXG4vLyAgICk7XG4vLyB9XG4iXX0=