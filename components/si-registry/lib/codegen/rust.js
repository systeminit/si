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
    key: "allEntityEditPropertyUpdates",
    value: function allEntityEditPropertyUpdates() {
      var _this4 = this;

      var results = this.entityEditMethods().flatMap(function (method) {
        return _this4.entityEditPropertyUpdates(method);
      });
      return Array.from(new Set(results)).sort(function (a, b) {
        return "".concat(a.from.name, ",").concat(a.to.name) > "".concat(b.from.name, ",").concat(b.to.name) ? 1 : -1;
      });
    }
  }, {
    key: "entityEditPropertyEithers",
    value: function entityEditPropertyEithers() {
      var results = new Map();
      var properties = this.systemObject.fields.getEntry("properties").properties.attrs;

      var _iterator = _createForOfIteratorHelper(properties),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var property = _step.value;
          var propEithers = property.relationships.all().filter(function (rel) {
            return rel instanceof PropPrelude.Either;
          });

          if (propEithers.length > 0) {
            var eithers = new Set();
            eithers.add(property);

            var _iterator2 = _createForOfIteratorHelper(propEithers),
                _step2;

            try {
              for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
                var _property = _step2.value;
                eithers.add(_property.partnerProp());
              }
            } catch (err) {
              _iterator2.e(err);
            } finally {
              _iterator2.f();
            }

            var eithersArray = Array.from(eithers).sort(function (a, b) {
              return a.name > b.name ? 1 : -1;
            });
            results.set(eithersArray.map(function (e) {
              return e.name;
            }).join(","), {
              entries: eithersArray
            });
          }
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      return Array.from(results.values()).sort();
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

      var _iterator3 = _createForOfIteratorHelper(propMethods),
          _step3;

      try {
        for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
          var propMethod = _step3.value;

          var output = _ejs["default"].render("<%- include('src/codegen/rust/serviceMethod.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
            fmt: this,
            propMethod: propMethod
          }, {
            filename: "."
          });

          results.push(output);
        }
      } catch (err) {
        _iterator3.e(err);
      } finally {
        _iterator3.f();
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
        var _iterator4 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step4;

        try {
          for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
            var prop = _step4.value;
            result.push("".concat((0, _changeCase.snakeCase)(prop.name), ": ").concat(this.rustTypeForProp(prop)));
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
    key: "implCreatePassNewArgs",
    value: function implCreatePassNewArgs() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator5 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step5;

        try {
          for (_iterator5.s(); !(_step5 = _iterator5.n()).done;) {
            var prop = _step5.value;
            result.push((0, _changeCase.snakeCase)(prop.name));
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
    key: "implServiceMethodListResultToReply",
    value: function implServiceMethodListResultToReply() {
      var result = [];
      var listMethod = this.systemObject.methods.getEntry("list");

      if (listMethod instanceof PropPrelude.PropMethod) {
        var _iterator6 = _createForOfIteratorHelper(listMethod.reply.properties.attrs),
            _step6;

        try {
          for (_iterator6.s(); !(_step6 = _iterator6.n()).done;) {
            var prop = _step6.value;
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
          _iterator6.e(err);
        } finally {
          _iterator6.f();
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
        var _iterator7 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step7;

        try {
          for (_iterator7.s(); !(_step7 = _iterator7.n()).done;) {
            var prop = _step7.value;
            var fieldName = (0, _changeCase.snakeCase)(prop.name);
            result.push("let ".concat(fieldName, " = inner.").concat(fieldName, ";"));
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
        var _iterator8 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step8;

        try {
          for (_iterator8.s(); !(_step8 = _iterator8.n()).done;) {
            var prop = _step8.value;
            var variableName = (0, _changeCase.snakeCase)(prop.name);

            if (prop instanceof PropPrelude.PropPassword) {
              result.push("result.".concat(variableName, " = Some(si_data::password::encrypt_password(").concat(variableName, ")?);"));
            } else {
              result.push("result.".concat(variableName, " = ").concat(variableName, ";"));
            }
          }
        } catch (err) {
          _iterator8.e(err);
        } finally {
          _iterator8.f();
        }
      }

      var _iterator9 = _createForOfIteratorHelper(this.systemObject.fields.attrs),
          _step9;

      try {
        for (_iterator9.s(); !(_step9 = _iterator9.n()).done;) {
          var _prop = _step9.value;

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
        _iterator9.e(err);
      } finally {
        _iterator9.f();
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

      var _iterator10 = _createForOfIteratorHelper(this.systemObject.fields.attrs),
          _step10;

      try {
        for (_iterator10.s(); !(_step10 = _iterator10.n()).done;) {
          var prop = _step10.value;

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
        _iterator10.e(err);
      } finally {
        _iterator10.f();
      }

      return result.join("\n");
    }
  }, {
    key: "storableOrderByFieldsByProp",
    value: function storableOrderByFieldsByProp(topProp, prefix) {
      var results = ['"siStorable.naturalKey"'];

      var _iterator11 = _createForOfIteratorHelper(topProp.properties.attrs),
          _step11;

      try {
        for (_iterator11.s(); !(_step11 = _iterator11.n()).done;) {
          var prop = _step11.value;

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
        _iterator11.e(err);
      } finally {
        _iterator11.f();
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

        var _iterator12 = _createForOfIteratorHelper(siProperties.properties.attrs),
            _step12;

        try {
          for (_iterator12.s(); !(_step12 = _iterator12.n()).done;) {
            var prop = _step12.value;

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
          _iterator12.e(err);
        } finally {
          _iterator12.f();
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

      var _iterator13 = _createForOfIteratorHelper(this.systemObjects),
          _step13;

      try {
        for (_iterator13.s(); !(_step13 = _iterator13.n()).done;) {
          var systemObj = _step13.value;

          if (this.isMigrateable(systemObj)) {
            result.push("crate::protobuf::".concat((0, _changeCase.pascalCase)(systemObj.typeName), "::migrate(&self.db).await?;"));
          }
        }
      } catch (err) {
        _iterator13.e(err);
      } finally {
        _iterator13.f();
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
      var _this5 = this;

      return this.systemObjects.some(function (obj) {
        return _this5.isMigrateable(obj);
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

      var _iterator14 = _createForOfIteratorHelper(this.actionProps()),
          _step14;

      try {
        for (_iterator14.s(); !(_step14 = _iterator14.n()).done;) {
          var prop = _step14.value;

          if (this.entityFormatter.isEntityEditMethod(prop)) {
            results.push(this.entityFormatter.entityEditMethodName(prop));
          } else {
            results.push(prop.name);
          }
        }
      } catch (err) {
        _iterator14.e(err);
      } finally {
        _iterator14.f();
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
      var _this6 = this;

      var integrationServices = new Set(this.entities().flatMap(function (entity) {
        return _this6.entityintegrationServicesFor(entity);
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

      var _iterator15 = _createForOfIteratorHelper(entity.integrationServices),
          _step15;

      try {
        for (_iterator15.s(); !(_step15 = _iterator15.n()).done;) {
          var integrationService = _step15.value;
          result.add(integrationService);
        }
      } catch (err) {
        _iterator15.e(err);
      } finally {
        _iterator15.f();
      }

      var _iterator16 = _createForOfIteratorHelper(this.entityActions(entity)),
          _step16;

      try {
        for (_iterator16.s(); !(_step16 = _iterator16.n()).done;) {
          var action = _step16.value;

          var _iterator17 = _createForOfIteratorHelper(action.integrationServices),
              _step17;

          try {
            for (_iterator17.s(); !(_step17 = _iterator17.n()).done;) {
              var _integrationService = _step17.value;
              result.add(_integrationService);
            }
          } catch (err) {
            _iterator17.e(err);
          } finally {
            _iterator17.f();
          }
        }
      } catch (err) {
        _iterator16.e(err);
      } finally {
        _iterator16.f();
      }

      return Array.from(result);
    }
  }, {
    key: "entityIntegrationServices",
    value: function entityIntegrationServices() {
      var _this7 = this;

      return this.entities().flatMap(function (entity) {
        return _this7.entityintegrationServicesFor(entity).map(function (integrationService) {
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
        var results, _iterator18, _step18, systemObject;

        return _regenerator["default"].wrap(function _callee2$(_context2) {
          while (1) {
            switch (_context2.prev = _context2.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator18 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(this.serviceName));

                try {
                  for (_iterator18.s(); !(_step18 = _iterator18.n()).done;) {
                    systemObject = _step18.value;

                    if (systemObject.kind() != "baseObject") {
                      results.push("pub mod ".concat((0, _changeCase.snakeCase)(systemObject.typeName), ";"));
                    }
                  }
                } catch (err) {
                  _iterator18.e(err);
                } finally {
                  _iterator18.f();
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
        var results, _iterator19, _step19, agent, _iterator20, _step20, _agent, fmt;

        return _regenerator["default"].wrap(function _callee5$(_context5) {
          while (1) {
            switch (_context5.prev = _context5.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator19 = _createForOfIteratorHelper(this.entityIntegrationServices());

                try {
                  for (_iterator19.s(); !(_step19 = _iterator19.n()).done;) {
                    agent = _step19.value;
                    results.push("pub mod ".concat(agent.agentName, ";"));
                  }
                } catch (err) {
                  _iterator19.e(err);
                } finally {
                  _iterator19.f();
                }

                results.push("");
                _iterator20 = _createForOfIteratorHelper(this.entityIntegrationServices());

                try {
                  for (_iterator20.s(); !(_step20 = _iterator20.n()).done;) {
                    _agent = _step20.value;
                    fmt = new RustFormatterAgent(this.serviceName, _agent);
                    results.push("pub use ".concat(_agent.agentName, "::{").concat(fmt.dispatchFunctionTraitName(), ", ").concat(fmt.dispatcherTypeName(), "};"));
                  }
                } catch (err) {
                  _iterator20.e(err);
                } finally {
                  _iterator20.f();
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsIm1ldGhvZHMiLCJnZXRFbnRyeSIsInByb3BBY3Rpb24iLCJlbnRpdHlFZGl0UHJvcGVydHkiLCJyZWxhdGlvbnNoaXBzIiwiYWxsIiwic29tZSIsInJlbCIsIlByb3BQcmVsdWRlIiwiRWl0aGVyIiwiVXBkYXRlcyIsImlzRW50aXR5T2JqZWN0IiwiZW50aXR5RWRpdE1ldGhvZHMiLCJoYXNFZGl0VXBkYXRlc0ZvckFjdGlvbiIsIkNvbXBvbmVudE9iamVjdCIsInByb3BNZXRob2QiLCJQcm9wQWN0aW9uIiwiaXNFbnRpdHlBY3Rpb25NZXRob2QiLCJuYW1lIiwiZW5kc1dpdGgiLCJFbnRpdHlFdmVudE9iamVjdCIsIkVudGl0eU9iamVjdCIsIlN5c3RlbU9iamVjdCIsIm1pZ3JhdGVhYmxlIiwiYXR0cnMiLCJmaWx0ZXIiLCJtIiwiYmFzZVR5cGVOYW1lIiwicnVzdEZpZWxkTmFtZUZvclByb3AiLCJyZXBsYWNlIiwiYWN0aW9uUHJvcHMiLCJwIiwiaXNFbnRpdHlFZGl0TWV0aG9kIiwicHJvcGVydHkiLCJyZXF1ZXN0IiwicHJvcGVydGllcyIsIlByb3BMaW5rIiwibG9va3VwTXlzZWxmIiwicnVzdFR5cGVGb3JQcm9wIiwib3B0aW9uIiwiciIsIm1hcCIsInVwZGF0ZSIsImZyb20iLCJ0byIsInBhcnRuZXJQcm9wIiwicmVzdWx0cyIsImZsYXRNYXAiLCJtZXRob2QiLCJlbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzIiwiQXJyYXkiLCJTZXQiLCJzb3J0IiwiYSIsImIiLCJNYXAiLCJmaWVsZHMiLCJwcm9wRWl0aGVycyIsImxlbmd0aCIsImVpdGhlcnMiLCJhZGQiLCJlaXRoZXJzQXJyYXkiLCJzZXQiLCJlIiwiam9pbiIsImVudHJpZXMiLCJ2YWx1ZXMiLCJwcm9wZXJ0eVVwZGF0ZSIsInNlcnZpY2VOYW1lIiwidHlwZU5hbWUiLCJQcm9wQ29kZSIsImxhbmd1YWdlIiwiUHJvcE9iamVjdCIsImtpbmQiLCJyZW5kZXJPcHRpb25zIiwibGlzdCIsInJlcGx5IiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZm10IiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicHJvcE1ldGhvZHMiLCJvdXRwdXQiLCJwdXNoIiwicHJvcCIsIlByb3BNZXRob2QiLCJwYXJlbnROYW1lIiwiUHJvcE51bWJlciIsIm51bWJlcktpbmQiLCJQcm9wQm9vbCIsInJlYWxQcm9wIiwicHJvcE93bmVyIiwibG9va3VwT2JqZWN0IiwicGF0aE5hbWUiLCJQcm9wTWFwIiwiUHJvcFRleHQiLCJQcm9wU2VsZWN0IiwicmVwZWF0ZWQiLCJyZXN1bHQiLCJjcmVhdGVNZXRob2QiLCJsaXN0TWV0aG9kIiwiZmllbGROYW1lIiwibGlzdFJlcGx5VmFsdWUiLCJuYXR1cmFsS2V5IiwidmFyaWFibGVOYW1lIiwiUHJvcFBhc3N3b3JkIiwiZGVmYXVsdFZhbHVlIiwiZW51bU5hbWUiLCJtdmNjIiwicmVxdWlyZWQiLCJwcm9wTmFtZSIsInRvcFByb3AiLCJwcmVmaXgiLCJoaWRkZW4iLCJzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AiLCJyb290UHJvcCIsImZldGNoUHJvcHMiLCJyZWZlcmVuY2VWZWMiLCJzaVByb3BlcnRpZXMiLCJpdGVtTmFtZSIsIkJhc2VPYmplY3QiLCJSdXN0Rm9ybWF0dGVyU2VydmljZSIsInN5c3RlbU9iamVjdHMiLCJyZWdpc3RyeSIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsIm8iLCJoYXNFbnRpdGllcyIsImltcGxTZXJ2aWNlVHJhaXROYW1lIiwic3lzdGVtT2JqIiwiaXNNaWdyYXRlYWJsZSIsIm9iaiIsIlJ1c3RGb3JtYXR0ZXJBZ2VudCIsImFnZW50IiwiYWdlbnROYW1lIiwiZW50aXR5IiwiZW50aXR5Rm9ybWF0dGVyIiwiaW50ZWdyYXRpb25OYW1lIiwiaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSIsImVudGl0eUVkaXRNZXRob2ROYW1lIiwiZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSIsIkNvZGVnZW5SdXN0IiwiaW50ZWdyYXRpb25TZXJ2aWNlcyIsImVudGl0aWVzIiwiZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvciIsInNpemUiLCJpbnRlZ3JhdGlvblNlcnZpY2UiLCJlbnRpdHlBY3Rpb25zIiwiYWN0aW9uIiwiaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMiLCJoYXNNb2RlbHMiLCJoYXNTZXJ2aWNlTWV0aG9kcyIsIndyaXRlQ29kZSIsImVudGl0eUludGVncmF0aW9uU2VydmljZXMiLCJkaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lIiwiZGlzcGF0Y2hlclR5cGVOYW1lIiwicGF0aFBhcnQiLCJwYXRoIiwiYWJzb2x1dGVQYXRoTmFtZSIsInJlc29sdmUiLCJmcyIsInByb21pc2VzIiwibWtkaXIiLCJyZWN1cnNpdmUiLCJjb2RlIiwicGF0aG5hbWUiLCJkaXJuYW1lIiwiYmFzZW5hbWUiLCJtYWtlUGF0aCIsImNyZWF0ZWRQYXRoIiwiY29kZUZpbGVuYW1lIiwid3JpdGVGaWxlIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7QUFRQTs7QUFDQTs7QUFHQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7Ozs7Ozs7QUFFQSxJQUFNQSxPQUFPLEdBQUdDLGlCQUFLQyxTQUFMLENBQWVDLDBCQUFhQyxJQUE1QixDQUFoQjs7SUF1QmFDLGE7QUFHWCx5QkFBWUMsWUFBWixFQUF5RDtBQUFBO0FBQUE7QUFDdkQsU0FBS0EsWUFBTCxHQUFvQkEsWUFBcEI7QUFDRDs7OztzQ0FFMEI7QUFDekIsVUFBSTtBQUNGLGFBQUtBLFlBQUwsQ0FBa0JDLE9BQWxCLENBQTBCQyxRQUExQixDQUFtQyxR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Q0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIE9iamVjdFR5cGVzLFxuICBCYXNlT2JqZWN0LFxuICBTeXN0ZW1PYmplY3QsXG4gIENvbXBvbmVudE9iamVjdCxcbiAgRW50aXR5T2JqZWN0LFxuICBFbnRpdHlFdmVudE9iamVjdCxcbn0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0ICogYXMgUHJvcFByZWx1ZGUgZnJvbSBcIi4uL2NvbXBvbmVudHMvcHJlbHVkZVwiO1xuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3BzLCBJbnRlZ3JhdGlvblNlcnZpY2UgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcblxuaW1wb3J0IHsgc25ha2VDYXNlLCBwYXNjYWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5pbXBvcnQgZWpzIGZyb20gXCJlanNcIjtcbmltcG9ydCBmcyBmcm9tIFwiZnNcIjtcbmltcG9ydCBwYXRoIGZyb20gXCJwYXRoXCI7XG5pbXBvcnQgY2hpbGRQcm9jZXNzIGZyb20gXCJjaGlsZF9wcm9jZXNzXCI7XG5pbXBvcnQgdXRpbCBmcm9tIFwidXRpbFwiO1xuXG5jb25zdCBleGVjQ21kID0gdXRpbC5wcm9taXNpZnkoY2hpbGRQcm9jZXNzLmV4ZWMpO1xuXG5pbnRlcmZhY2UgUnVzdFR5cGVBc1Byb3BPcHRpb25zIHtcbiAgcmVmZXJlbmNlPzogYm9vbGVhbjtcbiAgb3B0aW9uPzogYm9vbGVhbjtcbn1cblxuaW50ZXJmYWNlIEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlIHtcbiAgYWdlbnROYW1lOiBzdHJpbmc7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBpbnRlZ3JhdGlvbk5hbWU6IHN0cmluZztcbiAgaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTogc3RyaW5nO1xufVxuXG5pbnRlcmZhY2UgUHJvcGVydHlVcGRhdGUge1xuICBmcm9tOiBQcm9wUHJlbHVkZS5Qcm9wcztcbiAgdG86IFByb3BQcmVsdWRlLlByb3BzO1xufVxuXG5pbnRlcmZhY2UgUHJvcGVydHlFaXRoZXJTZXQge1xuICBlbnRyaWVzOiBQcm9wUHJlbHVkZS5Qcm9wc1tdO1xufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBoYXNDcmVhdGVNZXRob2QoKTogYm9vbGVhbiB7XG4gICAgdHJ5IHtcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGNhdGNoIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cblxuICBoYXNFZGl0RWl0aGVyc0ZvckFjdGlvbihwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLnNvbWUocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLkVpdGhlcik7XG4gIH1cblxuICBoYXNFZGl0VXBkYXRlc0ZvckFjdGlvbihwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLnNvbWUocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlVwZGF0ZXMpO1xuICB9XG5cbiAgaGFzRWRpdFVwZGF0ZXNBbmRFaXRoZXJzKCk6IGJvb2xlYW4ge1xuICAgIGlmICh0aGlzLmlzRW50aXR5T2JqZWN0KCkpIHtcbiAgICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRNZXRob2RzKCkuc29tZShcbiAgICAgICAgcHJvcEFjdGlvbiA9PlxuICAgICAgICAgIHRoaXMuaGFzRWRpdFVwZGF0ZXNGb3JBY3Rpb24ocHJvcEFjdGlvbikgJiZcbiAgICAgICAgICB0aGlzLmhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uKHByb3BBY3Rpb24pLFxuICAgICAgKTtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgcmFuICdoYXNFZGl0VXBkYXRlc0FuZEVpdGhlcnMoKScgb24gYSBub24tZW50aXR5IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBpc0NvbXBvbmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3Q7XG4gIH1cblxuICBpc0VudGl0eUFjdGlvbk1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHRoaXMuaXNFbnRpdHlPYmplY3QoKSAmJiBwcm9wTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvblxuICAgICk7XG4gIH1cblxuICBpc0VudGl0eUVkaXRNZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLmlzRW50aXR5QWN0aW9uTWV0aG9kKHByb3BNZXRob2QpICYmIHByb3BNZXRob2QubmFtZS5lbmRzV2l0aChcIkVkaXRcIilcbiAgICApO1xuICB9XG5cbiAgaXNFbnRpdHlFdmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdDtcbiAgfVxuXG4gIGlzRW50aXR5T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdDtcbiAgfVxuXG4gIGlzTWlncmF0ZWFibGUoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0ICYmIHRoaXMuc3lzdGVtT2JqZWN0Lm1pZ3JhdGVhYmxlXG4gICAgKTtcbiAgfVxuXG4gIGlzU3RvcmFibGUoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgYWN0aW9uUHJvcHMoKTogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5hdHRycy5maWx0ZXIoXG4gICAgICBtID0+IG0gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdO1xuICB9XG5cbiAgY29tcG9uZW50TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1Db21wb25lbnRgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gY29tcG9uZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBjb21wb25lbnRDb25zdHJhaW50c05hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9Q29tcG9uZW50Q29uc3RyYWludHNgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYSBjb21wb25lbnQgY29uc3RyYWludHMgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eUVkaXRNZXRob2ROYW1lKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkge1xuICAgICAgcmV0dXJuIGBlZGl0XyR7dGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wTWV0aG9kKS5yZXBsYWNlKFxuICAgICAgICBcIl9lZGl0XCIsXG4gICAgICAgIFwiXCIsXG4gICAgICApfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlZGl0IG1ldGhvZCBuYW1lIG9uIGEgbm9uLWVudGl0eSBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5RWRpdE1ldGhvZHMoKTogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdIHtcbiAgICByZXR1cm4gdGhpcy5hY3Rpb25Qcm9wcygpLmZpbHRlcihwID0+IHRoaXMuaXNFbnRpdHlFZGl0TWV0aG9kKHApKTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogUHJvcHMge1xuICAgIGxldCBwcm9wZXJ0eSA9IHByb3BBY3Rpb24ucmVxdWVzdC5wcm9wZXJ0aWVzLmdldEVudHJ5KFwicHJvcGVydHlcIik7XG4gICAgaWYgKHByb3BlcnR5IGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgIHByb3BlcnR5ID0gcHJvcGVydHkubG9va3VwTXlzZWxmKCk7XG4gICAgfVxuICAgIHJldHVybiBwcm9wZXJ0eTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eUZpZWxkKHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pKTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eVR5cGUocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pLCB7XG4gICAgICBvcHRpb246IGZhbHNlLFxuICAgIH0pO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5VXBkYXRlcyhcbiAgICBwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBQcm9wZXJ0eVVwZGF0ZVtdIHtcbiAgICByZXR1cm4gdGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbilcbiAgICAgIC5yZWxhdGlvbnNoaXBzLmFsbCgpXG4gICAgICAuZmlsdGVyKHIgPT4gciBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlVwZGF0ZXMpXG4gICAgICAubWFwKHVwZGF0ZSA9PiAoe1xuICAgICAgICBmcm9tOiB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKSxcbiAgICAgICAgdG86IHVwZGF0ZS5wYXJ0bmVyUHJvcCgpLFxuICAgICAgfSkpO1xuICB9XG5cbiAgYWxsRW50aXR5RWRpdFByb3BlcnR5VXBkYXRlcygpOiBQcm9wZXJ0eVVwZGF0ZVtdIHtcbiAgICBjb25zdCByZXN1bHRzID0gdGhpcy5lbnRpdHlFZGl0TWV0aG9kcygpLmZsYXRNYXAobWV0aG9kID0+XG4gICAgICB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZXMobWV0aG9kKSxcbiAgICApO1xuXG4gICAgcmV0dXJuIEFycmF5LmZyb20obmV3IFNldChyZXN1bHRzKSkuc29ydCgoYSwgYikgPT5cbiAgICAgIGAke2EuZnJvbS5uYW1lfSwke2EudG8ubmFtZX1gID4gYCR7Yi5mcm9tLm5hbWV9LCR7Yi50by5uYW1lfWAgPyAxIDogLTEsXG4gICAgKTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eUVpdGhlcnMoKTogUHJvcGVydHlFaXRoZXJTZXRbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IG5ldyBNYXAoKTtcbiAgICBjb25zdCBwcm9wZXJ0aWVzID0gKHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5nZXRFbnRyeShcbiAgICAgIFwicHJvcGVydGllc1wiLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkucHJvcGVydGllcy5hdHRycztcblxuICAgIGZvciAoY29uc3QgcHJvcGVydHkgb2YgcHJvcGVydGllcykge1xuICAgICAgY29uc3QgcHJvcEVpdGhlcnMgPSBwcm9wZXJ0eS5yZWxhdGlvbnNoaXBzXG4gICAgICAgIC5hbGwoKVxuICAgICAgICAuZmlsdGVyKHJlbCA9PiByZWwgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5FaXRoZXIpO1xuXG4gICAgICBpZiAocHJvcEVpdGhlcnMubGVuZ3RoID4gMCkge1xuICAgICAgICBjb25zdCBlaXRoZXJzID0gbmV3IFNldDxQcm9wUHJlbHVkZS5Qcm9wcz4oKTtcbiAgICAgICAgZWl0aGVycy5hZGQocHJvcGVydHkpO1xuICAgICAgICBmb3IgKGNvbnN0IHByb3BlcnR5IG9mIHByb3BFaXRoZXJzKSB7XG4gICAgICAgICAgZWl0aGVycy5hZGQocHJvcGVydHkucGFydG5lclByb3AoKSk7XG4gICAgICAgIH1cblxuICAgICAgICBjb25zdCBlaXRoZXJzQXJyYXkgPSBBcnJheS5mcm9tKGVpdGhlcnMpLnNvcnQoKGEsIGIpID0+XG4gICAgICAgICAgYS5uYW1lID4gYi5uYW1lID8gMSA6IC0xLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHRzLnNldChlaXRoZXJzQXJyYXkubWFwKGUgPT4gZS5uYW1lKS5qb2luKFwiLFwiKSwge1xuICAgICAgICAgIGVudHJpZXM6IGVpdGhlcnNBcnJheSxcbiAgICAgICAgfSk7XG4gICAgICB9XG4gICAgfVxuXG4gICAgcmV0dXJuIEFycmF5LmZyb20ocmVzdWx0cy52YWx1ZXMoKSkuc29ydCgpO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5VXBkYXRlTWV0aG9kTmFtZShwcm9wZXJ0eVVwZGF0ZTogUHJvcGVydHlVcGRhdGUpOiBzdHJpbmcge1xuICAgIHJldHVybiBgdXBkYXRlXyR7dGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChcbiAgICAgIHByb3BlcnR5VXBkYXRlLnRvLFxuICAgICl9X2Zyb21fJHt0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BlcnR5VXBkYXRlLmZyb20pfWA7XG4gIH1cblxuICBlbnRpdHlFdmVudE5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9RW50aXR5RXZlbnRgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5RXZlbnQgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eU5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9RW50aXR5YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eSBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5UHJvcGVydGllc05hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9RW50aXR5UHJvcGVydGllc2A7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHlQcm9wZXJ0aWVzIG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlcnJvclR5cGUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjplcnJvcjo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lKX1FcnJvcmA7XG4gIH1cblxuICBtb2RlbE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjptb2RlbDo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX1gO1xuICB9XG5cbiAgbW9kZWxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wTWV0aG9kKTtcbiAgfVxuXG4gIHN0cnVjdE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX1gO1xuICB9XG5cbiAgdHlwZU5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKTtcbiAgfVxuXG4gIGltcGxUcnlGcm9tRm9yUHJvcGVydHlVcGRhdGUocHJvcGVydHlVcGRhdGU6IFByb3BlcnR5VXBkYXRlKTogc3RyaW5nIHtcbiAgICBjb25zdCBmcm9tID0gcHJvcGVydHlVcGRhdGUuZnJvbTtcbiAgICBjb25zdCB0byA9IHByb3BlcnR5VXBkYXRlLnRvO1xuXG4gICAgLy8gRXZlcnkgZmFsbHRocm91Z2gvZGVmYXVsdC9lbHNlIG5lZWRzIGEgYHRocm93YCBjbGF1c2UgdG8gbG91ZGx5IHByb2NsYWltXG4gICAgLy8gdGhhdCBhIHNwZWNpZmljIGNvbnZlcnNpb24gaXMgbm90IHN1cHBvcnRlZC4gVGhpcyBhbGxvd3MgdXMgdG8gYWRkXG4gICAgLy8gY29udmVyc2lvbnMgYXMgd2UgZ28gd2l0aG91dCByb2d1ZSBhbmQgdW5leHBsYWluZWQgZXJyb3JzLiBJbiBzaG9ydCxcbiAgICAvLyB0cmVhdCB0aGlzIGxpa2UgUnVzdCBjb2RlIHdpdGggZnVsbHkgc2F0aXNmaWVkIG1hdGNoIGFybXMuIFRoYW5rIHlvdSxcbiAgICAvLyBsb3ZlLCB1cy5cbiAgICBpZiAoZnJvbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlKSB7XG4gICAgICBzd2l0Y2ggKGZyb20ubGFuZ3VhZ2UpIHtcbiAgICAgICAgY2FzZSBcInlhbWxcIjpcbiAgICAgICAgICBpZiAodG8gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgICAgICByZXR1cm4gYE9rKHNlcmRlX3lhbWw6OmZyb21fc3RyKHZhbHVlKT8pYDtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgdGhyb3cgYGNvbnZlcnNpb24gZnJvbSBsYW5ndWFnZSAnJHtcbiAgICAgICAgICAgICAgZnJvbS5sYW5ndWFnZVxuICAgICAgICAgICAgfScgdG8gdHlwZSAnJHt0by5raW5kKCl9JyBpcyBub3Qgc3VwcG9ydGVkYDtcbiAgICAgICAgICB9XG4gICAgICAgIGRlZmF1bHQ6XG4gICAgICAgICAgdGhyb3cgYGNvbnZlcnNpb24gZnJvbSBsYW5ndWFnZSAnJHtmcm9tLmxhbmd1YWdlfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChmcm9tIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgaWYgKHRvIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUpIHtcbiAgICAgICAgc3dpdGNoICh0by5sYW5ndWFnZSkge1xuICAgICAgICAgIGNhc2UgXCJ5YW1sXCI6XG4gICAgICAgICAgICByZXR1cm4gYE9rKHNlcmRlX3lhbWw6OnRvX3N0cmluZyh2YWx1ZSk/KWA7XG4gICAgICAgICAgZGVmYXVsdDpcbiAgICAgICAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gUHJvcE9iamVjdCB0byBsYW5ndWFnZSAnJHt0by5sYW5ndWFnZX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIFByb3BPYmplY3QgdG8gdHlwZSAnJHt0by5raW5kKCl9JyBpcyBub3Qgc3VwcG9ydGVkYDtcbiAgICAgIH1cbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgYGNvbnZlcnNpb24gZnJvbSB0eXBlICcke2Zyb20ua2luZCgpfScgdG8gdHlwZSAnJHt0by5raW5kKCl9JyBpcyBub3Qgc3VwcG9ydGVkYDtcbiAgICB9XG4gIH1cblxuICBpbXBsTGlzdFJlcXVlc3RUeXBlKHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9KTogc3RyaW5nIHtcbiAgICBjb25zdCBsaXN0ID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIFwibGlzdFwiLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZDtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AobGlzdC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxMaXN0UmVwbHlUeXBlKHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9KTogc3RyaW5nIHtcbiAgICBjb25zdCBsaXN0ID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIFwibGlzdFwiLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZDtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AobGlzdC5yZXBseSwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVJlcXVlc3RUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVxdWVzdCwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVJlcGx5VHlwZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKFxuICAgICAgdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZCwge1xuICAgICAgICBvcHRpb246IGZhbHNlLFxuICAgICAgICByZWZlcmVuY2U6IGZhbHNlLFxuICAgICAgfSksXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5QWN0aW9uKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUFjdGlvbi5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlFZGl0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUVkaXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUNvbW1vbkNyZWF0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5Q3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUdldChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VHZXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTGlzdChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VMaXN0LnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUNvbXBvbmVudFBpY2socHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tcG9uZW50UGljay5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDdXN0b21NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ3VzdG9tTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUF1dGgocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgaWYgKHByb3BNZXRob2Quc2tpcEF1dGgpIHtcbiAgICAgIHJldHVybiBgLy8gQXV0aGVudGljYXRpb24gaXMgc2tpcHBlZCBvbiBcXGAke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgICBwcm9wTWV0aG9kLFxuICAgICAgKX1cXGBcXG5gO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gdGhpcy5pbXBsU2VydmljZUF1dGhDYWxsKHByb3BNZXRob2QpO1xuICAgIH1cbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgbGV0IHByZWx1ZGUgPSBcInNpX2FjY291bnQ6OmF1dGhvcml6ZVwiO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSA9PSBcImFjY291bnRcIikge1xuICAgICAgcHJlbHVkZSA9IFwiY3JhdGU6OmF1dGhvcml6ZVwiO1xuICAgIH1cbiAgICByZXR1cm4gYCR7cHJlbHVkZX06OmF1dGhueigmc2VsZi5kYiwgJnJlcXVlc3QsIFwiJHt0aGlzLmltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICAgIHByb3BNZXRob2QsXG4gICAgKX1cIikuYXdhaXQ/O2A7XG4gIH1cblxuICBzZXJ2aWNlTWV0aG9kcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBjb25zdCBwcm9wTWV0aG9kcyA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMuc29ydCgoYSwgYikgPT5cbiAgICAgIGEubmFtZSA+IGIubmFtZSA/IDEgOiAtMSxcbiAgICApO1xuICAgIGZvciAoY29uc3QgcHJvcE1ldGhvZCBvZiBwcm9wTWV0aG9kcykge1xuICAgICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgICAge1xuICAgICAgICAgIGZtdDogdGhpcyxcbiAgICAgICAgICBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kLFxuICAgICAgICB9LFxuICAgICAgICB7XG4gICAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgICB9LFxuICAgICAgKTtcbiAgICAgIHJlc3VsdHMucHVzaChvdXRwdXQpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgcnVzdEZpZWxkTmFtZUZvclByb3AocHJvcDogUHJvcHMpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgfVxuXG4gIHJ1c3RUeXBlRm9yUHJvcChcbiAgICBwcm9wOiBQcm9wcyxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZWZlcmVuY2UgPSByZW5kZXJPcHRpb25zLnJlZmVyZW5jZSB8fCBmYWxzZTtcbiAgICBsZXQgb3B0aW9uID0gdHJ1ZTtcbiAgICBpZiAocmVuZGVyT3B0aW9ucy5vcHRpb24gPT09IGZhbHNlKSB7XG4gICAgICBvcHRpb24gPSBmYWxzZTtcbiAgICB9XG5cbiAgICBsZXQgdHlwZU5hbWU6IHN0cmluZztcblxuICAgIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTnVtYmVyKSB7XG4gICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTMyXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1MzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50NjRcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTY0XCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1NjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidTEyOFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1MTI4XCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQm9vbCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLm5hbWUsXG4gICAgICApfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgIGNvbnN0IHJlYWxQcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIGlmIChyZWFsUHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgY29uc3QgcHJvcE93bmVyID0gcHJvcC5sb29rdXBPYmplY3QoKTtcbiAgICAgICAgbGV0IHBhdGhOYW1lOiBzdHJpbmc7XG4gICAgICAgIGlmIChcbiAgICAgICAgICBwcm9wT3duZXIuc2VydmljZU5hbWUgJiZcbiAgICAgICAgICBwcm9wT3duZXIuc2VydmljZU5hbWUgPT0gdGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWVcbiAgICAgICAgKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBcImNyYXRlOjpwcm90b2J1ZlwiO1xuICAgICAgICB9IGVsc2UgaWYgKHByb3BPd25lci5zZXJ2aWNlTmFtZSkge1xuICAgICAgICAgIHBhdGhOYW1lID0gYHNpXyR7cHJvcE93bmVyLnNlcnZpY2VOYW1lfTo6cHJvdG9idWZgO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfVxuICAgICAgICB0eXBlTmFtZSA9IGAke3BhdGhOYW1lfTo6JHtwYXNjYWxDYXNlKHJlYWxQcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgICByZWFsUHJvcC5uYW1lLFxuICAgICAgICApfWA7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocmVhbFByb3AsIHJlbmRlck9wdGlvbnMpO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNYXApIHtcbiAgICAgIHR5cGVOYW1lID0gYHN0ZDo6Y29sbGVjdGlvbnM6Okhhc2hNYXA8U3RyaW5nLCBTdHJpbmc+YDtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BUZXh0IHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wU2VsZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IFwiU3RyaW5nXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IGBDYW5ub3QgZ2VuZXJhdGUgdHlwZSBmb3IgJHtwcm9wLm5hbWV9IGtpbmQgJHtwcm9wLmtpbmQoKX0gLSBCdWchYDtcbiAgICB9XG4gICAgaWYgKHJlZmVyZW5jZSkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICBpZiAodHlwZU5hbWUgPT0gXCJTdHJpbmdcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiJnN0clwiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICAgIHR5cGVOYW1lID0gYCYke3R5cGVOYW1lfWA7XG4gICAgICB9XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgIHR5cGVOYW1lID0gYFZlYzwke3R5cGVOYW1lfT5gO1xuICAgIH0gZWxzZSB7XG4gICAgICBpZiAob3B0aW9uKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGBPcHRpb248JHt0eXBlTmFtZX0+YDtcbiAgICAgIH1cbiAgICB9XG4gICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgcmV0dXJuIHR5cGVOYW1lO1xuICB9XG5cbiAgaW1wbENyZWF0ZU5ld0FyZ3MoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICByZXN1bHQucHVzaChgJHtzbmFrZUNhc2UocHJvcC5uYW1lKX06ICR7dGhpcy5ydXN0VHlwZUZvclByb3AocHJvcCl9YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbENyZWF0ZVBhc3NOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goc25ha2VDYXNlKHByb3AubmFtZSkpO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTGlzdFJlc3VsdFRvUmVwbHkoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBsaXN0TWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImxpc3RcIik7XG4gICAgaWYgKGxpc3RNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgbGlzdE1ldGhvZC5yZXBseS5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IGZpZWxkTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBsZXQgbGlzdFJlcGx5VmFsdWUgPSBgU29tZShvdXRwdXQuJHtmaWVsZE5hbWV9KWA7XG4gICAgICAgIGlmIChmaWVsZE5hbWUgPT0gXCJuZXh0X3BhZ2VfdG9rZW5cIikge1xuICAgICAgICAgIGxpc3RSZXBseVZhbHVlID0gXCJTb21lKG91dHB1dC5wYWdlX3Rva2VuKVwiO1xuICAgICAgICB9IGVsc2UgaWYgKGZpZWxkTmFtZSA9PSBcIml0ZW1zXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IGBvdXRwdXQuJHtmaWVsZE5hbWV9YDtcbiAgICAgICAgfVxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9OiAke2xpc3RSZXBseVZhbHVlfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kQ3JlYXRlRGVzdHJ1Y3R1cmUoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgcmVzdWx0LnB1c2goYGxldCAke2ZpZWxkTmFtZX0gPSBpbm5lci4ke2ZpZWxkTmFtZX07YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIG5hdHVyYWxLZXkoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QubmF0dXJhbEtleSk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIm5hbWVcIjtcbiAgICB9XG4gIH1cblxuICBpbXBsQ3JlYXRlU2V0UHJvcGVydGllcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IHZhcmlhYmxlTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BQYXNzd29yZCkge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSBTb21lKHNpX2RhdGE6OnBhc3N3b3JkOjplbmNyeXB0X3Bhc3N3b3JkKCR7dmFyaWFibGVOYW1lfSk/KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSAke3ZhcmlhYmxlTmFtZX07YCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5hdHRycykge1xuICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICBjb25zdCBkZWZhdWx0VmFsdWUgPSBwcm9wLmRlZmF1bHRWYWx1ZSgpO1xuICAgICAgaWYgKGRlZmF1bHRWYWx1ZSkge1xuICAgICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJ0ZXh0XCIpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gXCIke2RlZmF1bHRWYWx1ZX1cIi50b19zdHJpbmcoKTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJlbnVtXCIpIHtcbiAgICAgICAgICBjb25zdCBlbnVtTmFtZSA9IGAke3Bhc2NhbENhc2UoXG4gICAgICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSxcbiAgICAgICAgICApfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LnNldF8ke3ZhcmlhYmxlTmFtZX0oY3JhdGU6OnByb3RvYnVmOjoke2VudW1OYW1lfTo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgICBkZWZhdWx0VmFsdWUgYXMgc3RyaW5nLFxuICAgICAgICAgICAgKX0pO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsQ3JlYXRlQWRkVG9UZW5hbmN5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJiaWxsaW5nQWNjb3VudFwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uXCJcbiAgICApIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25TZXJ2aWNlXCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJjb21wb25lbnRPYmplY3RcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25fc2VydmljZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25fc2VydmljZV9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvblNlcnZpY2VJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ1c2VyXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiZ3JvdXBcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJvcmdhbml6YXRpb25cIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvbkluc3RhbmNlXCJcbiAgICApIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIndvcmtzcGFjZVwiKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5vcmdhbml6YXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhvcmdhbml6YXRpb25faWQpO2ApO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5vcmdhbml6YXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhvcmdhbml6YXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCB3b3Jrc3BhY2VfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLndvcmtzcGFjZV9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy53b3Jrc3BhY2VJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyh3b3Jrc3BhY2VfaWQpO2ApO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZUlzTXZjYygpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5tdmNjID09IHRydWUpIHtcbiAgICAgIHJldHVybiBcInRydWVcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiZmFsc2VcIjtcbiAgICB9XG4gIH1cblxuICBzdG9yYWJsZVZhbGlkYXRlRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuICAgICAgICBjb25zdCBwcm9wTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmxlbigpID09IDAge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5pc19ub25lKCkge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgdG9wUHJvcDogUHJvcFByZWx1ZGUuUHJvcE9iamVjdCxcbiAgICBwcmVmaXg6IHN0cmluZyxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gWydcInNpU3RvcmFibGUubmF0dXJhbEtleVwiJ107XG4gICAgZm9yIChsZXQgcHJvcCBvZiB0b3BQcm9wLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLmhpZGRlbikge1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgcHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIHByb3AubmFtZSkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICAgIHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIGAke3ByZWZpeH0uJHtwcm9wLm5hbWV9YCksXG4gICAgICAgICAgKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJlZml4fS4ke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3Qucm9vdFByb3AsXG4gICAgICBcIlwiLFxuICAgICk7XG4gICAgcmV0dXJuIGB2ZWMhWyR7cmVzdWx0c31dXFxuYDtcbiAgfVxuXG4gIHN0b3JhYmxlUmVmZXJlbnRpYWxGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IGZldGNoUHJvcHMgPSBbXTtcbiAgICBjb25zdCByZWZlcmVuY2VWZWMgPSBbXTtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0KSB7XG4gICAgICBsZXQgc2lQcm9wZXJ0aWVzID0gdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmdldEVudHJ5KFwic2lQcm9wZXJ0aWVzXCIpO1xuICAgICAgaWYgKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHNpUHJvcGVydGllcyA9IHNpUHJvcGVydGllcy5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmICghKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpKSB7XG4gICAgICAgIHRocm93IFwiQ2Fubm90IGdldCBwcm9wZXJ0aWVzIG9mIGEgbm9uIG9iamVjdCBpbiByZWYgY2hlY2tcIjtcbiAgICAgIH1cbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBzaVByb3BlcnRpZXMucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBpZiAocHJvcC5yZWZlcmVuY2UpIHtcbiAgICAgICAgICBjb25zdCBpdGVtTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc01hbnkoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNPbmUoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEJhc2VPYmplY3QpIHtcbiAgICB9XG5cbiAgICBpZiAoZmV0Y2hQcm9wcy5sZW5ndGggJiYgcmVmZXJlbmNlVmVjLmxlbmd0aCkge1xuICAgICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgICAgcmVzdWx0cy5wdXNoKGZldGNoUHJvcHMuam9pbihcIlxcblwiKSk7XG4gICAgICByZXN1bHRzLnB1c2goYHZlYyFbJHtyZWZlcmVuY2VWZWMuam9pbihcIixcIil9XWApO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiVmVjOjpuZXcoKVwiO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlclNlcnZpY2Uge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzeXN0ZW1PYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzXG4gICAgICAuc29ydCgoYSwgYikgPT4gKGEudHlwZU5hbWUgPiBiLnR5cGVOYW1lID8gMSA6IC0xKSlcbiAgICAgIC5tYXAobyA9PiBuZXcgUnVzdEZvcm1hdHRlcihvKSk7XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdEJvZHkoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXCJkYjogc2lfZGF0YTo6RGIsXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnQsXCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsU2VydmljZU5ld0NvbnN0cnVjdG9yQXJncygpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJldHVybiBcImRiOiBzaV9kYXRhOjpEYiwgYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnRcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiXCI7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VTdHJ1Y3RDb25zdHJ1Y3RvclJldHVybigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnRcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIixcIik7XG4gIH1cblxuICBpbXBsU2VydmljZVRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3NuYWtlQ2FzZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKX1fc2VydmVyOjoke3Bhc2NhbENhc2UodGhpcy5zZXJ2aWNlTmFtZSl9YDtcbiAgfVxuXG4gIGltcGxTZXJ2ZXJOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuaW1wbFNlcnZpY2VUcmFpdE5hbWUoKX1TZXJ2ZXJgO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNaWdyYXRlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgZm9yIChjb25zdCBzeXN0ZW1PYmogb2YgdGhpcy5zeXN0ZW1PYmplY3RzKSB7XG4gICAgICBpZiAodGhpcy5pc01pZ3JhdGVhYmxlKHN5c3RlbU9iaikpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgc3lzdGVtT2JqLnR5cGVOYW1lLFxuICAgICAgICAgICl9OjptaWdyYXRlKCZzZWxmLmRiKS5hd2FpdD87YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaGFzRW50aXRpZXMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0cy5zb21lKG9iaiA9PiBvYmogaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpO1xuICB9XG5cbiAgaXNNaWdyYXRlYWJsZShwcm9wOiBPYmplY3RUeXBlcyk6IGJvb2xlYW4ge1xuICAgIHJldHVybiBwcm9wIGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0ICYmIHByb3AubWlncmF0ZWFibGU7XG4gIH1cblxuICBoYXNNaWdyYXRhYmxlcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzLnNvbWUob2JqID0+IHRoaXMuaXNNaWdyYXRlYWJsZShvYmopKTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlckFnZW50IHtcbiAgYWdlbnROYW1lOiBzdHJpbmc7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBlbnRpdHlGb3JtYXR0ZXI6IFJ1c3RGb3JtYXR0ZXI7XG4gIGludGVncmF0aW9uTmFtZTogc3RyaW5nO1xuICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZywgYWdlbnQ6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlKSB7XG4gICAgdGhpcy5hZ2VudE5hbWUgPSBhZ2VudC5hZ2VudE5hbWU7XG4gICAgdGhpcy5lbnRpdHkgPSBhZ2VudC5lbnRpdHk7XG4gICAgdGhpcy5lbnRpdHlGb3JtYXR0ZXIgPSBuZXcgUnVzdEZvcm1hdHRlcih0aGlzLmVudGl0eSk7XG4gICAgdGhpcy5pbnRlZ3JhdGlvbk5hbWUgPSBhZ2VudC5pbnRlZ3JhdGlvbk5hbWU7XG4gICAgdGhpcy5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lID0gYWdlbnQuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzXG4gICAgICAuc29ydCgoYSwgYikgPT4gKGEudHlwZU5hbWUgPiBiLnR5cGVOYW1lID8gMSA6IC0xKSlcbiAgICAgIC5tYXAobyA9PiBuZXcgUnVzdEZvcm1hdHRlcihvKSk7XG4gIH1cblxuICBhY3Rpb25Qcm9wcygpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eS5tZXRob2RzLmF0dHJzLmZpbHRlcihcbiAgICAgIG0gPT4gbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW107XG4gIH1cblxuICBlbnRpdHlBY3Rpb25NZXRob2ROYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcImNyZWF0ZVwiXTtcblxuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLmFjdGlvblByb3BzKCkpIHtcbiAgICAgIGlmICh0aGlzLmVudGl0eUZvcm1hdHRlci5pc0VudGl0eUVkaXRNZXRob2QocHJvcCkpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuZW50aXR5Rm9ybWF0dGVyLmVudGl0eUVkaXRNZXRob2ROYW1lKHByb3ApKTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdHMucHVzaChwcm9wLm5hbWUpO1xuICAgICAgfVxuICAgIH1cblxuICAgIHJldHVybiByZXN1bHRzO1xuICB9XG5cbiAgZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMuaW50ZWdyYXRpb25OYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICB0aGlzLmludGVncmF0aW9uU2VydmljZU5hbWUsXG4gICAgKX0ke3Bhc2NhbENhc2UodGhpcy5lbnRpdHkuYmFzZVR5cGVOYW1lKX1gO1xuICB9XG5cbiAgZGlzcGF0Y2hlclR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpfURpc3BhdGNoZXJgO1xuICB9XG5cbiAgZGlzcGF0Y2hGdW5jdGlvblRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHt0aGlzLmRpc3BhdGNoZXJCYXNlVHlwZU5hbWUoKX1EaXNwYXRjaEZ1bmN0aW9uc2A7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIENvZGVnZW5SdXN0IHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICB9XG5cbiAgaGFzTW9kZWxzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiByZWdpc3RyeVxuICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgLnNvbWUobyA9PiBvLmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIik7XG4gIH1cblxuICBoYXNTZXJ2aWNlTWV0aG9kcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgcmVnaXN0cnlcbiAgICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgICAuZmxhdE1hcChvID0+IG8ubWV0aG9kcy5hdHRycykubGVuZ3RoID4gMFxuICAgICk7XG4gIH1cblxuICBoYXNFbnRpdHlJbnRlZ3JhdGlvblNlcnZjaWNlcygpOiBib29sZWFuIHtcbiAgICBjb25zdCBpbnRlZ3JhdGlvblNlcnZpY2VzID0gbmV3IFNldChcbiAgICAgIHRoaXMuZW50aXRpZXMoKS5mbGF0TWFwKGVudGl0eSA9PlxuICAgICAgICB0aGlzLmVudGl0eWludGVncmF0aW9uU2VydmljZXNGb3IoZW50aXR5KSxcbiAgICAgICksXG4gICAgKTtcbiAgICByZXR1cm4gaW50ZWdyYXRpb25TZXJ2aWNlcy5zaXplID4gMDtcbiAgfVxuXG4gIGVudGl0aWVzKCk6IEVudGl0eU9iamVjdFtdIHtcbiAgICByZXR1cm4gcmVnaXN0cnlcbiAgICAgIC5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUodGhpcy5zZXJ2aWNlTmFtZSlcbiAgICAgIC5maWx0ZXIobyA9PiBvIGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSBhcyBFbnRpdHlPYmplY3RbXTtcbiAgfVxuXG4gIGVudGl0eUFjdGlvbnMoZW50aXR5OiBFbnRpdHlPYmplY3QpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiBlbnRpdHkubWV0aG9kcy5hdHRycy5maWx0ZXIoXG4gICAgICBtID0+IG0gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdO1xuICB9XG5cbiAgZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHk6IEVudGl0eU9iamVjdCk6IEludGVncmF0aW9uU2VydmljZVtdIHtcbiAgICBjb25zdCByZXN1bHQ6IFNldDxJbnRlZ3JhdGlvblNlcnZpY2U+ID0gbmV3IFNldCgpO1xuICAgIGZvciAoY29uc3QgaW50ZWdyYXRpb25TZXJ2aWNlIG9mIGVudGl0eS5pbnRlZ3JhdGlvblNlcnZpY2VzKSB7XG4gICAgICByZXN1bHQuYWRkKGludGVncmF0aW9uU2VydmljZSk7XG4gICAgfVxuICAgIGZvciAoY29uc3QgYWN0aW9uIG9mIHRoaXMuZW50aXR5QWN0aW9ucyhlbnRpdHkpKSB7XG4gICAgICBmb3IgKGNvbnN0IGludGVncmF0aW9uU2VydmljZSBvZiBhY3Rpb24uaW50ZWdyYXRpb25TZXJ2aWNlcykge1xuICAgICAgICByZXN1bHQuYWRkKGludGVncmF0aW9uU2VydmljZSk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiBBcnJheS5mcm9tKHJlc3VsdCk7XG4gIH1cblxuICBlbnRpdHlJbnRlZ3JhdGlvblNlcnZpY2VzKCk6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0aWVzKCkuZmxhdE1hcChlbnRpdHkgPT5cbiAgICAgIHRoaXMuZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHkpLm1hcChpbnRlZ3JhdGlvblNlcnZpY2UgPT4gKHtcbiAgICAgICAgaW50ZWdyYXRpb25OYW1lOiBpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25OYW1lLFxuICAgICAgICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSxcbiAgICAgICAgZW50aXR5OiBlbnRpdHksXG4gICAgICAgIGFnZW50TmFtZTogYCR7c25ha2VDYXNlKFxuICAgICAgICAgIGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvbk5hbWUsXG4gICAgICAgICl9XyR7c25ha2VDYXNlKGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lKX1fJHtzbmFrZUNhc2UoXG4gICAgICAgICAgZW50aXR5LmJhc2VUeXBlTmFtZSxcbiAgICAgICAgKX1gLFxuICAgICAgfSkpLFxuICAgICk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKFwicHViIG1vZCBhZ2VudDtcIik7XG4gICAgfVxuICAgIGlmICh0aGlzLmhhc01vZGVscygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goXCJwdWIgbW9kIG1vZGVsO1wiKTtcbiAgICB9XG4gICAgaWYgKHRoaXMuaGFzU2VydmljZU1ldGhvZHMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKFwicHViIG1vZCBzZXJ2aWNlO1wiKTtcbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2RlbC9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWxNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iamVjdCBvZiByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoXG4gICAgICB0aGlzLnNlcnZpY2VOYW1lLFxuICAgICkpIHtcbiAgICAgIGlmIChzeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSB7XG4gICAgICAgIHJlc3VsdHMucHVzaChgcHViIG1vZCAke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfTtgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kZWwvbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlblNlcnZpY2UoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3Qvc2VydmljZS5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXJTZXJ2aWNlKHRoaXMuc2VydmljZU5hbWUpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vc2VydmljZS5yc2AsIG91dHB1dCk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9tb2RlbC5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXIoc3lzdGVtT2JqZWN0KSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcbiAgICAgIGBnZW4vbW9kZWwvJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ucnNgLFxuICAgICAgb3V0cHV0LFxuICAgICk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9hZ2VudC9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuQWdlbnRNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBmb3IgKGNvbnN0IGFnZW50IG9mIHRoaXMuZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goYHB1YiBtb2QgJHthZ2VudC5hZ2VudE5hbWV9O2ApO1xuICAgIH1cbiAgICByZXN1bHRzLnB1c2goXCJcIik7XG4gICAgZm9yIChjb25zdCBhZ2VudCBvZiB0aGlzLmVudGl0eUludGVncmF0aW9uU2VydmljZXMoKSkge1xuICAgICAgY29uc3QgZm10ID0gbmV3IFJ1c3RGb3JtYXR0ZXJBZ2VudCh0aGlzLnNlcnZpY2VOYW1lLCBhZ2VudCk7XG4gICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgIGBwdWIgdXNlICR7XG4gICAgICAgICAgYWdlbnQuYWdlbnROYW1lXG4gICAgICAgIH06Onske2ZtdC5kaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lKCl9LCAke2ZtdC5kaXNwYXRjaGVyVHlwZU5hbWUoKX19O2AsXG4gICAgICApO1xuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9hZ2VudC9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuQWdlbnQoYWdlbnQ6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvYWdlbnQucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyQWdlbnQodGhpcy5zZXJ2aWNlTmFtZSwgYWdlbnQpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vYWdlbnQvJHtzbmFrZUNhc2UoYWdlbnQuYWdlbnROYW1lKX0ucnNgLCBvdXRwdXQpO1xuICB9XG5cbiAgYXN5bmMgbWFrZVBhdGgocGF0aFBhcnQ6IHN0cmluZyk6IFByb21pc2U8c3RyaW5nPiB7XG4gICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXCIuLlwiLCBgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWAsIFwic3JjXCIsIHBhdGhQYXJ0KTtcbiAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbiAgICBhd2FpdCBmcy5wcm9taXNlcy5ta2RpcihwYXRoLnJlc29sdmUocGF0aE5hbWUpLCB7IHJlY3Vyc2l2ZTogdHJ1ZSB9KTtcbiAgICByZXR1cm4gYWJzb2x1dGVQYXRoTmFtZTtcbiAgfVxuXG4gIGFzeW5jIGZvcm1hdENvZGUoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgYXdhaXQgZXhlY0NtZChgY2FyZ28gZm10IC1wIHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gKTtcbiAgfVxuXG4gIGFzeW5jIHdyaXRlQ29kZShmaWxlbmFtZTogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBwYXRobmFtZSA9IHBhdGguZGlybmFtZShmaWxlbmFtZSk7XG4gICAgY29uc3QgYmFzZW5hbWUgPSBwYXRoLmJhc2VuYW1lKGZpbGVuYW1lKTtcbiAgICBjb25zdCBjcmVhdGVkUGF0aCA9IGF3YWl0IHRoaXMubWFrZVBhdGgocGF0aG5hbWUpO1xuICAgIGNvbnN0IGNvZGVGaWxlbmFtZSA9IHBhdGguam9pbihjcmVhdGVkUGF0aCwgYmFzZW5hbWUpO1xuICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGUpO1xuICB9XG59XG5cbi8vIGV4cG9ydCBjbGFzcyBDb2RlZ2VuUnVzdCB7XG4vLyAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG4vLyAgIGZvcm1hdHRlcjogUnVzdEZvcm1hdHRlcjtcbi8vXG4vLyAgIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpIHtcbi8vICAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbi8vICAgICB0aGlzLmZvcm1hdHRlciA9IG5ldyBSdXN0Rm9ybWF0dGVyKHN5c3RlbU9iamVjdCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIHdyaXRlQ29kZShwYXJ0OiBzdHJpbmcsIGNvZGU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IGNyZWF0ZWRQYXRoID0gYXdhaXQgdGhpcy5tYWtlUGF0aCgpO1xuLy8gICAgIGNvbnN0IGNvZGVGaWxlbmFtZSA9IHBhdGguam9pbihjcmVhdGVkUGF0aCwgYCR7c25ha2VDYXNlKHBhcnQpfS5yc2ApO1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGUpO1xuLy8gICAgIGF3YWl0IGV4ZWNDbWQoYHJ1c3RmbXQgJHtjb2RlRmlsZW5hbWV9YCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIG1ha2VQYXRoKCk6IFByb21pc2U8c3RyaW5nPiB7XG4vLyAgICAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4vLyAgICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICBcIi4uXCIsXG4vLyAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5zaVBhdGhOYW1lLFxuLy8gICAgICAgXCJzcmNcIixcbi8vICAgICAgIFwiZ2VuXCIsXG4vLyAgICAgICBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpLFxuLy8gICAgICk7XG4vLyAgICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgICAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIocGF0aC5yZXNvbHZlKHBhdGhOYW1lKSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4vLyAgICAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIGdlbmVyYXRlQ29tcG9uZW50SW1wbHMoKTogUHJvbWlzZTx2b2lkPiB7XG4vLyAgICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbi8vICAgICAgIFwiPCUtIGluY2x1ZGUoJ3J1c3QvY29tcG9uZW50LnJzLmVqcycsIHsgY29tcG9uZW50OiBjb21wb25lbnQgfSkgJT5cIixcbi8vICAgICAgIHtcbi8vICAgICAgICAgc3lzdGVtT2JqZWN0OiB0aGlzLnN5c3RlbU9iamVjdCxcbi8vICAgICAgICAgZm10OiB0aGlzLmZvcm1hdHRlcixcbi8vICAgICAgIH0sXG4vLyAgICAgICB7XG4vLyAgICAgICAgIGZpbGVuYW1lOiBfX2ZpbGVuYW1lLFxuLy8gICAgICAgfSxcbi8vICAgICApO1xuLy8gICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiY29tcG9uZW50XCIsIG91dHB1dCk7XG4vLyAgIH1cbi8vXG4vLyAgIGFzeW5jIGdlbmVyYXRlQ29tcG9uZW50TW9kKCk6IFByb21pc2U8dm9pZD4ge1xuLy8gICAgIGNvbnN0IG1vZHMgPSBbXCJjb21wb25lbnRcIl07XG4vLyAgICAgY29uc3QgbGluZXMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIFRvdWNoeSFcXG5cIl07XG4vLyAgICAgZm9yIChjb25zdCBtb2Qgb2YgbW9kcykge1xuLy8gICAgICAgbGluZXMucHVzaChgcHViIG1vZCAke21vZH07YCk7XG4vLyAgICAgfVxuLy8gICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwibW9kXCIsIGxpbmVzLmpvaW4oXCJcXG5cIikpO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXIge1xuLy8gICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuLy9cbi8vICAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4vLyAgICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudFR5cGVOYW1lKCk6IHN0cmluZyB7XG4vLyAgICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSk7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudE9yZGVyQnlGaWVsZHMoKTogc3RyaW5nIHtcbi8vICAgICBjb25zdCBvcmRlckJ5RmllbGRzID0gW107XG4vLyAgICAgY29uc3QgY29tcG9uZW50T2JqZWN0ID0gdGhpcy5jb21wb25lbnQuYXNDb21wb25lbnQoKTtcbi8vICAgICBmb3IgKGNvbnN0IHAgb2YgY29tcG9uZW50T2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbi8vICAgICAgIGlmIChwLmhpZGRlbikge1xuLy8gICAgICAgICBjb250aW51ZTtcbi8vICAgICAgIH1cbi8vICAgICAgIGlmIChwLm5hbWUgPT0gXCJzdG9yYWJsZVwiKSB7XG4vLyAgICAgICAgIG9yZGVyQnlGaWVsZHMucHVzaCgnXCJzdG9yYWJsZS5uYXR1cmFsS2V5XCInKTtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKCdcInN0b3JhYmxlLnR5cGVOYW1lXCInKTtcbi8vICAgICAgIH0gZWxzZSBpZiAocC5uYW1lID09IFwic2lQcm9wZXJ0aWVzXCIpIHtcbi8vICAgICAgICAgY29udGludWU7XG4vLyAgICAgICB9IGVsc2UgaWYgKHAubmFtZSA9PSBcImNvbnN0cmFpbnRzXCIgJiYgcC5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuLy8gICAgICAgICAvLyBAdHMtaWdub3JlIHRydXN0IHVzIC0gd2UgY2hlY2tlZFxuLy8gICAgICAgICBmb3IgKGNvbnN0IHBjIG9mIHAucHJvcGVydGllcy5hdHRycykge1xuLy8gICAgICAgICAgIGlmIChwYy5raW5kKCkgIT0gXCJvYmplY3RcIikge1xuLy8gICAgICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKGBcImNvbnN0cmFpbnRzLiR7cGMubmFtZX1cImApO1xuLy8gICAgICAgICAgIH1cbi8vICAgICAgICAgfVxuLy8gICAgICAgfSBlbHNlIHtcbi8vICAgICAgICAgb3JkZXJCeUZpZWxkcy5wdXNoKGBcIiR7cC5uYW1lfVwiYCk7XG4vLyAgICAgICB9XG4vLyAgICAgfVxuLy8gICAgIHJldHVybiBgdmVjIVske29yZGVyQnlGaWVsZHMuam9pbihcIixcIil9XVxcbmA7XG4vLyAgIH1cbi8vXG4vLyAgIGNvbXBvbmVudEltcG9ydHMoKTogc3RyaW5nIHtcbi8vICAgICBjb25zdCByZXN1bHQgPSBbXTtcbi8vICAgICByZXN1bHQucHVzaChcbi8vICAgICAgIGBwdWIgdXNlIGNyYXRlOjpwcm90b2J1Zjo6JHtzbmFrZUNhc2UodGhpcy5jb21wb25lbnQudHlwZU5hbWUpfTo6e2AsXG4vLyAgICAgICBgICBDb25zdHJhaW50cyxgLFxuLy8gICAgICAgYCAgTGlzdENvbXBvbmVudHNSZXBseSxgLFxuLy8gICAgICAgYCAgTGlzdENvbXBvbmVudHNSZXF1ZXN0LGAsXG4vLyAgICAgICBgICBQaWNrQ29tcG9uZW50UmVxdWVzdCxgLFxuLy8gICAgICAgYCAgQ29tcG9uZW50LGAsXG4vLyAgICAgICBgfTtgLFxuLy8gICAgICk7XG4vLyAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuLy8gICB9XG4vL1xuLy8gICBjb21wb25lbnRWYWxpZGF0aW9uKCk6IHN0cmluZyB7XG4vLyAgICAgcmV0dXJuIHRoaXMuZ2VuVmFsaWRhdGlvbih0aGlzLmNvbXBvbmVudC5hc0NvbXBvbmVudCgpKTtcbi8vICAgfVxuLy9cbi8vICAgZ2VuVmFsaWRhdGlvbihwcm9wT2JqZWN0OiBQcm9wT2JqZWN0KTogc3RyaW5nIHtcbi8vICAgICBjb25zdCByZXN1bHQgPSBbXTtcbi8vICAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4vLyAgICAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuLy8gICAgICAgICBjb25zdCBwcm9wTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuLy8gICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5pc19ub25lKCkge1xuLy8gICAgICAgICAgIHJldHVybiBFcnIoRGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4vLyAgICAgICAgIH1gKTtcbi8vICAgICAgIH1cbi8vICAgICB9XG4vLyAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdlbmVyYXRlR2VuTW9kKHdyaXR0ZW5Db21wb25lbnRzOiB7XG4vLyAgIFtrZXk6IHN0cmluZ106IHN0cmluZ1tdO1xuLy8gfSk6IFByb21pc2U8dm9pZD4ge1xuLy8gICBmb3IgKGNvbnN0IGNvbXBvbmVudCBpbiB3cml0dGVuQ29tcG9uZW50cykge1xuLy8gICAgIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFxuLy8gICAgICAgX19kaXJuYW1lLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgXCIuLlwiLFxuLy8gICAgICAgY29tcG9uZW50LFxuLy8gICAgICAgXCJzcmNcIixcbi8vICAgICAgIFwiZ2VuXCIsXG4vLyAgICAgKTtcbi8vICAgICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbi8vICAgICBjb25zdCBjb2RlID0gW1xuLy8gICAgICAgXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLFxuLy8gICAgICAgXCIvLyBObyB0b3VjaHkhXCIsXG4vLyAgICAgICBcIlwiLFxuLy8gICAgICAgXCJwdWIgbW9kIG1vZGVsO1wiLFxuLy8gICAgIF07XG4vL1xuLy8gICAgIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShcbi8vICAgICAgIHBhdGguam9pbihhYnNvbHV0ZVBhdGhOYW1lLCBcIm1vZC5yc1wiKSxcbi8vICAgICAgIGNvZGUuam9pbihcIlxcblwiKSxcbi8vICAgICApO1xuLy8gICB9XG4vLyB9XG4vL1xuLy8gZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdlbmVyYXRlR2VuTW9kTW9kZWwoc2VydmljZU5hbWU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuLy8gICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcbi8vICAgICBfX2Rpcm5hbWUsXG4vLyAgICAgXCIuLlwiLFxuLy8gICAgIFwiLi5cIixcbi8vICAgICBcIi4uXCIsXG4vLyAgICAgc2VydmljZU5hbWUsXG4vLyAgICAgXCJzcmNcIixcbi8vICAgICBcImdlblwiLFxuLy8gICAgIFwibW9kZWxcIixcbi8vICAgKTtcbi8vICAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4vLyAgIGNvbnN0IGNvZGUgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcXG5cIl07XG4vLyAgIGZvciAoY29uc3QgdHlwZU5hbWUgb2Ygd3JpdHRlbkNvbXBvbmVudHNbY29tcG9uZW50XSkge1xuLy8gICAgIGNvZGUucHVzaChgcHViIG1vZCAke3NuYWtlQ2FzZSh0eXBlTmFtZSl9O2ApO1xuLy8gICB9XG4vL1xuLy8gICBhd2FpdCBmcy5wcm9taXNlcy53cml0ZUZpbGUoXG4vLyAgICAgcGF0aC5qb2luKGFic29sdXRlUGF0aE5hbWUsIFwibW9kLnJzXCIpLFxuLy8gICAgIGNvZGUuam9pbihcIlxcblwiKSxcbi8vICAgKTtcbi8vIH1cbiJdfQ==