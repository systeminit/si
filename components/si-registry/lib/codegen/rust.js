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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInJlc3VsdHMiLCJraW5kIiwiZW50aXR5IiwicmVnaXN0cnkiLCJnZXQiLCJiYXNlVHlwZU5hbWUiLCJmbXQiLCJhY3Rpb25Qcm9wcyIsInByb3AiLCJpc0VudGl0eUVkaXRNZXRob2QiLCJwdXNoIiwiZW50aXR5RWRpdE1ldGhvZE5hbWUiLCJuYW1lIiwibWV0aG9kcyIsImdldEVudHJ5IiwicHJvcEFjdGlvbiIsImVudGl0eUVkaXRQcm9wZXJ0eSIsInJlbGF0aW9uc2hpcHMiLCJhbGwiLCJzb21lIiwicmVsIiwiUHJvcFByZWx1ZGUiLCJFaXRoZXIiLCJVcGRhdGVzIiwiaXNFbnRpdHlPYmplY3QiLCJlbnRpdHlFZGl0TWV0aG9kcyIsImhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uIiwiQ29tcG9uZW50T2JqZWN0IiwicHJvcE1ldGhvZCIsIlByb3BBY3Rpb24iLCJpc0VudGl0eUFjdGlvbk1ldGhvZCIsImVuZHNXaXRoIiwiRW50aXR5RXZlbnRPYmplY3QiLCJFbnRpdHlPYmplY3QiLCJTeXN0ZW1PYmplY3QiLCJtaWdyYXRlYWJsZSIsImF0dHJzIiwiZmlsdGVyIiwibSIsInJ1c3RGaWVsZE5hbWVGb3JQcm9wIiwicmVwbGFjZSIsInAiLCJwcm9wZXJ0eSIsInJlcXVlc3QiLCJwcm9wZXJ0aWVzIiwiUHJvcExpbmsiLCJsb29rdXBNeXNlbGYiLCJydXN0VHlwZUZvclByb3AiLCJvcHRpb24iLCJyIiwibWFwIiwidXBkYXRlIiwiZnJvbSIsInRvIiwicGFydG5lclByb3AiLCJmbGF0TWFwIiwibWV0aG9kIiwiZW50aXR5RWRpdFByb3BlcnR5VXBkYXRlcyIsIkFycmF5IiwiU2V0Iiwic29ydCIsImEiLCJiIiwiTWFwIiwiZmllbGRzIiwicHJvcEVpdGhlcnMiLCJsZW5ndGgiLCJlaXRoZXJzIiwiYWRkIiwiZWl0aGVyc0FycmF5Iiwic2V0IiwiZSIsImpvaW4iLCJlbnRyaWVzIiwidmFsdWVzIiwicHJvcGVydHlVcGRhdGUiLCJzZXJ2aWNlTmFtZSIsInR5cGVOYW1lIiwiUHJvcENvZGUiLCJsYW5ndWFnZSIsIlByb3BPYmplY3QiLCJyZW5kZXJPcHRpb25zIiwibGlzdCIsInJlcGx5IiwicmVmZXJlbmNlIiwiZWpzIiwicmVuZGVyIiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicHJvcE1ldGhvZHMiLCJvdXRwdXQiLCJQcm9wTWV0aG9kIiwicGFyZW50TmFtZSIsIlByb3BOdW1iZXIiLCJudW1iZXJLaW5kIiwiUHJvcEJvb2wiLCJyZWFsUHJvcCIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcE1hcCIsIlByb3BUZXh0IiwiUHJvcFNlbGVjdCIsInJlcGVhdGVkIiwicmVzdWx0IiwiY3JlYXRlTWV0aG9kIiwibGlzdE1ldGhvZCIsImZpZWxkTmFtZSIsImxpc3RSZXBseVZhbHVlIiwibmF0dXJhbEtleSIsInZhcmlhYmxlTmFtZSIsIlByb3BQYXNzd29yZCIsImRlZmF1bHRWYWx1ZSIsImVudW1OYW1lIiwibXZjYyIsInJlcXVpcmVkIiwicHJvcE5hbWUiLCJ0b3BQcm9wIiwicHJlZml4IiwiaGlkZGVuIiwic3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wIiwicm9vdFByb3AiLCJmZXRjaFByb3BzIiwicmVmZXJlbmNlVmVjIiwic2lQcm9wZXJ0aWVzIiwiaXRlbU5hbWUiLCJCYXNlT2JqZWN0IiwiUnVzdEZvcm1hdHRlclNlcnZpY2UiLCJzeXN0ZW1PYmplY3RzIiwiZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lIiwibyIsImhhc0VudGl0aWVzIiwiaW1wbFNlcnZpY2VUcmFpdE5hbWUiLCJzeXN0ZW1PYmoiLCJpc01pZ3JhdGVhYmxlIiwib2JqIiwiUnVzdEZvcm1hdHRlckFnZW50IiwiYWdlbnQiLCJhZ2VudE5hbWUiLCJlbnRpdHlGb3JtYXR0ZXIiLCJpbnRlZ3JhdGlvbk5hbWUiLCJpbnRlZ3JhdGlvblNlcnZpY2VOYW1lIiwiZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSIsIkNvZGVnZW5SdXN0IiwiaW50ZWdyYXRpb25TZXJ2aWNlcyIsImVudGl0aWVzIiwiZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvciIsInNpemUiLCJpbnRlZ3JhdGlvblNlcnZpY2UiLCJlbnRpdHlBY3Rpb25zIiwiYWN0aW9uIiwiaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMiLCJoYXNNb2RlbHMiLCJoYXNTZXJ2aWNlTWV0aG9kcyIsIndyaXRlQ29kZSIsImVudGl0eUludGVncmF0aW9uU2VydmljZXMiLCJkaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lIiwiZGlzcGF0Y2hlclR5cGVOYW1lIiwiY29kZSIsImZ1bGxQYXRoTmFtZSIsInBhdGgiLCJjb2RlRnMiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFBOztBQVFBOztBQUNBOztBQUdBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOzs7Ozs7OztBQUVBLElBQU1BLE9BQU8sR0FBR0MsaUJBQUtDLFNBQUwsQ0FBZUMsMEJBQWFDLElBQTVCLENBQWhCOztJQXVCYUMsYTtBQUdYLHlCQUFZQyxZQUFaLEVBQXlEO0FBQUE7QUFBQTtBQUN2RCxTQUFLQSxZQUFMLEdBQW9CQSxZQUFwQjtBQUNEOzs7OzhDQUVtQztBQUNsQyxVQUFNQyxPQUFPLEdBQUcsQ0FBQyxRQUFELENBQWhCOztBQUVBLFVBQUksS0FBS0QsWUFBTCxDQUFrQkUsSUFBbEIsTUFBNEIsbUJBQWhDLEVBQXFEO0FBQ25EO0FBQ0EsWUFBTUMsTUFBTSxHQUFHQyxtQkFBU0MsR0FBVCxXQUFnQixLQUFLTCxZQUFMLENBQWtCTSxZQUFsQyxZQUFmOztBQUNBLFlBQU1DLEdBQUcsR0FBRyxJ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RCxrQkFBQUEsUUFBUSxFQUFFO0FBRFosaUJBTGEsQzs7dUJBU1QsS0FBS3NFLFNBQUwscUJBQTRCLDJCQUFVakIsS0FBSyxDQUFDQyxTQUFoQixDQUE1QixVQUE2RGhELE1BQTdELEM7Ozs7Ozs7Ozs7Ozs7OztRQUdSO0FBQ0E7QUFDQTtBQUNBO0FBQ0E7QUFDQTs7Ozs7Ozs7Ozs7dUJBR1FwRyxPQUFPLDJCQUFvQixLQUFLbUYsV0FBekIsRTs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozt1SEFHQ1csUSxFQUFrQjBFLEk7Ozs7OztBQUMxQkMsZ0JBQUFBLFksR0FBZUMsaUJBQUszRixJQUFMLENBQ25CLElBRG1CLGVBRWIsS0FBS0ksV0FGUSxHQUduQixLQUhtQixFQUluQlcsUUFKbUIsQzs7dUJBTWY2RSxNQUFNLENBQUNQLFNBQVAsQ0FBaUJLLFlBQWpCLEVBQStCRCxJQUEvQixDIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHtcbiAgT2JqZWN0VHlwZXMsXG4gIEJhc2VPYmplY3QsXG4gIFN5c3RlbU9iamVjdCxcbiAgQ29tcG9uZW50T2JqZWN0LFxuICBFbnRpdHlPYmplY3QsXG4gIEVudGl0eUV2ZW50T2JqZWN0LFxufSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgKiBhcyBQcm9wUHJlbHVkZSBmcm9tIFwiLi4vY29tcG9uZW50cy9wcmVsdWRlXCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuaW1wb3J0IHsgUHJvcHMsIEludGVncmF0aW9uU2VydmljZSB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuXG5pbXBvcnQgeyBzbmFrZUNhc2UsIHBhc2NhbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBlanMgZnJvbSBcImVqc1wiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG5pbXBvcnQgKiBhcyBjb2RlRnMgZnJvbSBcIi4vZnNcIjtcblxuY29uc3QgZXhlY0NtZCA9IHV0aWwucHJvbWlzaWZ5KGNoaWxkUHJvY2Vzcy5leGVjKTtcblxuaW50ZXJmYWNlIFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyB7XG4gIHJlZmVyZW5jZT86IGJvb2xlYW47XG4gIG9wdGlvbj86IGJvb2xlYW47XG59XG5cbmludGVyZmFjZSBBZ2VudEludGVncmF0aW9uU2VydmljZSB7XG4gIGFnZW50TmFtZTogc3RyaW5nO1xuICBlbnRpdHk6IEVudGl0eU9iamVjdDtcbiAgaW50ZWdyYXRpb25OYW1lOiBzdHJpbmc7XG4gIGludGVncmF0aW9uU2VydmljZU5hbWU6IHN0cmluZztcbn1cblxuaW50ZXJmYWNlIFByb3BlcnR5VXBkYXRlIHtcbiAgZnJvbTogUHJvcFByZWx1ZGUuUHJvcHM7XG4gIHRvOiBQcm9wUHJlbHVkZS5Qcm9wcztcbn1cblxuaW50ZXJmYWNlIFByb3BlcnR5RWl0aGVyU2V0IHtcbiAgZW50cmllczogUHJvcFByZWx1ZGUuUHJvcHNbXTtcbn1cblxuZXhwb3J0IGNsYXNzIFJ1c3RGb3JtYXR0ZXIge1xuICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogUnVzdEZvcm1hdHRlcltcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgZW50aXR5QWN0aW9uTWV0aG9kTmFtZXMoKTogc3RyaW5nW10ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCJjcmVhdGVcIl07XG5cbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiZW50aXR5RXZlbnRPYmplY3RcIikge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgY29uc3QgZW50aXR5ID0gcmVnaXN0cnkuZ2V0KGAke3RoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZX1FbnRpdHlgKTtcbiAgICAgIGNvbnN0IGZtdCA9IG5ldyBSdXN0Rm9ybWF0dGVyKGVudGl0eSk7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgZm10LmFjdGlvblByb3BzKCkpIHtcbiAgICAgICAgaWYgKGZtdC5pc0VudGl0eUVkaXRNZXRob2QocHJvcCkpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goZm10LmVudGl0eUVkaXRNZXRob2ROYW1lKHByb3ApKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2gocHJvcC5uYW1lKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH0gZWxzZSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5hY3Rpb25Qcm9wcygpKSB7XG4gICAgICAgIGlmICh0aGlzLmlzRW50aXR5RWRpdE1ldGhvZChwcm9wKSkge1xuICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLmVudGl0eUVkaXRNZXRob2ROYW1lKHByb3ApKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2gocHJvcC5uYW1lKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cblxuICAgIHJldHVybiByZXN1bHRzO1xuICB9XG5cbiAgaGFzQ3JlYXRlTWV0aG9kKCk6IGJvb2xlYW4ge1xuICAgIHRyeSB7XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgICAgcmV0dXJuIHRydWU7XG4gICAgfSBjYXRjaCB7XG4gICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG5cbiAgaGFzRWRpdEVpdGhlcnNGb3JBY3Rpb24ocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKVxuICAgICAgLnJlbGF0aW9uc2hpcHMuYWxsKClcbiAgICAgIC5zb21lKHJlbCA9PiByZWwgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5FaXRoZXIpO1xuICB9XG5cbiAgaGFzRWRpdFVwZGF0ZXNGb3JBY3Rpb24ocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKVxuICAgICAgLnJlbGF0aW9uc2hpcHMuYWxsKClcbiAgICAgIC5zb21lKHJlbCA9PiByZWwgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5VcGRhdGVzKTtcbiAgfVxuXG4gIGhhc0VkaXRVcGRhdGVzQW5kRWl0aGVycygpOiBib29sZWFuIHtcbiAgICBpZiAodGhpcy5pc0VudGl0eU9iamVjdCgpKSB7XG4gICAgICByZXR1cm4gdGhpcy5lbnRpdHlFZGl0TWV0aG9kcygpLnNvbWUoXG4gICAgICAgIHByb3BBY3Rpb24gPT5cbiAgICAgICAgICB0aGlzLmhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uKHByb3BBY3Rpb24pICYmXG4gICAgICAgICAgdGhpcy5oYXNFZGl0VXBkYXRlc0ZvckFjdGlvbihwcm9wQWN0aW9uKSxcbiAgICAgICk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IHJhbiAnaGFzRWRpdFVwZGF0ZXNBbmRFaXRoZXJzKCknIG9uIGEgbm9uLWVudGl0eSBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgaXNDb21wb25lbnRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0O1xuICB9XG5cbiAgaXNFbnRpdHlBY3Rpb25NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLmlzRW50aXR5T2JqZWN0KCkgJiYgcHJvcE1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb25cbiAgICApO1xuICB9XG5cbiAgaXNFbnRpdHlFZGl0TWV0aG9kKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgdGhpcy5pc0VudGl0eUFjdGlvbk1ldGhvZChwcm9wTWV0aG9kKSAmJiBwcm9wTWV0aG9kLm5hbWUuZW5kc1dpdGgoXCJFZGl0XCIpXG4gICAgKTtcbiAgfVxuXG4gIGlzRW50aXR5RXZlbnRPYmplY3QoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3Q7XG4gIH1cblxuICBpc0VudGl0eU9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3Q7XG4gIH1cblxuICBpc01pZ3JhdGVhYmxlKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCAmJiB0aGlzLnN5c3RlbU9iamVjdC5taWdyYXRlYWJsZVxuICAgICk7XG4gIH1cblxuICBpc1N0b3JhYmxlKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdDtcbiAgfVxuXG4gIGFjdGlvblByb3BzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMuZmlsdGVyKFxuICAgICAgbSA9PiBtIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXTtcbiAgfVxuXG4gIGNvbXBvbmVudE5hbWUoKTogc3RyaW5nIHtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3RcbiAgICApIHtcbiAgICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LmJhc2VUeXBlTmFtZSxcbiAgICAgICl9Q29tcG9uZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGNvbXBvbmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgY29tcG9uZW50Q29uc3RyYWludHNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUNvbXBvbmVudENvbnN0cmFpbnRzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGEgY29tcG9uZW50IGNvbnN0cmFpbnRzIG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlFZGl0TWV0aG9kTmFtZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICAgIHJldHVybiBgZWRpdF8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcE1ldGhvZCkucmVwbGFjZShcbiAgICAgICAgXCJfZWRpdFwiLFxuICAgICAgICBcIlwiLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZWRpdCBtZXRob2QgbmFtZSBvbiBhIG5vbi1lbnRpdHkgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eUVkaXRNZXRob2RzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuYWN0aW9uUHJvcHMoKS5maWx0ZXIocCA9PiB0aGlzLmlzRW50aXR5RWRpdE1ldGhvZChwKSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IFByb3BzIHtcbiAgICBsZXQgcHJvcGVydHkgPSBwcm9wQWN0aW9uLnJlcXVlc3QucHJvcGVydGllcy5nZXRFbnRyeShcInByb3BlcnR5XCIpO1xuICAgIGlmIChwcm9wZXJ0eSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBwcm9wZXJ0eSA9IHByb3BlcnR5Lmxvb2t1cE15c2VsZigpO1xuICAgIH1cbiAgICByZXR1cm4gcHJvcGVydHk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlGaWVsZChwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcCh0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlUeXBlKHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcCh0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKSwge1xuICAgICAgb3B0aW9uOiBmYWxzZSxcbiAgICB9KTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZXMoXG4gICAgcHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogUHJvcGVydHlVcGRhdGVbXSB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLmZpbHRlcihyID0+IHIgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5VcGRhdGVzKVxuICAgICAgLm1hcCh1cGRhdGUgPT4gKHtcbiAgICAgICAgZnJvbTogdGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbiksXG4gICAgICAgIHRvOiB1cGRhdGUucGFydG5lclByb3AoKSxcbiAgICAgIH0pKTtcbiAgfVxuXG4gIGFsbEVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZXMoKTogUHJvcGVydHlVcGRhdGVbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IHRoaXMuZW50aXR5RWRpdE1ldGhvZHMoKS5mbGF0TWFwKG1ldGhvZCA9PlxuICAgICAgdGhpcy5lbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzKG1ldGhvZCksXG4gICAgKTtcblxuICAgIHJldHVybiBBcnJheS5mcm9tKG5ldyBTZXQocmVzdWx0cykpLnNvcnQoKGEsIGIpID0+XG4gICAgICBgJHthLmZyb20ubmFtZX0sJHthLnRvLm5hbWV9YCA+IGAke2IuZnJvbS5uYW1lfSwke2IudG8ubmFtZX1gID8gMSA6IC0xLFxuICAgICk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlFaXRoZXJzKCk6IFByb3BlcnR5RWl0aGVyU2V0W10ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBuZXcgTWFwKCk7XG4gICAgY29uc3QgcHJvcGVydGllcyA9ICh0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuZ2V0RW50cnkoXG4gICAgICBcInByb3BlcnRpZXNcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BPYmplY3QpLnByb3BlcnRpZXMuYXR0cnM7XG5cbiAgICBmb3IgKGNvbnN0IHByb3BlcnR5IG9mIHByb3BlcnRpZXMpIHtcbiAgICAgIGNvbnN0IHByb3BFaXRoZXJzID0gcHJvcGVydHkucmVsYXRpb25zaGlwc1xuICAgICAgICAuYWxsKClcbiAgICAgICAgLmZpbHRlcihyZWwgPT4gcmVsIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuRWl0aGVyKTtcblxuICAgICAgaWYgKHByb3BFaXRoZXJzLmxlbmd0aCA+IDApIHtcbiAgICAgICAgY29uc3QgZWl0aGVycyA9IG5ldyBTZXQ8UHJvcFByZWx1ZGUuUHJvcHM+KCk7XG4gICAgICAgIGVpdGhlcnMuYWRkKHByb3BlcnR5KTtcbiAgICAgICAgZm9yIChjb25zdCBwcm9wZXJ0eSBvZiBwcm9wRWl0aGVycykge1xuICAgICAgICAgIGVpdGhlcnMuYWRkKHByb3BlcnR5LnBhcnRuZXJQcm9wKCkpO1xuICAgICAgICB9XG5cbiAgICAgICAgY29uc3QgZWl0aGVyc0FycmF5ID0gQXJyYXkuZnJvbShlaXRoZXJzKS5zb3J0KChhLCBiKSA9PlxuICAgICAgICAgIGEubmFtZSA+IGIubmFtZSA/IDEgOiAtMSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0cy5zZXQoZWl0aGVyc0FycmF5Lm1hcChlID0+IGUubmFtZSkuam9pbihcIixcIiksIHtcbiAgICAgICAgICBlbnRyaWVzOiBlaXRoZXJzQXJyYXksXG4gICAgICAgIH0pO1xuICAgICAgfVxuICAgIH1cblxuICAgIHJldHVybiBBcnJheS5mcm9tKHJlc3VsdHMudmFsdWVzKCkpLnNvcnQoKTtcbiAgfVxuXG4gIGVudGl0eUVkaXRQcm9wZXJ0eVVwZGF0ZU1ldGhvZE5hbWUocHJvcGVydHlVcGRhdGU6IFByb3BlcnR5VXBkYXRlKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYHVwZGF0ZV8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AoXG4gICAgICBwcm9wZXJ0eVVwZGF0ZS50byxcbiAgICApfV9mcm9tXyR7dGhpcy5ydXN0RmllbGROYW1lRm9yUHJvcChwcm9wZXJ0eVVwZGF0ZS5mcm9tKX1gO1xuICB9XG5cbiAgZW50aXR5RXZlbnROYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eUV2ZW50YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eUV2ZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHkgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVudGl0eVByb3BlcnRpZXNOYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUVudGl0eVByb3BlcnRpZXNgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5UHJvcGVydGllcyBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZXJyb3JUeXBlKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6ZXJyb3I6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSl9RXJyb3JgO1xuICB9XG5cbiAgbW9kZWxOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6bW9kZWw6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIG1vZGVsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcE1ldGhvZCk7XG4gIH1cblxuICBzdHJ1Y3ROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9YDtcbiAgfVxuXG4gIHR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSk7XG4gIH1cblxuICBpbXBsVHJ5RnJvbUZvclByb3BlcnR5VXBkYXRlKHByb3BlcnR5VXBkYXRlOiBQcm9wZXJ0eVVwZGF0ZSk6IHN0cmluZyB7XG4gICAgY29uc3QgZnJvbSA9IHByb3BlcnR5VXBkYXRlLmZyb207XG4gICAgY29uc3QgdG8gPSBwcm9wZXJ0eVVwZGF0ZS50bztcblxuICAgIC8vIEV2ZXJ5IGZhbGx0aHJvdWdoL2RlZmF1bHQvZWxzZSBuZWVkcyBhIGB0aHJvd2AgY2xhdXNlIHRvIGxvdWRseSBwcm9jbGFpbVxuICAgIC8vIHRoYXQgYSBzcGVjaWZpYyBjb252ZXJzaW9uIGlzIG5vdCBzdXBwb3J0ZWQuIFRoaXMgYWxsb3dzIHVzIHRvIGFkZFxuICAgIC8vIGNvbnZlcnNpb25zIGFzIHdlIGdvIHdpdGhvdXQgcm9ndWUgYW5kIHVuZXhwbGFpbmVkIGVycm9ycy4gSW4gc2hvcnQsXG4gICAgLy8gdHJlYXQgdGhpcyBsaWtlIFJ1c3QgY29kZSB3aXRoIGZ1bGx5IHNhdGlzZmllZCBtYXRjaCBhcm1zLiBUaGFuayB5b3UsXG4gICAgLy8gbG92ZSwgdXMuXG4gICAgaWYgKGZyb20gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSkge1xuICAgICAgc3dpdGNoIChmcm9tLmxhbmd1YWdlKSB7XG4gICAgICAgIGNhc2UgXCJ5YW1sXCI6XG4gICAgICAgICAgaWYgKHRvIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICAgICAgcmV0dXJuIGBPayhzZXJkZV95YW1sOjpmcm9tX3N0cih2YWx1ZSk/KWA7XG4gICAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gbGFuZ3VhZ2UgJyR7XG4gICAgICAgICAgICAgIGZyb20ubGFuZ3VhZ2VcbiAgICAgICAgICAgIH0nIHRvIHR5cGUgJyR7dG8ua2luZCgpfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgICAgICAgfVxuICAgICAgICBkZWZhdWx0OlxuICAgICAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gbGFuZ3VhZ2UgJyR7ZnJvbS5sYW5ndWFnZX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoZnJvbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgIGlmICh0byBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlKSB7XG4gICAgICAgIHN3aXRjaCAodG8ubGFuZ3VhZ2UpIHtcbiAgICAgICAgICBjYXNlIFwieWFtbFwiOlxuICAgICAgICAgICAgcmV0dXJuIGBPayhzZXJkZV95YW1sOjp0b19zdHJpbmcodmFsdWUpPylgO1xuICAgICAgICAgIGRlZmF1bHQ6XG4gICAgICAgICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIFByb3BPYmplY3QgdG8gbGFuZ3VhZ2UgJyR7dG8ubGFuZ3VhZ2V9JyBpcyBub3Qgc3VwcG9ydGVkYDtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgdGhyb3cgYGNvbnZlcnNpb24gZnJvbSBQcm9wT2JqZWN0IHRvIHR5cGUgJyR7dG8ua2luZCgpfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgICB9XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gdHlwZSAnJHtmcm9tLmtpbmQoKX0nIHRvIHR5cGUgJyR7dG8ua2luZCgpfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgfVxuICB9XG5cbiAgaW1wbExpc3RSZXF1ZXN0VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVxdWVzdCwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsTGlzdFJlcGx5VHlwZShyZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSk6IHN0cmluZyB7XG4gICAgY29uc3QgbGlzdCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBcImxpc3RcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKGxpc3QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXF1ZXN0VHlwZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VSZXBseVR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXBseSwgcmVuZGVyT3B0aW9ucyk7XG4gIH1cblxuICBpbXBsU2VydmljZVRyYWNlTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWV9LiR7c25ha2VDYXNlKFxuICAgICAgdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZCwge1xuICAgICAgICBvcHRpb246IGZhbHNlLFxuICAgICAgICByZWZlcmVuY2U6IGZhbHNlLFxuICAgICAgfSksXG4gICAgKX1gO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UoXG4gICAgICB0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wTWV0aG9kLCB7XG4gICAgICAgIG9wdGlvbjogZmFsc2UsXG4gICAgICAgIHJlZmVyZW5jZTogZmFsc2UsXG4gICAgICB9KSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlBY3Rpb24ocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5QWN0aW9uLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUVkaXQocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5RWRpdC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDb21tb25DcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUNyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlR2V0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUdldC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VMaXN0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUxpc3QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tcG9uZW50UGljayhwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDb21wb25lbnRQaWNrLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUN1c3RvbU1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDdXN0b21NZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcE1ldGhvZC5za2lwQXV0aCkge1xuICAgICAgcmV0dXJuIGAvLyBBdXRoZW50aWNhdGlvbiBpcyBza2lwcGVkIG9uIFxcYCR7dGhpcy5pbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgICAgIHByb3BNZXRob2QsXG4gICAgICApfVxcYFxcbmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiB0aGlzLmltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZCk7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoQ2FsbChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBsZXQgcHJlbHVkZSA9IFwic2lfYWNjb3VudDo6YXV0aG9yaXplXCI7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lID09IFwiYWNjb3VudFwiKSB7XG4gICAgICBwcmVsdWRlID0gXCJjcmF0ZTo6YXV0aG9yaXplXCI7XG4gICAgfVxuICAgIHJldHVybiBgJHtwcmVsdWRlfTo6YXV0aG56KCZzZWxmLmRiLCAmcmVxdWVzdCwgXCIke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgcHJvcE1ldGhvZCxcbiAgICApfVwiKS5hd2FpdD87YDtcbiAgfVxuXG4gIHNlcnZpY2VNZXRob2RzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGNvbnN0IHByb3BNZXRob2RzID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5hdHRycy5zb3J0KChhLCBiKSA9PlxuICAgICAgYS5uYW1lID4gYi5uYW1lID8gMSA6IC0xLFxuICAgICk7XG4gICAgZm9yIChjb25zdCBwcm9wTWV0aG9kIG9mIHByb3BNZXRob2RzKSB7XG4gICAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2VNZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgICB7XG4gICAgICAgICAgZm10OiB0aGlzLFxuICAgICAgICAgIHByb3BNZXRob2Q6IHByb3BNZXRob2QsXG4gICAgICAgIH0sXG4gICAgICAgIHtcbiAgICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICAgIH0sXG4gICAgICApO1xuICAgICAgcmVzdWx0cy5wdXNoKG91dHB1dCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBydXN0RmllbGROYW1lRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICB9XG5cbiAgcnVzdFR5cGVGb3JQcm9wKFxuICAgIHByb3A6IFByb3BzLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlZmVyZW5jZSA9IHJlbmRlck9wdGlvbnMucmVmZXJlbmNlIHx8IGZhbHNlO1xuICAgIGxldCBvcHRpb24gPSB0cnVlO1xuICAgIGlmIChyZW5kZXJPcHRpb25zLm9wdGlvbiA9PT0gZmFsc2UpIHtcbiAgICAgIG9wdGlvbiA9IGZhbHNlO1xuICAgIH1cblxuICAgIGxldCB0eXBlTmFtZTogc3RyaW5nO1xuXG4gICAgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24gfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kXG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpMzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpNjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInU2NFwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1MTI4XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUxMjhcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BCb29sIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgY29uc3QgcmVhbFByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgaWYgKHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBjb25zdCBwcm9wT3duZXIgPSBwcm9wLmxvb2t1cE9iamVjdCgpO1xuICAgICAgICBsZXQgcGF0aE5hbWU6IHN0cmluZztcbiAgICAgICAgaWYgKFxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSAmJlxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSA9PSB0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZVxuICAgICAgICApIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH0gZWxzZSBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBgc2lfJHtwcm9wT3duZXIuc2VydmljZU5hbWV9Ojpwcm90b2J1ZmA7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBcImNyYXRlOjpwcm90b2J1ZlwiO1xuICAgICAgICB9XG4gICAgICAgIHR5cGVOYW1lID0gYCR7cGF0aE5hbWV9Ojoke3Bhc2NhbENhc2UocmVhbFByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICAgIHJlYWxQcm9wLm5hbWUsXG4gICAgICAgICl9YDtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChyZWFsUHJvcCwgcmVuZGVyT3B0aW9ucyk7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1hcCkge1xuICAgICAgdHlwZU5hbWUgPSBgc3RkOjpjb2xsZWN0aW9uczo6SGFzaE1hcDxTdHJpbmcsIFN0cmluZz5gO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFRleHQgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BTZWxlY3RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gXCJTdHJpbmdcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgYENhbm5vdCBnZW5lcmF0ZSB0eXBlIGZvciAke3Byb3AubmFtZX0ga2luZCAke3Byb3Aua2luZCgpfSAtIEJ1ZyFgO1xuICAgIH1cbiAgICBpZiAocmVmZXJlbmNlKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgIGlmICh0eXBlTmFtZSA9PSBcIlN0cmluZ1wiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCImc3RyXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgJiR7dHlwZU5hbWV9YDtcbiAgICAgIH1cbiAgICB9XG4gICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgdHlwZU5hbWUgPSBgVmVjPCR7dHlwZU5hbWV9PmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGlmIChvcHRpb24pIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICAgIHR5cGVOYW1lID0gYE9wdGlvbjwke3R5cGVOYW1lfT5gO1xuICAgICAgfVxuICAgIH1cbiAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICByZXR1cm4gdHlwZU5hbWU7XG4gIH1cblxuICBpbXBsQ3JlYXRlTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKGAke3NuYWtlQ2FzZShwcm9wLm5hbWUpfTogJHt0aGlzLnJ1c3RUeXBlRm9yUHJvcChwcm9wKX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsQ3JlYXRlUGFzc05ld0FyZ3MoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICByZXN1bHQucHVzaChzbmFrZUNhc2UocHJvcC5uYW1lKSk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2RMaXN0UmVzdWx0VG9SZXBseSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGxpc3RNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwibGlzdFwiKTtcbiAgICBpZiAobGlzdE1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBsaXN0TWV0aG9kLnJlcGx5LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGxldCBsaXN0UmVwbHlWYWx1ZSA9IGBTb21lKG91dHB1dC4ke2ZpZWxkTmFtZX0pYDtcbiAgICAgICAgaWYgKGZpZWxkTmFtZSA9PSBcIm5leHRfcGFnZV90b2tlblwiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBcIlNvbWUob3V0cHV0LnBhZ2VfdG9rZW4pXCI7XG4gICAgICAgIH0gZWxzZSBpZiAoZmllbGROYW1lID09IFwiaXRlbXNcIikge1xuICAgICAgICAgIGxpc3RSZXBseVZhbHVlID0gYG91dHB1dC4ke2ZpZWxkTmFtZX1gO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKGAke2ZpZWxkTmFtZX06ICR7bGlzdFJlcGx5VmFsdWV9YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNZXRob2RDcmVhdGVEZXN0cnVjdHVyZSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IGZpZWxkTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICByZXN1bHQucHVzaChgbGV0ICR7ZmllbGROYW1lfSA9IGlubmVyLiR7ZmllbGROYW1lfTtgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgbmF0dXJhbEtleSgpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIFN5c3RlbU9iamVjdCkge1xuICAgICAgcmV0dXJuIHNuYWtlQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC5uYXR1cmFsS2V5KTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwibmFtZVwiO1xuICAgIH1cbiAgfVxuXG4gIGltcGxDcmVhdGVTZXRQcm9wZXJ0aWVzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFBhc3N3b3JkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9IFNvbWUoc2lfZGF0YTo6cGFzc3dvcmQ6OmVuY3J5cHRfcGFzc3dvcmQoJHt2YXJpYWJsZU5hbWV9KT8pO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9ICR7dmFyaWFibGVOYW1lfTtgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmF0dHJzKSB7XG4gICAgICBjb25zdCB2YXJpYWJsZU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgIGNvbnN0IGRlZmF1bHRWYWx1ZSA9IHByb3AuZGVmYXVsdFZhbHVlKCk7XG4gICAgICBpZiAoZGVmYXVsdFZhbHVlKSB7XG4gICAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcInRleHRcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSBcIiR7ZGVmYXVsdFZhbHVlfVwiLnRvX3N0cmluZygpO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImVudW1cIikge1xuICAgICAgICAgIGNvbnN0IGVudW1OYW1lID0gYCR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lLFxuICAgICAgICAgICl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuc2V0XyR7dmFyaWFibGVOYW1lfShjcmF0ZTo6cHJvdG9idWY6OiR7ZW51bU5hbWV9Ojoke3Bhc2NhbENhc2UoXG4gICAgICAgICAgICAgIGRlZmF1bHRWYWx1ZSBhcyBzdHJpbmcsXG4gICAgICAgICAgICApfSk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVBZGRUb1RlbmFuY3koKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImJpbGxpbmdBY2NvdW50XCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25cIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblNlcnZpY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LmtpbmQoKSA9PSBcImNvbXBvbmVudE9iamVjdFwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uU2VydmljZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX3NlcnZpY2VfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcInVzZXJcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJncm91cFwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIm9yZ2FuaXphdGlvblwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uSW5zdGFuY2VcIlxuICAgICkge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwid29ya3NwYWNlXCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IG9yZ2FuaXphdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkub3JnYW5pemF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLm9yZ2FuaXphdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKG9yZ2FuaXphdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IHdvcmtzcGFjZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkud29ya3NwYWNlX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLndvcmtzcGFjZUlkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKHdvcmtzcGFjZV9pZCk7YCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlSXNNdmNjKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0Lm12Y2MgPT0gdHJ1ZSkge1xuICAgICAgcmV0dXJuIFwidHJ1ZVwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJmYWxzZVwiO1xuICAgIH1cbiAgfVxuXG4gIHN0b3JhYmxlVmFsaWRhdGVGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICAgIGNvbnN0IHByb3BOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0ubGVuKCkgPT0gMCB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmlzX25vbmUoKSB7XG4gICAgICAgICAgICAgcmV0dXJuIEVycihzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcIm1pc3NpbmcgcmVxdWlyZWQgJHtwcm9wTmFtZX0gdmFsdWVcIi5pbnRvKCkpKTtcbiAgICAgICAgICAgfWApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICB0b3BQcm9wOiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0LFxuICAgIHByZWZpeDogc3RyaW5nLFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbJ1wic2lTdG9yYWJsZS5uYXR1cmFsS2V5XCInXTtcbiAgICBmb3IgKGxldCBwcm9wIG9mIHRvcFByb3AucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKHByb3AuaGlkZGVuKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBwcm9wID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2godGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgcHJvcC5uYW1lKSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKFxuICAgICAgICAgICAgdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AocHJvcCwgYCR7cHJlZml4fS4ke3Byb3AubmFtZX1gKSxcbiAgICAgICAgICApO1xuICAgICAgICB9XG4gICAgICB9IGVsc2Uge1xuICAgICAgICBpZiAocHJlZml4ID09IFwiXCIpIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goYFwiJHtwcmVmaXh9LiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gdGhpcy5zdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC5yb290UHJvcCxcbiAgICAgIFwiXCIsXG4gICAgKTtcbiAgICByZXR1cm4gYHZlYyFbJHtyZXN1bHRzfV1cXG5gO1xuICB9XG5cbiAgc3RvcmFibGVSZWZlcmVudGlhbEZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgZmV0Y2hQcm9wcyA9IFtdO1xuICAgIGNvbnN0IHJlZmVyZW5jZVZlYyA9IFtdO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QpIHtcbiAgICAgIGxldCBzaVByb3BlcnRpZXMgPSB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuZ2V0RW50cnkoXCJzaVByb3BlcnRpZXNcIik7XG4gICAgICBpZiAoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgc2lQcm9wZXJ0aWVzID0gc2lQcm9wZXJ0aWVzLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKCEoc2lQcm9wZXJ0aWVzIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkpIHtcbiAgICAgICAgdGhyb3cgXCJDYW5ub3QgZ2V0IHByb3BlcnRpZXMgb2YgYSBub24gb2JqZWN0IGluIHJlZiBjaGVja1wiO1xuICAgICAgfVxuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIHNpUHJvcGVydGllcy5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGlmIChwcm9wLnJlZmVyZW5jZSkge1xuICAgICAgICAgIGNvbnN0IGl0ZW1OYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICAgIGZldGNoUHJvcHMucHVzaChgbGV0ICR7aXRlbU5hbWV9ID0gbWF0Y2ggJnNlbGYuc2lfcHJvcGVydGllcyB7XG4gICAgICAgICAgICAgICAgICAgICAgICAgICBTb21lKGNpcCkgPT4gY2lwXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuJHtpdGVtTmFtZX1cbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5hc19yZWYoKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLm1hcChTdHJpbmc6OmFzX3JlZilcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC51bndyYXBfb3IoXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIpLFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgICBOb25lID0+IFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiLFxuICAgICAgICAgICAgICAgICAgICAgICAgIH07YCk7XG4gICAgICAgICAgICByZWZlcmVuY2VWZWMucHVzaChcbiAgICAgICAgICAgICAgYHNpX2RhdGE6OlJlZmVyZW5jZTo6SGFzTWFueShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc09uZShcIiR7aXRlbU5hbWV9XCIsICR7aXRlbU5hbWV9KWAsXG4gICAgICAgICAgICApO1xuICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgfVxuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQmFzZU9iamVjdCkge1xuICAgIH1cblxuICAgIGlmIChmZXRjaFByb3BzLmxlbmd0aCAmJiByZWZlcmVuY2VWZWMubGVuZ3RoKSB7XG4gICAgICBjb25zdCByZXN1bHRzID0gW107XG4gICAgICByZXN1bHRzLnB1c2goZmV0Y2hQcm9wcy5qb2luKFwiXFxuXCIpKTtcbiAgICAgIHJlc3VsdHMucHVzaChgdmVjIVske3JlZmVyZW5jZVZlYy5qb2luKFwiLFwiKX1dYCk7XG4gICAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJWZWM6Om5ldygpXCI7XG4gICAgfVxuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyU2VydmljZSB7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZykge1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWUpO1xuICB9XG5cbiAgc3lzdGVtT2JqZWN0c0FzRm9ybWF0dGVycygpOiBSdXN0Rm9ybWF0dGVyW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNcbiAgICAgIC5zb3J0KChhLCBiKSA9PiAoYS50eXBlTmFtZSA+IGIudHlwZU5hbWUgPyAxIDogLTEpKVxuICAgICAgLm1hcChvID0+IG5ldyBSdXN0Rm9ybWF0dGVyKG8pKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlU3RydWN0Qm9keSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiOiBzaV9kYXRhOjpEYixcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudCxcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTmV3Q29uc3RydWN0b3JBcmdzKCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiLCBhZ2VudDogc2lfY2VhOjpBZ2VudENsaWVudFwiO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJkYjogc2lfZGF0YTo6RGJcIjtcbiAgICB9XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdENvbnN0cnVjdG9yUmV0dXJuKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW1wiZGJcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXRpZXMoKSkge1xuICAgICAgcmVzdWx0LnB1c2goXCJhZ2VudFwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlVHJhaXROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7c25ha2VDYXNlKFxuICAgICAgdGhpcy5zZXJ2aWNlTmFtZSxcbiAgICApfV9zZXJ2ZXI6OiR7cGFzY2FsQ2FzZSh0aGlzLnNlcnZpY2VOYW1lKX1gO1xuICB9XG5cbiAgaW1wbFNlcnZlck5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5pbXBsU2VydmljZVRyYWl0TmFtZSgpfVNlcnZlcmA7XG4gIH1cblxuICBpbXBsU2VydmljZU1pZ3JhdGUoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iaiBvZiB0aGlzLnN5c3RlbU9iamVjdHMpIHtcbiAgICAgIGlmICh0aGlzLmlzTWlncmF0ZWFibGUoc3lzdGVtT2JqKSkge1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UoXG4gICAgICAgICAgICBzeXN0ZW1PYmoudHlwZU5hbWUsXG4gICAgICAgICAgKX06Om1pZ3JhdGUoJnNlbGYuZGIpLmF3YWl0PztgLFxuICAgICAgICApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBoYXNFbnRpdGllcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzLnNvbWUob2JqID0+IG9iaiBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCk7XG4gIH1cblxuICBpc01pZ3JhdGVhYmxlKHByb3A6IE9iamVjdFR5cGVzKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHByb3AgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QgJiYgcHJvcC5taWdyYXRlYWJsZTtcbiAgfVxuXG4gIGhhc01pZ3JhdGFibGVzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHMuc29tZShvYmogPT4gdGhpcy5pc01pZ3JhdGVhYmxlKG9iaikpO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBSdXN0Rm9ybWF0dGVyQWdlbnQge1xuICBhZ2VudE5hbWU6IHN0cmluZztcbiAgZW50aXR5OiBFbnRpdHlPYmplY3Q7XG4gIGVudGl0eUZvcm1hdHRlcjogUnVzdEZvcm1hdHRlcjtcbiAgaW50ZWdyYXRpb25OYW1lOiBzdHJpbmc7XG4gIGludGVncmF0aW9uU2VydmljZU5hbWU6IHN0cmluZztcbiAgc2VydmljZU5hbWU6IHN0cmluZztcbiAgc3lzdGVtT2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nLCBhZ2VudDogQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2UpIHtcbiAgICB0aGlzLmFnZW50TmFtZSA9IGFnZW50LmFnZW50TmFtZTtcbiAgICB0aGlzLmVudGl0eSA9IGFnZW50LmVudGl0eTtcbiAgICB0aGlzLmVudGl0eUZvcm1hdHRlciA9IG5ldyBSdXN0Rm9ybWF0dGVyKHRoaXMuZW50aXR5KTtcbiAgICB0aGlzLmludGVncmF0aW9uTmFtZSA9IGFnZW50LmludGVncmF0aW9uTmFtZTtcbiAgICB0aGlzLmludGVncmF0aW9uU2VydmljZU5hbWUgPSBhZ2VudC5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lO1xuICAgIHRoaXMuc2VydmljZU5hbWUgPSBzZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnN5c3RlbU9iamVjdHMgPSByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWUpO1xuICB9XG5cbiAgc3lzdGVtT2JqZWN0c0FzRm9ybWF0dGVycygpOiBSdXN0Rm9ybWF0dGVyW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdHNcbiAgICAgIC5zb3J0KChhLCBiKSA9PiAoYS50eXBlTmFtZSA+IGIudHlwZU5hbWUgPyAxIDogLTEpKVxuICAgICAgLm1hcChvID0+IG5ldyBSdXN0Rm9ybWF0dGVyKG8pKTtcbiAgfVxuXG4gIGFjdGlvblByb3BzKCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5Lm1ldGhvZHMuYXR0cnMuZmlsdGVyKFxuICAgICAgbSA9PiBtIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXTtcbiAgfVxuXG4gIGVudGl0eUFjdGlvbk1ldGhvZE5hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiY3JlYXRlXCJdO1xuXG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuYWN0aW9uUHJvcHMoKSkge1xuICAgICAgaWYgKHRoaXMuZW50aXR5Rm9ybWF0dGVyLmlzRW50aXR5RWRpdE1ldGhvZChwcm9wKSkge1xuICAgICAgICByZXN1bHRzLnB1c2godGhpcy5lbnRpdHlGb3JtYXR0ZXIuZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcCkpO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKHByb3AubmFtZSk7XG4gICAgICB9XG4gICAgfVxuXG4gICAgcmV0dXJuIHJlc3VsdHM7XG4gIH1cblxuICBkaXNwYXRjaGVyQmFzZVR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3Bhc2NhbENhc2UodGhpcy5pbnRlZ3JhdGlvbk5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgIHRoaXMuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSxcbiAgICApfSR7cGFzY2FsQ2FzZSh0aGlzLmVudGl0eS5iYXNlVHlwZU5hbWUpfWA7XG4gIH1cblxuICBkaXNwYXRjaGVyVHlwZU5hbWUoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7dGhpcy5kaXNwYXRjaGVyQmFzZVR5cGVOYW1lKCl9RGlzcGF0Y2hlcmA7XG4gIH1cblxuICBkaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpfURpc3BhdGNoRnVuY3Rpb25zYDtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQ29kZWdlblJ1c3Qge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gIH1cblxuICBoYXNNb2RlbHMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHJlZ2lzdHJ5XG4gICAgICAuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHRoaXMuc2VydmljZU5hbWUpXG4gICAgICAuc29tZShvID0+IG8ua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKTtcbiAgfVxuXG4gIGhhc1NlcnZpY2VNZXRob2RzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICByZWdpc3RyeVxuICAgICAgICAuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHRoaXMuc2VydmljZU5hbWUpXG4gICAgICAgIC5mbGF0TWFwKG8gPT4gby5tZXRob2RzLmF0dHJzKS5sZW5ndGggPiAwXG4gICAgKTtcbiAgfVxuXG4gIGhhc0VudGl0eUludGVncmF0aW9uU2VydmNpY2VzKCk6IGJvb2xlYW4ge1xuICAgIGNvbnN0IGludGVncmF0aW9uU2VydmljZXMgPSBuZXcgU2V0KFxuICAgICAgdGhpcy5lbnRpdGllcygpLmZsYXRNYXAoZW50aXR5ID0+XG4gICAgICAgIHRoaXMuZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHkpLFxuICAgICAgKSxcbiAgICApO1xuICAgIHJldHVybiBpbnRlZ3JhdGlvblNlcnZpY2VzLnNpemUgPiAwO1xuICB9XG5cbiAgZW50aXRpZXMoKTogRW50aXR5T2JqZWN0W10ge1xuICAgIHJldHVybiByZWdpc3RyeVxuICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgLmZpbHRlcihvID0+IG8gaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIGFzIEVudGl0eU9iamVjdFtdO1xuICB9XG5cbiAgZW50aXR5QWN0aW9ucyhlbnRpdHk6IEVudGl0eU9iamVjdCk6IFByb3BQcmVsdWRlLlByb3BBY3Rpb25bXSB7XG4gICAgcmV0dXJuIGVudGl0eS5tZXRob2RzLmF0dHJzLmZpbHRlcihcbiAgICAgIG0gPT4gbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW107XG4gIH1cblxuICBlbnRpdHlpbnRlZ3JhdGlvblNlcnZpY2VzRm9yKGVudGl0eTogRW50aXR5T2JqZWN0KTogSW50ZWdyYXRpb25TZXJ2aWNlW10ge1xuICAgIGNvbnN0IHJlc3VsdDogU2V0PEludGVncmF0aW9uU2VydmljZT4gPSBuZXcgU2V0KCk7XG4gICAgZm9yIChjb25zdCBpbnRlZ3JhdGlvblNlcnZpY2Ugb2YgZW50aXR5LmludGVncmF0aW9uU2VydmljZXMpIHtcbiAgICAgIHJlc3VsdC5hZGQoaW50ZWdyYXRpb25TZXJ2aWNlKTtcbiAgICB9XG4gICAgZm9yIChjb25zdCBhY3Rpb24gb2YgdGhpcy5lbnRpdHlBY3Rpb25zKGVudGl0eSkpIHtcbiAgICAgIGZvciAoY29uc3QgaW50ZWdyYXRpb25TZXJ2aWNlIG9mIGFjdGlvbi5pbnRlZ3JhdGlvblNlcnZpY2VzKSB7XG4gICAgICAgIHJlc3VsdC5hZGQoaW50ZWdyYXRpb25TZXJ2aWNlKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIEFycmF5LmZyb20ocmVzdWx0KTtcbiAgfVxuXG4gIGVudGl0eUludGVncmF0aW9uU2VydmljZXMoKTogQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2VbXSB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXRpZXMoKS5mbGF0TWFwKGVudGl0eSA9PlxuICAgICAgdGhpcy5lbnRpdHlpbnRlZ3JhdGlvblNlcnZpY2VzRm9yKGVudGl0eSkubWFwKGludGVncmF0aW9uU2VydmljZSA9PiAoe1xuICAgICAgICBpbnRlZ3JhdGlvbk5hbWU6IGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvbk5hbWUsXG4gICAgICAgIGludGVncmF0aW9uU2VydmljZU5hbWU6IGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lLFxuICAgICAgICBlbnRpdHk6IGVudGl0eSxcbiAgICAgICAgYWdlbnROYW1lOiBgJHtzbmFrZUNhc2UoXG4gICAgICAgICAgaW50ZWdyYXRpb25TZXJ2aWNlLmludGVncmF0aW9uTmFtZSxcbiAgICAgICAgKX1fJHtzbmFrZUNhc2UoaW50ZWdyYXRpb25TZXJ2aWNlLmludGVncmF0aW9uU2VydmljZU5hbWUpfV8ke3NuYWtlQ2FzZShcbiAgICAgICAgICBlbnRpdHkuYmFzZVR5cGVOYW1lLFxuICAgICAgICApfWAsXG4gICAgICB9KSksXG4gICAgKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBpZiAodGhpcy5oYXNFbnRpdHlJbnRlZ3JhdGlvblNlcnZjaWNlcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goXCJwdWIgbW9kIGFnZW50O1wiKTtcbiAgICB9XG4gICAgaWYgKHRoaXMuaGFzTW9kZWxzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChcInB1YiBtb2QgbW9kZWw7XCIpO1xuICAgIH1cbiAgICBpZiAodGhpcy5oYXNTZXJ2aWNlTWV0aG9kcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goXCJwdWIgbW9kIHNlcnZpY2U7XCIpO1xuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL21vZGVsL21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5Nb2RlbE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXCIsIFwiXCJdO1xuICAgIGZvciAoY29uc3Qgc3lzdGVtT2JqZWN0IG9mIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKSkge1xuICAgICAgaWYgKHN5c3RlbU9iamVjdC5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKGBwdWIgbW9kICR7c25ha2VDYXNlKHN5c3RlbU9iamVjdC50eXBlTmFtZSl9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9tb2RlbC9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuU2VydmljZSgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlclNlcnZpY2UodGhpcy5zZXJ2aWNlTmFtZSksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoYGdlbi9zZXJ2aWNlLnJzYCwgb3V0cHV0KTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWwoc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcyk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IG91dHB1dCA9IGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L21vZGVsLnJzLmVqcycsIHsgZm10OiBmbXQgfSkgJT5cIixcbiAgICAgIHtcbiAgICAgICAgZm10OiBuZXcgUnVzdEZvcm1hdHRlcihzeXN0ZW1PYmplY3QpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFxuICAgICAgYGdlbi9tb2RlbC8ke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfS5yc2AsXG4gICAgICBvdXRwdXQsXG4gICAgKTtcbiAgfVxuXG4gIC8vIEdlbmVyYXRlIHRoZSAnZ2VuL2FnZW50L21vZC5ycydcbiAgYXN5bmMgZ2VuZXJhdGVHZW5BZ2VudE1vZCgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCByZXN1bHRzID0gW1wiLy8gQXV0by1nZW5lcmF0ZWQgY29kZSFcIiwgXCIvLyBObyB0b3VjaHkhXCIsIFwiXCJdO1xuICAgIGZvciAoY29uc3QgYWdlbnQgb2YgdGhpcy5lbnRpdHlJbnRlZ3JhdGlvblNlcnZpY2VzKCkpIHtcbiAgICAgIHJlc3VsdHMucHVzaChgcHViIG1vZCAke2FnZW50LmFnZW50TmFtZX07YCk7XG4gICAgfVxuICAgIHJlc3VsdHMucHVzaChcIlwiKTtcbiAgICBmb3IgKGNvbnN0IGFnZW50IG9mIHRoaXMuZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcygpKSB7XG4gICAgICBjb25zdCBmbXQgPSBuZXcgUnVzdEZvcm1hdHRlckFnZW50KHRoaXMuc2VydmljZU5hbWUsIGFnZW50KTtcbiAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgYHB1YiB1c2UgJHtcbiAgICAgICAgICBhZ2VudC5hZ2VudE5hbWVcbiAgICAgICAgfTo6eyR7Zm10LmRpc3BhdGNoRnVuY3Rpb25UcmFpdE5hbWUoKX0sICR7Zm10LmRpc3BhdGNoZXJUeXBlTmFtZSgpfX07YCxcbiAgICAgICk7XG4gICAgfVxuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKFwiZ2VuL2FnZW50L21vZC5yc1wiLCByZXN1bHRzLmpvaW4oXCJcXG5cIikpO1xuICB9XG5cbiAgYXN5bmMgZ2VuZXJhdGVHZW5BZ2VudChhZ2VudDogQWdlbnRJbnRlZ3JhdGlvblNlcnZpY2UpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9hZ2VudC5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXJBZ2VudCh0aGlzLnNlcnZpY2VOYW1lLCBhZ2VudCksXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICB9LFxuICAgICk7XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoYGdlbi9hZ2VudC8ke3NuYWtlQ2FzZShhZ2VudC5hZ2VudE5hbWUpfS5yc2AsIG91dHB1dCk7XG4gIH1cblxuICAvL2FzeW5jIG1ha2VQYXRoKHBhdGhQYXJ0OiBzdHJpbmcpOiBQcm9taXNlPHN0cmluZz4ge1xuICAvLyAgY29uc3QgcGF0aE5hbWUgPSBwYXRoLmpvaW4oXCIuLlwiLCBgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWAsIFwic3JjXCIsIHBhdGhQYXJ0KTtcbiAgLy8gIGNvbnN0IGFic29sdXRlUGF0aE5hbWUgPSBwYXRoLnJlc29sdmUocGF0aE5hbWUpO1xuICAvLyAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIocGF0aC5yZXNvbHZlKHBhdGhOYW1lKSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4gIC8vICByZXR1cm4gYWJzb2x1dGVQYXRoTmFtZTtcbiAgLy99XG5cbiAgYXN5bmMgZm9ybWF0Q29kZSgpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBhd2FpdCBleGVjQ21kKGBjYXJnbyBmbXQgLXAgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWApO1xuICB9XG5cbiAgYXN5bmMgd3JpdGVDb2RlKGZpbGVuYW1lOiBzdHJpbmcsIGNvZGU6IHN0cmluZyk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IGZ1bGxQYXRoTmFtZSA9IHBhdGguam9pbihcbiAgICAgIFwiLi5cIixcbiAgICAgIGBzaS0ke3RoaXMuc2VydmljZU5hbWV9YCxcbiAgICAgIFwic3JjXCIsXG4gICAgICBmaWxlbmFtZSxcbiAgICApO1xuICAgIGF3YWl0IGNvZGVGcy53cml0ZUNvZGUoZnVsbFBhdGhOYW1lLCBjb2RlKTtcbiAgfVxufVxuIl19