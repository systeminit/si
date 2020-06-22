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
    key: "componentContraintsEnums",
    value: function componentContraintsEnums() {
      if (this.systemObject instanceof _systemComponent.ComponentObject) {
        return this.systemObject.constraints.attrs.filter(function (c) {
          return c instanceof PropPrelude.PropEnum;
        }).map(function (c) {
          return c;
        });
      } else {
        throw new Error("You asked for component contraints on a non-component object; this is a bug!");
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
    key: "implProtobufEnum",
    value: function implProtobufEnum(propEnum) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implProtobufEnum.rs.ejs', { fmt: fmt, propEnum: propEnum }) %>", {
        fmt: this,
        propEnum: propEnum
      }, {
        filename: "."
      });
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
      } else if (prop instanceof PropPrelude.PropBool || prop instanceof PropPrelude.PropEnum || prop instanceof PropPrelude.PropObject) {
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
        throw new Error("All Props types covered; this code is unreachable!");
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
    key: "rustNameForEnumVariant",
    value: function rustNameForEnumVariant(variant) {
      return (0, _changeCase.pascalCase)(variant.replace(".", ""));
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInJlc3VsdHMiLCJraW5kIiwiZW50aXR5IiwicmVnaXN0cnkiLCJnZXQiLCJiYXNlVHlwZU5hbWUiLCJmbXQiLCJhY3Rpb25Qcm9wcyIsInByb3AiLCJpc0VudGl0eUVkaXRNZXRob2QiLCJwdXNoIiwiZW50aXR5RWRpdE1ldGhvZE5hbWUiLCJuYW1lIiwibWV0aG9kcyIsImdldEVudHJ5IiwicHJvcEFjdGlvbiIsImVudGl0eUVkaXRQcm9wZXJ0eSIsInJlbGF0aW9uc2hpcHMiLCJhbGwiLCJzb21lIiwicmVsIiwiUHJvcFByZWx1ZGUiLCJFaXRoZXIiLCJVcGRhdGVzIiwiaXNFbnRpdHlPYmplY3QiLCJlbnRpdHlFZGl0TWV0aG9kcyIsImhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uIiwiQ29tcG9uZW50T2JqZWN0IiwicHJvcE1ldGhvZCIsIlByb3BBY3Rpb24iLCJpc0VudGl0eUFjdGlvbk1ldGhvZCIsImVuZHNXaXRoIiwiRW50aXR5RXZlbnRPYmplY3QiLCJFbnRpdHlPYmplY3QiLCJ0eXBlTmFtZSIsIlN5c3RlbU9iamVjdCIsIm1pZ3JhdGVhYmxlIiwiYXR0cnMiLCJmaWx0ZXIiLCJtIiwiY29uc3RyYWludHMiLCJjIiwiUHJvcEVudW0iLCJtYXAiLCJFcnJvciIsInJ1c3RGaWVsZE5hbWVGb3JQcm9wIiwicmVwbGFjZSIsInAiLCJwcm9wZXJ0eSIsInJlcXVlc3QiLCJwcm9wZXJ0aWVzIiwiUHJvcExpbmsiLCJsb29rdXBNeXNlbGYiLCJydXN0VHlwZUZvclByb3AiLCJvcHRpb24iLCJyIiwidXBkYXRlIiwiZnJvbSIsInRvIiwicGFydG5lclByb3AiLCJmbGF0TWFwIiwibWV0aG9kIiwiZW50aXR5RWRpdFByb3BlcnR5VXBkYXRlcyIsIkFycmF5IiwiU2V0Iiwic29ydCIsImEiLCJiIiwiTWFwIiwiZmllbGRzIiwicHJvcEVpdGhlcnMiLCJsZW5ndGgiLCJlaXRoZXJzIiwiYWRkIiwiZWl0aGVyc0FycmF5Iiwic2V0IiwiZSIsImpvaW4iLCJlbnRyaWVzIiwidmFsdWVzIiwicHJvcGVydHlVcGRhdGUiLCJzZXJ2aWNlTmFtZSIsIlByb3BDb2RlIiwibGFuZ3VhZ2UiLCJQcm9wT2JqZWN0IiwicmVuZGVyT3B0aW9ucyIsImxpc3QiLCJyZXBseSIsInJlZmVyZW5jZSIsInByb3BFbnVtIiwiZWpzIiwicmVuZGVyIiwiZmlsZW5hbWUiLCJza2lwQXV0aCIsImltcGxTZXJ2aWNlTWV0aG9kTmFtZSIsImltcGxTZXJ2aWNlQXV0aENhbGwiLCJwcmVsdWRlIiwicHJvcE1ldGhvZHMiLCJvdXRwdXQiLCJQcm9wTWV0aG9kIiwicGFyZW50TmFtZSIsIlByb3BOdW1iZXIiLCJudW1iZXJLaW5kIiwiUHJvcEJvb2wiLCJyZWFsUHJvcCIsInByb3BPd25lciIsImxvb2t1cE9iamVjdCIsInBhdGhOYW1lIiwiUHJvcE1hcCIsIlByb3BUZXh0IiwiUHJvcFNlbGVjdCIsInJlcGVhdGVkIiwidmFyaWFudCIsInJlc3VsdCIsImNyZWF0ZU1ldGhvZCIsImxpc3RNZXRob2QiLCJmaWVsZE5hbWUiLCJsaXN0UmVwbHlWYWx1ZSIsIm5hdHVyYWxLZXkiLCJ2YXJpYWJsZU5hbWUiLCJQcm9wUGFzc3dvcmQiLCJkZWZhdWx0VmFsdWUiLCJlbnVtTmFtZSIsIm12Y2MiLCJyZXF1aXJlZCIsInByb3BOYW1lIiwidG9wUHJvcCIsInByZWZpeCIsImhpZGRlbiIsInN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcCIsInJvb3RQcm9wIiwiZmV0Y2hQcm9wcyIsInJlZmVyZW5jZVZlYyIsInNpUHJvcGVydGllcyIsIml0ZW1OYW1lIiwiQmFzZU9iamVjdCIsIlJ1c3RGb3JtYXR0ZXJTZXJ2aWNlIiwic3lzdGVtT2JqZWN0cyIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsIm8iLCJoYXNFbnRpdGllcyIsImltcGxTZXJ2aWNlVHJhaXROYW1lIiwic3lzdGVtT2JqIiwiaXNNaWdyYXRlYWJsZSIsIm9iaiIsIlJ1c3RGb3JtYXR0ZXJBZ2VudCIsImFnZW50IiwiYWdlbnROYW1lIiwiZW50aXR5Rm9ybWF0dGVyIiwiaW50ZWdyYXRpb25OYW1lIiwiaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSIsImRpc3BhdGNoZXJCYXNlVHlwZU5hbWUiLCJDb2RlZ2VuUnVzdCIsImludGVncmF0aW9uU2VydmljZXMiLCJlbnRpdGllcyIsImVudGl0eWludGVncmF0aW9uU2VydmljZXNGb3IiLCJzaXplIiwiaW50ZWdyYXRpb25TZXJ2aWNlIiwiZW50aXR5QWN0aW9ucyIsImFjdGlvbiIsImhhc0VudGl0eUludGVncmF0aW9uU2VydmNpY2VzIiwiaGFzTW9kZWxzIiwiaGFzU2VydmljZU1ldGhvZHMiLCJ3cml0ZUNvZGUiLCJlbnRpdHlJbnRlZ3JhdGlvblNlcnZpY2VzIiwiZGlzcGF0Y2hGdW5jdGlvblRyYWl0TmFtZSIsImRpc3BhdGNoZXJUeXBlTmFtZSIsImNvZGUiLCJmdWxsUGF0aE5hbWUiLCJwYXRoIiwiY29kZUZzIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7QUFRQTs7QUFDQTs7QUFHQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7Ozs7Ozs7QUFFQSxJQUFNQSxPQUFPLEdBQUdDLGlCQUFLQyxTQUFMLENBQWVDLDBCQUFhQyxJQUE1QixDQUFoQjs7SUF1QmFDLGE7QUFHWCx5QkFBWUMsWUFBWixFQUF5RDtBQUFBO0FBQUE7QUFDdkQsU0FBS0EsWUFBTCxHQUFvQkEsWUFBcEI7QUFDRDs7Ozs4Q0FFbUM7QUFDbEMsVUFBTUMsT0FBTyxHQUFHLENBQUMsUUFBRCxDQUFoQjs7QUFFQSxVQUFJLEtBQUtELFlBQUwsQ0FBa0JFLElBQWxCLE1BQTRCLG1CQUFoQyxFQUFxRDtBQUNuRDtBQUNBLFlBQU1DLE1BQU0sR0FBR0MsbUJBQVNDLEdBQVQs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WixpQkFMYSxDOzt1QkFTVCxLQUFLdUUsU0FBTCxxQkFBNEIsMkJBQVVqQixLQUFLLENBQUNDLFNBQWhCLENBQTVCLFVBQTZEakQsTUFBN0QsQzs7Ozs7Ozs7Ozs7Ozs7O1FBR1I7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOzs7Ozs7Ozs7Ozt1QkFHUXpHLE9BQU8sMkJBQW9CLEtBQUt3RixXQUF6QixFOzs7Ozs7Ozs7Ozs7Ozs7Ozs7O3VIQUdDVyxRLEVBQWtCMkUsSTs7Ozs7O0FBQzFCQyxnQkFBQUEsWSxHQUFlQyxpQkFBSzVGLElBQUwsQ0FDbkIsSUFEbUIsZUFFYixLQUFLSSxXQUZRLEdBR25CLEtBSG1CLEVBSW5CVyxRQUptQixDOzt1QkFNZjhFLE1BQU0sQ0FBQ1AsU0FBUCxDQUFpQkssWUFBakIsRUFBK0JELElBQS9CLEMiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQge1xuICBPYmplY3RUeXBlcyxcbiAgQmFzZU9iamVjdCxcbiAgU3lzdGVtT2JqZWN0LFxuICBDb21wb25lbnRPYmplY3QsXG4gIEVudGl0eU9iamVjdCxcbiAgRW50aXR5RXZlbnRPYmplY3QsXG59IGZyb20gXCIuLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCAqIGFzIFByb3BQcmVsdWRlIGZyb20gXCIuLi9jb21wb25lbnRzL3ByZWx1ZGVcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5pbXBvcnQgeyBQcm9wcywgSW50ZWdyYXRpb25TZXJ2aWNlIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5cbmltcG9ydCB7IHNuYWtlQ2FzZSwgcGFzY2FsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IGVqcyBmcm9tIFwiZWpzXCI7XG5pbXBvcnQgcGF0aCBmcm9tIFwicGF0aFwiO1xuaW1wb3J0IGNoaWxkUHJvY2VzcyBmcm9tIFwiY2hpbGRfcHJvY2Vzc1wiO1xuaW1wb3J0IHV0aWwgZnJvbSBcInV0aWxcIjtcbmltcG9ydCAqIGFzIGNvZGVGcyBmcm9tIFwiLi9mc1wiO1xuXG5jb25zdCBleGVjQ21kID0gdXRpbC5wcm9taXNpZnkoY2hpbGRQcm9jZXNzLmV4ZWMpO1xuXG5pbnRlcmZhY2UgUnVzdFR5cGVBc1Byb3BPcHRpb25zIHtcbiAgcmVmZXJlbmNlPzogYm9vbGVhbjtcbiAgb3B0aW9uPzogYm9vbGVhbjtcbn1cblxuaW50ZXJmYWNlIEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlIHtcbiAgYWdlbnROYW1lOiBzdHJpbmc7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBpbnRlZ3JhdGlvbk5hbWU6IHN0cmluZztcbiAgaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTogc3RyaW5nO1xufVxuXG5pbnRlcmZhY2UgUHJvcGVydHlVcGRhdGUge1xuICBmcm9tOiBQcm9wUHJlbHVkZS5Qcm9wcztcbiAgdG86IFByb3BQcmVsdWRlLlByb3BzO1xufVxuXG5pbnRlcmZhY2UgUHJvcGVydHlFaXRoZXJTZXQge1xuICBlbnRyaWVzOiBQcm9wUHJlbHVkZS5Qcm9wc1tdO1xufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBlbnRpdHlBY3Rpb25NZXRob2ROYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcImNyZWF0ZVwiXTtcblxuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJlbnRpdHlFdmVudE9iamVjdFwiKSB7XG4gICAgICAvLyBAdHMtaWdub3JlXG4gICAgICBjb25zdCBlbnRpdHkgPSByZWdpc3RyeS5nZXQoYCR7dGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lfUVudGl0eWApO1xuICAgICAgY29uc3QgZm10ID0gbmV3IFJ1c3RGb3JtYXR0ZXIoZW50aXR5KTtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBmbXQuYWN0aW9uUHJvcHMoKSkge1xuICAgICAgICBpZiAoZm10LmlzRW50aXR5RWRpdE1ldGhvZChwcm9wKSkge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChmbXQuZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcCkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChwcm9wLm5hbWUpO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfSBlbHNlIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLmFjdGlvblByb3BzKCkpIHtcbiAgICAgICAgaWYgKHRoaXMuaXNFbnRpdHlFZGl0TWV0aG9kKHByb3ApKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcCkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChwcm9wLm5hbWUpO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuXG4gICAgcmV0dXJuIHJlc3VsdHM7XG4gIH1cblxuICBoYXNDcmVhdGVNZXRob2QoKTogYm9vbGVhbiB7XG4gICAgdHJ5IHtcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGNhdGNoIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cblxuICBoYXNFZGl0RWl0aGVyc0ZvckFjdGlvbihwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLnNvbWUocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLkVpdGhlcik7XG4gIH1cblxuICBoYXNFZGl0VXBkYXRlc0ZvckFjdGlvbihwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLnNvbWUocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlVwZGF0ZXMpO1xuICB9XG5cbiAgaGFzRWRpdFVwZGF0ZXNBbmRFaXRoZXJzKCk6IGJvb2xlYW4ge1xuICAgIGlmICh0aGlzLmlzRW50aXR5T2JqZWN0KCkpIHtcbiAgICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRNZXRob2RzKCkuc29tZShcbiAgICAgICAgcHJvcEFjdGlvbiA9PlxuICAgICAgICAgIHRoaXMuaGFzRWRpdFVwZGF0ZXNGb3JBY3Rpb24ocHJvcEFjdGlvbikgJiZcbiAgICAgICAgICB0aGlzLmhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uKHByb3BBY3Rpb24pLFxuICAgICAgKTtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgcmFuICdoYXNFZGl0VXBkYXRlc0FuZEVpdGhlcnMoKScgb24gYSBub24tZW50aXR5IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBpc0NvbXBvbmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3Q7XG4gIH1cblxuICBpc0VudGl0eUFjdGlvbk1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHRoaXMuaXNFbnRpdHlPYmplY3QoKSAmJiBwcm9wTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvblxuICAgICk7XG4gIH1cblxuICBpc0VudGl0eUVkaXRNZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLmlzRW50aXR5QWN0aW9uTWV0aG9kKHByb3BNZXRob2QpICYmIHByb3BNZXRob2QubmFtZS5lbmRzV2l0aChcIkVkaXRcIilcbiAgICApO1xuICB9XG5cbiAgaXNFbnRpdHlFdmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdDtcbiAgfVxuXG4gIGlzRW50aXR5T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdDtcbiAgfVxuXG4gIGlzQ2hhbmdlU2V0T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImNoYW5nZVNldFwiO1xuICB9XG5cbiAgaXNNaWdyYXRlYWJsZSgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QgJiYgdGhpcy5zeXN0ZW1PYmplY3QubWlncmF0ZWFibGVcbiAgICApO1xuICB9XG5cbiAgaXNTdG9yYWJsZSgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBhY3Rpb25Qcm9wcygpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmF0dHJzLmZpbHRlcihcbiAgICAgIG0gPT4gbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW107XG4gIH1cblxuICBjb21wb25lbnROYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUNvbXBvbmVudGA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBjb21wb25lbnQgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGNvbXBvbmVudENvbnN0cmFpbnRzTmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1Db21wb25lbnRDb25zdHJhaW50c2A7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhIGNvbXBvbmVudCBjb25zdHJhaW50cyBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgY29tcG9uZW50Q29udHJhaW50c0VudW1zKCk6IFByb3BQcmVsdWRlLlByb3BFbnVtW10ge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCkge1xuICAgICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LmNvbnN0cmFpbnRzLmF0dHJzXG4gICAgICAgIC5maWx0ZXIoYyA9PiBjIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEVudW0pXG4gICAgICAgIC5tYXAoYyA9PiBjIGFzIFByb3BQcmVsdWRlLlByb3BFbnVtKTtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBcIllvdSBhc2tlZCBmb3IgY29tcG9uZW50IGNvbnRyYWludHMgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIixcbiAgICAgICk7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSB7XG4gICAgICByZXR1cm4gYGVkaXRfJHt0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BNZXRob2QpLnJlcGxhY2UoXG4gICAgICAgIFwiX2VkaXRcIixcbiAgICAgICAgXCJcIixcbiAgICAgICl9YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVkaXQgbWV0aG9kIG5hbWUgb24gYSBub24tZW50aXR5IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlFZGl0TWV0aG9kcygpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiB0aGlzLmFjdGlvblByb3BzKCkuZmlsdGVyKHAgPT4gdGhpcy5pc0VudGl0eUVkaXRNZXRob2QocCkpO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBQcm9wcyB7XG4gICAgbGV0IHByb3BlcnR5ID0gcHJvcEFjdGlvbi5yZXF1ZXN0LnByb3BlcnRpZXMuZ2V0RW50cnkoXCJwcm9wZXJ0eVwiKTtcbiAgICBpZiAocHJvcGVydHkgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgcHJvcGVydHkgPSBwcm9wZXJ0eS5sb29rdXBNeXNlbGYoKTtcbiAgICB9XG4gICAgcmV0dXJuIHByb3BlcnR5O1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5RmllbGQocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdEZpZWxkTmFtZUZvclByb3AodGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbikpO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5VHlwZShwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AodGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbiksIHtcbiAgICAgIG9wdGlvbjogZmFsc2UsXG4gICAgfSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzKFxuICAgIHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IFByb3BlcnR5VXBkYXRlW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKVxuICAgICAgLnJlbGF0aW9uc2hpcHMuYWxsKClcbiAgICAgIC5maWx0ZXIociA9PiByIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuVXBkYXRlcylcbiAgICAgIC5tYXAodXBkYXRlID0+ICh7XG4gICAgICAgIGZyb206IHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pLFxuICAgICAgICB0bzogdXBkYXRlLnBhcnRuZXJQcm9wKCksXG4gICAgICB9KSk7XG4gIH1cblxuICBhbGxFbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzKCk6IFByb3BlcnR5VXBkYXRlW10ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSB0aGlzLmVudGl0eUVkaXRNZXRob2RzKCkuZmxhdE1hcChtZXRob2QgPT5cbiAgICAgIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5VXBkYXRlcyhtZXRob2QpLFxuICAgICk7XG5cbiAgICByZXR1cm4gQXJyYXkuZnJvbShuZXcgU2V0KHJlc3VsdHMpKS5zb3J0KChhLCBiKSA9PlxuICAgICAgYCR7YS5mcm9tLm5hbWV9LCR7YS50by5uYW1lfWAgPiBgJHtiLmZyb20ubmFtZX0sJHtiLnRvLm5hbWV9YCA/IDEgOiAtMSxcbiAgICApO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5RWl0aGVycygpOiBQcm9wZXJ0eUVpdGhlclNldFtdIHtcbiAgICBjb25zdCByZXN1bHRzID0gbmV3IE1hcCgpO1xuICAgIGNvbnN0IHByb3BlcnRpZXMgPSAodGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmdldEVudHJ5KFxuICAgICAgXCJwcm9wZXJ0aWVzXCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KS5wcm9wZXJ0aWVzLmF0dHJzO1xuXG4gICAgZm9yIChjb25zdCBwcm9wZXJ0eSBvZiBwcm9wZXJ0aWVzKSB7XG4gICAgICBjb25zdCBwcm9wRWl0aGVycyA9IHByb3BlcnR5LnJlbGF0aW9uc2hpcHNcbiAgICAgICAgLmFsbCgpXG4gICAgICAgIC5maWx0ZXIocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLkVpdGhlcik7XG5cbiAgICAgIGlmIChwcm9wRWl0aGVycy5sZW5ndGggPiAwKSB7XG4gICAgICAgIGNvbnN0IGVpdGhlcnMgPSBuZXcgU2V0PFByb3BQcmVsdWRlLlByb3BzPigpO1xuICAgICAgICBlaXRoZXJzLmFkZChwcm9wZXJ0eSk7XG4gICAgICAgIGZvciAoY29uc3QgcHJvcGVydHkgb2YgcHJvcEVpdGhlcnMpIHtcbiAgICAgICAgICBlaXRoZXJzLmFkZChwcm9wZXJ0eS5wYXJ0bmVyUHJvcCgpKTtcbiAgICAgICAgfVxuXG4gICAgICAgIGNvbnN0IGVpdGhlcnNBcnJheSA9IEFycmF5LmZyb20oZWl0aGVycykuc29ydCgoYSwgYikgPT5cbiAgICAgICAgICBhLm5hbWUgPiBiLm5hbWUgPyAxIDogLTEsXG4gICAgICAgICk7XG4gICAgICAgIHJlc3VsdHMuc2V0KGVpdGhlcnNBcnJheS5tYXAoZSA9PiBlLm5hbWUpLmpvaW4oXCIsXCIpLCB7XG4gICAgICAgICAgZW50cmllczogZWl0aGVyc0FycmF5LFxuICAgICAgICB9KTtcbiAgICAgIH1cbiAgICB9XG5cbiAgICByZXR1cm4gQXJyYXkuZnJvbShyZXN1bHRzLnZhbHVlcygpKS5zb3J0KCk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVNZXRob2ROYW1lKHByb3BlcnR5VXBkYXRlOiBQcm9wZXJ0eVVwZGF0ZSk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGB1cGRhdGVfJHt0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKFxuICAgICAgcHJvcGVydHlVcGRhdGUudG8sXG4gICAgKX1fZnJvbV8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcGVydHlVcGRhdGUuZnJvbSl9YDtcbiAgfVxuXG4gIGVudGl0eUV2ZW50TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlFdmVudGA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHlFdmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCI7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIjtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlQcm9wZXJ0aWVzTmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlQcm9wZXJ0aWVzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eVByb3BlcnRpZXMgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiO1xuICAgIH1cbiAgfVxuXG4gIGVycm9yVHlwZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OmVycm9yOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWUpfUVycm9yYDtcbiAgfVxuXG4gIG1vZGVsTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6Om1vZGVsOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICBtb2RlbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BNZXRob2QpO1xuICB9XG5cbiAgc3RydWN0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICB0eXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuICB9XG5cbiAgaW1wbFRyeUZyb21Gb3JQcm9wZXJ0eVVwZGF0ZShwcm9wZXJ0eVVwZGF0ZTogUHJvcGVydHlVcGRhdGUpOiBzdHJpbmcge1xuICAgIGNvbnN0IGZyb20gPSBwcm9wZXJ0eVVwZGF0ZS5mcm9tO1xuICAgIGNvbnN0IHRvID0gcHJvcGVydHlVcGRhdGUudG87XG5cbiAgICAvLyBFdmVyeSBmYWxsdGhyb3VnaC9kZWZhdWx0L2Vsc2UgbmVlZHMgYSBgdGhyb3dgIGNsYXVzZSB0byBsb3VkbHkgcHJvY2xhaW1cbiAgICAvLyB0aGF0IGEgc3BlY2lmaWMgY29udmVyc2lvbiBpcyBub3Qgc3VwcG9ydGVkLiBUaGlzIGFsbG93cyB1cyB0byBhZGRcbiAgICAvLyBjb252ZXJzaW9ucyBhcyB3ZSBnbyB3aXRob3V0IHJvZ3VlIGFuZCB1bmV4cGxhaW5lZCBlcnJvcnMuIEluIHNob3J0LFxuICAgIC8vIHRyZWF0IHRoaXMgbGlrZSBSdXN0IGNvZGUgd2l0aCBmdWxseSBzYXRpc2ZpZWQgbWF0Y2ggYXJtcy4gVGhhbmsgeW91LFxuICAgIC8vIGxvdmUsIHVzLlxuICAgIGlmIChmcm9tIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUpIHtcbiAgICAgIHN3aXRjaCAoZnJvbS5sYW5ndWFnZSkge1xuICAgICAgICBjYXNlIFwieWFtbFwiOlxuICAgICAgICAgIGlmICh0byBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgICAgIHJldHVybiBgT2soc2VyZGVfeWFtbDo6ZnJvbV9zdHIodmFsdWUpPylgO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIGxhbmd1YWdlICcke1xuICAgICAgICAgICAgICBmcm9tLmxhbmd1YWdlXG4gICAgICAgICAgICB9JyB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgICAgICAgIH1cbiAgICAgICAgZGVmYXVsdDpcbiAgICAgICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIGxhbmd1YWdlICcke2Zyb20ubGFuZ3VhZ2V9JyBpcyBub3Qgc3VwcG9ydGVkYDtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKGZyb20gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICBpZiAodG8gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSkge1xuICAgICAgICBzd2l0Y2ggKHRvLmxhbmd1YWdlKSB7XG4gICAgICAgICAgY2FzZSBcInlhbWxcIjpcbiAgICAgICAgICAgIHJldHVybiBgT2soc2VyZGVfeWFtbDo6dG9fc3RyaW5nKHZhbHVlKT8pYDtcbiAgICAgICAgICBkZWZhdWx0OlxuICAgICAgICAgICAgdGhyb3cgYGNvbnZlcnNpb24gZnJvbSBQcm9wT2JqZWN0IHRvIGxhbmd1YWdlICcke3RvLmxhbmd1YWdlfScgaXMgbm90IHN1cHBvcnRlZGA7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHRocm93IGBjb252ZXJzaW9uIGZyb20gUHJvcE9iamVjdCB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgICAgfVxuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBgY29udmVyc2lvbiBmcm9tIHR5cGUgJyR7ZnJvbS5raW5kKCl9JyB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgO1xuICAgIH1cbiAgfVxuXG4gIGltcGxMaXN0UmVxdWVzdFR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbExpc3RSZXBseVR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVxdWVzdFR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVwbHlUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VUcmFjZU5hbWUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lfS4ke3NuYWtlQ2FzZShcbiAgICAgIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QsIHtcbiAgICAgICAgb3B0aW9uOiBmYWxzZSxcbiAgICAgICAgcmVmZXJlbmNlOiBmYWxzZSxcbiAgICAgIH0pLFxuICAgICl9YDtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKFxuICAgICAgdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZCwge1xuICAgICAgICBvcHRpb246IGZhbHNlLFxuICAgICAgICByZWZlcmVuY2U6IGZhbHNlLFxuICAgICAgfSksXG4gICAgKTtcbiAgfVxuXG4gIGltcGxQcm90b2J1ZkVudW0ocHJvcEVudW06IFByb3BQcmVsdWRlLlByb3BFbnVtKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFByb3RvYnVmRW51bS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wRW51bTogcHJvcEVudW0gfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wRW51bTogcHJvcEVudW0gfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5QWN0aW9uKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUFjdGlvbi5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlFZGl0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUVkaXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUNvbW1vbkNyZWF0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDaGFuZ2VTZXRDcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ2hhbmdlU2V0Q3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUNyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlR2V0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUdldC5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VMaXN0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUxpc3QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tcG9uZW50UGljayhwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDb21wb25lbnRQaWNrLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUN1c3RvbU1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VDdXN0b21NZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBpZiAocHJvcE1ldGhvZC5za2lwQXV0aCkge1xuICAgICAgcmV0dXJuIGAvLyBBdXRoZW50aWNhdGlvbiBpcyBza2lwcGVkIG9uIFxcYCR7dGhpcy5pbXBsU2VydmljZU1ldGhvZE5hbWUoXG4gICAgICAgIHByb3BNZXRob2QsXG4gICAgICApfVxcYFxcbmA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiB0aGlzLmltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZCk7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VBdXRoQ2FsbChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICBsZXQgcHJlbHVkZSA9IFwic2lfYWNjb3VudDo6YXV0aG9yaXplXCI7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lID09IFwiYWNjb3VudFwiKSB7XG4gICAgICBwcmVsdWRlID0gXCJjcmF0ZTo6YXV0aG9yaXplXCI7XG4gICAgfVxuICAgIHJldHVybiBgJHtwcmVsdWRlfTo6YXV0aG56KCZzZWxmLmRiLCAmcmVxdWVzdCwgXCIke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgcHJvcE1ldGhvZCxcbiAgICApfVwiKS5hd2FpdD87YDtcbiAgfVxuXG4gIHNlcnZpY2VNZXRob2RzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGNvbnN0IHByb3BNZXRob2RzID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5hdHRycy5zb3J0KChhLCBiKSA9PlxuICAgICAgYS5uYW1lID4gYi5uYW1lID8gMSA6IC0xLFxuICAgICk7XG4gICAgZm9yIChjb25zdCBwcm9wTWV0aG9kIG9mIHByb3BNZXRob2RzKSB7XG4gICAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L3NlcnZpY2VNZXRob2QucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgICB7XG4gICAgICAgICAgZm10OiB0aGlzLFxuICAgICAgICAgIHByb3BNZXRob2Q6IHByb3BNZXRob2QsXG4gICAgICAgIH0sXG4gICAgICAgIHtcbiAgICAgICAgICBmaWxlbmFtZTogXCIuXCIsXG4gICAgICAgIH0sXG4gICAgICApO1xuICAgICAgcmVzdWx0cy5wdXNoKG91dHB1dCk7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBydXN0RmllbGROYW1lRm9yUHJvcChwcm9wOiBQcm9wcyk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICB9XG5cbiAgcnVzdFR5cGVGb3JQcm9wKFxuICAgIHByb3A6IFByb3BzLFxuICAgIHJlbmRlck9wdGlvbnM6IFJ1c3RUeXBlQXNQcm9wT3B0aW9ucyA9IHt9LFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlZmVyZW5jZSA9IHJlbmRlck9wdGlvbnMucmVmZXJlbmNlIHx8IGZhbHNlO1xuICAgIGxldCBvcHRpb24gPSB0cnVlO1xuICAgIGlmIChyZW5kZXJPcHRpb25zLm9wdGlvbiA9PT0gZmFsc2UpIHtcbiAgICAgIG9wdGlvbiA9IGZhbHNlO1xuICAgIH1cblxuICAgIGxldCB0eXBlTmFtZTogc3RyaW5nO1xuXG4gICAgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24gfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kXG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BOdW1iZXIpIHtcbiAgICAgIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpMzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDMyXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUzMlwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJpNjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidWludDY0XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInU2NFwiO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLm51bWJlcktpbmQgPT0gXCJ1MTI4XCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcInUxMjhcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BCb29sIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEVudW0gfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0XG4gICAgKSB7XG4gICAgICB0eXBlTmFtZSA9IGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX1gO1xuICAgIH0gZWxzZSBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICBpZiAocmVhbFByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGNvbnN0IHByb3BPd25lciA9IHByb3AubG9va3VwT2JqZWN0KCk7XG4gICAgICAgIGxldCBwYXRoTmFtZTogc3RyaW5nO1xuICAgICAgICBpZiAoXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lICYmXG4gICAgICAgICAgcHJvcE93bmVyLnNlcnZpY2VOYW1lID09IHRoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lXG4gICAgICAgICkge1xuICAgICAgICAgIHBhdGhOYW1lID0gXCJjcmF0ZTo6cHJvdG9idWZcIjtcbiAgICAgICAgfSBlbHNlIGlmIChwcm9wT3duZXIuc2VydmljZU5hbWUpIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IGBzaV8ke3Byb3BPd25lci5zZXJ2aWNlTmFtZX06OnByb3RvYnVmYDtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH1cbiAgICAgICAgdHlwZU5hbWUgPSBgJHtwYXRoTmFtZX06OiR7cGFzY2FsQ2FzZShyZWFsUHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgICAgcmVhbFByb3AubmFtZSxcbiAgICAgICAgKX1gO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHJlYWxQcm9wLCByZW5kZXJPcHRpb25zKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWFwKSB7XG4gICAgICB0eXBlTmFtZSA9IGBzdGQ6OmNvbGxlY3Rpb25zOjpIYXNoTWFwPFN0cmluZywgU3RyaW5nPmA7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wVGV4dCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BDb2RlIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFNlbGVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoXCJBbGwgUHJvcHMgdHlwZXMgY292ZXJlZDsgdGhpcyBjb2RlIGlzIHVucmVhY2hhYmxlIVwiKTtcbiAgICB9XG4gICAgaWYgKHJlZmVyZW5jZSkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICBpZiAodHlwZU5hbWUgPT0gXCJTdHJpbmdcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiJnN0clwiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICAgIHR5cGVOYW1lID0gYCYke3R5cGVOYW1lfWA7XG4gICAgICB9XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgIHR5cGVOYW1lID0gYFZlYzwke3R5cGVOYW1lfT5gO1xuICAgIH0gZWxzZSB7XG4gICAgICBpZiAob3B0aW9uKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGBPcHRpb248JHt0eXBlTmFtZX0+YDtcbiAgICAgIH1cbiAgICB9XG4gICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgcmV0dXJuIHR5cGVOYW1lO1xuICB9XG5cbiAgcnVzdE5hbWVGb3JFbnVtVmFyaWFudCh2YXJpYW50OiBzdHJpbmcpOiBzdHJpbmcge1xuICAgIHJldHVybiBwYXNjYWxDYXNlKHZhcmlhbnQucmVwbGFjZShcIi5cIiwgXCJcIikpO1xuICB9XG5cbiAgaW1wbENyZWF0ZU5ld0FyZ3MoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICByZXN1bHQucHVzaChgJHtzbmFrZUNhc2UocHJvcC5uYW1lKX06ICR7dGhpcy5ydXN0VHlwZUZvclByb3AocHJvcCl9YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIiwgXCIpO1xuICB9XG5cbiAgaW1wbENyZWF0ZVBhc3NOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goc25ha2VDYXNlKHByb3AubmFtZSkpO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTGlzdFJlc3VsdFRvUmVwbHkoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBsaXN0TWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImxpc3RcIik7XG4gICAgaWYgKGxpc3RNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgbGlzdE1ldGhvZC5yZXBseS5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IGZpZWxkTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBsZXQgbGlzdFJlcGx5VmFsdWUgPSBgU29tZShvdXRwdXQuJHtmaWVsZE5hbWV9KWA7XG4gICAgICAgIGlmIChmaWVsZE5hbWUgPT0gXCJuZXh0X3BhZ2VfdG9rZW5cIikge1xuICAgICAgICAgIGxpc3RSZXBseVZhbHVlID0gXCJTb21lKG91dHB1dC5wYWdlX3Rva2VuKVwiO1xuICAgICAgICB9IGVsc2UgaWYgKGZpZWxkTmFtZSA9PSBcIml0ZW1zXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IGBvdXRwdXQuJHtmaWVsZE5hbWV9YDtcbiAgICAgICAgfVxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9OiAke2xpc3RSZXBseVZhbHVlfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kQ3JlYXRlRGVzdHJ1Y3R1cmUoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgcmVzdWx0LnB1c2goYGxldCAke2ZpZWxkTmFtZX0gPSBpbm5lci4ke2ZpZWxkTmFtZX07YCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIlxcblwiKTtcbiAgfVxuXG4gIG5hdHVyYWxLZXkoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QpIHtcbiAgICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QubmF0dXJhbEtleSk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIm5hbWVcIjtcbiAgICB9XG4gIH1cblxuICBpbXBsQ3JlYXRlU2V0UHJvcGVydGllcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIGNvbnN0IHZhcmlhYmxlTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BQYXNzd29yZCkge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSBTb21lKHNpX2RhdGE6OnBhc3N3b3JkOjplbmNyeXB0X3Bhc3N3b3JkKCR7dmFyaWFibGVOYW1lfSk/KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYHJlc3VsdC4ke3ZhcmlhYmxlTmFtZX0gPSAke3ZhcmlhYmxlTmFtZX07YCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5hdHRycykge1xuICAgICAgY29uc3QgdmFyaWFibGVOYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICBjb25zdCBkZWZhdWx0VmFsdWUgPSBwcm9wLmRlZmF1bHRWYWx1ZSgpO1xuICAgICAgaWYgKGRlZmF1bHRWYWx1ZSkge1xuICAgICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJ0ZXh0XCIpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gXCIke2RlZmF1bHRWYWx1ZX1cIi50b19zdHJpbmcoKTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJlbnVtXCIpIHtcbiAgICAgICAgICBjb25zdCBlbnVtTmFtZSA9IGAke3Bhc2NhbENhc2UoXG4gICAgICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSxcbiAgICAgICAgICApfSR7cGFzY2FsQ2FzZShwcm9wLm5hbWUpfWA7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LnNldF8ke3ZhcmlhYmxlTmFtZX0oY3JhdGU6OnByb3RvYnVmOjoke2VudW1OYW1lfTo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgICBkZWZhdWx0VmFsdWUgYXMgc3RyaW5nLFxuICAgICAgICAgICAgKX0pO2AsXG4gICAgICAgICAgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsQ3JlYXRlQWRkVG9UZW5hbmN5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJiaWxsaW5nQWNjb3VudFwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uXCJcbiAgICApIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25TZXJ2aWNlXCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9pZCk7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJjb21wb25lbnRPYmplY3RcIikge1xuICAgICAgcmVzdWx0LnB1c2goYHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKFwiZ2xvYmFsXCIpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5pbnRlZ3JhdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvbklkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGludGVncmF0aW9uX2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25fc2VydmljZV9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25fc2VydmljZV9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5pbnRlZ3JhdGlvblNlcnZpY2VJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9zZXJ2aWNlX2lkKTtgKTtcbiAgICB9IGVsc2UgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ1c2VyXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiZ3JvdXBcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJvcmdhbml6YXRpb25cIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvbkluc3RhbmNlXCJcbiAgICApIHtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBiaWxsaW5nX2FjY291bnRfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmJpbGxpbmdfYWNjb3VudF9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5iaWxsaW5nQWNjb3VudElkXCIuaW50bygpKSxcbiAgICAgICAgKT87XG4gICAgICAgIHNpX3N0b3JhYmxlLmFkZF90b190ZW5hbnRfaWRzKGJpbGxpbmdfYWNjb3VudF9pZCk7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcIndvcmtzcGFjZVwiKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5vcmdhbml6YXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhvcmdhbml6YXRpb25faWQpO2ApO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBvcmdhbml6YXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLm9yZ2FuaXphdGlvbl9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy5vcmdhbml6YXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhvcmdhbml6YXRpb25faWQpO2ApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCB3b3Jrc3BhY2VfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLndvcmtzcGFjZV9pZC5hc19yZWYoKS5va19vcl9lbHNlKHx8XG4gICAgICAgICAgICBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllcy53b3Jrc3BhY2VJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyh3b3Jrc3BhY2VfaWQpO2ApO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZUlzTXZjYygpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5tdmNjID09IHRydWUpIHtcbiAgICAgIHJldHVybiBcInRydWVcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiZmFsc2VcIjtcbiAgICB9XG4gIH1cblxuICBzdG9yYWJsZVZhbGlkYXRlRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuICAgICAgICBjb25zdCBwcm9wTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGBpZiBzZWxmLiR7cHJvcE5hbWV9LmxlbigpID09IDAge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5pc19ub25lKCkge1xuICAgICAgICAgICAgIHJldHVybiBFcnIoc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJtaXNzaW5nIHJlcXVpcmVkICR7cHJvcE5hbWV9IHZhbHVlXCIuaW50bygpKSk7XG4gICAgICAgICAgIH1gKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AoXG4gICAgdG9wUHJvcDogUHJvcFByZWx1ZGUuUHJvcE9iamVjdCxcbiAgICBwcmVmaXg6IHN0cmluZyxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHRzID0gWydcInNpU3RvcmFibGUubmF0dXJhbEtleVwiJ107XG4gICAgZm9yIChsZXQgcHJvcCBvZiB0b3BQcm9wLnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLmhpZGRlbikge1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcExpbmspIHtcbiAgICAgICAgcHJvcCA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIHByb3AubmFtZSkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChcbiAgICAgICAgICAgIHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKHByb3AsIGAke3ByZWZpeH0uJHtwcm9wLm5hbWV9YCksXG4gICAgICAgICAgKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgaWYgKHByZWZpeCA9PSBcIlwiKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJvcC5uYW1lfVwiYCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKGBcIiR7cHJlZml4fS4ke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzLmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIHN0b3JhYmxlT3JkZXJCeUZpZWxkc0Z1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IHRoaXMuc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3Qucm9vdFByb3AsXG4gICAgICBcIlwiLFxuICAgICk7XG4gICAgcmV0dXJuIGB2ZWMhWyR7cmVzdWx0c31dXFxuYDtcbiAgfVxuXG4gIHN0b3JhYmxlUmVmZXJlbnRpYWxGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IGZldGNoUHJvcHMgPSBbXTtcbiAgICBjb25zdCByZWZlcmVuY2VWZWMgPSBbXTtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdCkge1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0KSB7XG4gICAgICBsZXQgc2lQcm9wZXJ0aWVzID0gdGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmdldEVudHJ5KFwic2lQcm9wZXJ0aWVzXCIpO1xuICAgICAgaWYgKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHNpUHJvcGVydGllcyA9IHNpUHJvcGVydGllcy5sb29rdXBNeXNlbGYoKTtcbiAgICAgIH1cbiAgICAgIGlmICghKHNpUHJvcGVydGllcyBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpKSB7XG4gICAgICAgIHRocm93IFwiQ2Fubm90IGdldCBwcm9wZXJ0aWVzIG9mIGEgbm9uIG9iamVjdCBpbiByZWYgY2hlY2tcIjtcbiAgICAgIH1cbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBzaVByb3BlcnRpZXMucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBpZiAocHJvcC5yZWZlcmVuY2UpIHtcbiAgICAgICAgICBjb25zdCBpdGVtTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc01hbnkoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNPbmUoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEJhc2VPYmplY3QpIHtcbiAgICB9XG5cbiAgICBpZiAoZmV0Y2hQcm9wcy5sZW5ndGggJiYgcmVmZXJlbmNlVmVjLmxlbmd0aCkge1xuICAgICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgICAgcmVzdWx0cy5wdXNoKGZldGNoUHJvcHMuam9pbihcIlxcblwiKSk7XG4gICAgICByZXN1bHRzLnB1c2goYHZlYyFbJHtyZWZlcmVuY2VWZWMuam9pbihcIixcIil9XWApO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiVmVjOjpuZXcoKVwiO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlclNlcnZpY2Uge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzeXN0ZW1PYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzXG4gICAgICAuc29ydCgoYSwgYikgPT4gKGEudHlwZU5hbWUgPiBiLnR5cGVOYW1lID8gMSA6IC0xKSlcbiAgICAgIC5tYXAobyA9PiBuZXcgUnVzdEZvcm1hdHRlcihvKSk7XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdEJvZHkoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXCJkYjogc2lfZGF0YTo6RGIsXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnQsXCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsU2VydmljZU5ld0NvbnN0cnVjdG9yQXJncygpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJldHVybiBcImRiOiBzaV9kYXRhOjpEYiwgYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnRcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiXCI7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VTdHJ1Y3RDb25zdHJ1Y3RvclJldHVybigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnRcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIixcIik7XG4gIH1cblxuICBpbXBsU2VydmljZVRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3NuYWtlQ2FzZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKX1fc2VydmVyOjoke3Bhc2NhbENhc2UodGhpcy5zZXJ2aWNlTmFtZSl9YDtcbiAgfVxuXG4gIGltcGxTZXJ2ZXJOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuaW1wbFNlcnZpY2VUcmFpdE5hbWUoKX1TZXJ2ZXJgO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNaWdyYXRlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgZm9yIChjb25zdCBzeXN0ZW1PYmogb2YgdGhpcy5zeXN0ZW1PYmplY3RzKSB7XG4gICAgICBpZiAodGhpcy5pc01pZ3JhdGVhYmxlKHN5c3RlbU9iaikpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgc3lzdGVtT2JqLnR5cGVOYW1lLFxuICAgICAgICAgICl9OjptaWdyYXRlKCZzZWxmLmRiKS5hd2FpdD87YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaGFzRW50aXRpZXMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0cy5zb21lKG9iaiA9PiBvYmogaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpO1xuICB9XG5cbiAgaXNNaWdyYXRlYWJsZShwcm9wOiBPYmplY3RUeXBlcyk6IGJvb2xlYW4ge1xuICAgIHJldHVybiBwcm9wIGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0ICYmIHByb3AubWlncmF0ZWFibGU7XG4gIH1cblxuICBoYXNNaWdyYXRhYmxlcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzLnNvbWUob2JqID0+IHRoaXMuaXNNaWdyYXRlYWJsZShvYmopKTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlckFnZW50IHtcbiAgYWdlbnROYW1lOiBzdHJpbmc7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBlbnRpdHlGb3JtYXR0ZXI6IFJ1c3RGb3JtYXR0ZXI7XG4gIGludGVncmF0aW9uTmFtZTogc3RyaW5nO1xuICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZywgYWdlbnQ6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlKSB7XG4gICAgdGhpcy5hZ2VudE5hbWUgPSBhZ2VudC5hZ2VudE5hbWU7XG4gICAgdGhpcy5lbnRpdHkgPSBhZ2VudC5lbnRpdHk7XG4gICAgdGhpcy5lbnRpdHlGb3JtYXR0ZXIgPSBuZXcgUnVzdEZvcm1hdHRlcih0aGlzLmVudGl0eSk7XG4gICAgdGhpcy5pbnRlZ3JhdGlvbk5hbWUgPSBhZ2VudC5pbnRlZ3JhdGlvbk5hbWU7XG4gICAgdGhpcy5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lID0gYWdlbnQuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzXG4gICAgICAuc29ydCgoYSwgYikgPT4gKGEudHlwZU5hbWUgPiBiLnR5cGVOYW1lID8gMSA6IC0xKSlcbiAgICAgIC5tYXAobyA9PiBuZXcgUnVzdEZvcm1hdHRlcihvKSk7XG4gIH1cblxuICBhY3Rpb25Qcm9wcygpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eS5tZXRob2RzLmF0dHJzLmZpbHRlcihcbiAgICAgIG0gPT4gbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW107XG4gIH1cblxuICBlbnRpdHlBY3Rpb25NZXRob2ROYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcImNyZWF0ZVwiXTtcblxuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLmFjdGlvblByb3BzKCkpIHtcbiAgICAgIGlmICh0aGlzLmVudGl0eUZvcm1hdHRlci5pc0VudGl0eUVkaXRNZXRob2QocHJvcCkpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuZW50aXR5Rm9ybWF0dGVyLmVudGl0eUVkaXRNZXRob2ROYW1lKHByb3ApKTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdHMucHVzaChwcm9wLm5hbWUpO1xuICAgICAgfVxuICAgIH1cblxuICAgIHJldHVybiByZXN1bHRzO1xuICB9XG5cbiAgZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMuaW50ZWdyYXRpb25OYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICB0aGlzLmludGVncmF0aW9uU2VydmljZU5hbWUsXG4gICAgKX0ke3Bhc2NhbENhc2UodGhpcy5lbnRpdHkuYmFzZVR5cGVOYW1lKX1gO1xuICB9XG5cbiAgZGlzcGF0Y2hlclR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpfURpc3BhdGNoZXJgO1xuICB9XG5cbiAgZGlzcGF0Y2hGdW5jdGlvblRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHt0aGlzLmRpc3BhdGNoZXJCYXNlVHlwZU5hbWUoKX1EaXNwYXRjaEZ1bmN0aW9uc2A7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIENvZGVnZW5SdXN0IHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICB9XG5cbiAgaGFzTW9kZWxzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiByZWdpc3RyeVxuICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgLnNvbWUobyA9PiBvLmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIik7XG4gIH1cblxuICBoYXNTZXJ2aWNlTWV0aG9kcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgcmVnaXN0cnlcbiAgICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgICAuZmxhdE1hcChvID0+IG8ubWV0aG9kcy5hdHRycykubGVuZ3RoID4gMFxuICAgICk7XG4gIH1cblxuICBoYXNFbnRpdHlJbnRlZ3JhdGlvblNlcnZjaWNlcygpOiBib29sZWFuIHtcbiAgICBjb25zdCBpbnRlZ3JhdGlvblNlcnZpY2VzID0gbmV3IFNldChcbiAgICAgIHRoaXMuZW50aXRpZXMoKS5mbGF0TWFwKGVudGl0eSA9PlxuICAgICAgICB0aGlzLmVudGl0eWludGVncmF0aW9uU2VydmljZXNGb3IoZW50aXR5KSxcbiAgICAgICksXG4gICAgKTtcbiAgICByZXR1cm4gaW50ZWdyYXRpb25TZXJ2aWNlcy5zaXplID4gMDtcbiAgfVxuXG4gIGVudGl0aWVzKCk6IEVudGl0eU9iamVjdFtdIHtcbiAgICByZXR1cm4gcmVnaXN0cnlcbiAgICAgIC5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUodGhpcy5zZXJ2aWNlTmFtZSlcbiAgICAgIC5maWx0ZXIobyA9PiBvIGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSBhcyBFbnRpdHlPYmplY3RbXTtcbiAgfVxuXG4gIGVudGl0eUFjdGlvbnMoZW50aXR5OiBFbnRpdHlPYmplY3QpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiBlbnRpdHkubWV0aG9kcy5hdHRycy5maWx0ZXIoXG4gICAgICBtID0+IG0gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdO1xuICB9XG5cbiAgZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHk6IEVudGl0eU9iamVjdCk6IEludGVncmF0aW9uU2VydmljZVtdIHtcbiAgICBjb25zdCByZXN1bHQ6IFNldDxJbnRlZ3JhdGlvblNlcnZpY2U+ID0gbmV3IFNldCgpO1xuICAgIGZvciAoY29uc3QgaW50ZWdyYXRpb25TZXJ2aWNlIG9mIGVudGl0eS5pbnRlZ3JhdGlvblNlcnZpY2VzKSB7XG4gICAgICByZXN1bHQuYWRkKGludGVncmF0aW9uU2VydmljZSk7XG4gICAgfVxuICAgIGZvciAoY29uc3QgYWN0aW9uIG9mIHRoaXMuZW50aXR5QWN0aW9ucyhlbnRpdHkpKSB7XG4gICAgICBmb3IgKGNvbnN0IGludGVncmF0aW9uU2VydmljZSBvZiBhY3Rpb24uaW50ZWdyYXRpb25TZXJ2aWNlcykge1xuICAgICAgICByZXN1bHQuYWRkKGludGVncmF0aW9uU2VydmljZSk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiBBcnJheS5mcm9tKHJlc3VsdCk7XG4gIH1cblxuICBlbnRpdHlJbnRlZ3JhdGlvblNlcnZpY2VzKCk6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0aWVzKCkuZmxhdE1hcChlbnRpdHkgPT5cbiAgICAgIHRoaXMuZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHkpLm1hcChpbnRlZ3JhdGlvblNlcnZpY2UgPT4gKHtcbiAgICAgICAgaW50ZWdyYXRpb25OYW1lOiBpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25OYW1lLFxuICAgICAgICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSxcbiAgICAgICAgZW50aXR5OiBlbnRpdHksXG4gICAgICAgIGFnZW50TmFtZTogYCR7c25ha2VDYXNlKFxuICAgICAgICAgIGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvbk5hbWUsXG4gICAgICAgICl9XyR7c25ha2VDYXNlKGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lKX1fJHtzbmFrZUNhc2UoXG4gICAgICAgICAgZW50aXR5LmJhc2VUeXBlTmFtZSxcbiAgICAgICAgKX1gLFxuICAgICAgfSkpLFxuICAgICk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKFwicHViIG1vZCBhZ2VudDtcIik7XG4gICAgfVxuICAgIGlmICh0aGlzLmhhc01vZGVscygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goXCJwdWIgbW9kIG1vZGVsO1wiKTtcbiAgICB9XG4gICAgaWYgKHRoaXMuaGFzU2VydmljZU1ldGhvZHMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKFwicHViIG1vZCBzZXJ2aWNlO1wiKTtcbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2RlbC9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWxNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iamVjdCBvZiByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoXG4gICAgICB0aGlzLnNlcnZpY2VOYW1lLFxuICAgICkpIHtcbiAgICAgIGlmIChzeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSB7XG4gICAgICAgIHJlc3VsdHMucHVzaChgcHViIG1vZCAke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfTtgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kZWwvbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlblNlcnZpY2UoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3Qvc2VydmljZS5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXJTZXJ2aWNlKHRoaXMuc2VydmljZU5hbWUpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vc2VydmljZS5yc2AsIG91dHB1dCk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9tb2RlbC5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXIoc3lzdGVtT2JqZWN0KSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcbiAgICAgIGBnZW4vbW9kZWwvJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ucnNgLFxuICAgICAgb3V0cHV0LFxuICAgICk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9hZ2VudC9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuQWdlbnRNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBmb3IgKGNvbnN0IGFnZW50IG9mIHRoaXMuZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goYHB1YiBtb2QgJHthZ2VudC5hZ2VudE5hbWV9O2ApO1xuICAgIH1cbiAgICByZXN1bHRzLnB1c2goXCJcIik7XG4gICAgZm9yIChjb25zdCBhZ2VudCBvZiB0aGlzLmVudGl0eUludGVncmF0aW9uU2VydmljZXMoKSkge1xuICAgICAgY29uc3QgZm10ID0gbmV3IFJ1c3RGb3JtYXR0ZXJBZ2VudCh0aGlzLnNlcnZpY2VOYW1lLCBhZ2VudCk7XG4gICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgIGBwdWIgdXNlICR7XG4gICAgICAgICAgYWdlbnQuYWdlbnROYW1lXG4gICAgICAgIH06Onske2ZtdC5kaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lKCl9LCAke2ZtdC5kaXNwYXRjaGVyVHlwZU5hbWUoKX19O2AsXG4gICAgICApO1xuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9hZ2VudC9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuQWdlbnQoYWdlbnQ6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvYWdlbnQucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyQWdlbnQodGhpcy5zZXJ2aWNlTmFtZSwgYWdlbnQpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vYWdlbnQvJHtzbmFrZUNhc2UoYWdlbnQuYWdlbnROYW1lKX0ucnNgLCBvdXRwdXQpO1xuICB9XG5cbiAgLy9hc3luYyBtYWtlUGF0aChwYXRoUGFydDogc3RyaW5nKTogUHJvbWlzZTxzdHJpbmc+IHtcbiAgLy8gIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFwiLi5cIiwgYHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gLCBcInNyY1wiLCBwYXRoUGFydCk7XG4gIC8vICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbiAgLy8gIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKHBhdGgucmVzb2x2ZShwYXRoTmFtZSksIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuICAvLyAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4gIC8vfVxuXG4gIGFzeW5jIGZvcm1hdENvZGUoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgYXdhaXQgZXhlY0NtZChgY2FyZ28gZm10IC1wIHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gKTtcbiAgfVxuXG4gIGFzeW5jIHdyaXRlQ29kZShmaWxlbmFtZTogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBmdWxsUGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4gICAgICBcIi4uXCIsXG4gICAgICBgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWAsXG4gICAgICBcInNyY1wiLFxuICAgICAgZmlsZW5hbWUsXG4gICAgKTtcbiAgICBhd2FpdCBjb2RlRnMud3JpdGVDb2RlKGZ1bGxQYXRoTmFtZSwgY29kZSk7XG4gIH1cbn1cbiJdfQ==