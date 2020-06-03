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

var _path = _interopRequireDefault(require("path"));

var _child_process = _interopRequireDefault(require("child_process"));

var _util = _interopRequireDefault(require("util"));

var codeFs = _interopRequireWildcard(require("./fs"));

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
    key: "entityActionMethodNames",
    value: function entityActionMethodNames() {
      var results = ["create"];

      if (this.systemObject.kind() == "entityEventObject") {
        // @ts-ignore
        var entity = _registry.registry.get("".concat(this.systemObject.baseTypeName, "Entity"));

        var fmt = new RustFormatter(entity);

        var _iterator = _createForOfIteratorHelper(fmt.actionProps()),
            _step;

        try {
          for (_iterator.s(); !(_step = _iterator.n()).done;) {
            var prop = _step.value;

            if (fmt.isEntityEditMethod(prop)) {
              results.push(fmt.entityEditMethodName(prop));
            } else {
              results.push(prop.name);
            }
          }
        } catch (err) {
          _iterator.e(err);
        } finally {
          _iterator.f();
        }
      } else {
        var _iterator2 = _createForOfIteratorHelper(this.actionProps()),
            _step2;

        try {
          for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
            var _prop = _step2.value;

            if (this.isEntityEditMethod(_prop)) {
              results.push(this.entityEditMethodName(_prop));
            } else {
              results.push(_prop.name);
            }
          }
        } catch (err) {
          _iterator2.e(err);
        } finally {
          _iterator2.f();
        }
      }

      return results;
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
    key: "isChangeSetObject",
    value: function isChangeSetObject() {
      return this.systemObject.typeName == "changeSet";
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

      var _iterator3 = _createForOfIteratorHelper(properties),
          _step3;

      try {
        for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
          var property = _step3.value;
          var propEithers = property.relationships.all().filter(function (rel) {
            return rel instanceof PropPrelude.Either;
          });

          if (propEithers.length > 0) {
            var eithers = new Set();
            eithers.add(property);

            var _iterator4 = _createForOfIteratorHelper(propEithers),
                _step4;

            try {
              for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
                var _property = _step4.value;
                eithers.add(_property.partnerProp());
              }
            } catch (err) {
              _iterator4.e(err);
            } finally {
              _iterator4.f();
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
        _iterator3.e(err);
      } finally {
        _iterator3.f();
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
    key: "implServiceTraceName",
    value: function implServiceTraceName(propMethod) {
      return "".concat(this.systemObject.serviceName, ".").concat((0, _changeCase.snakeCase)(this.rustTypeForProp(propMethod, {
        option: false,
        reference: false
      })));
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
    key: "implServiceChangeSetCreate",
    value: function implServiceChangeSetCreate(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceChangeSetCreate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
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

      var _iterator5 = _createForOfIteratorHelper(propMethods),
          _step5;

      try {
        for (_iterator5.s(); !(_step5 = _iterator5.n()).done;) {
          var propMethod = _step5.value;

          var output = _ejs["default"].render("<%- include('src/codegen/rust/serviceMethod.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
            fmt: this,
            propMethod: propMethod
          }, {
            filename: "."
          });

          results.push(output);
        }
      } catch (err) {
        _iterator5.e(err);
      } finally {
        _iterator5.f();
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
        var _iterator6 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step6;

        try {
          for (_iterator6.s(); !(_step6 = _iterator6.n()).done;) {
            var prop = _step6.value;
            result.push("".concat((0, _changeCase.snakeCase)(prop.name), ": ").concat(this.rustTypeForProp(prop)));
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
    key: "implCreatePassNewArgs",
    value: function implCreatePassNewArgs() {
      var result = [];
      var createMethod = this.systemObject.methods.getEntry("create");

      if (createMethod instanceof PropPrelude.PropMethod) {
        var _iterator7 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step7;

        try {
          for (_iterator7.s(); !(_step7 = _iterator7.n()).done;) {
            var prop = _step7.value;
            result.push((0, _changeCase.snakeCase)(prop.name));
          }
        } catch (err) {
          _iterator7.e(err);
        } finally {
          _iterator7.f();
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
        var _iterator8 = _createForOfIteratorHelper(listMethod.reply.properties.attrs),
            _step8;

        try {
          for (_iterator8.s(); !(_step8 = _iterator8.n()).done;) {
            var prop = _step8.value;
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
          _iterator8.e(err);
        } finally {
          _iterator8.f();
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
        var _iterator9 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step9;

        try {
          for (_iterator9.s(); !(_step9 = _iterator9.n()).done;) {
            var prop = _step9.value;
            var fieldName = (0, _changeCase.snakeCase)(prop.name);
            result.push("let ".concat(fieldName, " = inner.").concat(fieldName, ";"));
          }
        } catch (err) {
          _iterator9.e(err);
        } finally {
          _iterator9.f();
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
        var _iterator10 = _createForOfIteratorHelper(createMethod.request.properties.attrs),
            _step10;

        try {
          for (_iterator10.s(); !(_step10 = _iterator10.n()).done;) {
            var prop = _step10.value;
            var variableName = (0, _changeCase.snakeCase)(prop.name);

            if (prop instanceof PropPrelude.PropPassword) {
              result.push("result.".concat(variableName, " = Some(si_data::password::encrypt_password(").concat(variableName, ")?);"));
            } else {
              result.push("result.".concat(variableName, " = ").concat(variableName, ";"));
            }
          }
        } catch (err) {
          _iterator10.e(err);
        } finally {
          _iterator10.f();
        }
      }

      var _iterator11 = _createForOfIteratorHelper(this.systemObject.fields.attrs),
          _step11;

      try {
        for (_iterator11.s(); !(_step11 = _iterator11.n()).done;) {
          var _prop2 = _step11.value;

          var _variableName = (0, _changeCase.snakeCase)(_prop2.name);

          var defaultValue = _prop2.defaultValue();

          if (defaultValue) {
            if (_prop2.kind() == "text") {
              result.push("result.".concat(_variableName, " = \"").concat(defaultValue, "\".to_string();"));
            } else if (_prop2.kind() == "enum") {
              var enumName = "".concat((0, _changeCase.pascalCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(_prop2.name));
              result.push("result.set_".concat(_variableName, "(crate::protobuf::").concat(enumName, "::").concat((0, _changeCase.pascalCase)(defaultValue), ");"));
            }
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

      var _iterator12 = _createForOfIteratorHelper(this.systemObject.fields.attrs),
          _step12;

      try {
        for (_iterator12.s(); !(_step12 = _iterator12.n()).done;) {
          var prop = _step12.value;

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
        _iterator12.e(err);
      } finally {
        _iterator12.f();
      }

      return result.join("\n");
    }
  }, {
    key: "storableOrderByFieldsByProp",
    value: function storableOrderByFieldsByProp(topProp, prefix) {
      var results = ['"siStorable.naturalKey"'];

      var _iterator13 = _createForOfIteratorHelper(topProp.properties.attrs),
          _step13;

      try {
        for (_iterator13.s(); !(_step13 = _iterator13.n()).done;) {
          var prop = _step13.value;

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
        _iterator13.e(err);
      } finally {
        _iterator13.f();
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

        var _iterator14 = _createForOfIteratorHelper(siProperties.properties.attrs),
            _step14;

        try {
          for (_iterator14.s(); !(_step14 = _iterator14.n()).done;) {
            var prop = _step14.value;

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
          _iterator14.e(err);
        } finally {
          _iterator14.f();
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

      var _iterator15 = _createForOfIteratorHelper(this.systemObjects),
          _step15;

      try {
        for (_iterator15.s(); !(_step15 = _iterator15.n()).done;) {
          var systemObj = _step15.value;

          if (this.isMigrateable(systemObj)) {
            result.push("crate::protobuf::".concat((0, _changeCase.pascalCase)(systemObj.typeName), "::migrate(&self.db).await?;"));
          }
        }
      } catch (err) {
        _iterator15.e(err);
      } finally {
        _iterator15.f();
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

      var _iterator16 = _createForOfIteratorHelper(this.actionProps()),
          _step16;

      try {
        for (_iterator16.s(); !(_step16 = _iterator16.n()).done;) {
          var prop = _step16.value;

          if (this.entityFormatter.isEntityEditMethod(prop)) {
            results.push(this.entityFormatter.entityEditMethodName(prop));
          } else {
            results.push(prop.name);
          }
        }
      } catch (err) {
        _iterator16.e(err);
      } finally {
        _iterator16.f();
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

      var _iterator17 = _createForOfIteratorHelper(entity.integrationServices),
          _step17;

      try {
        for (_iterator17.s(); !(_step17 = _iterator17.n()).done;) {
          var integrationService = _step17.value;
          result.add(integrationService);
        }
      } catch (err) {
        _iterator17.e(err);
      } finally {
        _iterator17.f();
      }

      var _iterator18 = _createForOfIteratorHelper(this.entityActions(entity)),
          _step18;

      try {
        for (_iterator18.s(); !(_step18 = _iterator18.n()).done;) {
          var action = _step18.value;

          var _iterator19 = _createForOfIteratorHelper(action.integrationServices),
              _step19;

          try {
            for (_iterator19.s(); !(_step19 = _iterator19.n()).done;) {
              var _integrationService = _step19.value;
              result.add(_integrationService);
            }
          } catch (err) {
            _iterator19.e(err);
          } finally {
            _iterator19.f();
          }
        }
      } catch (err) {
        _iterator18.e(err);
      } finally {
        _iterator18.f();
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
        var results, _iterator20, _step20, systemObject;

        return _regenerator["default"].wrap(function _callee2$(_context2) {
          while (1) {
            switch (_context2.prev = _context2.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator20 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(this.serviceName));

                try {
                  for (_iterator20.s(); !(_step20 = _iterator20.n()).done;) {
                    systemObject = _step20.value;

                    if (systemObject.kind() != "baseObject") {
                      results.push("pub mod ".concat((0, _changeCase.snakeCase)(systemObject.typeName), ";"));
                    }
                  }
                } catch (err) {
                  _iterator20.e(err);
                } finally {
                  _iterator20.f();
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
        var results, _iterator21, _step21, agent, _iterator22, _step22, _agent, fmt;

        return _regenerator["default"].wrap(function _callee5$(_context5) {
          while (1) {
            switch (_context5.prev = _context5.next) {
              case 0:
                results = ["// Auto-generated code!", "// No touchy!", ""];
                _iterator21 = _createForOfIteratorHelper(this.entityIntegrationServices());

                try {
                  for (_iterator21.s(); !(_step21 = _iterator21.n()).done;) {
                    agent = _step21.value;
                    results.push("pub mod ".concat(agent.agentName, ";"));
                  }
                } catch (err) {
                  _iterator21.e(err);
                } finally {
                  _iterator21.f();
                }

                results.push("");
                _iterator22 = _createForOfIteratorHelper(this.entityIntegrationServices());

                try {
                  for (_iterator22.s(); !(_step22 = _iterator22.n()).done;) {
                    _agent = _step22.value;
                    fmt = new RustFormatterAgent(this.serviceName, _agent);
                    results.push("pub use ".concat(_agent.agentName, "::{").concat(fmt.dispatchFunctionTraitName(), ", ").concat(fmt.dispatcherTypeName(), "};"));
                  }
                } catch (err) {
                  _iterator22.e(err);
                } finally {
                  _iterator22.f();
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
    }() //async makePath(pathPart: string): Promise<string> {
    //  const pathName = path.join("..", `si-${this.serviceName}`, "src", pathPart);
    //  const absolutePathName = path.resolve(pathName);
    //  await fs.promises.mkdir(path.resolve(pathName), { recursive: true });
    //  return absolutePathName;
    //}

  }, {
    key: "formatCode",
    value: function () {
      var _formatCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee7() {
        return _regenerator["default"].wrap(function _callee7$(_context7) {
          while (1) {
            switch (_context7.prev = _context7.next) {
              case 0:
                _context7.next = 2;
                return execCmd("cargo fmt -p si-".concat(this.serviceName));

              case 2:
              case "end":
                return _context7.stop();
            }
          }
        }, _callee7, this);
      }));

      function formatCode() {
        return _formatCode.apply(this, arguments);
      }

      return formatCode;
    }()
  }, {
    key: "writeCode",
    value: function () {
      var _writeCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee8(filename, code) {
        var fullPathName;
        return _regenerator["default"].wrap(function _callee8$(_context8) {
          while (1) {
            switch (_context8.prev = _context8.next) {
              case 0:
                fullPathName = _path["default"].join("..", "si-".concat(this.serviceName), "src", filename);
                _context8.next = 3;
                return codeFs.writeCode(fullPathName, code);

              case 3:
              case "end":
                return _context8.stop();
            }
          }
        }, _callee8, this);
      }));

      function writeCode(_x3, _x4) {
        return _writeCode.apply(this, arguments);
      }

      return writeCode;
    }()
  }]);
  return CodegenRust;
}();

exports.CodegenRust = CodegenRust;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInJlc3VsdHMiLCJraW5kIiwiZW50aXR5IiwicmVnaXN0cnkiLCJnZXQiLCJiYXNlVHlwZU5hbWUiLCJmbXQiLCJhY3Rpb25Qcm9wcyIsInByb3AiLCJpc0VudGl0eUVkaXRNZXRob2QiLCJwdXNoIiwiZW50aXR5RWRpdE1ldGhvZE5hbWUiLCJuYW1lIiwibWV0aG9kcyIsImdldEVudHJ5IiwicHJvcEFjdGlvbiIsImVudGl0eUVkaXRQcm9wZXJ0eSIsInJlbGF0aW9uc2hpcHMiLCJhbGwiLCJzb21lIiwicmVsIiwiUHJvcFByZWx1ZGUiLCJFaXRoZXIiLCJVcGRhdGVzIiwiaXNFbnRpdHlPYmplY3QiLCJlbnRpdHlFZGl0TWV0aG9kcyIsImhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uIiwiQ29tcG9uZW50T2JqZWN0IiwicHJvcE1ldGhvZCIsIlByb3BBY3Rpb24iLCJpc0VudGl0eUFjdGlvbk1ldGhvZCIsImVuZHNXaXRoIiwiRW50aXR5RXZlbnRPYmplY3QiLCJFbnRpdHlPYmplY3QiLCJ0eXBlTmFtZSIsIlN5c3RlbU9iamVjdCIsIm1pZ3JhdGVhYmxlIiwiYXR0cnMiLCJmaWx0ZXIiLCJtIiwicnVzdEZpZWxkTmFtZUZvclByb3AiLCJyZXBsYWNlIiwicCIsInByb3BlcnR5IiwicmVxdWVzdCIsInByb3BlcnRpZXMiLCJQcm9wTGluayIsImxvb2t1cE15c2VsZiIsInJ1c3RUeXBlRm9yUHJvcCIsIm9wdGlvbiIsInIiLCJtYXAiLCJ1cGRhdGUiLCJmcm9tIiwidG8iLCJwYXJ0bmVyUHJvcCIsImZsYXRNYXAiLCJtZXRob2QiLCJlbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzIiwiQXJyYXkiLCJTZXQiLCJzb3J0IiwiYSIsImIiLCJNYXAiLCJmaWVsZHMiLCJwcm9wRWl0aGVycyIsImxlbmd0aCIsImVpdGhlcnMiLCJhZGQiLCJlaXRoZXJzQXJyYXkiLCJzZXQiLCJlIiwiam9pbiIsImVudHJpZXMiLCJ2YWx1ZXMiLCJwcm9wZXJ0eVVwZGF0ZSIsInNlcnZpY2VOYW1lIiwiUHJvcENvZGUiLCJsYW5ndWFnZSIsIlByb3BPYmplY3QiLCJyZW5kZXJPcHRpb25zIiwibGlzdCIsInJlcGx5IiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicHJvcE1ldGhvZHMiLCJvdXRwdXQiLCJQcm9wTWV0aG9kIiwicGFyZW50TmFtZSIsIlByb3BOdW1iZXIiLCJudW1iZXJLaW5kIiwiUHJvcEJvb2wiLCJyZWFsUHJvcCIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcE1hcCIsIlByb3BUZXh0IiwiUHJvcFNlbGVjdCIsInJlcGVhdGVkIiwicmVzdWx0IiwiY3JlYXRlTWV0aG9kIiwibGlzdE1ldGhvZCIsImZpZWxkTmFtZSIsImxpc3RSZXBseVZhbHVlIiwibmF0dXJhbEtleSIsInZhcmlhYmxlTmFtZSIsIlByb3BQYXNzd29yZCIsImRlZmF1bHRWYWx1ZSIsImVudW1OYW1lIiwibXZjYyIsInJlcXVpcmVkIiwicHJvcE5hbWUiLCJ0b3BQcm9wIiwicHJlZml4IiwiaGlkZGVuIiwic3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wIiwicm9vdFByb3AiLCJmZXRjaFByb3BzIiwicmVmZXJlbmNlVmVjIiwic2lQcm9wZXJ0aWVzIiwiaXRlbU5hbWUiLCJCYXNlT2JqZWN0IiwiUnVzdEZvcm1hdHRlclNlcnZpY2UiLCJzeXN0ZW1PYmplY3RzIiwiZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lIiwibyIsImhhc0VudGl0aWVzIiwiaW1wbFNlcnZpY2VUcmFpdE5hbWUiLCJzeXN0ZW1PYmoiLCJpc01pZ3JhdGVhYmxlIiwib2JqIiwiUnVzdEZvcm1hdHRlckFnZW50IiwiYWdlbnQiLCJhZ2VudE5hbWUiLCJlbnRpdHlGb3JtYXR0ZXIiLCJpbnRlZ3JhdGlvbk5hbWUiLCJpbnRlZ3JhdGlvblNlcnZpY2VOYW1lIiwiZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSIsIkNvZGVnZW5SdXN0IiwiaW50ZWdyYXRpb25TZXJ2aWNlcyIsImVudGl0aWVzIiwiZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvciIsInNpemUiLCJpbnRlZ3JhdGlvblNlcnZpY2UiLCJlbnRpdHlBY3Rpb25zIiwiYWN0aW9uIiwiaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMiLCJoYXNNb2RlbHMiLCJoYXNTZXJ2aWNlTWV0aG9kcyIsIndyaXRlQ29kZSIsImVudGl0eUludGVncmF0aW9uU2VydmljZXMiLCJkaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lIiwiZGlzcGF0Y2hlclR5cGVOYW1lIiwiY29kZSIsImZ1bGxQYXRoTmFtZSIsInBhdGgiLCJjb2RlRnMiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFBOztBQVFBOztBQUNBOztBQUdBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOzs7Ozs7OztBQUVBLElBQU1BLE9BQU8sR0FBR0MsaUJBQUtDLFNBQUwsQ0FBZUMsMEJBQWFDLElBQTVCLENBQWhCOztJQXVCYUMsYTtBQUdYLHlCQUFZQyxZQUFaLEVBQXlEO0FBQUE7QUFBQTtBQUN2RCxTQUFLQSxZQUFMLEdBQW9CQSxZQUFwQjtBQUNEOzs7OzhDQUVtQztBQUNsQyxVQUFNQyxPQUFPLEdBQUcsQ0FBQyxRQUFELENBQWhCOztBQUVBLFVBQUksS0FBS0QsWUFBTCxDQUFrQkUsSUFBbEIsTUFBNEIsbUJBQWhDLEVBQXFEO0FBQ25EO0FBQ0EsWUFBTUMsTUFBTSxHQUFHQyxtQkFBU0MsR0FBVCxXQUFnQixLQUFLTCxZQUFMLENBQWtCTSxZQUFsQyxZQUFmOztBQUNBLFlBQU1DLEdBQUcsR0FBRyxJ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QUFLSCxTQUFMLENBQWUsa0JBQWYsRUFBbUM3SixPQUFPLENBQUN5RSxJQUFSLENBQWEsSUFBYixDQUFuQyxDOzs7Ozs7Ozs7Ozs7Ozs7Ozs7OzhIQUdlbUUsSzs7Ozs7O0FBQ2YvQyxnQkFBQUEsTSxHQUFTUixnQkFBSUMsTUFBSixDQUNiLCtEQURhLEVBRWI7QUFDRWhGLGtCQUFBQSxHQUFHLEVBQUUsSUFBSXFJLGtCQUFKLENBQXVCLEtBQUs5RCxXQUE1QixFQUF5QytELEtBQXpDO0FBRFAsaUJBRmEsRUFLYjtBQUNFckQsa0JBQUFBLFFBQVEsRUFBRTtBQURaLGlCQUxhLEM7O3VCQVNULEtBQUtzRSxTQUFMLHFCQUE0QiwyQkFBVWpCLEtBQUssQ0FBQ0MsU0FBaEIsQ0FBNUIsVUFBNkRoRCxNQUE3RCxDOzs7Ozs7Ozs7Ozs7Ozs7UUFHUjtBQUNBO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7Ozs7Ozs7Ozs7O3VCQUdRcEcsT0FBTywyQkFBb0IsS0FBS29GLFdBQXpCLEU7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7dUhBR0NVLFEsRUFBa0IwRSxJOzs7Ozs7QUFDMUJDLGdCQUFBQSxZLEdBQWVDLGlCQUFLMUYsSUFBTCxDQUNuQixJQURtQixlQUViLEtBQUtJLFdBRlEsR0FHbkIsS0FIbUIsRUFJbkJVLFFBSm1CLEM7O3VCQU1mNkUsTUFBTSxDQUFDUCxTQUFQLENBQWlCSyxZQUFqQixFQUErQkQsSUFBL0IsQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIE9iamVjdFR5cGVzLFxuICBCYXNlT2JqZWN0LFxuICBTeXN0ZW1PYmplY3QsXG4gIENvbXBvbmVudE9iamVjdCxcbiAgRW50aXR5T2JqZWN0LFxuICBFbnRpdHlFdmVudE9iamVjdCxcbn0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0ICogYXMgUHJvcFByZWx1ZGUgZnJvbSBcIi4uL2NvbXBvbmVudHMvcHJlbHVkZVwiO1xuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3BzLCBJbnRlZ3JhdGlvblNlcnZpY2UgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcblxuaW1wb3J0IHsgc25ha2VDYXNlLCBwYXNjYWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5pbXBvcnQgZWpzIGZyb20gXCJlanNcIjtcbmltcG9ydCBwYXRoIGZyb20gXCJwYXRoXCI7XG5pbXBvcnQgY2hpbGRQcm9jZXNzIGZyb20gXCJjaGlsZF9wcm9jZXNzXCI7XG5pbXBvcnQgdXRpbCBmcm9tIFwidXRpbFwiO1xuaW1wb3J0ICogYXMgY29kZUZzIGZyb20gXCIuL2ZzXCI7XG5cbmNvbnN0IGV4ZWNDbWQgPSB1dGlsLnByb21pc2lmeShjaGlsZFByb2Nlc3MuZXhlYyk7XG5cbmludGVyZmFjZSBSdXN0VHlwZUFzUHJvcE9wdGlvbnMge1xuICByZWZlcmVuY2U/OiBib29sZWFuO1xuICBvcHRpb24/OiBib29sZWFuO1xufVxuXG5pbnRlcmZhY2UgQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2Uge1xuICBhZ2VudE5hbWU6IHN0cmluZztcbiAgZW50aXR5OiBFbnRpdHlPYmplY3Q7XG4gIGludGVncmF0aW9uTmFtZTogc3RyaW5nO1xuICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBzdHJpbmc7XG59XG5cbmludGVyZmFjZSBQcm9wZXJ0eVVwZGF0ZSB7XG4gIGZyb206IFByb3BQcmVsdWRlLlByb3BzO1xuICB0bzogUHJvcFByZWx1ZGUuUHJvcHM7XG59XG5cbmludGVyZmFjZSBQcm9wZXJ0eUVpdGhlclNldCB7XG4gIGVudHJpZXM6IFByb3BQcmVsdWRlLlByb3BzW107XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyIHtcbiAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcblxuICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFJ1c3RGb3JtYXR0ZXJbXCJzeXN0ZW1PYmplY3RcIl0pIHtcbiAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbiAgfVxuXG4gIGVudGl0eUFjdGlvbk1ldGhvZE5hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiY3JlYXRlXCJdO1xuXG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImVudGl0eUV2ZW50T2JqZWN0XCIpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgIGNvbnN0IGVudGl0eSA9IHJlZ2lzdHJ5LmdldChgJHt0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWV9RW50aXR5YCk7XG4gICAgICBjb25zdCBmbXQgPSBuZXcgUnVzdEZvcm1hdHRlcihlbnRpdHkpO1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGZtdC5hY3Rpb25Qcm9wcygpKSB7XG4gICAgICAgIGlmIChmbXQuaXNFbnRpdHlFZGl0TWV0aG9kKHByb3ApKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGZtdC5lbnRpdHlFZGl0TWV0aG9kTmFtZShwcm9wKSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHByb3AubmFtZSk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9IGVsc2Uge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuYWN0aW9uUHJvcHMoKSkge1xuICAgICAgICBpZiAodGhpcy5pc0VudGl0eUVkaXRNZXRob2QocHJvcCkpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5lbnRpdHlFZGl0TWV0aG9kTmFtZShwcm9wKSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHByb3AubmFtZSk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG5cbiAgICByZXR1cm4gcmVzdWx0cztcbiAgfVxuXG4gIGhhc0NyZWF0ZU1ldGhvZCgpOiBib29sZWFuIHtcbiAgICB0cnkge1xuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICAgIHJldHVybiB0cnVlO1xuICAgIH0gY2F0Y2gge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuXG4gIGhhc0VkaXRFaXRoZXJzRm9yQWN0aW9uKHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbilcbiAgICAgIC5yZWxhdGlvbnNoaXBzLmFsbCgpXG4gICAgICAuc29tZShyZWwgPT4gcmVsIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuRWl0aGVyKTtcbiAgfVxuXG4gIGhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uKHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbilcbiAgICAgIC5yZWxhdGlvbnNoaXBzLmFsbCgpXG4gICAgICAuc29tZShyZWwgPT4gcmVsIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuVXBkYXRlcyk7XG4gIH1cblxuICBoYXNFZGl0VXBkYXRlc0FuZEVpdGhlcnMoKTogYm9vbGVhbiB7XG4gICAgaWYgKHRoaXMuaXNFbnRpdHlPYmplY3QoKSkge1xuICAgICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdE1ldGhvZHMoKS5zb21lKFxuICAgICAgICBwcm9wQWN0aW9uID0+XG4gICAgICAgICAgdGhpcy5oYXNFZGl0VXBkYXRlc0ZvckFjdGlvbihwcm9wQWN0aW9uKSAmJlxuICAgICAgICAgIHRoaXMuaGFzRWRpdFVwZGF0ZXNGb3JBY3Rpb24ocHJvcEFjdGlvbiksXG4gICAgICApO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSByYW4gJ2hhc0VkaXRVcGRhdGVzQW5kRWl0aGVycygpJyBvbiBhIG5vbi1lbnRpdHkgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGlzQ29tcG9uZW50T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdDtcbiAgfVxuXG4gIGlzRW50aXR5QWN0aW9uTWV0aG9kKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgdGhpcy5pc0VudGl0eU9iamVjdCgpICYmIHByb3BNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uXG4gICAgKTtcbiAgfVxuXG4gIGlzRW50aXR5RWRpdE1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHRoaXMuaXNFbnRpdHlBY3Rpb25NZXRob2QocHJvcE1ldGhvZCkgJiYgcHJvcE1ldGhvZC5uYW1lLmVuZHNXaXRoKFwiRWRpdFwiKVxuICAgICk7XG4gIH1cblxuICBpc0VudGl0eUV2ZW50T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0O1xuICB9XG5cbiAgaXNFbnRpdHlPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0O1xuICB9XG5cbiAgaXNDaGFuZ2VTZXRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiY2hhbmdlU2V0XCI7XG4gIH1cblxuICBpc01pZ3JhdGVhYmxlKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCAmJiB0aGlzLnN5c3RlbU9iamVjdC5taWdyYXRlYWJsZVxuICAgICk7XG4gIH1cblxuICBpc1N0b3JhYmxlKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdDtcbiAgfVxuXG4gIGFjdGlvblByb3BzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMuZmlsdGVyKFxuICAgICAgbSA9PiBtIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXTtcbiAgfVxuXG4gIGNvbXBvbmVudE5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9Q29tcG9uZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGNvbXBvbmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgY29tcG9uZW50Q29uc3RyYWludHNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUNvbXBvbmVudENvbnN0cmFpbnRzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGEgY29tcG9uZW50IGNvbnN0cmFpbnRzIG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlFZGl0TWV0aG9kTmFtZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICAgIHJldHVybiBgZWRpdF8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcE1ldGhvZCkucmVwbGFjZShcbiAgICAgICAgXCJfZWRpdFwiLFxuICAgICAgICBcIlwiLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZWRpdCBtZXRob2QgbmFtZSBvbiBhIG5vbi1lbnRpdHkgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eUVkaXRNZXRob2RzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuYWN0aW9uUHJvcHMoKS5maWx0ZXIocCA9PiB0aGlzLmlzRW50aXR5RWRpdE1ldGhvZChwKSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IFByb3BzIHtcbiAgICBsZXQgcHJvcGVydHkgPSBwcm9wQWN0aW9uLnJlcXVlc3QucHJvcGVydGllcy5nZXRFbnRyeShcInByb3BlcnR5XCIpO1xuICAgIGlmIChwcm9wZXJ0eSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBwcm9wZXJ0eSA9IHByb3BlcnR5Lmxvb2t1cE15c2VsZigpO1xuICAgIH1cbiAgICByZXR1cm4gcHJvcGVydHk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlGaWVsZChwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcCh0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlUeXBlKHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcCh0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKSwge1xuICAgICAgb3B0aW9uOiBmYWxzZSxcbiAgICB9KTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZXMoXG4gICAgcHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogUHJvcGVydHlVcGRhdGVbXSB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLmZpbHRlcihyID0+IHIgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5VcGRhdGVzKVxuICAgICAgLm1hcCh1cGRhdGUgPT4gKHtcbiAgICAgICAgZnJvbTogdGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbiksXG4gICAgICAgIHRvOiB1cGRhdGUucGFydG5lclByb3AoKSxcbiAgICAgIH0pKTtcbiAgfVxuXG4gIGFsbEVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZXMoKTogUHJvcGVydHlVcGRhdGVbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IHRoaXMuZW50aXR5RWRpdE1ldGhvZHMoKS5mbGF0TWFwKG1ldGhvZCA9PlxuICAgICAgdGhpcy5lbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzKG1ldGhvZCksXG4gICAgKTtcblxuICAgIHJldHVybiBBcnJheS5mcm9tKG5ldyBTZXQocmVzdWx0cykpLnNvcnQoKGEsIGIpID0+XG4gICAgICBgJHthLmZyb20ubmFtZX0sJHthLnRvLm5hbWV9YCA+IGAke2IuZnJvbS5uYW1lfSwke2IudG8ubmFtZX1gID8gMSA6IC0xLFxuICAgICk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlFaXRoZXJzKCk6IFByb3BlcnR5RWl0aGVyU2V0W10ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBuZXcgTWFwKCk7XG4gICAgY29uc3QgcHJvcGVydGllcyA9ICh0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuZ2V0RW50cnkoXG4gICAgICBcInByb3BlcnRpZXNcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BPYmplY3QpLnByb3BlcnRpZXMuYXR0cnM7XG5cbiAgICBmb3IgKGNvbnN0IHByb3BlcnR5IG9mIHByb3BlcnRpZXMpIHtcbiAgICAgIGNvbnN0IHByb3BFaXRoZXJzID0gcHJvcGVydHkucmVsYXRpb25zaGlwc1xuICAgICAgICAuYWxsKClcbiAgICAgICAgLmZpbHRlcihyZWwgPT4gcmVsIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuRWl0aGVyKTtcblxuICAgICAgaWYgKHByb3BFaXRoZXJzLmxlbmd0aCA+IDApIHtcbiAgICAgICAgY29uc3QgZWl0aGVycyA9IG5ldyBTZXQ8UHJvcFByZWx1ZGUuUHJvcHM+KCk7XG4gICAgICAgIGVpdGhlcnMuYWRkKHByb3BlcnR5KTtcbiAgICAgICAgZm9yIChjb25zdCBwcm9wZXJ0eSBvZiBwcm9wRWl0aGVycykge1xuICAgICAgICAgIGVpdGhlcnMuYWRkKHByb3BlcnR5LnBhcnRuZXJQcm9wKCkpO1xuICAgICAgICB9XG5cbiAgICAgICAgY29uc3QgZWl0aGVyc0FycmF5ID0gQXJyYXkuZnJvbShlaXRoZXJzKS5zb3J0KChhLCBiKSA9PlxuICAgICAgICAgIGEubmFtZSA+IGIubmFtZSA/IDEgOiAtMSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0cy5zZXQoZWl0aGVyc0FycmF5Lm1hcChlID0+IGUubmFtZSkuam9pbihcIixcIiksIHtcbiAgICAgICAgICBlbnRyaWVzOiBlaXRoZXJzQXJyYXksXG4gICAgICAgIH0pO1xuICAgICAgfVxuICAgIH1cblxuICAgIHJldHVybiBBcnJheS5mcm9tKHJlc3VsdHMudmFsdWVzKCkpLnNvcnQoKTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZU1ldGhvZE5hbWUocHJvcGVydHlVcGRhdGU6IFByb3BlcnR5VXBkYXRlKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYHVwZGF0ZV8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AoXG4gICAgICBwcm9wZXJ0eVVwZGF0ZS50byxcbiAgICApfV9mcm9tXyR7dGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wZXJ0eVVwZGF0ZS5mcm9tKX1gO1xuICB9XG5cbiAgZW50aXR5RXZlbnROYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eUV2ZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eUV2ZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHkgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eVByb3BlcnRpZXNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eVByb3BlcnRpZXNgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5UHJvcGVydGllcyBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZXJyb3JUeXBlKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6ZXJyb3I6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSl9RXJyb3JgO1xuICB9XG5cbiAgbW9kZWxOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6bW9kZWw6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIG1vZGVsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcE1ldGhvZCk7XG4gIH1cblxuICBzdHJ1Y3ROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIHR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSk7XG4gIH1cblxuICBpbXBsVHJ5RnJvbUZvclByb3BlcnR5VXBkYXRlKHByb3BlcnR5VXBkYXRlOiBQcm9wZXJ0eVVwZGF0ZSk6IHN0cmluZyB7XG4gICAgY29uc3QgZnJvbSA9IHByb3BlcnR5VXBkYXRlLmZyb207XG4gICAgY29uc3QgdG8gPSBwcm9wZXJ0eVVwZGF0ZS50bztcblxuICAgIC8vIEV2ZXJ5IGZhbGx0aHJvdWdoL2RlZmF1bHQvZWxzZSBuZWVkcyBhIGB0aHJvd2AgY2xhdXNlIHRvIGxvdWRseSBwcm9jbGFpbVxuICAgIC8vIHRoYXQgYSBzcGVjaWZpYyBjb252ZXJzaW9uIGlzIG5vdCBzdXBwb3J0ZWQuIFRoaXMgYWxsb3dzIHVzIHRvIGFkZFxuICAgIC8vIGNvbnZlcnNpb25zIGFzIHdlIGdvIHdpdGhvdXQgcm9ndWUgYW5kIHVuZXhwbGFpbmVkIGVycm9ycy4gSW4gc2hvcnQsXG4gICAgLy8gdHJlYXQgdGhpcyBsaWtlIFJ1c3QgY29kZSB3aXRoIGZ1bGx5IHNhdGlzZmllZCBtYXRjaCBhcm1zLiBUaGFuayB5b3UsXG4gICAgLy8gbG92ZSwgdXMuXG4gICAgaWYgKGZyb20gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSkge1xuICAgICAgc3dpdGNoIChmcm9tLmxhbmd1YWdlKSB7XG4gICAgICAgIGNhc2UgXCJ5YW1sXCI6XG4gICAgICAgICAgaWYgKHRvIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICAgICAgcmV0dXJuIGBPayhzZXJkZV95YW1sOjpmcm9tX3N0cih2YWx1ZSk/KWA7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gbGFuZ3VhZ2UgJyR7XG4gICAgICAgICAgICAgIGZyb20ubGFuZ3VhZ2VcbiAgICAgICAgICAgIH0nIHRvIHR5cGUgJyR7dG8ua2luZCgpfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgICAgICAgfVxuICAgICAgICBkZWZhdWx0OlxuICAgICAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gbGFuZ3VhZ2UgJyR7ZnJvbS5sYW5ndWFnZX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoZnJvbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgIGlmICh0byBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlKSB7XG4gICAgICAgIHN3aXRjaCAodG8ubGFuZ3VhZ2UpIHtcbiAgICAgICAgICBjYXNlIFwieWFtbFwiOlxuICAgICAgICAgICAgcmV0dXJuIGBPayhzZXJkZV95YW1sOjp0b19zdHJpbmcodmFsdWUpPylgO1xuICAgICAgICAgIGRlZmF1bHQ6XG4gICAgICAgICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIFByb3BPYmplY3QgdG8gbGFuZ3VhZ2UgJyR7dG8ubGFuZ3VhZ2V9JyBpcyBub3Qgc3VwcG9ydGVkYDtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgdGhyb3cgYGNvbnZlcnNpb24gZnJvbSBQcm9wT2JqZWN0IHRvIHR5cGUgJyR7dG8ua2luZCgpfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgICB9XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gdHlwZSAnJHtmcm9tLmtpbmQoKX0nIHRvIHR5cGUgJyR7dG8ua2luZCgpfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgfVxuICB9XG5cbiAgaW1wbExpc3RSZXF1ZXN0VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVxdWVzdCwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsTGlzdFJlcGx5VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXF1ZXN0VHlwZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXBseVR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXBseSwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVRyYWNlTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWV9LiR7c25ha2VDYXNlKFxuICAgICAgdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZCwge1xuICAgICAgICBvcHRpb246IGZhbHNlLFxuICAgICAgICByZWZlcmVuY2U6IGZhbHNlLFxuICAgICAgfSksXG4gICAgKX1gO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UoXG4gICAgICB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLCB7XG4gICAgICAgIG9wdGlvbjogZmFsc2UsXG4gICAgICAgIHJlZmVyZW5jZTogZmFsc2UsXG4gICAgICB9KSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlBY3Rpb24ocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5QWN0aW9uLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUVkaXQocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5RWRpdC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDb21tb25DcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUNoYW5nZVNldENyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDaGFuZ2VTZXRDcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5Q3JlYXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUNyZWF0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VHZXQocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlR2V0LnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUxpc3QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlTGlzdC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDb21wb25lbnRQaWNrKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUNvbXBvbmVudFBpY2sucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ3VzdG9tTWV0aG9kKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUN1c3RvbU1ldGhvZC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGlmIChwcm9wTWV0aG9kLnNraXBBdXRoKSB7XG4gICAgICByZXR1cm4gYC8vIEF1dGhlbnRpY2F0aW9uIGlzIHNraXBwZWQgb24gXFxgJHt0aGlzLmltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICAgICAgcHJvcE1ldGhvZCxcbiAgICAgICl9XFxgXFxuYDtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHRoaXMuaW1wbFNlcnZpY2VBdXRoQ2FsbChwcm9wTWV0aG9kKTtcbiAgICB9XG4gIH1cblxuICBpbXBsU2VydmljZUF1dGhDYWxsKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIGxldCBwcmVsdWRlID0gXCJzaV9hY2NvdW50OjphdXRob3JpemVcIjtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWUgPT0gXCJhY2NvdW50XCIpIHtcbiAgICAgIHByZWx1ZGUgPSBcImNyYXRlOjphdXRob3JpemVcIjtcbiAgICB9XG4gICAgcmV0dXJuIGAke3ByZWx1ZGV9OjphdXRobnooJnNlbGYuZGIsICZyZXF1ZXN0LCBcIiR7dGhpcy5pbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgICBwcm9wTWV0aG9kLFxuICAgICl9XCIpLmF3YWl0PztgO1xuICB9XG5cbiAgc2VydmljZU1ldGhvZHMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgY29uc3QgcHJvcE1ldGhvZHMgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmF0dHJzLnNvcnQoKGEsIGIpID0+XG4gICAgICBhLm5hbWUgPiBiLm5hbWUgPyAxIDogLTEsXG4gICAgKTtcbiAgICBmb3IgKGNvbnN0IHByb3BNZXRob2Qgb2YgcHJvcE1ldGhvZHMpIHtcbiAgICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3Qvc2VydmljZU1ldGhvZC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICAgIHtcbiAgICAgICAgICBmbXQ6IHRoaXMsXG4gICAgICAgICAgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCxcbiAgICAgICAgfSxcbiAgICAgICAge1xuICAgICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgICAgfSxcbiAgICAgICk7XG4gICAgICByZXN1bHRzLnB1c2gob3V0cHV0KTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3A6IFByb3BzKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gIH1cblxuICBydXN0VHlwZUZvclByb3AoXG4gICAgcHJvcDogUHJvcHMsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVmZXJlbmNlID0gcmVuZGVyT3B0aW9ucy5yZWZlcmVuY2UgfHwgZmFsc2U7XG4gICAgbGV0IG9wdGlvbiA9IHRydWU7XG4gICAgaWYgKHJlbmRlck9wdGlvbnMub3B0aW9uID09PSBmYWxzZSkge1xuICAgICAgb3B0aW9uID0gZmFsc2U7XG4gICAgfVxuXG4gICAgbGV0IHR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgICBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbiB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gYCR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE51bWJlcikge1xuICAgICAgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcImludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcImkzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50MzJcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwidTMyXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcImludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcImk2NFwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1aW50NjRcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwidTY0XCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInUxMjhcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwidTEyOFwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEJvb2wgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAocmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICAgIGxldCBwYXRoTmFtZTogc3RyaW5nO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lICYmXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lID09IHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lXG4gICAgICAgICkge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IGBzaV8ke3Byb3BPd25lci5zZXJ2aWNlTmFtZX06OnByb3RvYnVmYDtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH1cbiAgICAgICAgdHlwZU5hbWUgPSBgJHtwYXRoTmFtZX06OiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHJlYWxQcm9wLCByZW5kZXJPcHRpb25zKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWFwKSB7XG4gICAgICB0eXBlTmFtZSA9IGBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoTWFwPFN0cmluZywgU3RyaW5nPmA7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgQ2Fubm90IGdlbmVyYXRlIHR5cGUgZm9yICR7cHJvcC5uYW1lfSBraW5kICR7cHJvcC5raW5kKCl9IC0gQnVnIWA7XG4gICAgfVxuICAgIGlmIChyZWZlcmVuY2UpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgaWYgKHR5cGVOYW1lID09IFwiU3RyaW5nXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcIiZzdHJcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGAmJHt0eXBlTmFtZX1gO1xuICAgICAgfVxuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICB0eXBlTmFtZSA9IGBWZWM8JHt0eXBlTmFtZX0+YDtcbiAgICB9IGVsc2Uge1xuICAgICAgaWYgKG9wdGlvbikge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgT3B0aW9uPCR7dHlwZU5hbWV9PmA7XG4gICAgICB9XG4gICAgfVxuICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgIHJldHVybiB0eXBlTmFtZTtcbiAgfVxuXG4gIGltcGxDcmVhdGVOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goYCR7c25ha2VDYXNlKHByb3AubmFtZSl9OiAke3RoaXMucnVzdFR5cGVGb3JQcm9wKHByb3ApfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVQYXNzTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKHNuYWtlQ2FzZShwcm9wLm5hbWUpKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZExpc3RSZXN1bHRUb1JlcGx5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgbGlzdE1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJsaXN0XCIpO1xuICAgIGlmIChsaXN0TWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGxpc3RNZXRob2QucmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgbGV0IGxpc3RSZXBseVZhbHVlID0gYFNvbWUob3V0cHV0LiR7ZmllbGROYW1lfSlgO1xuICAgICAgICBpZiAoZmllbGROYW1lID09IFwibmV4dF9wYWdlX3Rva2VuXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IFwiU29tZShvdXRwdXQucGFnZV90b2tlbilcIjtcbiAgICAgICAgfSBlbHNlIGlmIChmaWVsZE5hbWUgPT0gXCJpdGVtc1wiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBgb3V0cHV0LiR7ZmllbGROYW1lfWA7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfTogJHtsaXN0UmVwbHlWYWx1ZX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZENyZWF0ZURlc3RydWN0dXJlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKGBsZXQgJHtmaWVsZE5hbWV9ID0gaW5uZXIuJHtmaWVsZE5hbWV9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBuYXR1cmFsS2V5KCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0Lm5hdHVyYWxLZXkpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJuYW1lXCI7XG4gICAgfVxuICB9XG5cbiAgaW1wbENyZWF0ZVNldFByb3BlcnRpZXMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCB2YXJpYWJsZU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wUGFzc3dvcmQpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gU29tZShzaV9kYXRhOjpwYXNzd29yZDo6ZW5jcnlwdF9wYXNzd29yZCgke3ZhcmlhYmxlTmFtZX0pPyk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gJHt2YXJpYWJsZU5hbWV9O2ApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuYXR0cnMpIHtcbiAgICAgIGNvbnN0IHZhcmlhYmxlTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgY29uc3QgZGVmYXVsdFZhbHVlID0gcHJvcC5kZWZhdWx0VmFsdWUoKTtcbiAgICAgIGlmIChkZWZhdWx0VmFsdWUpIHtcbiAgICAgICAgaWYgKHByb3Aua2luZCgpID09IFwidGV4dFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9IFwiJHtkZWZhdWx0VmFsdWV9XCIudG9fc3RyaW5nKCk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwiZW51bVwiKSB7XG4gICAgICAgICAgY29uc3QgZW51bU5hbWUgPSBgJHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUsXG4gICAgICAgICAgKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC5zZXRfJHt2YXJpYWJsZU5hbWV9KGNyYXRlOjpwcm90b2J1Zjo6JHtlbnVtTmFtZX06OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgICAgZGVmYXVsdFZhbHVlIGFzIHN0cmluZyxcbiAgICAgICAgICAgICl9KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaW1wbENyZWF0ZUFkZFRvVGVuYW5jeSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiYmlsbGluZ0FjY291bnRcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblwiXG4gICAgKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uU2VydmljZVwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiY29tcG9uZW50T2JqZWN0XCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX3NlcnZpY2VfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX3NlcnZpY2VfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25TZXJ2aWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25fc2VydmljZV9pZCk7YCk7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwidXNlclwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImdyb3VwXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwib3JnYW5pemF0aW9uXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25JbnN0YW5jZVwiXG4gICAgKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ3b3Jrc3BhY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgb3JnYW5pemF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5vcmdhbml6YXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMub3JnYW5pemF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMob3JnYW5pemF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgb3JnYW5pemF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5vcmdhbml6YXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMub3JnYW5pemF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMob3JnYW5pemF0aW9uX2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgd29ya3NwYWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS53b3Jrc3BhY2VfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMud29ya3NwYWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMod29ya3NwYWNlX2lkKTtgKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVJc012Y2MoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QubXZjYyA9PSB0cnVlKSB7XG4gICAgICByZXR1cm4gXCJ0cnVlXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcImZhbHNlXCI7XG4gICAgfVxuICB9XG5cbiAgc3RvcmFibGVWYWxpZGF0ZUZ1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5hdHRycykge1xuICAgICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbiAgICAgICAgY29uc3QgcHJvcE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5sZW4oKSA9PSAwIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0uaXNfbm9uZSgpIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgIHRvcFByb3A6IFByb3BQcmVsdWRlLlByb3BPYmplY3QsXG4gICAgcHJlZml4OiBzdHJpbmcsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFsnXCJzaVN0b3JhYmxlLm5hdHVyYWxLZXlcIiddO1xuICAgIGZvciAobGV0IHByb3Agb2YgdG9wUHJvcC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4pIHtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBwcm9wLm5hbWUpKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBgJHtwcmVmaXh9LiR7cHJvcC5uYW1lfWApLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3ByZWZpeH0uJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiLCBcIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnJvb3RQcm9wLFxuICAgICAgXCJcIixcbiAgICApO1xuICAgIHJldHVybiBgdmVjIVske3Jlc3VsdHN9XVxcbmA7XG4gIH1cblxuICBzdG9yYWJsZVJlZmVyZW50aWFsRmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCBmZXRjaFByb3BzID0gW107XG4gICAgY29uc3QgcmVmZXJlbmNlVmVjID0gW107XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCkge1xuICAgICAgbGV0IHNpUHJvcGVydGllcyA9IHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5nZXRFbnRyeShcInNpUHJvcGVydGllc1wiKTtcbiAgICAgIGlmIChzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBzaVByb3BlcnRpZXMgPSBzaVByb3BlcnRpZXMubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAoIShzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSkge1xuICAgICAgICB0aHJvdyBcIkNhbm5vdCBnZXQgcHJvcGVydGllcyBvZiBhIG5vbiBvYmplY3QgaW4gcmVmIGNoZWNrXCI7XG4gICAgICB9XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2Ygc2lQcm9wZXJ0aWVzLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgaWYgKHByb3AucmVmZXJlbmNlKSB7XG4gICAgICAgICAgY29uc3QgaXRlbU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNNYW55KFwiJHtpdGVtTmFtZX1cIiwgJHtpdGVtTmFtZX0pYCxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIGZldGNoUHJvcHMucHVzaChgbGV0ICR7aXRlbU5hbWV9ID0gbWF0Y2ggJnNlbGYuc2lfcHJvcGVydGllcyB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICBTb21lKGNpcCkgPT4gY2lwXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuJHtpdGVtTmFtZX1cbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5hc19yZWYoKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLm1hcChTdHJpbmc6OmFzX3JlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC51bndyYXBfb3IoXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICBOb25lID0+IFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiLFxuICAgICAgICAgICAgICAgICAgICAgICAgIH07YCk7XG4gICAgICAgICAgICByZWZlcmVuY2VWZWMucHVzaChcbiAgICAgICAgICAgICAgYHNpX2RhdGE6OlJlZmVyZW5jZTo6SGFzT25lKFwiJHtpdGVtTmFtZX1cIiwgJHtpdGVtTmFtZX0pYCxcbiAgICAgICAgICAgICk7XG4gICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICB9XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBCYXNlT2JqZWN0KSB7XG4gICAgfVxuXG4gICAgaWYgKGZldGNoUHJvcHMubGVuZ3RoICYmIHJlZmVyZW5jZVZlYy5sZW5ndGgpIHtcbiAgICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICAgIHJlc3VsdHMucHVzaChmZXRjaFByb3BzLmpvaW4oXCJcXG5cIikpO1xuICAgICAgcmVzdWx0cy5wdXNoKGB2ZWMhWyR7cmVmZXJlbmNlVmVjLmpvaW4oXCIsXCIpfV1gKTtcbiAgICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlZlYzo6bmV3KClcIjtcbiAgICB9XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXJTZXJ2aWNlIHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0cyA9IHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSk7XG4gIH1cblxuICBzeXN0ZW1PYmplY3RzQXNGb3JtYXR0ZXJzKCk6IFJ1c3RGb3JtYXR0ZXJbXSB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0c1xuICAgICAgLnNvcnQoKGEsIGIpID0+IChhLnR5cGVOYW1lID4gYi50eXBlTmFtZSA/IDEgOiAtMSkpXG4gICAgICAubWFwKG8gPT4gbmV3IFJ1c3RGb3JtYXR0ZXIobykpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VTdHJ1Y3RCb2R5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wiZGI6IHNpX2RhdGE6OkRiLFwiXTtcbiAgICBpZiAodGhpcy5oYXNFbnRpdGllcygpKSB7XG4gICAgICByZXN1bHQucHVzaChcImFnZW50OiBzaV9jZWE6OkFnZW50Q2xpZW50LFwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VOZXdDb25zdHJ1Y3RvckFyZ3MoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5oYXNFbnRpdGllcygpKSB7XG4gICAgICByZXR1cm4gXCJkYjogc2lfZGF0YTo6RGIsIGFnZW50OiBzaV9jZWE6OkFnZW50Q2xpZW50XCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcImRiOiBzaV9kYXRhOjpEYlwiO1xuICAgIH1cbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Q29uc3RydWN0b3JSZXR1cm4oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXCJkYlwiXTtcbiAgICBpZiAodGhpcy5oYXNFbnRpdGllcygpKSB7XG4gICAgICByZXN1bHQucHVzaChcImFnZW50XCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VUcmFpdE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtzbmFrZUNhc2UoXG4gICAgICB0aGlzLnNlcnZpY2VOYW1lLFxuICAgICl9X3NlcnZlcjo6JHtwYXNjYWxDYXNlKHRoaXMuc2VydmljZU5hbWUpfWA7XG4gIH1cblxuICBpbXBsU2VydmVyTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHt0aGlzLmltcGxTZXJ2aWNlVHJhaXROYW1lKCl9U2VydmVyYDtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWlncmF0ZSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqIG9mIHRoaXMuc3lzdGVtT2JqZWN0cykge1xuICAgICAgaWYgKHRoaXMuaXNNaWdyYXRlYWJsZShzeXN0ZW1PYmopKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgIHN5c3RlbU9iai50eXBlTmFtZSxcbiAgICAgICAgICApfTo6bWlncmF0ZSgmc2VsZi5kYikuYXdhaXQ/O2AsXG4gICAgICAgICk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGhhc0VudGl0aWVzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHMuc29tZShvYmogPT4gb2JqIGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KTtcbiAgfVxuXG4gIGlzTWlncmF0ZWFibGUocHJvcDogT2JqZWN0VHlwZXMpOiBib29sZWFuIHtcbiAgICByZXR1cm4gcHJvcCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCAmJiBwcm9wLm1pZ3JhdGVhYmxlO1xuICB9XG5cbiAgaGFzTWlncmF0YWJsZXMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0cy5zb21lKG9iaiA9PiB0aGlzLmlzTWlncmF0ZWFibGUob2JqKSk7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXJBZ2VudCB7XG4gIGFnZW50TmFtZTogc3RyaW5nO1xuICBlbnRpdHk6IEVudGl0eU9iamVjdDtcbiAgZW50aXR5Rm9ybWF0dGVyOiBSdXN0Rm9ybWF0dGVyO1xuICBpbnRlZ3JhdGlvbk5hbWU6IHN0cmluZztcbiAgaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzeXN0ZW1PYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcsIGFnZW50OiBBZ2VudEludGVncmF0aW9uU2VydmljZSkge1xuICAgIHRoaXMuYWdlbnROYW1lID0gYWdlbnQuYWdlbnROYW1lO1xuICAgIHRoaXMuZW50aXR5ID0gYWdlbnQuZW50aXR5O1xuICAgIHRoaXMuZW50aXR5Rm9ybWF0dGVyID0gbmV3IFJ1c3RGb3JtYXR0ZXIodGhpcy5lbnRpdHkpO1xuICAgIHRoaXMuaW50ZWdyYXRpb25OYW1lID0gYWdlbnQuaW50ZWdyYXRpb25OYW1lO1xuICAgIHRoaXMuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSA9IGFnZW50LmludGVncmF0aW9uU2VydmljZU5hbWU7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0cyA9IHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSk7XG4gIH1cblxuICBzeXN0ZW1PYmplY3RzQXNGb3JtYXR0ZXJzKCk6IFJ1c3RGb3JtYXR0ZXJbXSB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0c1xuICAgICAgLnNvcnQoKGEsIGIpID0+IChhLnR5cGVOYW1lID4gYi50eXBlTmFtZSA/IDEgOiAtMSkpXG4gICAgICAubWFwKG8gPT4gbmV3IFJ1c3RGb3JtYXR0ZXIobykpO1xuICB9XG5cbiAgYWN0aW9uUHJvcHMoKTogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdIHtcbiAgICByZXR1cm4gdGhpcy5lbnRpdHkubWV0aG9kcy5hdHRycy5maWx0ZXIoXG4gICAgICBtID0+IG0gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdO1xuICB9XG5cbiAgZW50aXR5QWN0aW9uTWV0aG9kTmFtZXMoKTogc3RyaW5nW10ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCJjcmVhdGVcIl07XG5cbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5hY3Rpb25Qcm9wcygpKSB7XG4gICAgICBpZiAodGhpcy5lbnRpdHlGb3JtYXR0ZXIuaXNFbnRpdHlFZGl0TWV0aG9kKHByb3ApKSB7XG4gICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLmVudGl0eUZvcm1hdHRlci5lbnRpdHlFZGl0TWV0aG9kTmFtZShwcm9wKSk7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHRzLnB1c2gocHJvcC5uYW1lKTtcbiAgICAgIH1cbiAgICB9XG5cbiAgICByZXR1cm4gcmVzdWx0cztcbiAgfVxuXG4gIGRpc3BhdGNoZXJCYXNlVHlwZU5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7cGFzY2FsQ2FzZSh0aGlzLmludGVncmF0aW9uTmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgdGhpcy5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lLFxuICAgICl9JHtwYXNjYWxDYXNlKHRoaXMuZW50aXR5LmJhc2VUeXBlTmFtZSl9YDtcbiAgfVxuXG4gIGRpc3BhdGNoZXJUeXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHt0aGlzLmRpc3BhdGNoZXJCYXNlVHlwZU5hbWUoKX1EaXNwYXRjaGVyYDtcbiAgfVxuXG4gIGRpc3BhdGNoRnVuY3Rpb25UcmFpdE5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5kaXNwYXRjaGVyQmFzZVR5cGVOYW1lKCl9RGlzcGF0Y2hGdW5jdGlvbnNgO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBDb2RlZ2VuUnVzdCB7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZykge1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgfVxuXG4gIGhhc01vZGVscygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gcmVnaXN0cnlcbiAgICAgIC5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUodGhpcy5zZXJ2aWNlTmFtZSlcbiAgICAgIC5zb21lKG8gPT4gby5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIpO1xuICB9XG5cbiAgaGFzU2VydmljZU1ldGhvZHMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHJlZ2lzdHJ5XG4gICAgICAgIC5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUodGhpcy5zZXJ2aWNlTmFtZSlcbiAgICAgICAgLmZsYXRNYXAobyA9PiBvLm1ldGhvZHMuYXR0cnMpLmxlbmd0aCA+IDBcbiAgICApO1xuICB9XG5cbiAgaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMoKTogYm9vbGVhbiB7XG4gICAgY29uc3QgaW50ZWdyYXRpb25TZXJ2aWNlcyA9IG5ldyBTZXQoXG4gICAgICB0aGlzLmVudGl0aWVzKCkuZmxhdE1hcChlbnRpdHkgPT5cbiAgICAgICAgdGhpcy5lbnRpdHlpbnRlZ3JhdGlvblNlcnZpY2VzRm9yKGVudGl0eSksXG4gICAgICApLFxuICAgICk7XG4gICAgcmV0dXJuIGludGVncmF0aW9uU2VydmljZXMuc2l6ZSA+IDA7XG4gIH1cblxuICBlbnRpdGllcygpOiBFbnRpdHlPYmplY3RbXSB7XG4gICAgcmV0dXJuIHJlZ2lzdHJ5XG4gICAgICAuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHRoaXMuc2VydmljZU5hbWUpXG4gICAgICAuZmlsdGVyKG8gPT4gbyBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkgYXMgRW50aXR5T2JqZWN0W107XG4gIH1cblxuICBlbnRpdHlBY3Rpb25zKGVudGl0eTogRW50aXR5T2JqZWN0KTogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdIHtcbiAgICByZXR1cm4gZW50aXR5Lm1ldGhvZHMuYXR0cnMuZmlsdGVyKFxuICAgICAgbSA9PiBtIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXTtcbiAgfVxuXG4gIGVudGl0eWludGVncmF0aW9uU2VydmljZXNGb3IoZW50aXR5OiBFbnRpdHlPYmplY3QpOiBJbnRlZ3JhdGlvblNlcnZpY2VbXSB7XG4gICAgY29uc3QgcmVzdWx0OiBTZXQ8SW50ZWdyYXRpb25TZXJ2aWNlPiA9IG5ldyBTZXQoKTtcbiAgICBmb3IgKGNvbnN0IGludGVncmF0aW9uU2VydmljZSBvZiBlbnRpdHkuaW50ZWdyYXRpb25TZXJ2aWNlcykge1xuICAgICAgcmVzdWx0LmFkZChpbnRlZ3JhdGlvblNlcnZpY2UpO1xuICAgIH1cbiAgICBmb3IgKGNvbnN0IGFjdGlvbiBvZiB0aGlzLmVudGl0eUFjdGlvbnMoZW50aXR5KSkge1xuICAgICAgZm9yIChjb25zdCBpbnRlZ3JhdGlvblNlcnZpY2Ugb2YgYWN0aW9uLmludGVncmF0aW9uU2VydmljZXMpIHtcbiAgICAgICAgcmVzdWx0LmFkZChpbnRlZ3JhdGlvblNlcnZpY2UpO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gQXJyYXkuZnJvbShyZXN1bHQpO1xuICB9XG5cbiAgZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcygpOiBBZ2VudEludGVncmF0aW9uU2VydmljZVtdIHtcbiAgICByZXR1cm4gdGhpcy5lbnRpdGllcygpLmZsYXRNYXAoZW50aXR5ID0+XG4gICAgICB0aGlzLmVudGl0eWludGVncmF0aW9uU2VydmljZXNGb3IoZW50aXR5KS5tYXAoaW50ZWdyYXRpb25TZXJ2aWNlID0+ICh7XG4gICAgICAgIGludGVncmF0aW9uTmFtZTogaW50ZWdyYXRpb25TZXJ2aWNlLmludGVncmF0aW9uTmFtZSxcbiAgICAgICAgaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTogaW50ZWdyYXRpb25TZXJ2aWNlLmludGVncmF0aW9uU2VydmljZU5hbWUsXG4gICAgICAgIGVudGl0eTogZW50aXR5LFxuICAgICAgICBhZ2VudE5hbWU6IGAke3NuYWtlQ2FzZShcbiAgICAgICAgICBpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25OYW1lLFxuICAgICAgICApfV8ke3NuYWtlQ2FzZShpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSl9XyR7c25ha2VDYXNlKFxuICAgICAgICAgIGVudGl0eS5iYXNlVHlwZU5hbWUsXG4gICAgICAgICl9YCxcbiAgICAgIH0pKSxcbiAgICApO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbk1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXCIsIFwiXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0eUludGVncmF0aW9uU2VydmNpY2VzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChcInB1YiBtb2QgYWdlbnQ7XCIpO1xuICAgIH1cbiAgICBpZiAodGhpcy5oYXNNb2RlbHMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKFwicHViIG1vZCBtb2RlbDtcIik7XG4gICAgfVxuICAgIGlmICh0aGlzLmhhc1NlcnZpY2VNZXRob2RzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChcInB1YiBtb2Qgc2VydmljZTtcIik7XG4gICAgfVxuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vbW9kZWwvbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgZm9yIChjb25zdCBzeXN0ZW1PYmplY3Qgb2YgcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApKSB7XG4gICAgICBpZiAoc3lzdGVtT2JqZWN0LmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIikge1xuICAgICAgICByZXN1bHRzLnB1c2goYHB1YiBtb2QgJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX07YCk7XG4gICAgICB9XG4gICAgfVxuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL21vZGVsL21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5TZXJ2aWNlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2UucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyU2VydmljZSh0aGlzLnNlcnZpY2VOYW1lKSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShgZ2VuL3NlcnZpY2UucnNgLCBvdXRwdXQpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2RlbChzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvbW9kZWwucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyKHN5c3RlbU9iamVjdCksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXG4gICAgICBgZ2VuL21vZGVsLyR7c25ha2VDYXNlKHN5c3RlbU9iamVjdC50eXBlTmFtZSl9LnJzYCxcbiAgICAgIG91dHB1dCxcbiAgICApO1xuICB9XG5cbiAgLy8gR2VuZXJhdGUgdGhlICdnZW4vYWdlbnQvbW9kLnJzJ1xuICBhc3luYyBnZW5lcmF0ZUdlbkFnZW50TW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgZm9yIChjb25zdCBhZ2VudCBvZiB0aGlzLmVudGl0eUludGVncmF0aW9uU2VydmljZXMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKGBwdWIgbW9kICR7YWdlbnQuYWdlbnROYW1lfTtgKTtcbiAgICB9XG4gICAgcmVzdWx0cy5wdXNoKFwiXCIpO1xuICAgIGZvciAoY29uc3QgYWdlbnQgb2YgdGhpcy5lbnRpdHlJbnRlZ3JhdGlvblNlcnZpY2VzKCkpIHtcbiAgICAgIGNvbnN0IGZtdCA9IG5ldyBSdXN0Rm9ybWF0dGVyQWdlbnQodGhpcy5zZXJ2aWNlTmFtZSwgYWdlbnQpO1xuICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICBgcHViIHVzZSAke1xuICAgICAgICAgIGFnZW50LmFnZW50TmFtZVxuICAgICAgICB9Ojp7JHtmbXQuZGlzcGF0Y2hGdW5jdGlvblRyYWl0TmFtZSgpfSwgJHtmbXQuZGlzcGF0Y2hlclR5cGVOYW1lKCl9fTtgLFxuICAgICAgKTtcbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vYWdlbnQvbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlbkFnZW50KGFnZW50OiBBZ2VudEludGVncmF0aW9uU2VydmljZSk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2FnZW50LnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlckFnZW50KHRoaXMuc2VydmljZU5hbWUsIGFnZW50KSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShgZ2VuL2FnZW50LyR7c25ha2VDYXNlKGFnZW50LmFnZW50TmFtZSl9LnJzYCwgb3V0cHV0KTtcbiAgfVxuXG4gIC8vYXN5bmMgbWFrZVBhdGgocGF0aFBhcnQ6IHN0cmluZyk6IFByb21pc2U8c3RyaW5nPiB7XG4gIC8vICBjb25zdCBwYXRoTmFtZSA9IHBhdGguam9pbihcIi4uXCIsIGBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCwgXCJzcmNcIiwgcGF0aFBhcnQpO1xuICAvLyAgY29uc3QgYWJzb2x1dGVQYXRoTmFtZSA9IHBhdGgucmVzb2x2ZShwYXRoTmFtZSk7XG4gIC8vICBhd2FpdCBmcy5wcm9taXNlcy5ta2RpcihwYXRoLnJlc29sdmUocGF0aE5hbWUpLCB7IHJlY3Vyc2l2ZTogdHJ1ZSB9KTtcbiAgLy8gIHJldHVybiBhYnNvbHV0ZVBhdGhOYW1lO1xuICAvL31cblxuICBhc3luYyBmb3JtYXRDb2RlKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGF3YWl0IGV4ZWNDbWQoYGNhcmdvIGZtdCAtcCBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCk7XG4gIH1cblxuICBhc3luYyB3cml0ZUNvZGUoZmlsZW5hbWU6IHN0cmluZywgY29kZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgZnVsbFBhdGhOYW1lID0gcGF0aC5qb2luKFxuICAgICAgXCIuLlwiLFxuICAgICAgYHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gLFxuICAgICAgXCJzcmNcIixcbiAgICAgIGZpbGVuYW1lLFxuICAgICk7XG4gICAgYXdhaXQgY29kZUZzLndyaXRlQ29kZShmdWxsUGF0aE5hbWUsIGNvZGUpO1xuICB9XG59XG4iXX0=