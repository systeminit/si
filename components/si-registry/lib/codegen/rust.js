"use strict";

var _interopRequireWildcard = require("@babel/runtime/helpers/interopRequireWildcard");

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.CodegenRust = exports.RustFormatterAgent = exports.RustFormatterService = exports.RustFormatter = void 0;

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
    key: "hasEditEithersForAction",
    value: function hasEditEithersForAction(propAction) {
      return this.entityEditProperty(propAction).relationships.all().some(function (rel) {
        return rel instanceof PropPrelude.Either;
      });
    }
  }, {
    key: "hasEditUpdatesForAction",
    value: function hasEditUpdatesForAction(propAction) {
      return this.entityEditProperty(propAction).relationships.all().some(function (rel) {
        return rel instanceof PropPrelude.Updates;
      });
    }
  }, {
    key: "hasEditUpdatesAndEithers",
    value: function hasEditUpdatesAndEithers() {
      var _this = this;

      if (this.isEntityObject()) {
        return this.entityEditMethods().some(function (propAction) {
          return _this.hasEditUpdatesForAction(propAction) && _this.hasEditUpdatesForAction(propAction);
        });
      } else {
        throw "You ran 'hasEditUpdatesAndEithers()' on a non-entity object; this is a bug!";
      }
    }
  }, {
    key: "isComponentObject",
    value: function isComponentObject() {
      return this.systemObject instanceof _systemComponent.ComponentObject;
    }
  }, {
    key: "isEntityActionMethod",
    value: function isEntityActionMethod(propMethod) {
      return this.isEntityObject() && propMethod instanceof PropPrelude.PropAction;
    }
  }, {
    key: "isEntityEditMethod",
    value: function isEntityEditMethod(propMethod) {
      return this.isEntityActionMethod(propMethod) && propMethod.name.endsWith("Edit");
    }
  }, {
    key: "isEntityEventObject",
    value: function isEntityEventObject() {
      return this.systemObject instanceof _systemComponent.EntityEventObject;
    }
  }, {
    key: "isEntityObject",
    value: function isEntityObject() {
      return this.systemObject instanceof _systemComponent.EntityObject;
    }
  }, {
    key: "isMigrateable",
    value: function isMigrateable() {
      return this.systemObject instanceof _systemComponent.SystemObject && this.systemObject.migrateable;
    }
  }, {
    key: "isStorable",
    value: function isStorable() {
      return this.systemObject instanceof _systemComponent.SystemObject;
    }
  }, {
    key: "actionProps",
    value: function actionProps() {
      return this.systemObject.methods.attrs.filter(function (m) {
        return m instanceof PropPrelude.PropAction;
      });
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
    key: "entityEditMethods",
    value: function entityEditMethods() {
      var _this2 = this;

      return this.actionProps().filter(function (p) {
        return _this2.isEntityEditMethod(p);
      });
    }
  }, {
    key: "entityEditProperty",
    value: function entityEditProperty(propAction) {
      var property = propAction.request.properties.getEntry("property");

      if (property instanceof PropPrelude.PropLink) {
        property = property.lookupMyself();
      }

      return property;
    }
  }, {
    key: "entityEditPropertyField",
    value: function entityEditPropertyField(propAction) {
      return this.rustFieldNameForProp(this.entityEditProperty(propAction));
    }
  }, {
    key: "entityEditPropertyType",
    value: function entityEditPropertyType(propAction) {
      return this.rustTypeForProp(this.entityEditProperty(propAction), {
        option: false
      });
    }
  }, {
    key: "entityEditPropertyUpdates",
    value: function entityEditPropertyUpdates(propAction) {
      var _this3 = this;

      return this.entityEditProperty(propAction).relationships.all().filter(function (r) {
        return r instanceof PropPrelude.Updates;
      }).map(function (update) {
        return {
          from: _this3.entityEditProperty(propAction),
          to: update.partnerProp()
        };
      });
    }
  }, {
    key: "entityEditPropertyEithers",
    value: function entityEditPropertyEithers() {
      var results = new Set();
      return Array.from(results).sort();
    }
  }, {
    key: "entityEditPropertyUpdateMethodName",
    value: function entityEditPropertyUpdateMethodName(propertyUpdate) {
      return "update_".concat(this.rustFieldNameForProp(propertyUpdate.to), "_from_").concat(this.rustFieldNameForProp(propertyUpdate.from));
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
    key: "errorType",
    value: function errorType() {
      return "crate::error::".concat((0, _changeCase.pascalCase)(this.systemObject.serviceName), "Error");
    }
  }, {
    key: "modelName",
    value: function modelName() {
      return "crate::model::".concat((0, _changeCase.pascalCase)(this.systemObject.typeName));
    }
  }, {
    key: "modelServiceMethodName",
    value: function modelServiceMethodName(propMethod) {
      return this.rustFieldNameForProp(propMethod);
    }
  }, {
    key: "structName",
    value: function structName() {
      return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.typeName));
    }
  }, {
    key: "typeName",
    value: function typeName() {
      return (0, _changeCase.snakeCase)(this.systemObject.typeName);
    }
  }, {
    key: "implTryFromForPropertyUpdate",
    value: function implTryFromForPropertyUpdate(propertyUpdate) {
      var from = propertyUpdate.from;
      var to = propertyUpdate.to; // Every fallthrough/default/else needs a `throw` clause to loudly proclaim
      // that a specific conversion is not supported. This allows us to add
      // conversions as we go without rogue and unexplained errors. In short,
      // treat this like Rust code with fully satisfied match arms. Thank you,
      // love, us.

      if (from instanceof PropPrelude.PropCode) {
        switch (from.language) {
          case "yaml":
            if (to instanceof PropPrelude.PropObject) {
              return "Ok(serde_yaml::from_str(value)?)";
            } else {
              throw "conversion from language '".concat(from.language, "' to type '").concat(to.kind(), "' is not supported");
            }

          default:
            throw "conversion from language '".concat(from.language, "' is not supported");
        }
      } else if (from instanceof PropPrelude.PropObject) {
        if (to instanceof PropPrelude.PropCode) {
          switch (to.language) {
            case "yaml":
              return "Ok(serde_yaml::to_string(value)?)";

            default:
              throw "conversion from PropObject to language '".concat(to.language, "' is not supported");
          }
        } else {
          throw "conversion from PropObject to type '".concat(to.kind(), "' is not supported");
        }
      } else {
        throw "conversion from type '".concat(from.kind(), "' to type '").concat(to.kind(), "' is not supported");
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

          if (this.isMigrateable(systemObj)) {
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
      return this.systemObjects.some(function (obj) {
        return obj instanceof _systemComponent.EntityObject;
      });
    }
  }, {
    key: "isMigrateable",
    value: function isMigrateable(prop) {
      return prop instanceof _systemComponent.SystemObject && prop.migrateable;
    }
  }, {
    key: "hasMigratables",
    value: function hasMigratables() {
      var _this4 = this;

      return this.systemObjects.some(function (obj) {
        return _this4.isMigrateable(obj);
      });
    }
  }]);
  return RustFormatterService;
}();

exports.RustFormatterService = RustFormatterService;

var RustFormatterAgent = /*#__PURE__*/function () {
  function RustFormatterAgent(serviceName, agent) {
    (0, _classCallCheck2["default"])(this, RustFormatterAgent);
    (0, _defineProperty2["default"])(this, "agentName", void 0);
    (0, _defineProperty2["default"])(this, "entity", void 0);
    (0, _defineProperty2["default"])(this, "entityFormatter", void 0);
    (0, _defineProperty2["default"])(this, "integrationName", void 0);
    (0, _defineProperty2["default"])(this, "integrationServiceName", void 0);
    (0, _defineProperty2["default"])(this, "serviceName", void 0);
    (0, _defineProperty2["default"])(this, "systemObjects", void 0);
    this.agentName = agent.agentName;
    this.entity = agent.entity;
    this.entityFormatter = new RustFormatter(this.entity);
    this.integrationName = agent.integrationName;
    this.integrationServiceName = agent.integrationServiceName;
    this.serviceName = serviceName;
    this.systemObjects = _registry.registry.getObjectsForServiceName(serviceName);
  }

  (0, _createClass2["default"])(RustFormatterAgent, [{
    key: "systemObjectsAsFormatters",
    value: function systemObjectsAsFormatters() {
      return this.systemObjects.sort(function (a, b) {
        return a.typeName > b.typeName ? 1 : -1;
      }).map(function (o) {
        return new RustFormatter(o);
      });
    }
  }, {
    key: "actionProps",
    value: function actionProps() {
      return this.entity.methods.attrs.filter(function (m) {
        return m instanceof PropPrelude.PropAction;
      });
    }
  }, {
    key: "entityActionMethodNames",
    value: function entityActionMethodNames() {
      var results = ["create"];

      var _iterator12 = _createForOfIteratorHelper(this.actionProps()),
          _step12;

      try {
        for (_iterator12.s(); !(_step12 = _iterator12.n()).done;) {
          var prop = _step12.value;

          if (this.entityFormatter.isEntityEditMethod(prop)) {
            results.push(this.entityFormatter.entityEditMethodName(prop));
          } else {
            results.push(prop.name);
          }
        }
      } catch (err) {
        _iterator12.e(err);
      } finally {
        _iterator12.f();
      }

      return results;
    }
  }, {
    key: "dispatcherBaseTypeName",
    value: function dispatcherBaseTypeName() {
      return "".concat((0, _changeCase.pascalCase)(this.integrationName)).concat((0, _changeCase.pascalCase)(this.integrationServiceName)).concat((0, _changeCase.pascalCase)(this.entity.baseTypeName));
    }
  }, {
    key: "dispatcherTypeName",
    value: function dispatcherTypeName() {
      return "".concat(this.dispatcherBaseTypeName(), "Dispatcher");
    }
  }, {
    key: "dispatchFunctionTraitName",
    value: function dispatchFunctionTraitName() {
      return "".concat(this.dispatcherBaseTypeName(), "DispatchFunctions");
    }
  }]);
  return RustFormatterAgent;
}();

exports.RustFormatterAgent = RustFormatterAgent;

var CodegenRust = /*#__PURE__*/function () {
  function CodegenRust(serviceName) {
    (0, _classCallCheck2["default"])(this, CodegenRust);
    (0, _defineProperty2["default"])(this, "serviceName", void 0);
    this.serviceName = serviceName;
  }

  (0, _createClass2["default"])(CodegenRust, [{
    key: "hasModels",
    value: function hasModels() {
      return _registry.registry.getObjectsForServiceName(this.serviceName).some(function (o) {
        return o.kind() != "baseObject";
      });
    }
  }, {
    key: "hasServiceMethods",
    value: function hasServiceMethods() {
      return _registry.registry.getObjectsForServiceName(this.serviceName).flatMap(function (o) {
        return o.methods.attrs;
      }).length > 0;
    }
  }, {
    key: "hasEntityIntegrationServcices",
    value: function hasEntityIntegrationServcices() {
      var _this5 = this;

      var integrationServices = new Set(this.entities().flatMap(function (entity) {
        return _this5.entityintegrationServicesFor(entity);
      }));
      return integrationServices.size > 0;
    }
  }, {
    key: "entities",
    value: function entities() {
      return _registry.registry.getObjectsForServiceName(this.serviceName).filter(function (o) {
        return o instanceof _systemComponent.EntityObject;
      });
    }
  }, {
    key: "entityActions",
    value: function entityActions(entity) {
      return entity.methods.attrs.filter(function (m) {
        return m instanceof PropPrelude.PropAction;
      });
    }
  }, {
    key: "entityintegrationServicesFor",
    value: function entityintegrationServicesFor(entity) {
      var result = new Set();

      var _iterator13 = _createForOfIteratorHelper(entity.integrationServices),
          _step13;

      try {
        for (_iterator13.s(); !(_step13 = _iterator13.n()).done;) {
          var integrationService = _step13.value;
          result.add(integrationService);
        }
      } catch (err) {
        _iterator13.e(err);
      } finally {
        _iterator13.f();
      }

      var _iterator14 = _createForOfIteratorHelper(this.entityActions(entity)),
          _step14;

      try {
        for (_iterator14.s(); !(_step14 = _iterator14.n()).done;) {
          var action = _step14.value;

          var _iterator15 = _createForOfIteratorHelper(action.integrationServices),
              _step15;

          try {
            for (_iterator15.s(); !(_step15 = _iterator15.n()).done;) {
              var _integrationService = _step15.value;
              result.add(_integrationService);
            }
          } catch (err) {
            _iterator15.e(err);
          } finally {
            _iterator15.f();
          }
        }
      } catch (err) {
        _iterator14.e(err);
      } finally {
        _iterator14.f();
      }

      return Array.from(result);
    }
  }, {
    key: "entityIntegrationServices",
    value: function entityIntegrationServices() {
      var _this6 = this;

      return this.entities().flatMap(function (entity) {
        return _this6.entityintegrationServicesFor(entity).map(function (integrationService) {
          return {
            integrationName: integrationService.integrationName,
            integrationServiceName: integrationService.integrationServiceName,
            entity: entity,
            agentName: "".concat((0, _changeCase.snakeCase)(integrationService.integrationName), "_").concat((0, _changeCase.snakeCase)(integrationService.integrationServiceName), "_").concat((0, _changeCase.snakeCase)(entity.baseTypeName))
          };
        });
      });
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
                results = ["// Auto-generated code!", "// No touchy!", ""];

                if (this.hasEntityIntegrationServcices()) {
                  results.push("pub mod agent;");
                }

                if (this.hasModels()) {
                  results.push("pub mod model;");
                }

                if (this.hasServiceMethods()) {
                  results.push("pub mod service;");
                }

                _context.next = 6;
                return this.writeCode("gen/mod.rs", results.join("\n"));

              case 6:
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
        var results, _iterator16, _step16, systemObject;

        return _regenerator["default"].wrap(function _callee2$(_context2) {
          while (1) {
            switch (_context2.prev = _context2.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator16 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(this.serviceName));

                try {
                  for (_iterator16.s(); !(_step16 = _iterator16.n()).done;) {
                    systemObject = _step16.value;

                    if (systemObject.kind() != "baseObject") {
                      results.push("pub mod ".concat((0, _changeCase.snakeCase)(systemObject.typeName), ";"));
                    }
                  }
                } catch (err) {
                  _iterator16.e(err);
                } finally {
                  _iterator16.f();
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
    }() // Generate the 'gen/agent/mod.rs'

  }, {
    key: "generateGenAgentMod",
    value: function () {
      var _generateGenAgentMod = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee5() {
        var results, _iterator17, _step17, agent, _iterator18, _step18, _agent, fmt;

        return _regenerator["default"].wrap(function _callee5$(_context5) {
          while (1) {
            switch (_context5.prev = _context5.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator17 = _createForOfIteratorHelper(this.entityIntegrationServices());

                try {
                  for (_iterator17.s(); !(_step17 = _iterator17.n()).done;) {
                    agent = _step17.value;
                    results.push("pub mod ".concat(agent.agentName, ";"));
                  }
                } catch (err) {
                  _iterator17.e(err);
                } finally {
                  _iterator17.f();
                }

                results.push("");
                _iterator18 = _createForOfIteratorHelper(this.entityIntegrationServices());

                try {
                  for (_iterator18.s(); !(_step18 = _iterator18.n()).done;) {
                    _agent = _step18.value;
                    fmt = new RustFormatterAgent(this.serviceName, _agent);
                    results.push("pub use ".concat(_agent.agentName, "::{").concat(fmt.dispatchFunctionTraitName(), ", ").concat(fmt.dispatcherTypeName(), "};"));
                  }
                } catch (err) {
                  _iterator18.e(err);
                } finally {
                  _iterator18.f();
                }

                _context5.next = 8;
                return this.writeCode("gen/agent/mod.rs", results.join("\n"));

              case 8:
              case "end":
                return _context5.stop();
            }
          }
        }, _callee5, this);
      }));

      function generateGenAgentMod() {
        return _generateGenAgentMod.apply(this, arguments);
      }

      return generateGenAgentMod;
    }()
  }, {
    key: "generateGenAgent",
    value: function () {
      var _generateGenAgent = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee6(agent) {
        var output;
        return _regenerator["default"].wrap(function _callee6$(_context6) {
          while (1) {
            switch (_context6.prev = _context6.next) {
              case 0:
                output = _ejs["default"].render("<%- include('src/codegen/rust/agent.rs.ejs', { fmt: fmt }) %>", {
                  fmt: new RustFormatterAgent(this.serviceName, agent)
                }, {
                  filename: "."
                });
                _context6.next = 3;
                return this.writeCode("gen/agent/".concat((0, _changeCase.snakeCase)(agent.agentName), ".rs"), output);

              case 3:
              case "end":
                return _context6.stop();
            }
          }
        }, _callee6, this);
      }));

      function generateGenAgent(_x2) {
        return _generateGenAgent.apply(this, arguments);
      }

      return generateGenAgent;
    }()
  }, {
    key: "makePath",
    value: function () {
      var _makePath = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee7(pathPart) {
        var pathName, absolutePathName;
        return _regenerator["default"].wrap(function _callee7$(_context7) {
          while (1) {
            switch (_context7.prev = _context7.next) {
              case 0:
                pathName = _path["default"].join("..", "si-".concat(this.serviceName), "src", pathPart);
                absolutePathName = _path["default"].resolve(pathName);
                _context7.next = 4;
                return _fs["default"].promises.mkdir(_path["default"].resolve(pathName), {
                  recursive: true
                });

              case 4:
                return _context7.abrupt("return", absolutePathName);

              case 5:
              case "end":
                return _context7.stop();
            }
          }
        }, _callee7, this);
      }));

      function makePath(_x3) {
        return _makePath.apply(this, arguments);
      }

      return makePath;
    }()
  }, {
    key: "formatCode",
    value: function () {
      var _formatCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee8() {
        return _regenerator["default"].wrap(function _callee8$(_context8) {
          while (1) {
            switch (_context8.prev = _context8.next) {
              case 0:
                _context8.next = 2;
                return execCmd("cargo fmt -p si-".concat(this.serviceName));

              case 2:
              case "end":
                return _context8.stop();
            }
          }
        }, _callee8, this);
      }));

      function formatCode() {
        return _formatCode.apply(this, arguments);
      }

      return formatCode;
    }()
  }, {
    key: "writeCode",
    value: function () {
      var _writeCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee9(filename, code) {
        var pathname, basename, createdPath, codeFilename;
        return _regenerator["default"].wrap(function _callee9$(_context9) {
          while (1) {
            switch (_context9.prev = _context9.next) {
              case 0:
                pathname = _path["default"].dirname(filename);
                basename = _path["default"].basename(filename);
                _context9.next = 4;
                return this.makePath(pathname);

              case 4:
                createdPath = _context9.sent;
                codeFilename = _path["default"].join(createdPath, basename);
                _context9.next = 8;
                return _fs["default"].promises.writeFile(codeFilename, code);

              case 8:
              case "end":
                return _context9.stop();
            }
          }
        }, _callee9, this);
      }));

      function writeCode(_x4, _x5) {
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsIm1ldGhvZHMiLCJnZXRFbnRyeSIsInByb3BBY3Rpb24iLCJlbnRpdHlFZGl0UHJvcGVydHkiLCJyZWxhdGlvbnNoaXBzIiwiYWxsIiwic29tZSIsInJlbCIsIlByb3BQcmVsdWRlIiwiRWl0aGVyIiwiVXBkYXRlcyIsImlzRW50aXR5T2JqZWN0IiwiZW50aXR5RWRpdE1ldGhvZHMiLCJoYXNFZGl0VXBkYXRlc0ZvckFjdGlvbiIsIkNvbXBvbmVudE9iamVjdCIsInByb3BNZXRob2QiLCJQcm9wQWN0aW9uIiwiaXNFbnRpdHlBY3Rpb25NZXRob2QiLCJuYW1lIiwiZW5kc1dpdGgiLCJFbnRpdHlFdmVudE9iamVjdCIsIkVudGl0eU9iamVjdCIsIlN5c3RlbU9iamVjdCIsIm1pZ3JhdGVhYmxlIiwiYXR0cnMiLCJmaWx0ZXIiLCJtIiwiYmFzZVR5cGVOYW1lIiwicnVzdEZpZWxkTmFtZUZvclByb3AiLCJyZXBsYWNlIiwiYWN0aW9uUHJvcHMiLCJwIiwiaXNFbnRpdHlFZGl0TWV0aG9kIiwicHJvcGVydHkiLCJyZXF1ZXN0IiwicHJvcGVydGllcyIsIlByb3BMaW5rIiwibG9va3VwTXlzZWxmIiwicnVzdFR5cGVGb3JQcm9wIiwib3B0aW9uIiwiciIsIm1hcCIsInVwZGF0ZSIsImZyb20iLCJ0byIsInBhcnRuZXJQcm9wIiwicmVzdWx0cyIsIlNldCIsIkFycmF5Iiwic29ydCIsInByb3BlcnR5VXBkYXRlIiwic2VydmljZU5hbWUiLCJ0eXBlTmFtZSIsIlByb3BDb2RlIiwibGFuZ3VhZ2UiLCJQcm9wT2JqZWN0Iiwia2luZCIsInJlbmRlck9wdGlvbnMiLCJsaXN0IiwicmVwbHkiLCJyZWZlcmVuY2UiLCJlanMiLCJyZW5kZXIiLCJmbXQiLCJmaWxlbmFtZSIsInNraXBBdXRoIiwiaW1wbFNlcnZpY2VNZXRob2ROYW1lIiwiaW1wbFNlcnZpY2VBdXRoQ2FsbCIsInByZWx1ZGUiLCJwcm9wTWV0aG9kcyIsImEiLCJiIiwib3V0cHV0IiwicHVzaCIsImpvaW4iLCJwcm9wIiwiUHJvcE1ldGhvZCIsInBhcmVudE5hbWUiLCJQcm9wTnVtYmVyIiwibnVtYmVyS2luZCIsIlByb3BCb29sIiwicmVhbFByb3AiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BNYXAiLCJQcm9wVGV4dCIsIlByb3BTZWxlY3QiLCJyZXBlYXRlZCIsInJlc3VsdCIsImNyZWF0ZU1ldGhvZCIsImxpc3RNZXRob2QiLCJmaWVsZE5hbWUiLCJsaXN0UmVwbHlWYWx1ZSIsIm5hdHVyYWxLZXkiLCJ2YXJpYWJsZU5hbWUiLCJQcm9wUGFzc3dvcmQiLCJmaWVsZHMiLCJkZWZhdWx0VmFsdWUiLCJlbnVtTmFtZSIsIm12Y2MiLCJyZXF1aXJlZCIsInByb3BOYW1lIiwidG9wUHJvcCIsInByZWZpeCIsImhpZGRlbiIsInN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcCIsInJvb3RQcm9wIiwiZmV0Y2hQcm9wcyIsInJlZmVyZW5jZVZlYyIsInNpUHJvcGVydGllcyIsIml0ZW1OYW1lIiwiQmFzZU9iamVjdCIsImxlbmd0aCIsIlJ1c3RGb3JtYXR0ZXJTZXJ2aWNlIiwic3lzdGVtT2JqZWN0cyIsInJlZ2lzdHJ5IiwiZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lIiwibyIsImhhc0VudGl0aWVzIiwiaW1wbFNlcnZpY2VUcmFpdE5hbWUiLCJzeXN0ZW1PYmoiLCJpc01pZ3JhdGVhYmxlIiwib2JqIiwiUnVzdEZvcm1hdHRlckFnZW50IiwiYWdlbnQiLCJhZ2VudE5hbWUiLCJlbnRpdHkiLCJlbnRpdHlGb3JtYXR0ZXIiLCJpbnRlZ3JhdGlvbk5hbWUiLCJpbnRlZ3JhdGlvblNlcnZpY2VOYW1lIiwiZW50aXR5RWRpdE1ldGhvZE5hbWUiLCJkaXNwYXRjaGVyQmFzZVR5cGVOYW1lIiwiQ29kZWdlblJ1c3QiLCJmbGF0TWFwIiwiaW50ZWdyYXRpb25TZXJ2aWNlcyIsImVudGl0aWVzIiwiZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvciIsInNpemUiLCJpbnRlZ3JhdGlvblNlcnZpY2UiLCJhZGQiLCJlbnRpdHlBY3Rpb25zIiwiYWN0aW9uIiwiaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMiLCJoYXNNb2RlbHMiLCJoYXNTZXJ2aWNlTWV0aG9kcyIsIndyaXRlQ29kZSIsImVudGl0eUludGVncmF0aW9uU2VydmljZXMiLCJkaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lIiwiZGlzcGF0Y2hlclR5cGVOYW1lIiwicGF0aFBhcnQiLCJwYXRoIiwiYWJzb2x1dGVQYXRoTmFtZSIsInJlc29sdmUiLCJmcyIsInByb21pc2VzIiwibWtkaXIiLCJyZWN1cnNpdmUiLCJjb2RlIiwicGF0aG5hbWUiLCJkaXJuYW1lIiwiYmFzZW5hbWUiLCJtYWtlUGF0aCIsImNyZWF0ZWRQYXRoIiwiY29kZUZpbGVuYW1lIiwid3JpdGVGaWxlIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7QUFRQTs7QUFDQTs7QUFHQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7Ozs7Ozs7QUFFQSxJQUFNQSxPQUFPLEdBQUdDLGlCQUFLQyxTQUFMLENBQWVDLDBCQUFhQyxJQUE1QixDQUFoQjs7SUF1QmFDLGE7QUFHWCx5QkFBWUMsWUFBWixFQUF5RDtBQUFBO0FBQUE7QUFDdkQsU0FBS0EsWUFBTCxHQUFvQkEsWUFBcEI7QUFDRDs7OztzQ0FFMEI7QUFDekIsVUFBSTtBQUNGLGFBQUtBLFlBQUwsQ0FBa0JDLE9BQWxCLENBQTBCQyxRQUExQixDQUFtQyxRQUFuQztBQUNBLGVBQU8sSUFBUDtBQUNELE9BSEQsQ0FHRSxnQkFBTTtBQUNOLGVBQU8sS0FBUDtBQUNEO0FBQ0Y7Ozs0Q0FFdUJDLFUsRUFBNkM7QUFDbkUsYUFBTyxLQUFLQyxrQkFBTCxDQUF3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WixDQUFzQkQsWUFBdEIsRUFBb0NOLElBQXBDLEM7Ozs7Ozs7Ozs7Ozs7Ozs7OztLQUlWO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHtcbiAgT2JqZWN0VHlwZXMsXG4gIEJhc2VPYmplY3QsXG4gIFN5c3RlbU9iamVjdCxcbiAgQ29tcG9uZW50T2JqZWN0LFxuICBFbnRpdHlPYmplY3QsXG4gIEVudGl0eUV2ZW50T2JqZWN0LFxufSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuaW1wb3J0IHsgUHJvcHMsIEludGVncmF0aW9uU2VydmljZSB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuXG5pbXBvcnQgeyBzbmFrZUNhc2UsIHBhc2NhbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBlanMgZnJvbSBcImVqc1wiO1xuaW1wb3J0IGZzIGZyb20gXCJmc1wiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG5cbmNvbnN0IGV4ZWNDbWQgPSB1dGlsLnByb21pc2lmeShjaGlsZFByb2Nlc3MuZXhlYyk7XG5cbmludGVyZmFjZSBSdXN0VHlwZUFzUHJvcE9wdGlvbnMge1xuICByZWZlcmVuY2U/OiBib29sZWFuO1xuICBvcHRpb24/OiBib29sZWFuO1xufVxuXG5pbnRlcmZhY2UgQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2Uge1xuICBhZ2VudE5hbWU6IHN0cmluZztcbiAgZW50aXR5OiBFbnRpdHlPYmplY3Q7XG4gIGludGVncmF0aW9uTmFtZTogc3RyaW5nO1xuICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBzdHJpbmc7XG59XG5cbmludGVyZmFjZSBQcm9wZXJ0eVVwZGF0ZSB7XG4gIGZyb206IFByb3BQcmVsdWRlLlByb3BzO1xuICB0bzogUHJvcFByZWx1ZGUuUHJvcHM7XG59XG5cbmludGVyZmFjZSBQcm9wZXJ0eUVpdGhlclNldCB7XG4gIHNldDogUHJvcFByZWx1ZGUuUHJvcHNbXTtcbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXIge1xuICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogUnVzdEZvcm1hdHRlcltcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgaGFzQ3JlYXRlTWV0aG9kKCk6IGJvb2xlYW4ge1xuICAgIHRyeSB7XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBjYXRjaCB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaGFzRWRpdEVpdGhlcnNGb3JBY3Rpb24ocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKVxuICAgICAgLnJlbGF0aW9uc2hpcHMuYWxsKClcbiAgICAgIC5zb21lKHJlbCA9PiByZWwgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5FaXRoZXIpO1xuICB9XG5cbiAgaGFzRWRpdFVwZGF0ZXNGb3JBY3Rpb24ocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKVxuICAgICAgLnJlbGF0aW9uc2hpcHMuYWxsKClcbiAgICAgIC5zb21lKHJlbCA9PiByZWwgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5VcGRhdGVzKTtcbiAgfVxuXG4gIGhhc0VkaXRVcGRhdGVzQW5kRWl0aGVycygpOiBib29sZWFuIHtcbiAgICBpZiAodGhpcy5pc0VudGl0eU9iamVjdCgpKSB7XG4gICAgICByZXR1cm4gdGhpcy5lbnRpdHlFZGl0TWV0aG9kcygpLnNvbWUoXG4gICAgICAgIHByb3BBY3Rpb24gPT5cbiAgICAgICAgICB0aGlzLmhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uKHByb3BBY3Rpb24pICYmXG4gICAgICAgICAgdGhpcy5oYXNFZGl0VXBkYXRlc0ZvckFjdGlvbihwcm9wQWN0aW9uKSxcbiAgICAgICk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IHJhbiAnaGFzRWRpdFVwZGF0ZXNBbmRFaXRoZXJzKCknIG9uIGEgbm9uLWVudGl0eSBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgaXNDb21wb25lbnRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0O1xuICB9XG5cbiAgaXNFbnRpdHlBY3Rpb25NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLmlzRW50aXR5T2JqZWN0KCkgJiYgcHJvcE1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb25cbiAgICApO1xuICB9XG5cbiAgaXNFbnRpdHlFZGl0TWV0aG9kKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgdGhpcy5pc0VudGl0eUFjdGlvbk1ldGhvZChwcm9wTWV0aG9kKSAmJiBwcm9wTWV0aG9kLm5hbWUuZW5kc1dpdGgoXCJFZGl0XCIpXG4gICAgKTtcbiAgfVxuXG4gIGlzRW50aXR5RXZlbnRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3Q7XG4gIH1cblxuICBpc0VudGl0eU9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3Q7XG4gIH1cblxuICBpc01pZ3JhdGVhYmxlKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCAmJiB0aGlzLnN5c3RlbU9iamVjdC5taWdyYXRlYWJsZVxuICAgICk7XG4gIH1cblxuICBpc1N0b3JhYmxlKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdDtcbiAgfVxuXG4gIGFjdGlvblByb3BzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMuZmlsdGVyKFxuICAgICAgbSA9PiBtIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXTtcbiAgfVxuXG4gIGNvbXBvbmVudE5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9Q29tcG9uZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGNvbXBvbmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgY29tcG9uZW50Q29uc3RyYWludHNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUNvbXBvbmVudENvbnN0cmFpbnRzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGEgY29tcG9uZW50IGNvbnN0cmFpbnRzIG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlFZGl0TWV0aG9kTmFtZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICAgIHJldHVybiBgZWRpdF8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcE1ldGhvZCkucmVwbGFjZShcbiAgICAgICAgXCJfZWRpdFwiLFxuICAgICAgICBcIlwiLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZWRpdCBtZXRob2QgbmFtZSBvbiBhIG5vbi1lbnRpdHkgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eUVkaXRNZXRob2RzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuYWN0aW9uUHJvcHMoKS5maWx0ZXIocCA9PiB0aGlzLmlzRW50aXR5RWRpdE1ldGhvZChwKSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IFByb3BzIHtcbiAgICBsZXQgcHJvcGVydHkgPSBwcm9wQWN0aW9uLnJlcXVlc3QucHJvcGVydGllcy5nZXRFbnRyeShcInByb3BlcnR5XCIpO1xuICAgIGlmIChwcm9wZXJ0eSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBwcm9wZXJ0eSA9IHByb3BlcnR5Lmxvb2t1cE15c2VsZigpO1xuICAgIH1cbiAgICByZXR1cm4gcHJvcGVydHk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlGaWVsZChwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcCh0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlUeXBlKHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcCh0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKSwge1xuICAgICAgb3B0aW9uOiBmYWxzZSxcbiAgICB9KTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZXMoXG4gICAgcHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogUHJvcGVydHlVcGRhdGVbXSB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLmZpbHRlcihyID0+IHIgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5VcGRhdGVzKVxuICAgICAgLm1hcCh1cGRhdGUgPT4gKHtcbiAgICAgICAgZnJvbTogdGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbiksXG4gICAgICAgIHRvOiB1cGRhdGUucGFydG5lclByb3AoKSxcbiAgICAgIH0pKTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eUVpdGhlcnMoKTogUHJvcGVydHlFaXRoZXJTZXRbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IG5ldyBTZXQ8UHJvcGVydHlFaXRoZXJTZXQ+KCk7XG5cbiAgICByZXR1cm4gQXJyYXkuZnJvbShyZXN1bHRzKS5zb3J0KCk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVNZXRob2ROYW1lKHByb3BlcnR5VXBkYXRlOiBQcm9wZXJ0eVVwZGF0ZSk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGB1cGRhdGVfJHt0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKFxuICAgICAgcHJvcGVydHlVcGRhdGUudG8sXG4gICAgKX1fZnJvbV8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcGVydHlVcGRhdGUuZnJvbSl9YDtcbiAgfVxuXG4gIGVudGl0eUV2ZW50TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlFdmVudGA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHlFdmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlQcm9wZXJ0aWVzTmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlQcm9wZXJ0aWVzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eVByb3BlcnRpZXMgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVycm9yVHlwZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OmVycm9yOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWUpfUVycm9yYDtcbiAgfVxuXG4gIG1vZGVsTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6Om1vZGVsOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICBtb2RlbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BNZXRob2QpO1xuICB9XG5cbiAgc3RydWN0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICB0eXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuICB9XG5cbiAgaW1wbFRyeUZyb21Gb3JQcm9wZXJ0eVVwZGF0ZShwcm9wZXJ0eVVwZGF0ZTogUHJvcGVydHlVcGRhdGUpOiBzdHJpbmcge1xuICAgIGNvbnN0IGZyb20gPSBwcm9wZXJ0eVVwZGF0ZS5mcm9tO1xuICAgIGNvbnN0IHRvID0gcHJvcGVydHlVcGRhdGUudG87XG5cbiAgICAvLyBFdmVyeSBmYWxsdGhyb3VnaC9kZWZhdWx0L2Vsc2UgbmVlZHMgYSBgdGhyb3dgIGNsYXVzZSB0byBsb3VkbHkgcHJvY2xhaW1cbiAgICAvLyB0aGF0IGEgc3BlY2lmaWMgY29udmVyc2lvbiBpcyBub3Qgc3VwcG9ydGVkLiBUaGlzIGFsbG93cyB1cyB0byBhZGRcbiAgICAvLyBjb252ZXJzaW9ucyBhcyB3ZSBnbyB3aXRob3V0IHJvZ3VlIGFuZCB1bmV4cGxhaW5lZCBlcnJvcnMuIEluIHNob3J0LFxuICAgIC8vIHRyZWF0IHRoaXMgbGlrZSBSdXN0IGNvZGUgd2l0aCBmdWxseSBzYXRpc2ZpZWQgbWF0Y2ggYXJtcy4gVGhhbmsgeW91LFxuICAgIC8vIGxvdmUsIHVzLlxuICAgIGlmIChmcm9tIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUpIHtcbiAgICAgIHN3aXRjaCAoZnJvbS5sYW5ndWFnZSkge1xuICAgICAgICBjYXNlIFwieWFtbFwiOlxuICAgICAgICAgIGlmICh0byBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgICAgIHJldHVybiBgT2soc2VyZGVfeWFtbDo6ZnJvbV9zdHIodmFsdWUpPylgO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIGxhbmd1YWdlICcke1xuICAgICAgICAgICAgICBmcm9tLmxhbmd1YWdlXG4gICAgICAgICAgICB9JyB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgICAgICAgIH1cbiAgICAgICAgZGVmYXVsdDpcbiAgICAgICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIGxhbmd1YWdlICcke2Zyb20ubGFuZ3VhZ2V9JyBpcyBub3Qgc3VwcG9ydGVkYDtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKGZyb20gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICBpZiAodG8gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSkge1xuICAgICAgICBzd2l0Y2ggKHRvLmxhbmd1YWdlKSB7XG4gICAgICAgICAgY2FzZSBcInlhbWxcIjpcbiAgICAgICAgICAgIHJldHVybiBgT2soc2VyZGVfeWFtbDo6dG9fc3RyaW5nKHZhbHVlKT8pYDtcbiAgICAgICAgICBkZWZhdWx0OlxuICAgICAgICAgICAgdGhyb3cgYGNvbnZlcnNpb24gZnJvbSBQcm9wT2JqZWN0IHRvIGxhbmd1YWdlICcke3RvLmxhbmd1YWdlfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gUHJvcE9iamVjdCB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgICAgfVxuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIHR5cGUgJyR7ZnJvbS5raW5kKCl9JyB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgIH1cbiAgfVxuXG4gIGltcGxMaXN0UmVxdWVzdFR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbExpc3RSZXBseVR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVxdWVzdFR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVwbHlUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UoXG4gICAgICB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLCB7XG4gICAgICAgIG9wdGlvbjogZmFsc2UsXG4gICAgICAgIHJlZmVyZW5jZTogZmFsc2UsXG4gICAgICB9KSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlBY3Rpb24ocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5QWN0aW9uLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUVkaXQocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5RWRpdC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDb21tb25DcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUNyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlR2V0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUdldC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VMaXN0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUxpc3QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tcG9uZW50UGljayhwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDb21wb25lbnRQaWNrLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUN1c3RvbU1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDdXN0b21NZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcE1ldGhvZC5za2lwQXV0aCkge1xuICAgICAgcmV0dXJuIGAvLyBBdXRoZW50aWNhdGlvbiBpcyBza2lwcGVkIG9uIFxcYCR7dGhpcy5pbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgICAgIHByb3BNZXRob2QsXG4gICAgICApfVxcYFxcbmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiB0aGlzLmltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZCk7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoQ2FsbChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBsZXQgcHJlbHVkZSA9IFwic2lfYWNjb3VudDo6YXV0aG9yaXplXCI7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lID09IFwiYWNjb3VudFwiKSB7XG4gICAgICBwcmVsdWRlID0gXCJjcmF0ZTo6YXV0aG9yaXplXCI7XG4gICAgfVxuICAgIHJldHVybiBgJHtwcmVsdWRlfTo6YXV0aG56KCZzZWxmLmRiLCAmcmVxdWVzdCwgXCIke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgcHJvcE1ldGhvZCxcbiAgICApfVwiKS5hd2FpdD87YDtcbiAgfVxuXG4gIHNlcnZpY2VNZXRob2RzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGNvbnN0IHByb3BNZXRob2RzID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5hdHRycy5zb3J0KChhLCBiKSA9PlxuICAgICAgYS5uYW1lID4gYi5uYW1lID8gMSA6IC0xLFxuICAgICk7XG4gICAgZm9yIChjb25zdCBwcm9wTWV0aG9kIG9mIHByb3BNZXRob2RzKSB7XG4gICAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2VNZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgICB7XG4gICAgICAgICAgZm10OiB0aGlzLFxuICAgICAgICAgIHByb3BNZXRob2Q6IHByb3BNZXRob2QsXG4gICAgICAgIH0sXG4gICAgICAgIHtcbiAgICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICAgIH0sXG4gICAgICApO1xuICAgICAgcmVzdWx0cy5wdXNoKG91dHB1dCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBydXN0RmllbGROYW1lRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICB9XG5cbiAgcnVzdFR5cGVGb3JQcm9wKFxuICAgIHByb3A6IFByb3BzLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlZmVyZW5jZSA9IHJlbmRlck9wdGlvbnMucmVmZXJlbmNlIHx8IGZhbHNlO1xuICAgIGxldCBvcHRpb24gPSB0cnVlO1xuICAgIGlmIChyZW5kZXJPcHRpb25zLm9wdGlvbiA9PT0gZmFsc2UpIHtcbiAgICAgIG9wdGlvbiA9IGZhbHNlO1xuICAgIH1cblxuICAgIGxldCB0eXBlTmFtZTogc3RyaW5nO1xuXG4gICAgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24gfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kXG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpMzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpNjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInU2NFwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1MTI4XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUxMjhcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BCb29sIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgY29uc3QgcmVhbFByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgaWYgKHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBjb25zdCBwcm9wT3duZXIgPSBwcm9wLmxvb2t1cE9iamVjdCgpO1xuICAgICAgICBsZXQgcGF0aE5hbWU6IHN0cmluZztcbiAgICAgICAgaWYgKFxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSAmJlxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSA9PSB0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZVxuICAgICAgICApIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH0gZWxzZSBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBgc2lfJHtwcm9wT3duZXIuc2VydmljZU5hbWV9Ojpwcm90b2J1ZmA7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBcImNyYXRlOjpwcm90b2J1ZlwiO1xuICAgICAgICB9XG4gICAgICAgIHR5cGVOYW1lID0gYCR7cGF0aE5hbWV9Ojoke3Bhc2NhbENhc2UocmVhbFByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICAgIHJlYWxQcm9wLm5hbWUsXG4gICAgICAgICl9YDtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChyZWFsUHJvcCwgcmVuZGVyT3B0aW9ucyk7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1hcCkge1xuICAgICAgdHlwZU5hbWUgPSBgc3RkOjpjb2xsZWN0aW9uczo6SGFzaE1hcDxTdHJpbmcsIFN0cmluZz5gO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFRleHQgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BTZWxlY3RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gXCJTdHJpbmdcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgYENhbm5vdCBnZW5lcmF0ZSB0eXBlIGZvciAke3Byb3AubmFtZX0ga2luZCAke3Byb3Aua2luZCgpfSAtIEJ1ZyFgO1xuICAgIH1cbiAgICBpZiAocmVmZXJlbmNlKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgIGlmICh0eXBlTmFtZSA9PSBcIlN0cmluZ1wiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCImc3RyXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgJiR7dHlwZU5hbWV9YDtcbiAgICAgIH1cbiAgICB9XG4gICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgdHlwZU5hbWUgPSBgVmVjPCR7dHlwZU5hbWV9PmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGlmIChvcHRpb24pIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICAgIHR5cGVOYW1lID0gYE9wdGlvbjwke3R5cGVOYW1lfT5gO1xuICAgICAgfVxuICAgIH1cbiAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICByZXR1cm4gdHlwZU5hbWU7XG4gIH1cblxuICBpbXBsQ3JlYXRlTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKGAke3NuYWtlQ2FzZShwcm9wLm5hbWUpfTogJHt0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wKX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsQ3JlYXRlUGFzc05ld0FyZ3MoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICByZXN1bHQucHVzaChzbmFrZUNhc2UocHJvcC5uYW1lKSk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2RMaXN0UmVzdWx0VG9SZXBseSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGxpc3RNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwibGlzdFwiKTtcbiAgICBpZiAobGlzdE1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBsaXN0TWV0aG9kLnJlcGx5LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGxldCBsaXN0UmVwbHlWYWx1ZSA9IGBTb21lKG91dHB1dC4ke2ZpZWxkTmFtZX0pYDtcbiAgICAgICAgaWYgKGZpZWxkTmFtZSA9PSBcIm5leHRfcGFnZV90b2tlblwiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBcIlNvbWUob3V0cHV0LnBhZ2VfdG9rZW4pXCI7XG4gICAgICAgIH0gZWxzZSBpZiAoZmllbGROYW1lID09IFwiaXRlbXNcIikge1xuICAgICAgICAgIGxpc3RSZXBseVZhbHVlID0gYG91dHB1dC4ke2ZpZWxkTmFtZX1gO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKGAke2ZpZWxkTmFtZX06ICR7bGlzdFJlcGx5VmFsdWV9YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2RDcmVhdGVEZXN0cnVjdHVyZSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IGZpZWxkTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICByZXN1bHQucHVzaChgbGV0ICR7ZmllbGROYW1lfSA9IGlubmVyLiR7ZmllbGROYW1lfTtgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgbmF0dXJhbEtleSgpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCkge1xuICAgICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5uYXR1cmFsS2V5KTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwibmFtZVwiO1xuICAgIH1cbiAgfVxuXG4gIGltcGxDcmVhdGVTZXRQcm9wZXJ0aWVzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFBhc3N3b3JkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9IFNvbWUoc2lfZGF0YTo6cGFzc3dvcmQ6OmVuY3J5cHRfcGFzc3dvcmQoJHt2YXJpYWJsZU5hbWV9KT8pO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9ICR7dmFyaWFibGVOYW1lfTtgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmF0dHJzKSB7XG4gICAgICBjb25zdCB2YXJpYWJsZU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgIGNvbnN0IGRlZmF1bHRWYWx1ZSA9IHByb3AuZGVmYXVsdFZhbHVlKCk7XG4gICAgICBpZiAoZGVmYXVsdFZhbHVlKSB7XG4gICAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcInRleHRcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSBcIiR7ZGVmYXVsdFZhbHVlfVwiLnRvX3N0cmluZygpO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImVudW1cIikge1xuICAgICAgICAgIGNvbnN0IGVudW1OYW1lID0gYCR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lLFxuICAgICAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuc2V0XyR7dmFyaWFibGVOYW1lfShjcmF0ZTo6cHJvdG9idWY6OiR7ZW51bU5hbWV9Ojoke3Bhc2NhbENhc2UoXG4gICAgICAgICAgICAgIGRlZmF1bHRWYWx1ZSBhcyBzdHJpbmcsXG4gICAgICAgICAgICApfSk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVBZGRUb1RlbmFuY3koKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImJpbGxpbmdBY2NvdW50XCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25cIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblNlcnZpY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImNvbXBvbmVudE9iamVjdFwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uU2VydmljZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX3NlcnZpY2VfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcInVzZXJcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJncm91cFwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIm9yZ2FuaXphdGlvblwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uSW5zdGFuY2VcIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwid29ya3NwYWNlXCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IHdvcmtzcGFjZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkud29ya3NwYWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLndvcmtzcGFjZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKHdvcmtzcGFjZV9pZCk7YCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlSXNNdmNjKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0Lm12Y2MgPT0gdHJ1ZSkge1xuICAgICAgcmV0dXJuIFwidHJ1ZVwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJmYWxzZVwiO1xuICAgIH1cbiAgfVxuXG4gIHN0b3JhYmxlVmFsaWRhdGVGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICAgIGNvbnN0IHByb3BOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0ubGVuKCkgPT0gMCB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmlzX25vbmUoKSB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICB0b3BQcm9wOiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0LFxuICAgIHByZWZpeDogc3RyaW5nLFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbJ1wic2lTdG9yYWJsZS5uYXR1cmFsS2V5XCInXTtcbiAgICBmb3IgKGxldCBwcm9wIG9mIHRvcFByb3AucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKHByb3AuaGlkZGVuKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBwcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgcHJvcC5uYW1lKSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgICAgdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgYCR7cHJlZml4fS4ke3Byb3AubmFtZX1gKSxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcmVmaXh9LiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5yb290UHJvcCxcbiAgICAgIFwiXCIsXG4gICAgKTtcbiAgICByZXR1cm4gYHZlYyFbJHtyZXN1bHRzfV1cXG5gO1xuICB9XG5cbiAgc3RvcmFibGVSZWZlcmVudGlhbEZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgZmV0Y2hQcm9wcyA9IFtdO1xuICAgIGNvbnN0IHJlZmVyZW5jZVZlYyA9IFtdO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QpIHtcbiAgICAgIGxldCBzaVByb3BlcnRpZXMgPSB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuZ2V0RW50cnkoXCJzaVByb3BlcnRpZXNcIik7XG4gICAgICBpZiAoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgc2lQcm9wZXJ0aWVzID0gc2lQcm9wZXJ0aWVzLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKCEoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkpIHtcbiAgICAgICAgdGhyb3cgXCJDYW5ub3QgZ2V0IHByb3BlcnRpZXMgb2YgYSBub24gb2JqZWN0IGluIHJlZiBjaGVja1wiO1xuICAgICAgfVxuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIHNpUHJvcGVydGllcy5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGlmIChwcm9wLnJlZmVyZW5jZSkge1xuICAgICAgICAgIGNvbnN0IGl0ZW1OYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICAgIGZldGNoUHJvcHMucHVzaChgbGV0ICR7aXRlbU5hbWV9ID0gbWF0Y2ggJnNlbGYuc2lfcHJvcGVydGllcyB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICBTb21lKGNpcCkgPT4gY2lwXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuJHtpdGVtTmFtZX1cbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5hc19yZWYoKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLm1hcChTdHJpbmc6OmFzX3JlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC51bndyYXBfb3IoXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICBOb25lID0+IFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiLFxuICAgICAgICAgICAgICAgICAgICAgICAgIH07YCk7XG4gICAgICAgICAgICByZWZlcmVuY2VWZWMucHVzaChcbiAgICAgICAgICAgICAgYHNpX2RhdGE6OlJlZmVyZW5jZTo6SGFzTWFueShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc09uZShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgfVxuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQmFzZU9iamVjdCkge1xuICAgIH1cblxuICAgIGlmIChmZXRjaFByb3BzLmxlbmd0aCAmJiByZWZlcmVuY2VWZWMubGVuZ3RoKSB7XG4gICAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgICByZXN1bHRzLnB1c2goZmV0Y2hQcm9wcy5qb2luKFwiXFxuXCIpKTtcbiAgICAgIHJlc3VsdHMucHVzaChgdmVjIVske3JlZmVyZW5jZVZlYy5qb2luKFwiLFwiKX1dYCk7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJWZWM6Om5ldygpXCI7XG4gICAgfVxuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyU2VydmljZSB7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZykge1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWUpO1xuICB9XG5cbiAgc3lzdGVtT2JqZWN0c0FzRm9ybWF0dGVycygpOiBSdXN0Rm9ybWF0dGVyW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNcbiAgICAgIC5zb3J0KChhLCBiKSA9PiAoYS50eXBlTmFtZSA+IGIudHlwZU5hbWUgPyAxIDogLTEpKVxuICAgICAgLm1hcChvID0+IG5ldyBSdXN0Rm9ybWF0dGVyKG8pKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Qm9keSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiOiBzaV9kYXRhOjpEYixcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudCxcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTmV3Q29uc3RydWN0b3JBcmdzKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiLCBhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudFwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJkYjogc2lfZGF0YTo6RGJcIjtcbiAgICB9XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdENvbnN0cnVjdG9yUmV0dXJuKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wiZGJcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudFwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlVHJhaXROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7c25ha2VDYXNlKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApfV9zZXJ2ZXI6OiR7cGFzY2FsQ2FzZSh0aGlzLnNlcnZpY2VOYW1lKX1gO1xuICB9XG5cbiAgaW1wbFNlcnZlck5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5pbXBsU2VydmljZVRyYWl0TmFtZSgpfVNlcnZlcmA7XG4gIH1cblxuICBpbXBsU2VydmljZU1pZ3JhdGUoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iaiBvZiB0aGlzLnN5c3RlbU9iamVjdHMpIHtcbiAgICAgIGlmICh0aGlzLmlzTWlncmF0ZWFibGUoc3lzdGVtT2JqKSkge1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgICAgICBzeXN0ZW1PYmoudHlwZU5hbWUsXG4gICAgICAgICAgKX06Om1pZ3JhdGUoJnNlbGYuZGIpLmF3YWl0PztgLFxuICAgICAgICApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBoYXNFbnRpdGllcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzLnNvbWUob2JqID0+IG9iaiBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCk7XG4gIH1cblxuICBpc01pZ3JhdGVhYmxlKHByb3A6IE9iamVjdFR5cGVzKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHByb3AgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QgJiYgcHJvcC5taWdyYXRlYWJsZTtcbiAgfVxuXG4gIGhhc01pZ3JhdGFibGVzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHMuc29tZShvYmogPT4gdGhpcy5pc01pZ3JhdGVhYmxlKG9iaikpO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyQWdlbnQge1xuICBhZ2VudE5hbWU6IHN0cmluZztcbiAgZW50aXR5OiBFbnRpdHlPYmplY3Q7XG4gIGVudGl0eUZvcm1hdHRlcjogUnVzdEZvcm1hdHRlcjtcbiAgaW50ZWdyYXRpb25OYW1lOiBzdHJpbmc7XG4gIGludGVncmF0aW9uU2VydmljZU5hbWU6IHN0cmluZztcbiAgc2VydmljZU5hbWU6IHN0cmluZztcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nLCBhZ2VudDogQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2UpIHtcbiAgICB0aGlzLmFnZW50TmFtZSA9IGFnZW50LmFnZW50TmFtZTtcbiAgICB0aGlzLmVudGl0eSA9IGFnZW50LmVudGl0eTtcbiAgICB0aGlzLmVudGl0eUZvcm1hdHRlciA9IG5ldyBSdXN0Rm9ybWF0dGVyKHRoaXMuZW50aXR5KTtcbiAgICB0aGlzLmludGVncmF0aW9uTmFtZSA9IGFnZW50LmludGVncmF0aW9uTmFtZTtcbiAgICB0aGlzLmludGVncmF0aW9uU2VydmljZU5hbWUgPSBhZ2VudC5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lO1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWUpO1xuICB9XG5cbiAgc3lzdGVtT2JqZWN0c0FzRm9ybWF0dGVycygpOiBSdXN0Rm9ybWF0dGVyW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNcbiAgICAgIC5zb3J0KChhLCBiKSA9PiAoYS50eXBlTmFtZSA+IGIudHlwZU5hbWUgPyAxIDogLTEpKVxuICAgICAgLm1hcChvID0+IG5ldyBSdXN0Rm9ybWF0dGVyKG8pKTtcbiAgfVxuXG4gIGFjdGlvblByb3BzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5Lm1ldGhvZHMuYXR0cnMuZmlsdGVyKFxuICAgICAgbSA9PiBtIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXTtcbiAgfVxuXG4gIGVudGl0eUFjdGlvbk1ldGhvZE5hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiY3JlYXRlXCJdO1xuXG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuYWN0aW9uUHJvcHMoKSkge1xuICAgICAgaWYgKHRoaXMuZW50aXR5Rm9ybWF0dGVyLmlzRW50aXR5RWRpdE1ldGhvZChwcm9wKSkge1xuICAgICAgICByZXN1bHRzLnB1c2godGhpcy5lbnRpdHlGb3JtYXR0ZXIuZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcCkpO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKHByb3AubmFtZSk7XG4gICAgICB9XG4gICAgfVxuXG4gICAgcmV0dXJuIHJlc3VsdHM7XG4gIH1cblxuICBkaXNwYXRjaGVyQmFzZVR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3Bhc2NhbENhc2UodGhpcy5pbnRlZ3JhdGlvbk5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgIHRoaXMuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSxcbiAgICApfSR7cGFzY2FsQ2FzZSh0aGlzLmVudGl0eS5iYXNlVHlwZU5hbWUpfWA7XG4gIH1cblxuICBkaXNwYXRjaGVyVHlwZU5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5kaXNwYXRjaGVyQmFzZVR5cGVOYW1lKCl9RGlzcGF0Y2hlcmA7XG4gIH1cblxuICBkaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpfURpc3BhdGNoRnVuY3Rpb25zYDtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gIH1cblxuICBoYXNNb2RlbHMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHJlZ2lzdHJ5XG4gICAgICAuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHRoaXMuc2VydmljZU5hbWUpXG4gICAgICAuc29tZShvID0+IG8ua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKTtcbiAgfVxuXG4gIGhhc1NlcnZpY2VNZXRob2RzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICByZWdpc3RyeVxuICAgICAgICAuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHRoaXMuc2VydmljZU5hbWUpXG4gICAgICAgIC5mbGF0TWFwKG8gPT4gby5tZXRob2RzLmF0dHJzKS5sZW5ndGggPiAwXG4gICAgKTtcbiAgfVxuXG4gIGhhc0VudGl0eUludGVncmF0aW9uU2VydmNpY2VzKCk6IGJvb2xlYW4ge1xuICAgIGNvbnN0IGludGVncmF0aW9uU2VydmljZXMgPSBuZXcgU2V0KFxuICAgICAgdGhpcy5lbnRpdGllcygpLmZsYXRNYXAoZW50aXR5ID0+XG4gICAgICAgIHRoaXMuZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHkpLFxuICAgICAgKSxcbiAgICApO1xuICAgIHJldHVybiBpbnRlZ3JhdGlvblNlcnZpY2VzLnNpemUgPiAwO1xuICB9XG5cbiAgZW50aXRpZXMoKTogRW50aXR5T2JqZWN0W10ge1xuICAgIHJldHVybiByZWdpc3RyeVxuICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgLmZpbHRlcihvID0+IG8gaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIGFzIEVudGl0eU9iamVjdFtdO1xuICB9XG5cbiAgZW50aXR5QWN0aW9ucyhlbnRpdHk6IEVudGl0eU9iamVjdCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIGVudGl0eS5tZXRob2RzLmF0dHJzLmZpbHRlcihcbiAgICAgIG0gPT4gbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW107XG4gIH1cblxuICBlbnRpdHlpbnRlZ3JhdGlvblNlcnZpY2VzRm9yKGVudGl0eTogRW50aXR5T2JqZWN0KTogSW50ZWdyYXRpb25TZXJ2aWNlW10ge1xuICAgIGNvbnN0IHJlc3VsdDogU2V0PEludGVncmF0aW9uU2VydmljZT4gPSBuZXcgU2V0KCk7XG4gICAgZm9yIChjb25zdCBpbnRlZ3JhdGlvblNlcnZpY2Ugb2YgZW50aXR5LmludGVncmF0aW9uU2VydmljZXMpIHtcbiAgICAgIHJlc3VsdC5hZGQoaW50ZWdyYXRpb25TZXJ2aWNlKTtcbiAgICB9XG4gICAgZm9yIChjb25zdCBhY3Rpb24gb2YgdGhpcy5lbnRpdHlBY3Rpb25zKGVudGl0eSkpIHtcbiAgICAgIGZvciAoY29uc3QgaW50ZWdyYXRpb25TZXJ2aWNlIG9mIGFjdGlvbi5pbnRlZ3JhdGlvblNlcnZpY2VzKSB7XG4gICAgICAgIHJlc3VsdC5hZGQoaW50ZWdyYXRpb25TZXJ2aWNlKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIEFycmF5LmZyb20ocmVzdWx0KTtcbiAgfVxuXG4gIGVudGl0eUludGVncmF0aW9uU2VydmljZXMoKTogQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2VbXSB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXRpZXMoKS5mbGF0TWFwKGVudGl0eSA9PlxuICAgICAgdGhpcy5lbnRpdHlpbnRlZ3JhdGlvblNlcnZpY2VzRm9yKGVudGl0eSkubWFwKGludGVncmF0aW9uU2VydmljZSA9PiAoe1xuICAgICAgICBpbnRlZ3JhdGlvbk5hbWU6IGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvbk5hbWUsXG4gICAgICAgIGludGVncmF0aW9uU2VydmljZU5hbWU6IGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lLFxuICAgICAgICBlbnRpdHk6IGVudGl0eSxcbiAgICAgICAgYWdlbnROYW1lOiBgJHtzbmFrZUNhc2UoXG4gICAgICAgICAgaW50ZWdyYXRpb25TZXJ2aWNlLmludGVncmF0aW9uTmFtZSxcbiAgICAgICAgKX1fJHtzbmFrZUNhc2UoaW50ZWdyYXRpb25TZXJ2aWNlLmludGVncmF0aW9uU2VydmljZU5hbWUpfV8ke3NuYWtlQ2FzZShcbiAgICAgICAgICBlbnRpdHkuYmFzZVR5cGVOYW1lLFxuICAgICAgICApfWAsXG4gICAgICB9KSksXG4gICAgKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBpZiAodGhpcy5oYXNFbnRpdHlJbnRlZ3JhdGlvblNlcnZjaWNlcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goXCJwdWIgbW9kIGFnZW50O1wiKTtcbiAgICB9XG4gICAgaWYgKHRoaXMuaGFzTW9kZWxzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChcInB1YiBtb2QgbW9kZWw7XCIpO1xuICAgIH1cbiAgICBpZiAodGhpcy5oYXNTZXJ2aWNlTWV0aG9kcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goXCJwdWIgbW9kIHNlcnZpY2U7XCIpO1xuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL21vZGVsL21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2RlbE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXCIsIFwiXCJdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqZWN0IG9mIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKSkge1xuICAgICAgaWYgKHN5c3RlbU9iamVjdC5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKGBwdWIgbW9kICR7c25ha2VDYXNlKHN5c3RlbU9iamVjdC50eXBlTmFtZSl9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2RlbC9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuU2VydmljZSgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlclNlcnZpY2UodGhpcy5zZXJ2aWNlTmFtZSksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoYGdlbi9zZXJ2aWNlLnJzYCwgb3V0cHV0KTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWwoc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcyk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L21vZGVsLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlcihzeXN0ZW1PYmplY3QpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFxuICAgICAgYGdlbi9tb2RlbC8ke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfS5yc2AsXG4gICAgICBvdXRwdXQsXG4gICAgKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL2FnZW50L21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5BZ2VudE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXCIsIFwiXCJdO1xuICAgIGZvciAoY29uc3QgYWdlbnQgb2YgdGhpcy5lbnRpdHlJbnRlZ3JhdGlvblNlcnZpY2VzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChgcHViIG1vZCAke2FnZW50LmFnZW50TmFtZX07YCk7XG4gICAgfVxuICAgIHJlc3VsdHMucHVzaChcIlwiKTtcbiAgICBmb3IgKGNvbnN0IGFnZW50IG9mIHRoaXMuZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcygpKSB7XG4gICAgICBjb25zdCBmbXQgPSBuZXcgUnVzdEZvcm1hdHRlckFnZW50KHRoaXMuc2VydmljZU5hbWUsIGFnZW50KTtcbiAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgYHB1YiB1c2UgJHtcbiAgICAgICAgICBhZ2VudC5hZ2VudE5hbWVcbiAgICAgICAgfTo6eyR7Zm10LmRpc3BhdGNoRnVuY3Rpb25UcmFpdE5hbWUoKX0sICR7Zm10LmRpc3BhdGNoZXJUeXBlTmFtZSgpfX07YCxcbiAgICAgICk7XG4gICAgfVxuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL2FnZW50L21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5BZ2VudChhZ2VudDogQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2UpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9hZ2VudC5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXJBZ2VudCh0aGlzLnNlcnZpY2VOYW1lLCBhZ2VudCksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoYGdlbi9hZ2VudC8ke3NuYWtlQ2FzZShhZ2VudC5hZ2VudE5hbWUpfS5yc2AsIG91dHB1dCk7XG4gIH1cblxuICBhc3luYyBtYWtlUGF0aChwYXRoUGFydDogc3RyaW5nKTogUHJvbWlzZTxzdHJpbmc+IHtcbiAgICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcIi4uXCIsIGBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCwgXCJzcmNcIiwgcGF0aFBhcnQpO1xuICAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuICAgIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKHBhdGgucmVzb2x2ZShwYXRoTmFtZSksIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuICAgIHJldHVybiBhYnNvbHV0ZVBhdGhOYW1lO1xuICB9XG5cbiAgYXN5bmMgZm9ybWF0Q29kZSgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBhd2FpdCBleGVjQ21kKGBjYXJnbyBmbXQgLXAgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWApO1xuICB9XG5cbiAgYXN5bmMgd3JpdGVDb2RlKGZpbGVuYW1lOiBzdHJpbmcsIGNvZGU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHBhdGhuYW1lID0gcGF0aC5kaXJuYW1lKGZpbGVuYW1lKTtcbiAgICBjb25zdCBiYXNlbmFtZSA9IHBhdGguYmFzZW5hbWUoZmlsZW5hbWUpO1xuICAgIGNvbnN0IGNyZWF0ZWRQYXRoID0gYXdhaXQgdGhpcy5tYWtlUGF0aChwYXRobmFtZSk7XG4gICAgY29uc3QgY29kZUZpbGVuYW1lID0gcGF0aC5qb2luKGNyZWF0ZWRQYXRoLCBiYXNlbmFtZSk7XG4gICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKGNvZGVGaWxlbmFtZSwgY29kZSk7XG4gIH1cbn1cblxuLy8gZXhwb3J0IGNsYXNzIENvZGVnZW5SdXN0IHtcbi8vICAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcbi8vICAgZm9ybWF0dGVyOiBSdXN0Rm9ybWF0dGVyO1xuLy9cbi8vICAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcykge1xuLy8gICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuLy8gICAgIHRoaXMuZm9ybWF0dGVyID0gbmV3IFJ1c3RGb3JtYXR0ZXIoc3lzdGVtT2JqZWN0KTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgd3JpdGVDb2RlKHBhcnQ6IHN0cmluZywgY29kZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgICAgY29uc3QgY3JlYXRlZFBhdGggPSBhd2FpdCB0aGlzLm1ha2VQYXRoKCk7XG4vLyAgICAgY29uc3QgY29kZUZpbGVuYW1lID0gcGF0aC5qb2luKGNyZWF0ZWRQYXRoLCBgJHtzbmFrZUNhc2UocGFydCl9LnJzYCk7XG4vLyAgICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKGNvZGVGaWxlbmFtZSwgY29kZSk7XG4vLyAgICAgYXdhaXQgZXhlY0NtZChgcnVzdGZtdCAke2NvZGVGaWxlbmFtZX1gKTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgbWFrZVBhdGgoKTogUHJvbWlzZTxzdHJpbmc+IHtcbi8vICAgICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICAgIF9fZGlybmFtZSxcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIFwiLi5cIixcbi8vICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnNpUGF0aE5hbWUsXG4vLyAgICAgICBcInNyY1wiLFxuLy8gICAgICAgXCJnZW5cIixcbi8vICAgICAgIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSksXG4vLyAgICAgKTtcbi8vICAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbi8vICAgICBhd2FpdCBmcy5wcm9taXNlcy5ta2RpcihwYXRoLnJlc29sdmUocGF0aE5hbWUpLCB7IHJlY3Vyc2l2ZTogdHJ1ZSB9KTtcbi8vICAgICByZXR1cm4gYWJzb2x1dGVQYXRoTmFtZTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgZ2VuZXJhdGVDb21wb25lbnRJbXBscygpOiBQcm9taXNlPHZvaWQ+IHtcbi8vICAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuLy8gICAgICAgXCI8JS0gaW5jbHVkZSgncnVzdC9jb21wb25lbnQucnMuZWpzJywgeyBjb21wb25lbnQ6IGNvbXBvbmVudCB9KSAlPlwiLFxuLy8gICAgICAge1xuLy8gICAgICAgICBzeXN0ZW1PYmplY3Q6IHRoaXMuc3lzdGVtT2JqZWN0LFxuLy8gICAgICAgICBmbXQ6IHRoaXMuZm9ybWF0dGVyLFxuLy8gICAgICAgfSxcbi8vICAgICAgIHtcbi8vICAgICAgICAgZmlsZW5hbWU6IF9fZmlsZW5hbWUsXG4vLyAgICAgICB9LFxuLy8gICAgICk7XG4vLyAgICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJjb21wb25lbnRcIiwgb3V0cHV0KTtcbi8vICAgfVxuLy9cbi8vICAgYXN5bmMgZ2VuZXJhdGVDb21wb25lbnRNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgICAgY29uc3QgbW9kcyA9IFtcImNvbXBvbmVudFwiXTtcbi8vICAgICBjb25zdCBsaW5lcyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gVG91Y2h5IVxcblwiXTtcbi8vICAgICBmb3IgKGNvbnN0IG1vZCBvZiBtb2RzKSB7XG4vLyAgICAgICBsaW5lcy5wdXNoKGBwdWIgbW9kICR7bW9kfTtgKTtcbi8vICAgICB9XG4vLyAgICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJtb2RcIiwgbGluZXMuam9pbihcIlxcblwiKSk7XG4vLyAgIH1cbi8vIH1cbi8vXG4vLyBleHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4vLyAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG4vL1xuLy8gICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFJ1c3RGb3JtYXR0ZXJbXCJzeXN0ZW1PYmplY3RcIl0pIHtcbi8vICAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50VHlwZU5hbWUoKTogc3RyaW5nIHtcbi8vICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKTtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50T3JkZXJCeUZpZWxkcygpOiBzdHJpbmcge1xuLy8gICAgIGNvbnN0IG9yZGVyQnlGaWVsZHMgPSBbXTtcbi8vICAgICBjb25zdCBjb21wb25lbnRPYmplY3QgPSB0aGlzLmNvbXBvbmVudC5hc0NvbXBvbmVudCgpO1xuLy8gICAgIGZvciAoY29uc3QgcCBvZiBjb21wb25lbnRPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgaWYgKHAuaGlkZGVuKSB7XG4vLyAgICAgICAgIGNvbnRpbnVlO1xuLy8gICAgICAgfVxuLy8gICAgICAgaWYgKHAubmFtZSA9PSBcInN0b3JhYmxlXCIpIHtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKCdcInN0b3JhYmxlLm5hdHVyYWxLZXlcIicpO1xuLy8gICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goJ1wic3RvcmFibGUudHlwZU5hbWVcIicpO1xuLy8gICAgICAgfSBlbHNlIGlmIChwLm5hbWUgPT0gXCJzaVByb3BlcnRpZXNcIikge1xuLy8gICAgICAgICBjb250aW51ZTtcbi8vICAgICAgIH0gZWxzZSBpZiAocC5uYW1lID09IFwiY29uc3RyYWludHNcIiAmJiBwLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4vLyAgICAgICAgIC8vIEB0cy1pZ25vcmUgdHJ1c3QgdXMgLSB3ZSBjaGVja2VkXG4vLyAgICAgICAgIGZvciAoY29uc3QgcGMgb2YgcC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICAgICAgaWYgKHBjLmtpbmQoKSAhPSBcIm9iamVjdFwiKSB7XG4vLyAgICAgICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goYFwiY29uc3RyYWludHMuJHtwYy5uYW1lfVwiYCk7XG4vLyAgICAgICAgICAgfVxuLy8gICAgICAgICB9XG4vLyAgICAgICB9IGVsc2Uge1xuLy8gICAgICAgICBvcmRlckJ5RmllbGRzLnB1c2goYFwiJHtwLm5hbWV9XCJgKTtcbi8vICAgICAgIH1cbi8vICAgICB9XG4vLyAgICAgcmV0dXJuIGB2ZWMhWyR7b3JkZXJCeUZpZWxkcy5qb2luKFwiLFwiKX1dXFxuYDtcbi8vICAgfVxuLy9cbi8vICAgY29tcG9uZW50SW1wb3J0cygpOiBzdHJpbmcge1xuLy8gICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuLy8gICAgIHJlc3VsdC5wdXNoKFxuLy8gICAgICAgYHB1YiB1c2UgY3JhdGU6OnByb3RvYnVmOjoke3NuYWtlQ2FzZSh0aGlzLmNvbXBvbmVudC50eXBlTmFtZSl9Ojp7YCxcbi8vICAgICAgIGAgIENvbnN0cmFpbnRzLGAsXG4vLyAgICAgICBgICBMaXN0Q29tcG9uZW50c1JlcGx5LGAsXG4vLyAgICAgICBgICBMaXN0Q29tcG9uZW50c1JlcXVlc3QsYCxcbi8vICAgICAgIGAgIFBpY2tDb21wb25lbnRSZXF1ZXN0LGAsXG4vLyAgICAgICBgICBDb21wb25lbnQsYCxcbi8vICAgICAgIGB9O2AsXG4vLyAgICAgKTtcbi8vICAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudFZhbGlkYXRpb24oKTogc3RyaW5nIHtcbi8vICAgICByZXR1cm4gdGhpcy5nZW5WYWxpZGF0aW9uKHRoaXMuY29tcG9uZW50LmFzQ29tcG9uZW50KCkpO1xuLy8gICB9XG4vL1xuLy8gICBnZW5WYWxpZGF0aW9uKHByb3BPYmplY3Q6IFByb3BPYmplY3QpOiBzdHJpbmcge1xuLy8gICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuLy8gICAgIGZvciAoY29uc3QgcHJvcCBvZiBwcm9wT2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4vLyAgICAgICAgIGNvbnN0IHByb3BOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4vLyAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmlzX25vbmUoKSB7XG4vLyAgICAgICAgICAgcmV0dXJuIEVycihEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbi8vICAgICAgICAgfWApO1xuLy8gICAgICAgfVxuLy8gICAgIH1cbi8vICAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4vLyAgIH1cbi8vIH1cbi8vXG4vLyBleHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2VuZXJhdGVHZW5Nb2Qod3JpdHRlbkNvbXBvbmVudHM6IHtcbi8vICAgW2tleTogc3RyaW5nXTogc3RyaW5nW107XG4vLyB9KTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgIGZvciAoY29uc3QgY29tcG9uZW50IGluIHdyaXR0ZW5Db21wb25lbnRzKSB7XG4vLyAgICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBjb21wb25lbnQsXG4vLyAgICAgICBcInNyY1wiLFxuLy8gICAgICAgXCJnZW5cIixcbi8vICAgICApO1xuLy8gICAgIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuLy8gICAgIGNvbnN0IGNvZGUgPSBbXG4vLyAgICAgICBcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsXG4vLyAgICAgICBcIi8vIE5vIHRvdWNoeSFcIixcbi8vICAgICAgIFwiXCIsXG4vLyAgICAgICBcInB1YiBtb2QgbW9kZWw7XCIsXG4vLyAgICAgXTtcbi8vXG4vLyAgICAgYXdhaXQgZnMucHJvbWlzZXMud3JpdGVGaWxlKFxuLy8gICAgICAgcGF0aC5qb2luKGFic29sdXRlUGF0aE5hbWUsIFwibW9kLnJzXCIpLFxuLy8gICAgICAgY29kZS5qb2luKFwiXFxuXCIpLFxuLy8gICAgICk7XG4vLyAgIH1cbi8vIH1cbi8vXG4vLyBleHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2VuZXJhdGVHZW5Nb2RNb2RlbChzZXJ2aWNlTmFtZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgIF9fZGlybmFtZSxcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBzZXJ2aWNlTmFtZSxcbi8vICAgICBcInNyY1wiLFxuLy8gICAgIFwiZ2VuXCIsXG4vLyAgICAgXCJtb2RlbFwiLFxuLy8gICApO1xuLy8gICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbi8vICAgY29uc3QgY29kZSA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVxcblwiXTtcbi8vICAgZm9yIChjb25zdCB0eXBlTmFtZSBvZiB3cml0dGVuQ29tcG9uZW50c1tjb21wb25lbnRdKSB7XG4vLyAgICAgY29kZS5wdXNoKGBwdWIgbW9kICR7c25ha2VDYXNlKHR5cGVOYW1lKX07YCk7XG4vLyAgIH1cbi8vXG4vLyAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShcbi8vICAgICBwYXRoLmpvaW4oYWJzb2x1dGVQYXRoTmFtZSwgXCJtb2QucnNcIiksXG4vLyAgICAgY29kZS5qb2luKFwiXFxuXCIpLFxuLy8gICApO1xuLy8gfVxuIl19