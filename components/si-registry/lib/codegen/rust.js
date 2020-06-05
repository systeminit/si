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
        throw new Error("You ran 'hasEditUpdatesAndEithers()' on a non-entity object; this is a bug!");
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
        throw new Error("You asked for an component name on a non-component object; this is a bug!");
      }
    }
  }, {
    key: "componentConstraintsName",
    value: function componentConstraintsName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "ComponentConstraints");
      } else {
        throw new Error("You asked for a component constraints name on a non-component object; this is a bug!");
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
        throw new Error("You asked for an edit method name on a non-entity object; this is a bug!");
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
        throw new Error("You asked for an entityEvent name on a non-component object; this is a bug!");
      }
    }
  }, {
    key: "entityName",
    value: function entityName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "Entity");
      } else {
        throw new Error("You asked for an entity name on a non-component object; this is a bug!");
      }
    }
  }, {
    key: "entityPropertiesName",
    value: function entityPropertiesName() {
      if (this.systemObject instanceof _systemComponent.ComponentObject || this.systemObject instanceof _systemComponent.EntityObject || this.systemObject instanceof _systemComponent.EntityEventObject) {
        return "crate::protobuf::".concat((0, _changeCase.pascalCase)(this.systemObject.baseTypeName), "EntityProperties");
      } else {
        throw new Error("You asked for an entityProperties name on a non-component object; this is a bug!");
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
              throw new Error("conversion from language '".concat(from.language, "' to type '").concat(to.kind(), "' is not supported"));
            }

          default:
            throw new Error("conversion from language '".concat(from.language, "' is not supported"));
        }
      } else if (from instanceof PropPrelude.PropObject) {
        if (to instanceof PropPrelude.PropCode) {
          switch (to.language) {
            case "yaml":
              return "Ok(serde_yaml::to_string(value)?)";

            default:
              throw new Error("conversion from PropObject to language '".concat(to.language, "' is not supported"));
          }
        } else {
          throw new Error("conversion from PropObject to type '".concat(to.kind(), "' is not supported"));
        }
      } else {
        throw new Error("conversion from type '".concat(from.kind(), "' to type '").concat(to.kind(), "' is not supported"));
      }
    }
  }, {
    key: "implUpdateRequestType",
    value: function implUpdateRequestType() {
      var renderOptions = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : {};
      var list = this.systemObject.methods.getEntry("update");
      var updateProp = list.request.properties.getEntry("update");
      return this.rustTypeForProp(updateProp, renderOptions);
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
    key: "implServiceEntityDelete",
    value: function implServiceEntityDelete(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceEntityDelete.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
        fmt: this,
        propMethod: propMethod
      }, {
        filename: "."
      });
    }
  }, {
    key: "implServiceEntityUpdate",
    value: function implServiceEntityUpdate(propMethod) {
      return _ejs["default"].render("<%- include('src/codegen/rust/implServiceEntityUpdate.rs.ejs', { fmt: fmt, propMethod: propMethod }) %>", {
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
          throw new Error("Cannot get properties of a non object in ref check");
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL3J1c3QudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiUnVzdEZvcm1hdHRlciIsInN5c3RlbU9iamVjdCIsInJlc3VsdHMiLCJraW5kIiwiZW50aXR5IiwicmVnaXN0cnkiLCJnZXQiLCJiYXNlVHlwZU5hbWUiLCJmbXQiLCJhY3Rpb25Qcm9wcyIsInByb3AiLCJpc0VudGl0eUVkaXRNZXRob2QiLCJwdXNoIiwiZW50aXR5RWRpdE1ldGhvZE5hbWUiLCJuYW1lIiwibWV0aG9kcyIsImdldEVudHJ5IiwicHJvcEFjdGlvbiIsImVudGl0eUVkaXRQcm9wZXJ0eSIsInJlbGF0aW9uc2hpcHMiLCJhbGwiLCJzb21lIiwicmVsIiwiUHJvcFByZWx1ZGUiLCJFaXRoZXIiLCJVcGRhdGVzIiwiaXNFbnRpdHlPYmplY3QiLCJlbnRpdHlFZGl0TWV0aG9kcyIsImhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uIiwiRXJyb3IiLCJDb21wb25lbnRPYmplY3QiLCJwcm9wTWV0aG9kIiwiUHJvcEFjdGlvbiIsImlzRW50aXR5QWN0aW9uTWV0aG9kIiwiZW5kc1dpdGgiLCJFbnRpdHlFdmVudE9iamVjdCIsIkVudGl0eU9iamVjdCIsInR5cGVOYW1lIiwiU3lzdGVtT2JqZWN0IiwibWlncmF0ZWFibGUiLCJhdHRycyIsImZpbHRlciIsIm0iLCJjb25zdHJhaW50cyIsImMiLCJQcm9wRW51bSIsIm1hcCIsInJ1c3RGaWVsZE5hbWVGb3JQcm9wIiwicmVwbGFjZSIsInAiLCJwcm9wZXJ0eSIsInJlcXVlc3QiLCJwcm9wZXJ0aWVzIiwiUHJvcExpbmsiLCJsb29rdXBNeXNlbGYiLCJydXN0VHlwZUZvclByb3AiLCJvcHRpb24iLCJyIiwidXBkYXRlIiwiZnJvbSIsInRvIiwicGFydG5lclByb3AiLCJmbGF0TWFwIiwibWV0aG9kIiwiZW50aXR5RWRpdFByb3BlcnR5VXBkYXRlcyIsIkFycmF5IiwiU2V0Iiwic29ydCIsImEiLCJiIiwiTWFwIiwiZmllbGRzIiwicHJvcEVpdGhlcnMiLCJsZW5ndGgiLCJlaXRoZXJzIiwiYWRkIiwiZWl0aGVyc0FycmF5Iiwic2V0IiwiZSIsImpvaW4iLCJlbnRyaWVzIiwidmFsdWVzIiwicHJvcGVydHlVcGRhdGUiLCJzZXJ2aWNlTmFtZSIsIlByb3BDb2RlIiwibGFuZ3VhZ2UiLCJQcm9wT2JqZWN0IiwicmVuZGVyT3B0aW9ucyIsImxpc3QiLCJ1cGRhdGVQcm9wIiwicmVwbHkiLCJyZWZlcmVuY2UiLCJwcm9wRW51bSIsImVqcyIsInJlbmRlciIsImZpbGVuYW1lIiwic2tpcEF1dGgiLCJpbXBsU2VydmljZU1ldGhvZE5hbWUiLCJpbXBsU2VydmljZUF1dGhDYWxsIiwicHJlbHVkZSIsInByb3BNZXRob2RzIiwib3V0cHV0IiwiUHJvcE1ldGhvZCIsInBhcmVudE5hbWUiLCJQcm9wTnVtYmVyIiwibnVtYmVyS2luZCIsIlByb3BCb29sIiwicmVhbFByb3AiLCJwcm9wT3duZXIiLCJsb29rdXBPYmplY3QiLCJwYXRoTmFtZSIsIlByb3BNYXAiLCJQcm9wVGV4dCIsIlByb3BTZWxlY3QiLCJyZXBlYXRlZCIsInZhcmlhbnQiLCJyZXN1bHQiLCJjcmVhdGVNZXRob2QiLCJsaXN0TWV0aG9kIiwiZmllbGROYW1lIiwibGlzdFJlcGx5VmFsdWUiLCJuYXR1cmFsS2V5IiwidmFyaWFibGVOYW1lIiwiUHJvcFBhc3N3b3JkIiwiZGVmYXVsdFZhbHVlIiwiZW51bU5hbWUiLCJtdmNjIiwicmVxdWlyZWQiLCJwcm9wTmFtZSIsInRvcFByb3AiLCJwcmVmaXgiLCJoaWRkZW4iLCJzdG9yYWJsZU9yZGVyQnlGaWVsZHNCeVByb3AiLCJyb290UHJvcCIsImZldGNoUHJvcHMiLCJyZWZlcmVuY2VWZWMiLCJzaVByb3BlcnRpZXMiLCJpdGVtTmFtZSIsIkJhc2VPYmplY3QiLCJSdXN0Rm9ybWF0dGVyU2VydmljZSIsInN5c3RlbU9iamVjdHMiLCJnZXRPYmplY3RzRm9yU2VydmljZU5hbWUiLCJvIiwiaGFzRW50aXRpZXMiLCJpbXBsU2VydmljZVRyYWl0TmFtZSIsInN5c3RlbU9iaiIsImlzTWlncmF0ZWFibGUiLCJvYmoiLCJSdXN0Rm9ybWF0dGVyQWdlbnQiLCJhZ2VudCIsImFnZW50TmFtZSIsImVudGl0eUZvcm1hdHRlciIsImludGVncmF0aW9uTmFtZSIsImludGVncmF0aW9uU2VydmljZU5hbWUiLCJkaXNwYXRjaGVyQmFzZVR5cGVOYW1lIiwiQ29kZWdlblJ1c3QiLCJpbnRlZ3JhdGlvblNlcnZpY2VzIiwiZW50aXRpZXMiLCJlbnRpdHlpbnRlZ3JhdGlvblNlcnZpY2VzRm9yIiwic2l6ZSIsImludGVncmF0aW9uU2VydmljZSIsImVudGl0eUFjdGlvbnMiLCJhY3Rpb24iLCJoYXNFbnRpdHlJbnRlZ3JhdGlvblNlcnZjaWNlcyIsImhhc01vZGVscyIsImhhc1NlcnZpY2VNZXRob2RzIiwid3JpdGVDb2RlIiwiZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcyIsImRpc3BhdGNoRnVuY3Rpb25UcmFpdE5hbWUiLCJkaXNwYXRjaGVyVHlwZU5hbWUiLCJjb2RlIiwiZnVsbFBhdGhOYW1lIiwicGF0aCIsImNvZGVGcyJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0FBUUE7O0FBQ0E7O0FBR0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7Ozs7Ozs7O0FBRUEsSUFBTUEsT0FBTyxHQUFHQyxpQkFBS0MsU0FBTCxDQUFlQywwQkFBYUMsSUFBNUIsQ0FBaEI7O0lBdUJhQyxhO0FBR1gseUJBQVlDLFlBQVosRUFBeUQ7QUFBQTtBQUFBO0FBQ3ZELFNBQUtBLFlBQUwsR0FBb0JBLFlBQXBCO0FBQ0Q7Ozs7OENBRW1DO0FBQ2xDLFVBQU1DLE9BQU8sR0FBRyxDQUFDLFFBQUQsQ0FBaEI7O0FBRUEsVUFBSSxLQUFLRCxZQUFMLENBQWtCRSxJQUFsQixNQUE0QixtQkFBaEMsRUFBcUQ7QUFDbkQ7QUFDQSxZQUFNQyxNQUFNLEdBQUdD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QUFRLEVBQUU7QUFEWixpQkFMYSxDOzt1QkFTVCxLQUFLdUUsU0FBTCxxQkFBNEIsMkJBQVVqQixLQUFLLENBQUNDLFNBQWhCLENBQTVCLFVBQTZEakQsTUFBN0QsQzs7Ozs7Ozs7Ozs7Ozs7O1FBR1I7QUFDQTtBQUNBO0FBQ0E7QUFDQTtBQUNBOzs7Ozs7Ozs7Ozt1QkFHUTFHLE9BQU8sMkJBQW9CLEtBQUt3RixXQUF6QixFOzs7Ozs7Ozs7Ozs7Ozs7Ozs7O3VIQUdDWSxRLEVBQWtCMkUsSTs7Ozs7O0FBQzFCQyxnQkFBQUEsWSxHQUFlQyxpQkFBSzdGLElBQUwsQ0FDbkIsSUFEbUIsZUFFYixLQUFLSSxXQUZRLEdBR25CLEtBSG1CLEVBSW5CWSxRQUptQixDOzt1QkFNZjhFLE1BQU0sQ0FBQ1AsU0FBUCxDQUFpQkssWUFBakIsRUFBK0JELElBQS9CLEMiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQge1xuICBPYmplY3RUeXBlcyxcbiAgQmFzZU9iamVjdCxcbiAgU3lzdGVtT2JqZWN0LFxuICBDb21wb25lbnRPYmplY3QsXG4gIEVudGl0eU9iamVjdCxcbiAgRW50aXR5RXZlbnRPYmplY3QsXG59IGZyb20gXCIuLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCAqIGFzIFByb3BQcmVsdWRlIGZyb20gXCIuLi9jb21wb25lbnRzL3ByZWx1ZGVcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5pbXBvcnQgeyBQcm9wcywgSW50ZWdyYXRpb25TZXJ2aWNlIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5cbmltcG9ydCB7IHNuYWtlQ2FzZSwgcGFzY2FsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IGVqcyBmcm9tIFwiZWpzXCI7XG5pbXBvcnQgcGF0aCBmcm9tIFwicGF0aFwiO1xuaW1wb3J0IGNoaWxkUHJvY2VzcyBmcm9tIFwiY2hpbGRfcHJvY2Vzc1wiO1xuaW1wb3J0IHV0aWwgZnJvbSBcInV0aWxcIjtcbmltcG9ydCAqIGFzIGNvZGVGcyBmcm9tIFwiLi9mc1wiO1xuXG5jb25zdCBleGVjQ21kID0gdXRpbC5wcm9taXNpZnkoY2hpbGRQcm9jZXNzLmV4ZWMpO1xuXG5pbnRlcmZhY2UgUnVzdFR5cGVBc1Byb3BPcHRpb25zIHtcbiAgcmVmZXJlbmNlPzogYm9vbGVhbjtcbiAgb3B0aW9uPzogYm9vbGVhbjtcbn1cblxuaW50ZXJmYWNlIEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlIHtcbiAgYWdlbnROYW1lOiBzdHJpbmc7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBpbnRlZ3JhdGlvbk5hbWU6IHN0cmluZztcbiAgaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTogc3RyaW5nO1xufVxuXG5pbnRlcmZhY2UgUHJvcGVydHlVcGRhdGUge1xuICBmcm9tOiBQcm9wUHJlbHVkZS5Qcm9wcztcbiAgdG86IFByb3BQcmVsdWRlLlByb3BzO1xufVxuXG5pbnRlcmZhY2UgUHJvcGVydHlFaXRoZXJTZXQge1xuICBlbnRyaWVzOiBQcm9wUHJlbHVkZS5Qcm9wc1tdO1xufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlciB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBSdXN0Rm9ybWF0dGVyW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBlbnRpdHlBY3Rpb25NZXRob2ROYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcImNyZWF0ZVwiXTtcblxuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5raW5kKCkgPT0gXCJlbnRpdHlFdmVudE9iamVjdFwiKSB7XG4gICAgICAvLyBAdHMtaWdub3JlXG4gICAgICBjb25zdCBlbnRpdHkgPSByZWdpc3RyeS5nZXQoYCR7dGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lfUVudGl0eWApO1xuICAgICAgY29uc3QgZm10ID0gbmV3IFJ1c3RGb3JtYXR0ZXIoZW50aXR5KTtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBmbXQuYWN0aW9uUHJvcHMoKSkge1xuICAgICAgICBpZiAoZm10LmlzRW50aXR5RWRpdE1ldGhvZChwcm9wKSkge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChmbXQuZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcCkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChwcm9wLm5hbWUpO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfSBlbHNlIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLmFjdGlvblByb3BzKCkpIHtcbiAgICAgICAgaWYgKHRoaXMuaXNFbnRpdHlFZGl0TWV0aG9kKHByb3ApKSB7XG4gICAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcCkpO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChwcm9wLm5hbWUpO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuXG4gICAgcmV0dXJuIHJlc3VsdHM7XG4gIH1cblxuICBoYXNDcmVhdGVNZXRob2QoKTogYm9vbGVhbiB7XG4gICAgdHJ5IHtcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9IGNhdGNoIHtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cblxuICBoYXNFZGl0RWl0aGVyc0ZvckFjdGlvbihwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLnNvbWUocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLkVpdGhlcik7XG4gIH1cblxuICBoYXNFZGl0VXBkYXRlc0ZvckFjdGlvbihwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pXG4gICAgICAucmVsYXRpb25zaGlwcy5hbGwoKVxuICAgICAgLnNvbWUocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlVwZGF0ZXMpO1xuICB9XG5cbiAgaGFzRWRpdFVwZGF0ZXNBbmRFaXRoZXJzKCk6IGJvb2xlYW4ge1xuICAgIGlmICh0aGlzLmlzRW50aXR5T2JqZWN0KCkpIHtcbiAgICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRNZXRob2RzKCkuc29tZShcbiAgICAgICAgcHJvcEFjdGlvbiA9PlxuICAgICAgICAgIHRoaXMuaGFzRWRpdFVwZGF0ZXNGb3JBY3Rpb24ocHJvcEFjdGlvbikgJiZcbiAgICAgICAgICB0aGlzLmhhc0VkaXRVcGRhdGVzRm9yQWN0aW9uKHByb3BBY3Rpb24pLFxuICAgICAgKTtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBcIllvdSByYW4gJ2hhc0VkaXRVcGRhdGVzQW5kRWl0aGVycygpJyBvbiBhIG5vbi1lbnRpdHkgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiLFxuICAgICAgKTtcbiAgICB9XG4gIH1cblxuICBpc0NvbXBvbmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3Q7XG4gIH1cblxuICBpc0VudGl0eUFjdGlvbk1ldGhvZChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIChcbiAgICAgIHRoaXMuaXNFbnRpdHlPYmplY3QoKSAmJiBwcm9wTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEFjdGlvblxuICAgICk7XG4gIH1cblxuICBpc0VudGl0eUVkaXRNZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiAoXG4gICAgICB0aGlzLmlzRW50aXR5QWN0aW9uTWV0aG9kKHByb3BNZXRob2QpICYmIHByb3BNZXRob2QubmFtZS5lbmRzV2l0aChcIkVkaXRcIilcbiAgICApO1xuICB9XG5cbiAgaXNFbnRpdHlFdmVudE9iamVjdCgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdDtcbiAgfVxuXG4gIGlzRW50aXR5T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdDtcbiAgfVxuXG4gIGlzQ2hhbmdlU2V0T2JqZWN0KCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImNoYW5nZVNldFwiO1xuICB9XG5cbiAgaXNNaWdyYXRlYWJsZSgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3QgJiYgdGhpcy5zeXN0ZW1PYmplY3QubWlncmF0ZWFibGVcbiAgICApO1xuICB9XG5cbiAgaXNTdG9yYWJsZSgpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBTeXN0ZW1PYmplY3Q7XG4gIH1cblxuICBhY3Rpb25Qcm9wcygpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmF0dHJzLmZpbHRlcihcbiAgICAgIG0gPT4gbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW107XG4gIH1cblxuICBjb21wb25lbnROYW1lKCk6IHN0cmluZyB7XG4gICAgaWYgKFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBDb21wb25lbnRPYmplY3QgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eUV2ZW50T2JqZWN0XG4gICAgKSB7XG4gICAgICByZXR1cm4gYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICB0aGlzLnN5c3RlbU9iamVjdC5iYXNlVHlwZU5hbWUsXG4gICAgICApfUNvbXBvbmVudGA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgXCJZb3UgYXNrZWQgZm9yIGFuIGNvbXBvbmVudCBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCIsXG4gICAgICApO1xuICAgIH1cbiAgfVxuXG4gIGNvbXBvbmVudENvbnN0cmFpbnRzTmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1Db21wb25lbnRDb25zdHJhaW50c2A7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgXCJZb3UgYXNrZWQgZm9yIGEgY29tcG9uZW50IGNvbnN0cmFpbnRzIG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIixcbiAgICAgICk7XG4gICAgfVxuICB9XG5cbiAgY29tcG9uZW50Q29udHJhaW50c0VudW1zKCk6IFByb3BQcmVsdWRlLlByb3BFbnVtW10ge1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCkge1xuICAgICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0LmNvbnN0cmFpbnRzLmF0dHJzXG4gICAgICAgIC5maWx0ZXIoYyA9PiBjIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcEVudW0pXG4gICAgICAgIC5tYXAoYyA9PiBjIGFzIFByb3BQcmVsdWRlLlByb3BFbnVtKTtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBcIllvdSBhc2tlZCBmb3IgY29tcG9uZW50IGNvbnRyYWludHMgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIixcbiAgICAgICk7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5RWRpdE1ldGhvZE5hbWUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSB7XG4gICAgICByZXR1cm4gYGVkaXRfJHt0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BNZXRob2QpLnJlcGxhY2UoXG4gICAgICAgIFwiX2VkaXRcIixcbiAgICAgICAgXCJcIixcbiAgICAgICl9YDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBcIllvdSBhc2tlZCBmb3IgYW4gZWRpdCBtZXRob2QgbmFtZSBvbiBhIG5vbi1lbnRpdHkgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiLFxuICAgICAgKTtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlFZGl0TWV0aG9kcygpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiB0aGlzLmFjdGlvblByb3BzKCkuZmlsdGVyKHAgPT4gdGhpcy5pc0VudGl0eUVkaXRNZXRob2QocCkpO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24pOiBQcm9wcyB7XG4gICAgbGV0IHByb3BlcnR5ID0gcHJvcEFjdGlvbi5yZXF1ZXN0LnByb3BlcnRpZXMuZ2V0RW50cnkoXCJwcm9wZXJ0eVwiKTtcbiAgICBpZiAocHJvcGVydHkgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgcHJvcGVydHkgPSBwcm9wZXJ0eS5sb29rdXBNeXNlbGYoKTtcbiAgICB9XG4gICAgcmV0dXJuIHByb3BlcnR5O1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5RmllbGQocHJvcEFjdGlvbjogUHJvcFByZWx1ZGUuUHJvcEFjdGlvbik6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdEZpZWxkTmFtZUZvclByb3AodGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbikpO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5VHlwZShwcm9wQWN0aW9uOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AodGhpcy5lbnRpdHlFZGl0UHJvcGVydHkocHJvcEFjdGlvbiksIHtcbiAgICAgIG9wdGlvbjogZmFsc2UsXG4gICAgfSk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzKFxuICAgIHByb3BBY3Rpb246IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IFByb3BlcnR5VXBkYXRlW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eUVkaXRQcm9wZXJ0eShwcm9wQWN0aW9uKVxuICAgICAgLnJlbGF0aW9uc2hpcHMuYWxsKClcbiAgICAgIC5maWx0ZXIociA9PiByIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuVXBkYXRlcylcbiAgICAgIC5tYXAodXBkYXRlID0+ICh7XG4gICAgICAgIGZyb206IHRoaXMuZW50aXR5RWRpdFByb3BlcnR5KHByb3BBY3Rpb24pLFxuICAgICAgICB0bzogdXBkYXRlLnBhcnRuZXJQcm9wKCksXG4gICAgICB9KSk7XG4gIH1cblxuICBhbGxFbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVzKCk6IFByb3BlcnR5VXBkYXRlW10ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSB0aGlzLmVudGl0eUVkaXRNZXRob2RzKCkuZmxhdE1hcChtZXRob2QgPT5cbiAgICAgIHRoaXMuZW50aXR5RWRpdFByb3BlcnR5VXBkYXRlcyhtZXRob2QpLFxuICAgICk7XG5cbiAgICByZXR1cm4gQXJyYXkuZnJvbShuZXcgU2V0KHJlc3VsdHMpKS5zb3J0KChhLCBiKSA9PlxuICAgICAgYCR7YS5mcm9tLm5hbWV9LCR7YS50by5uYW1lfWAgPiBgJHtiLmZyb20ubmFtZX0sJHtiLnRvLm5hbWV9YCA/IDEgOiAtMSxcbiAgICApO1xuICB9XG5cbiAgZW50aXR5RWRpdFByb3BlcnR5RWl0aGVycygpOiBQcm9wZXJ0eUVpdGhlclNldFtdIHtcbiAgICBjb25zdCByZXN1bHRzID0gbmV3IE1hcCgpO1xuICAgIGNvbnN0IHByb3BlcnRpZXMgPSAodGhpcy5zeXN0ZW1PYmplY3QuZmllbGRzLmdldEVudHJ5KFxuICAgICAgXCJwcm9wZXJ0aWVzXCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KS5wcm9wZXJ0aWVzLmF0dHJzO1xuXG4gICAgZm9yIChjb25zdCBwcm9wZXJ0eSBvZiBwcm9wZXJ0aWVzKSB7XG4gICAgICBjb25zdCBwcm9wRWl0aGVycyA9IHByb3BlcnR5LnJlbGF0aW9uc2hpcHNcbiAgICAgICAgLmFsbCgpXG4gICAgICAgIC5maWx0ZXIocmVsID0+IHJlbCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLkVpdGhlcik7XG5cbiAgICAgIGlmIChwcm9wRWl0aGVycy5sZW5ndGggPiAwKSB7XG4gICAgICAgIGNvbnN0IGVpdGhlcnMgPSBuZXcgU2V0PFByb3BQcmVsdWRlLlByb3BzPigpO1xuICAgICAgICBlaXRoZXJzLmFkZChwcm9wZXJ0eSk7XG4gICAgICAgIGZvciAoY29uc3QgcHJvcGVydHkgb2YgcHJvcEVpdGhlcnMpIHtcbiAgICAgICAgICBlaXRoZXJzLmFkZChwcm9wZXJ0eS5wYXJ0bmVyUHJvcCgpKTtcbiAgICAgICAgfVxuXG4gICAgICAgIGNvbnN0IGVpdGhlcnNBcnJheSA9IEFycmF5LmZyb20oZWl0aGVycykuc29ydCgoYSwgYikgPT5cbiAgICAgICAgICBhLm5hbWUgPiBiLm5hbWUgPyAxIDogLTEsXG4gICAgICAgICk7XG4gICAgICAgIHJlc3VsdHMuc2V0KGVpdGhlcnNBcnJheS5tYXAoZSA9PiBlLm5hbWUpLmpvaW4oXCIsXCIpLCB7XG4gICAgICAgICAgZW50cmllczogZWl0aGVyc0FycmF5LFxuICAgICAgICB9KTtcbiAgICAgIH1cbiAgICB9XG5cbiAgICByZXR1cm4gQXJyYXkuZnJvbShyZXN1bHRzLnZhbHVlcygpKS5zb3J0KCk7XG4gIH1cblxuICBlbnRpdHlFZGl0UHJvcGVydHlVcGRhdGVNZXRob2ROYW1lKHByb3BlcnR5VXBkYXRlOiBQcm9wZXJ0eVVwZGF0ZSk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGB1cGRhdGVfJHt0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKFxuICAgICAgcHJvcGVydHlVcGRhdGUudG8sXG4gICAgKX1fZnJvbV8ke3RoaXMucnVzdEZpZWxkTmFtZUZvclByb3AocHJvcGVydHlVcGRhdGUuZnJvbSl9YDtcbiAgfVxuXG4gIGVudGl0eUV2ZW50TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlFdmVudGA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgXCJZb3UgYXNrZWQgZm9yIGFuIGVudGl0eUV2ZW50IG5hbWUgb24gYSBub24tY29tcG9uZW50IG9iamVjdDsgdGhpcyBpcyBhIGJ1ZyFcIixcbiAgICAgICk7XG4gICAgfVxuICB9XG5cbiAgZW50aXR5TmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlgO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoXG4gICAgICAgIFwiWW91IGFza2VkIGZvciBhbiBlbnRpdHkgbmFtZSBvbiBhIG5vbi1jb21wb25lbnQgb2JqZWN0OyB0aGlzIGlzIGEgYnVnIVwiLFxuICAgICAgKTtcbiAgICB9XG4gIH1cblxuICBlbnRpdHlQcm9wZXJ0aWVzTmFtZSgpOiBzdHJpbmcge1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgQ29tcG9uZW50T2JqZWN0IHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEVudGl0eU9iamVjdCB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QgaW5zdGFuY2VvZiBFbnRpdHlFdmVudE9iamVjdFxuICAgICkge1xuICAgICAgcmV0dXJuIGBjcmF0ZTo6cHJvdG9idWY6OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QuYmFzZVR5cGVOYW1lLFxuICAgICAgKX1FbnRpdHlQcm9wZXJ0aWVzYDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBcIllvdSBhc2tlZCBmb3IgYW4gZW50aXR5UHJvcGVydGllcyBuYW1lIG9uIGEgbm9uLWNvbXBvbmVudCBvYmplY3Q7IHRoaXMgaXMgYSBidWchXCIsXG4gICAgICApO1xuICAgIH1cbiAgfVxuXG4gIGVycm9yVHlwZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OmVycm9yOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3Quc2VydmljZU5hbWUpfUVycm9yYDtcbiAgfVxuXG4gIG1vZGVsTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6Om1vZGVsOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICBtb2RlbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QgfCBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICApOiBzdHJpbmcge1xuICAgIHJldHVybiB0aGlzLnJ1c3RGaWVsZE5hbWVGb3JQcm9wKHByb3BNZXRob2QpO1xuICB9XG5cbiAgc3RydWN0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfWA7XG4gIH1cblxuICB0eXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpO1xuICB9XG5cbiAgaW1wbFRyeUZyb21Gb3JQcm9wZXJ0eVVwZGF0ZShwcm9wZXJ0eVVwZGF0ZTogUHJvcGVydHlVcGRhdGUpOiBzdHJpbmcge1xuICAgIGNvbnN0IGZyb20gPSBwcm9wZXJ0eVVwZGF0ZS5mcm9tO1xuICAgIGNvbnN0IHRvID0gcHJvcGVydHlVcGRhdGUudG87XG5cbiAgICAvLyBFdmVyeSBmYWxsdGhyb3VnaC9kZWZhdWx0L2Vsc2UgbmVlZHMgYSBgdGhyb3dgIGNsYXVzZSB0byBsb3VkbHkgcHJvY2xhaW1cbiAgICAvLyB0aGF0IGEgc3BlY2lmaWMgY29udmVyc2lvbiBpcyBub3Qgc3VwcG9ydGVkLiBUaGlzIGFsbG93cyB1cyB0byBhZGRcbiAgICAvLyBjb252ZXJzaW9ucyBhcyB3ZSBnbyB3aXRob3V0IHJvZ3VlIGFuZCB1bmV4cGxhaW5lZCBlcnJvcnMuIEluIHNob3J0LFxuICAgIC8vIHRyZWF0IHRoaXMgbGlrZSBSdXN0IGNvZGUgd2l0aCBmdWxseSBzYXRpc2ZpZWQgbWF0Y2ggYXJtcy4gVGhhbmsgeW91LFxuICAgIC8vIGxvdmUsIHVzLlxuICAgIGlmIChmcm9tIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcENvZGUpIHtcbiAgICAgIHN3aXRjaCAoZnJvbS5sYW5ndWFnZSkge1xuICAgICAgICBjYXNlIFwieWFtbFwiOlxuICAgICAgICAgIGlmICh0byBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BPYmplY3QpIHtcbiAgICAgICAgICAgIHJldHVybiBgT2soc2VyZGVfeWFtbDo6ZnJvbV9zdHIodmFsdWUpPylgO1xuICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXG4gICAgICAgICAgICAgIGBjb252ZXJzaW9uIGZyb20gbGFuZ3VhZ2UgJyR7XG4gICAgICAgICAgICAgICAgZnJvbS5sYW5ndWFnZVxuICAgICAgICAgICAgICB9JyB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9XG4gICAgICAgIGRlZmF1bHQ6XG4gICAgICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICAgICAgYGNvbnZlcnNpb24gZnJvbSBsYW5ndWFnZSAnJHtmcm9tLmxhbmd1YWdlfScgaXMgbm90IHN1cHBvcnRlZGAsXG4gICAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKGZyb20gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICBpZiAodG8gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSkge1xuICAgICAgICBzd2l0Y2ggKHRvLmxhbmd1YWdlKSB7XG4gICAgICAgICAgY2FzZSBcInlhbWxcIjpcbiAgICAgICAgICAgIHJldHVybiBgT2soc2VyZGVfeWFtbDo6dG9fc3RyaW5nKHZhbHVlKT8pYDtcbiAgICAgICAgICBkZWZhdWx0OlxuICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICAgICAgICBgY29udmVyc2lvbiBmcm9tIFByb3BPYmplY3QgdG8gbGFuZ3VhZ2UgJyR7dG8ubGFuZ3VhZ2V9JyBpcyBub3Qgc3VwcG9ydGVkYCxcbiAgICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgICBgY29udmVyc2lvbiBmcm9tIFByb3BPYmplY3QgdG8gdHlwZSAnJHt0by5raW5kKCl9JyBpcyBub3Qgc3VwcG9ydGVkYCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBgY29udmVyc2lvbiBmcm9tIHR5cGUgJyR7ZnJvbS5raW5kKCl9JyB0byB0eXBlICcke3RvLmtpbmQoKX0nIGlzIG5vdCBzdXBwb3J0ZWRgLFxuICAgICAgKTtcbiAgICB9XG4gIH1cblxuICBpbXBsVXBkYXRlUmVxdWVzdFR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJ1cGRhdGVcIixcbiAgICApIGFzIFByb3BQcmVsdWRlLlByb3BNZXRob2Q7XG4gICAgY29uc3QgdXBkYXRlUHJvcCA9IGxpc3QucmVxdWVzdC5wcm9wZXJ0aWVzLmdldEVudHJ5KFwidXBkYXRlXCIpO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcCh1cGRhdGVQcm9wLCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxMaXN0UmVxdWVzdFR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcXVlc3QsIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbExpc3RSZXBseVR5cGUocmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30pOiBzdHJpbmcge1xuICAgIGNvbnN0IGxpc3QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgXCJsaXN0XCIsXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kO1xuICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChsaXN0LnJlcGx5LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVxdWVzdFR5cGUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZC5yZXF1ZXN0LCByZW5kZXJPcHRpb25zKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlUmVwbHlUeXBlKFxuICAgIHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QsXG4gICAgcmVuZGVyT3B0aW9uczogUnVzdFR5cGVBc1Byb3BPcHRpb25zID0ge30sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QucmVwbHksIHJlbmRlck9wdGlvbnMpO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VUcmFjZU5hbWUoXG4gICAgcHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCB8IFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuc3lzdGVtT2JqZWN0LnNlcnZpY2VOYW1lfS4ke3NuYWtlQ2FzZShcbiAgICAgIHRoaXMucnVzdFR5cGVGb3JQcm9wKHByb3BNZXRob2QsIHtcbiAgICAgICAgb3B0aW9uOiBmYWxzZSxcbiAgICAgICAgcmVmZXJlbmNlOiBmYWxzZSxcbiAgICAgIH0pLFxuICAgICl9YDtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICBwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kIHwgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbixcbiAgKTogc3RyaW5nIHtcbiAgICByZXR1cm4gc25ha2VDYXNlKFxuICAgICAgdGhpcy5ydXN0VHlwZUZvclByb3AocHJvcE1ldGhvZCwge1xuICAgICAgICBvcHRpb246IGZhbHNlLFxuICAgICAgICByZWZlcmVuY2U6IGZhbHNlLFxuICAgICAgfSksXG4gICAgKTtcbiAgfVxuXG4gIGltcGxQcm90b2J1ZkVudW0ocHJvcEVudW06IFByb3BQcmVsdWRlLlByb3BFbnVtKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFByb3RvYnVmRW51bS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wRW51bTogcHJvcEVudW0gfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wRW51bTogcHJvcEVudW0gfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5QWN0aW9uKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUFjdGlvbi5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlFZGl0KHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eUVkaXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlQ29tbW9uQ3JlYXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUNvbW1vbkNyZWF0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDaGFuZ2VTZXRDcmVhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ2hhbmdlU2V0Q3JlYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUVudGl0eUNyZWF0ZShwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VFbnRpdHlDcmVhdGUucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlRW50aXR5RGVsZXRlKHByb3BNZXRob2Q6IFByb3BQcmVsdWRlLlByb3BNZXRob2QpOiBzdHJpbmcge1xuICAgIHJldHVybiBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9pbXBsU2VydmljZUVudGl0eURlbGV0ZS5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VFbnRpdHlVcGRhdGUocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlRW50aXR5VXBkYXRlLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUdldChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VHZXQucnMuZWpzJywgeyBmbXQ6IGZtdCwgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9KSAlPlwiLFxuICAgICAgeyBmbXQ6IHRoaXMsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSxcbiAgICAgIHsgZmlsZW5hbWU6IFwiLlwiIH0sXG4gICAgKTtcbiAgfVxuXG4gIGltcGxTZXJ2aWNlTGlzdChwcm9wTWV0aG9kOiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKTogc3RyaW5nIHtcbiAgICByZXR1cm4gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvaW1wbFNlcnZpY2VMaXN0LnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUNvbXBvbmVudFBpY2socHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ29tcG9uZW50UGljay5ycy5lanMnLCB7IGZtdDogZm10LCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0pICU+XCIsXG4gICAgICB7IGZtdDogdGhpcywgcHJvcE1ldGhvZDogcHJvcE1ldGhvZCB9LFxuICAgICAgeyBmaWxlbmFtZTogXCIuXCIgfSxcbiAgICApO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VDdXN0b21NZXRob2QocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGVqcy5yZW5kZXIoXG4gICAgICBcIjwlLSBpbmNsdWRlKCdzcmMvY29kZWdlbi9ydXN0L2ltcGxTZXJ2aWNlQ3VzdG9tTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgIHsgZm10OiB0aGlzLCBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kIH0sXG4gICAgICB7IGZpbGVuYW1lOiBcIi5cIiB9LFxuICAgICk7XG4gIH1cblxuICBpbXBsU2VydmljZUF1dGgocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgaWYgKHByb3BNZXRob2Quc2tpcEF1dGgpIHtcbiAgICAgIHJldHVybiBgLy8gQXV0aGVudGljYXRpb24gaXMgc2tpcHBlZCBvbiBcXGAke3RoaXMuaW1wbFNlcnZpY2VNZXRob2ROYW1lKFxuICAgICAgICBwcm9wTWV0aG9kLFxuICAgICAgKX1cXGBcXG5gO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gdGhpcy5pbXBsU2VydmljZUF1dGhDYWxsKHByb3BNZXRob2QpO1xuICAgIH1cbiAgfVxuXG4gIGltcGxTZXJ2aWNlQXV0aENhbGwocHJvcE1ldGhvZDogUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCk6IHN0cmluZyB7XG4gICAgbGV0IHByZWx1ZGUgPSBcInNpX2FjY291bnQ6OmF1dGhvcml6ZVwiO1xuICAgIGlmICh0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZSA9PSBcImFjY291bnRcIikge1xuICAgICAgcHJlbHVkZSA9IFwiY3JhdGU6OmF1dGhvcml6ZVwiO1xuICAgIH1cbiAgICByZXR1cm4gYCR7cHJlbHVkZX06OmF1dGhueigmc2VsZi5kYiwgJnJlcXVlc3QsIFwiJHt0aGlzLmltcGxTZXJ2aWNlTWV0aG9kTmFtZShcbiAgICAgIHByb3BNZXRob2QsXG4gICAgKX1cIikuYXdhaXQ/O2A7XG4gIH1cblxuICBzZXJ2aWNlTWV0aG9kcygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBjb25zdCBwcm9wTWV0aG9kcyA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuYXR0cnMuc29ydCgoYSwgYikgPT5cbiAgICAgIGEubmFtZSA+IGIubmFtZSA/IDEgOiAtMSxcbiAgICApO1xuICAgIGZvciAoY29uc3QgcHJvcE1ldGhvZCBvZiBwcm9wTWV0aG9kcykge1xuICAgICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9zZXJ2aWNlTWV0aG9kLnJzLmVqcycsIHsgZm10OiBmbXQsIHByb3BNZXRob2Q6IHByb3BNZXRob2QgfSkgJT5cIixcbiAgICAgICAge1xuICAgICAgICAgIGZtdDogdGhpcyxcbiAgICAgICAgICBwcm9wTWV0aG9kOiBwcm9wTWV0aG9kLFxuICAgICAgICB9LFxuICAgICAgICB7XG4gICAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgICB9LFxuICAgICAgKTtcbiAgICAgIHJlc3VsdHMucHVzaChvdXRwdXQpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgcnVzdEZpZWxkTmFtZUZvclByb3AocHJvcDogUHJvcHMpOiBzdHJpbmcge1xuICAgIHJldHVybiBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgfVxuXG4gIHJ1c3RUeXBlRm9yUHJvcChcbiAgICBwcm9wOiBQcm9wcyxcbiAgICByZW5kZXJPcHRpb25zOiBSdXN0VHlwZUFzUHJvcE9wdGlvbnMgPSB7fSxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCByZWZlcmVuY2UgPSByZW5kZXJPcHRpb25zLnJlZmVyZW5jZSB8fCBmYWxzZTtcbiAgICBsZXQgb3B0aW9uID0gdHJ1ZTtcbiAgICBpZiAocmVuZGVyT3B0aW9ucy5vcHRpb24gPT09IGZhbHNlKSB7XG4gICAgICBvcHRpb24gPSBmYWxzZTtcbiAgICB9XG5cbiAgICBsZXQgdHlwZU5hbWU6IHN0cmluZztcblxuICAgIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHByb3AubmFtZSl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTnVtYmVyKSB7XG4gICAgICBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50MzJcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTMyXCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQzMlwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1MzJcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwiaW50NjRcIikge1xuICAgICAgICB0eXBlTmFtZSA9IFwiaTY0XCI7XG4gICAgICB9IGVsc2UgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcInVpbnQ2NFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1NjRcIjtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5udW1iZXJLaW5kID09IFwidTEyOFwiKSB7XG4gICAgICAgIHR5cGVOYW1lID0gXCJ1MTI4XCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQm9vbCB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BFbnVtIHx8XG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdFxuICAgICkge1xuICAgICAgdHlwZU5hbWUgPSBgY3JhdGU6OnByb3RvYnVmOjoke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9YDtcbiAgICB9IGVsc2UgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgY29uc3QgcmVhbFByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgaWYgKHJlYWxQcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE9iamVjdCkge1xuICAgICAgICBjb25zdCBwcm9wT3duZXIgPSBwcm9wLmxvb2t1cE9iamVjdCgpO1xuICAgICAgICBsZXQgcGF0aE5hbWU6IHN0cmluZztcbiAgICAgICAgaWYgKFxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSAmJlxuICAgICAgICAgIHByb3BPd25lci5zZXJ2aWNlTmFtZSA9PSB0aGlzLnN5c3RlbU9iamVjdC5zZXJ2aWNlTmFtZVxuICAgICAgICApIHtcbiAgICAgICAgICBwYXRoTmFtZSA9IFwiY3JhdGU6OnByb3RvYnVmXCI7XG4gICAgICAgIH0gZWxzZSBpZiAocHJvcE93bmVyLnNlcnZpY2VOYW1lKSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBgc2lfJHtwcm9wT3duZXIuc2VydmljZU5hbWV9Ojpwcm90b2J1ZmA7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcGF0aE5hbWUgPSBcImNyYXRlOjpwcm90b2J1ZlwiO1xuICAgICAgICB9XG4gICAgICAgIHR5cGVOYW1lID0gYCR7cGF0aE5hbWV9Ojoke3Bhc2NhbENhc2UocmVhbFByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICAgIHJlYWxQcm9wLm5hbWUsXG4gICAgICAgICl9YDtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiB0aGlzLnJ1c3RUeXBlRm9yUHJvcChyZWFsUHJvcCwgcmVuZGVyT3B0aW9ucyk7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1hcCkge1xuICAgICAgdHlwZU5hbWUgPSBgc3RkOjpjb2xsZWN0aW9uczo6SGFzaE1hcDxTdHJpbmcsIFN0cmluZz5gO1xuICAgIH0gZWxzZSBpZiAoXG4gICAgICBwcm9wIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcFRleHQgfHxcbiAgICAgIHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQ29kZSB8fFxuICAgICAgcHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BTZWxlY3RcbiAgICApIHtcbiAgICAgIHR5cGVOYW1lID0gXCJTdHJpbmdcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFwiQWxsIFByb3BzIHR5cGVzIGNvdmVyZWQ7IHRoaXMgY29kZSBpcyB1bnJlYWNoYWJsZSFcIik7XG4gICAgfVxuICAgIGlmIChyZWZlcmVuY2UpIHtcbiAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgaWYgKHR5cGVOYW1lID09IFwiU3RyaW5nXCIpIHtcbiAgICAgICAgdHlwZU5hbWUgPSBcIiZzdHJcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgICAgICB0eXBlTmFtZSA9IGAmJHt0eXBlTmFtZX1gO1xuICAgICAgfVxuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCkge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvIGFzc2lnbiBpdCwgeW91IGp1c3QgY2FudCB0ZWxsXG4gICAgICB0eXBlTmFtZSA9IGBWZWM8JHt0eXBlTmFtZX0+YDtcbiAgICB9IGVsc2Uge1xuICAgICAgaWYgKG9wdGlvbikge1xuICAgICAgICAvLyBAdHMtaWdub3JlIC0gd2UgZG8gYXNzaWduIGl0LCB5b3UganVzdCBjYW50IHRlbGxcbiAgICAgICAgdHlwZU5hbWUgPSBgT3B0aW9uPCR7dHlwZU5hbWV9PmA7XG4gICAgICB9XG4gICAgfVxuICAgIC8vIEB0cy1pZ25vcmUgLSB3ZSBkbyBhc3NpZ24gaXQsIHlvdSBqdXN0IGNhbnQgdGVsbFxuICAgIHJldHVybiB0eXBlTmFtZTtcbiAgfVxuXG4gIHJ1c3ROYW1lRm9yRW51bVZhcmlhbnQodmFyaWFudDogc3RyaW5nKTogc3RyaW5nIHtcbiAgICByZXR1cm4gcGFzY2FsQ2FzZSh2YXJpYW50LnJlcGxhY2UoXCIuXCIsIFwiXCIpKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVOZXdBcmdzKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goYCR7c25ha2VDYXNlKHByb3AubmFtZSl9OiAke3RoaXMucnVzdFR5cGVGb3JQcm9wKHByb3ApfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIsIFwiKTtcbiAgfVxuXG4gIGltcGxDcmVhdGVQYXNzTmV3QXJncygpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGNvbnN0IGNyZWF0ZU1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJjcmVhdGVcIik7XG4gICAgaWYgKGNyZWF0ZU1ldGhvZCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BNZXRob2QpIHtcbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBjcmVhdGVNZXRob2QucmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKHNuYWtlQ2FzZShwcm9wLm5hbWUpKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZExpc3RSZXN1bHRUb1JlcGx5KCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgbGlzdE1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXCJsaXN0XCIpO1xuICAgIGlmIChsaXN0TWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGxpc3RNZXRob2QucmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgbGV0IGxpc3RSZXBseVZhbHVlID0gYFNvbWUob3V0cHV0LiR7ZmllbGROYW1lfSlgO1xuICAgICAgICBpZiAoZmllbGROYW1lID09IFwibmV4dF9wYWdlX3Rva2VuXCIpIHtcbiAgICAgICAgICBsaXN0UmVwbHlWYWx1ZSA9IFwiU29tZShvdXRwdXQucGFnZV90b2tlbilcIjtcbiAgICAgICAgfSBlbHNlIGlmIChmaWVsZE5hbWUgPT0gXCJpdGVtc1wiKSB7XG4gICAgICAgICAgbGlzdFJlcGx5VmFsdWUgPSBgb3V0cHV0LiR7ZmllbGROYW1lfWA7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfTogJHtsaXN0UmVwbHlWYWx1ZX1gKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiLCBcIik7XG4gIH1cblxuICBpbXBsU2VydmljZU1ldGhvZENyZWF0ZURlc3RydWN0dXJlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgY29uc3QgY3JlYXRlTWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcImNyZWF0ZVwiKTtcbiAgICBpZiAoY3JlYXRlTWV0aG9kIGluc3RhbmNlb2YgUHJvcFByZWx1ZGUuUHJvcE1ldGhvZCkge1xuICAgICAgZm9yIChjb25zdCBwcm9wIG9mIGNyZWF0ZU1ldGhvZC5yZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgICAgY29uc3QgZmllbGROYW1lID0gc25ha2VDYXNlKHByb3AubmFtZSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKGBsZXQgJHtmaWVsZE5hbWV9ID0gaW5uZXIuJHtmaWVsZE5hbWV9O2ApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBuYXR1cmFsS2V5KCk6IHN0cmluZyB7XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgICByZXR1cm4gc25ha2VDYXNlKHRoaXMuc3lzdGVtT2JqZWN0Lm5hdHVyYWxLZXkpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJuYW1lXCI7XG4gICAgfVxuICB9XG5cbiAgaW1wbENyZWF0ZVNldFByb3BlcnRpZXMoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXTtcbiAgICBjb25zdCBjcmVhdGVNZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFwiY3JlYXRlXCIpO1xuICAgIGlmIChjcmVhdGVNZXRob2QgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTWV0aG9kKSB7XG4gICAgICBmb3IgKGNvbnN0IHByb3Agb2YgY3JlYXRlTWV0aG9kLnJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCB2YXJpYWJsZU5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wUGFzc3dvcmQpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICAgIGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gU29tZShzaV9kYXRhOjpwYXNzd29yZDo6ZW5jcnlwdF9wYXNzd29yZCgke3ZhcmlhYmxlTmFtZX0pPyk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKGByZXN1bHQuJHt2YXJpYWJsZU5hbWV9ID0gJHt2YXJpYWJsZU5hbWV9O2ApO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLnN5c3RlbU9iamVjdC5maWVsZHMuYXR0cnMpIHtcbiAgICAgIGNvbnN0IHZhcmlhYmxlTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgY29uc3QgZGVmYXVsdFZhbHVlID0gcHJvcC5kZWZhdWx0VmFsdWUoKTtcbiAgICAgIGlmIChkZWZhdWx0VmFsdWUpIHtcbiAgICAgICAgaWYgKHByb3Aua2luZCgpID09IFwidGV4dFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgICBgcmVzdWx0LiR7dmFyaWFibGVOYW1lfSA9IFwiJHtkZWZhdWx0VmFsdWV9XCIudG9fc3RyaW5nKCk7YCxcbiAgICAgICAgICApO1xuICAgICAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwiZW51bVwiKSB7XG4gICAgICAgICAgY29uc3QgZW51bU5hbWUgPSBgJHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUsXG4gICAgICAgICAgKX0ke3Bhc2NhbENhc2UocHJvcC5uYW1lKX1gO1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgICAgYHJlc3VsdC5zZXRfJHt2YXJpYWJsZU5hbWV9KGNyYXRlOjpwcm90b2J1Zjo6JHtlbnVtTmFtZX06OiR7cGFzY2FsQ2FzZShcbiAgICAgICAgICAgICAgZGVmYXVsdFZhbHVlIGFzIHN0cmluZyxcbiAgICAgICAgICAgICl9KTtgLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaW1wbENyZWF0ZUFkZFRvVGVuYW5jeSgpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtdO1xuICAgIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiYmlsbGluZ0FjY291bnRcIiB8fFxuICAgICAgdGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJpbnRlZ3JhdGlvblwiXG4gICAgKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImludGVncmF0aW9uU2VydmljZVwiKSB7XG4gICAgICByZXN1bHQucHVzaChgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoXCJnbG9iYWxcIik7YCk7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgaW50ZWdyYXRpb25faWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmludGVncmF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25faWQpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3Qua2luZCgpID09IFwiY29tcG9uZW50T2JqZWN0XCIpIHtcbiAgICAgIHJlc3VsdC5wdXNoKGBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhcImdsb2JhbFwiKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICBgc2lfcHJvcGVydGllcy5hc19yZWYoKS5va19vcl9lbHNlKHx8IHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzXCIuaW50bygpKSk/O2AsXG4gICAgICApO1xuICAgICAgcmVzdWx0LnB1c2goYGxldCBpbnRlZ3JhdGlvbl9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuaW50ZWdyYXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25JZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhpbnRlZ3JhdGlvbl9pZCk7YCk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGludGVncmF0aW9uX3NlcnZpY2VfaWQgPSBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLnVud3JhcCgpLmludGVncmF0aW9uX3NlcnZpY2VfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuaW50ZWdyYXRpb25TZXJ2aWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoaW50ZWdyYXRpb25fc2VydmljZV9pZCk7YCk7XG4gICAgfSBlbHNlIGlmIChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwidXNlclwiIHx8XG4gICAgICB0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSA9PSBcImdyb3VwXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwib3JnYW5pemF0aW9uXCIgfHxcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lID09IFwiaW50ZWdyYXRpb25JbnN0YW5jZVwiXG4gICAgKSB7XG4gICAgICByZXN1bHQucHVzaChcbiAgICAgICAgYHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkub2tfb3JfZWxzZSh8fCBzaV9kYXRhOjpEYXRhRXJyb3I6OlZhbGlkYXRpb25FcnJvcihcInNpUHJvcGVydGllc1wiLmludG8oKSkpPztgLFxuICAgICAgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgYmlsbGluZ19hY2NvdW50X2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5iaWxsaW5nX2FjY291bnRfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMuYmlsbGluZ0FjY291bnRJZFwiLmludG8oKSksXG4gICAgICAgICk/O1xuICAgICAgICBzaV9zdG9yYWJsZS5hZGRfdG9fdGVuYW50X2lkcyhiaWxsaW5nX2FjY291bnRfaWQpO2ApO1xuICAgIH0gZWxzZSBpZiAodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUgPT0gXCJ3b3Jrc3BhY2VcIikge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgb3JnYW5pemF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5vcmdhbml6YXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMub3JnYW5pemF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMob3JnYW5pemF0aW9uX2lkKTtgKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgIGBzaV9wcm9wZXJ0aWVzLmFzX3JlZigpLm9rX29yX2Vsc2UofHwgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXNcIi5pbnRvKCkpKT87YCxcbiAgICAgICk7XG4gICAgICByZXN1bHQucHVzaChgbGV0IGJpbGxpbmdfYWNjb3VudF9pZCA9IHNpX3Byb3BlcnRpZXMuYXNfcmVmKCkudW53cmFwKCkuYmlsbGluZ19hY2NvdW50X2lkLmFzX3JlZigpLm9rX29yX2Vsc2UofHxcbiAgICAgICAgICAgIHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwic2lQcm9wZXJ0aWVzLmJpbGxpbmdBY2NvdW50SWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMoYmlsbGluZ19hY2NvdW50X2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgb3JnYW5pemF0aW9uX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS5vcmdhbml6YXRpb25faWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMub3JnYW5pemF0aW9uSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMob3JnYW5pemF0aW9uX2lkKTtgKTtcbiAgICAgIHJlc3VsdC5wdXNoKGBsZXQgd29ya3NwYWNlX2lkID0gc2lfcHJvcGVydGllcy5hc19yZWYoKS51bndyYXAoKS53b3Jrc3BhY2VfaWQuYXNfcmVmKCkub2tfb3JfZWxzZSh8fFxuICAgICAgICAgICAgc2lfZGF0YTo6RGF0YUVycm9yOjpWYWxpZGF0aW9uRXJyb3IoXCJzaVByb3BlcnRpZXMud29ya3NwYWNlSWRcIi5pbnRvKCkpLFxuICAgICAgICApPztcbiAgICAgICAgc2lfc3RvcmFibGUuYWRkX3RvX3RlbmFudF9pZHMod29ya3NwYWNlX2lkKTtgKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVJc012Y2MoKTogc3RyaW5nIHtcbiAgICBpZiAodGhpcy5zeXN0ZW1PYmplY3QubXZjYyA9PSB0cnVlKSB7XG4gICAgICByZXR1cm4gXCJ0cnVlXCI7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcImZhbHNlXCI7XG4gICAgfVxuICB9XG5cbiAgc3RvcmFibGVWYWxpZGF0ZUZ1bmN0aW9uKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5hdHRycykge1xuICAgICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbiAgICAgICAgY29uc3QgcHJvcE5hbWUgPSBzbmFrZUNhc2UocHJvcC5uYW1lKTtcbiAgICAgICAgaWYgKHByb3AucmVwZWF0ZWQpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChgaWYgc2VsZi4ke3Byb3BOYW1lfS5sZW4oKSA9PSAwIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goYGlmIHNlbGYuJHtwcm9wTmFtZX0uaXNfbm9uZSgpIHtcbiAgICAgICAgICAgICByZXR1cm4gRXJyKHNpX2RhdGE6OkRhdGFFcnJvcjo6VmFsaWRhdGlvbkVycm9yKFwibWlzc2luZyByZXF1aXJlZCAke3Byb3BOYW1lfSB2YWx1ZVwiLmludG8oKSkpO1xuICAgICAgICAgICB9YCk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgc3RvcmFibGVPcmRlckJ5RmllbGRzQnlQcm9wKFxuICAgIHRvcFByb3A6IFByb3BQcmVsdWRlLlByb3BPYmplY3QsXG4gICAgcHJlZml4OiBzdHJpbmcsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFsnXCJzaVN0b3JhYmxlLm5hdHVyYWxLZXlcIiddO1xuICAgIGZvciAobGV0IHByb3Agb2YgdG9wUHJvcC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4pIHtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9XG4gICAgICBpZiAocHJvcCBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BMaW5rKSB7XG4gICAgICAgIHByb3AgPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgfVxuICAgICAgaWYgKHByb3AgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaCh0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBwcm9wLm5hbWUpKTtcbiAgICAgICAgfSBlbHNlIHtcbiAgICAgICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgICAgICB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChwcm9wLCBgJHtwcmVmaXh9LiR7cHJvcC5uYW1lfWApLFxuICAgICAgICAgICk7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIGlmIChwcmVmaXggPT0gXCJcIikge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3Byb3AubmFtZX1cImApO1xuICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgIHJlc3VsdHMucHVzaChgXCIke3ByZWZpeH0uJHtwcm9wLm5hbWV9XCJgKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0cy5qb2luKFwiLCBcIik7XG4gIH1cblxuICBzdG9yYWJsZU9yZGVyQnlGaWVsZHNGdW5jdGlvbigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdHMgPSB0aGlzLnN0b3JhYmxlT3JkZXJCeUZpZWxkc0J5UHJvcChcbiAgICAgIHRoaXMuc3lzdGVtT2JqZWN0LnJvb3RQcm9wLFxuICAgICAgXCJcIixcbiAgICApO1xuICAgIHJldHVybiBgdmVjIVske3Jlc3VsdHN9XVxcbmA7XG4gIH1cblxuICBzdG9yYWJsZVJlZmVyZW50aWFsRmllbGRzRnVuY3Rpb24oKTogc3RyaW5nIHtcbiAgICBjb25zdCBmZXRjaFByb3BzID0gW107XG4gICAgY29uc3QgcmVmZXJlbmNlVmVjID0gW107XG4gICAgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5RXZlbnRPYmplY3QpIHtcbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIENvbXBvbmVudE9iamVjdCkge1xuICAgICAgbGV0IHNpUHJvcGVydGllcyA9IHRoaXMuc3lzdGVtT2JqZWN0LmZpZWxkcy5nZXRFbnRyeShcInNpUHJvcGVydGllc1wiKTtcbiAgICAgIGlmIChzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wTGluaykge1xuICAgICAgICBzaVByb3BlcnRpZXMgPSBzaVByb3BlcnRpZXMubG9va3VwTXlzZWxmKCk7XG4gICAgICB9XG4gICAgICBpZiAoIShzaVByb3BlcnRpZXMgaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wT2JqZWN0KSkge1xuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJDYW5ub3QgZ2V0IHByb3BlcnRpZXMgb2YgYSBub24gb2JqZWN0IGluIHJlZiBjaGVja1wiKTtcbiAgICAgIH1cbiAgICAgIGZvciAoY29uc3QgcHJvcCBvZiBzaVByb3BlcnRpZXMucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBpZiAocHJvcC5yZWZlcmVuY2UpIHtcbiAgICAgICAgICBjb25zdCBpdGVtTmFtZSA9IHNuYWtlQ2FzZShwcm9wLm5hbWUpO1xuICAgICAgICAgIGlmIChwcm9wLnJlcGVhdGVkKSB7XG4gICAgICAgICAgICBmZXRjaFByb3BzLnB1c2goYGxldCAke2l0ZW1OYW1lfSA9IG1hdGNoICZzZWxmLnNpX3Byb3BlcnRpZXMge1xuICAgICAgICAgICAgICAgICAgICAgICAgICAgU29tZShjaXApID0+IGNpcFxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLiR7aXRlbU5hbWV9XG4gICAgICAgICAgICAgICAgICAgICAgICAgICAuYXNfcmVmKClcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC5tYXAoU3RyaW5nOjphc19yZWYpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAudW53cmFwX29yKFwiTm8gJHtpdGVtTmFtZX0gZm91bmQgZm9yIHJlZmVyZW50aWFsIGludGVncml0eSBjaGVja1wiKSxcbiAgICAgICAgICAgICAgICAgICAgICAgICAgICAgTm9uZSA9PiBcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIixcbiAgICAgICAgICAgICAgICAgICAgICAgICB9O2ApO1xuICAgICAgICAgICAgcmVmZXJlbmNlVmVjLnB1c2goXG4gICAgICAgICAgICAgIGBzaV9kYXRhOjpSZWZlcmVuY2U6Okhhc01hbnkoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9IGVsc2Uge1xuICAgICAgICAgICAgZmV0Y2hQcm9wcy5wdXNoKGBsZXQgJHtpdGVtTmFtZX0gPSBtYXRjaCAmc2VsZi5zaV9wcm9wZXJ0aWVzIHtcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIFNvbWUoY2lwKSA9PiBjaXBcbiAgICAgICAgICAgICAgICAgICAgICAgICAgIC4ke2l0ZW1OYW1lfVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLmFzX3JlZigpXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAubWFwKFN0cmluZzo6YXNfcmVmKVxuICAgICAgICAgICAgICAgICAgICAgICAgICAgLnVud3JhcF9vcihcIk5vICR7aXRlbU5hbWV9IGZvdW5kIGZvciByZWZlcmVudGlhbCBpbnRlZ3JpdHkgY2hlY2tcIiksXG4gICAgICAgICAgICAgICAgICAgICAgICAgICAgIE5vbmUgPT4gXCJObyAke2l0ZW1OYW1lfSBmb3VuZCBmb3IgcmVmZXJlbnRpYWwgaW50ZWdyaXR5IGNoZWNrXCIsXG4gICAgICAgICAgICAgICAgICAgICAgICAgfTtgKTtcbiAgICAgICAgICAgIHJlZmVyZW5jZVZlYy5wdXNoKFxuICAgICAgICAgICAgICBgc2lfZGF0YTo6UmVmZXJlbmNlOjpIYXNPbmUoXCIke2l0ZW1OYW1lfVwiLCAke2l0ZW1OYW1lfSlgLFxuICAgICAgICAgICAgKTtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHRoaXMuc3lzdGVtT2JqZWN0IGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0KSB7XG4gICAgfSBlbHNlIGlmICh0aGlzLnN5c3RlbU9iamVjdCBpbnN0YW5jZW9mIEJhc2VPYmplY3QpIHtcbiAgICB9XG5cbiAgICBpZiAoZmV0Y2hQcm9wcy5sZW5ndGggJiYgcmVmZXJlbmNlVmVjLmxlbmd0aCkge1xuICAgICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgICAgcmVzdWx0cy5wdXNoKGZldGNoUHJvcHMuam9pbihcIlxcblwiKSk7XG4gICAgICByZXN1bHRzLnB1c2goYHZlYyFbJHtyZWZlcmVuY2VWZWMuam9pbihcIixcIil9XWApO1xuICAgICAgcmV0dXJuIHJlc3VsdHMuam9pbihcIlxcblwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiVmVjOjpuZXcoKVwiO1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlclNlcnZpY2Uge1xuICBzZXJ2aWNlTmFtZTogc3RyaW5nO1xuICBzeXN0ZW1PYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKHNlcnZpY2VOYW1lOiBzdHJpbmcpIHtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzXG4gICAgICAuc29ydCgoYSwgYikgPT4gKGEudHlwZU5hbWUgPiBiLnR5cGVOYW1lID8gMSA6IC0xKSlcbiAgICAgIC5tYXAobyA9PiBuZXcgUnVzdEZvcm1hdHRlcihvKSk7XG4gIH1cblxuICBpbXBsU2VydmljZVN0cnVjdEJvZHkoKTogc3RyaW5nIHtcbiAgICBjb25zdCByZXN1bHQgPSBbXCJkYjogc2lfZGF0YTo6RGIsXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnQsXCIpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCJcXG5cIik7XG4gIH1cblxuICBpbXBsU2VydmljZU5ld0NvbnN0cnVjdG9yQXJncygpOiBzdHJpbmcge1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJldHVybiBcImRiOiBzaV9kYXRhOjpEYiwgYWdlbnQ6IHNpX2NlYTo6QWdlbnRDbGllbnRcIjtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiZGI6IHNpX2RhdGE6OkRiXCI7XG4gICAgfVxuICB9XG5cbiAgaW1wbFNlcnZpY2VTdHJ1Y3RDb25zdHJ1Y3RvclJldHVybigpOiBzdHJpbmcge1xuICAgIGNvbnN0IHJlc3VsdCA9IFtcImRiXCJdO1xuICAgIGlmICh0aGlzLmhhc0VudGl0aWVzKCkpIHtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYWdlbnRcIik7XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQuam9pbihcIixcIik7XG4gIH1cblxuICBpbXBsU2VydmljZVRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgY3JhdGU6OnByb3RvYnVmOjoke3NuYWtlQ2FzZShcbiAgICAgIHRoaXMuc2VydmljZU5hbWUsXG4gICAgKX1fc2VydmVyOjoke3Bhc2NhbENhc2UodGhpcy5zZXJ2aWNlTmFtZSl9YDtcbiAgfVxuXG4gIGltcGxTZXJ2ZXJOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuaW1wbFNlcnZpY2VUcmFpdE5hbWUoKX1TZXJ2ZXJgO1xuICB9XG5cbiAgaW1wbFNlcnZpY2VNaWdyYXRlKCk6IHN0cmluZyB7XG4gICAgY29uc3QgcmVzdWx0ID0gW107XG4gICAgZm9yIChjb25zdCBzeXN0ZW1PYmogb2YgdGhpcy5zeXN0ZW1PYmplY3RzKSB7XG4gICAgICBpZiAodGhpcy5pc01pZ3JhdGVhYmxlKHN5c3RlbU9iaikpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgYGNyYXRlOjpwcm90b2J1Zjo6JHtwYXNjYWxDYXNlKFxuICAgICAgICAgICAgc3lzdGVtT2JqLnR5cGVOYW1lLFxuICAgICAgICAgICl9OjptaWdyYXRlKCZzZWxmLmRiKS5hd2FpdD87YCxcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiXFxuXCIpO1xuICB9XG5cbiAgaGFzRW50aXRpZXMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuc3lzdGVtT2JqZWN0cy5zb21lKG9iaiA9PiBvYmogaW5zdGFuY2VvZiBFbnRpdHlPYmplY3QpO1xuICB9XG5cbiAgaXNNaWdyYXRlYWJsZShwcm9wOiBPYmplY3RUeXBlcyk6IGJvb2xlYW4ge1xuICAgIHJldHVybiBwcm9wIGluc3RhbmNlb2YgU3lzdGVtT2JqZWN0ICYmIHByb3AubWlncmF0ZWFibGU7XG4gIH1cblxuICBoYXNNaWdyYXRhYmxlcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzLnNvbWUob2JqID0+IHRoaXMuaXNNaWdyYXRlYWJsZShvYmopKTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUnVzdEZvcm1hdHRlckFnZW50IHtcbiAgYWdlbnROYW1lOiBzdHJpbmc7XG4gIGVudGl0eTogRW50aXR5T2JqZWN0O1xuICBlbnRpdHlGb3JtYXR0ZXI6IFJ1c3RGb3JtYXR0ZXI7XG4gIGludGVncmF0aW9uTmFtZTogc3RyaW5nO1xuICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHNlcnZpY2VOYW1lOiBzdHJpbmc7XG4gIHN5c3RlbU9iamVjdHM6IE9iamVjdFR5cGVzW107XG5cbiAgY29uc3RydWN0b3Ioc2VydmljZU5hbWU6IHN0cmluZywgYWdlbnQ6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlKSB7XG4gICAgdGhpcy5hZ2VudE5hbWUgPSBhZ2VudC5hZ2VudE5hbWU7XG4gICAgdGhpcy5lbnRpdHkgPSBhZ2VudC5lbnRpdHk7XG4gICAgdGhpcy5lbnRpdHlGb3JtYXR0ZXIgPSBuZXcgUnVzdEZvcm1hdHRlcih0aGlzLmVudGl0eSk7XG4gICAgdGhpcy5pbnRlZ3JhdGlvbk5hbWUgPSBhZ2VudC5pbnRlZ3JhdGlvbk5hbWU7XG4gICAgdGhpcy5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lID0gYWdlbnQuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTtcbiAgICB0aGlzLnNlcnZpY2VOYW1lID0gc2VydmljZU5hbWU7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3RzID0gcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKTtcbiAgfVxuXG4gIHN5c3RlbU9iamVjdHNBc0Zvcm1hdHRlcnMoKTogUnVzdEZvcm1hdHRlcltdIHtcbiAgICByZXR1cm4gdGhpcy5zeXN0ZW1PYmplY3RzXG4gICAgICAuc29ydCgoYSwgYikgPT4gKGEudHlwZU5hbWUgPiBiLnR5cGVOYW1lID8gMSA6IC0xKSlcbiAgICAgIC5tYXAobyA9PiBuZXcgUnVzdEZvcm1hdHRlcihvKSk7XG4gIH1cblxuICBhY3Rpb25Qcm9wcygpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0eS5tZXRob2RzLmF0dHJzLmZpbHRlcihcbiAgICAgIG0gPT4gbSBpbnN0YW5jZW9mIFByb3BQcmVsdWRlLlByb3BBY3Rpb24sXG4gICAgKSBhcyBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW107XG4gIH1cblxuICBlbnRpdHlBY3Rpb25NZXRob2ROYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcImNyZWF0ZVwiXTtcblxuICAgIGZvciAoY29uc3QgcHJvcCBvZiB0aGlzLmFjdGlvblByb3BzKCkpIHtcbiAgICAgIGlmICh0aGlzLmVudGl0eUZvcm1hdHRlci5pc0VudGl0eUVkaXRNZXRob2QocHJvcCkpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKHRoaXMuZW50aXR5Rm9ybWF0dGVyLmVudGl0eUVkaXRNZXRob2ROYW1lKHByb3ApKTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdHMucHVzaChwcm9wLm5hbWUpO1xuICAgICAgfVxuICAgIH1cblxuICAgIHJldHVybiByZXN1bHRzO1xuICB9XG5cbiAgZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMuaW50ZWdyYXRpb25OYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICB0aGlzLmludGVncmF0aW9uU2VydmljZU5hbWUsXG4gICAgKX0ke3Bhc2NhbENhc2UodGhpcy5lbnRpdHkuYmFzZVR5cGVOYW1lKX1gO1xuICB9XG5cbiAgZGlzcGF0Y2hlclR5cGVOYW1lKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3RoaXMuZGlzcGF0Y2hlckJhc2VUeXBlTmFtZSgpfURpc3BhdGNoZXJgO1xuICB9XG5cbiAgZGlzcGF0Y2hGdW5jdGlvblRyYWl0TmFtZSgpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHt0aGlzLmRpc3BhdGNoZXJCYXNlVHlwZU5hbWUoKX1EaXNwYXRjaEZ1bmN0aW9uc2A7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIENvZGVnZW5SdXN0IHtcbiAgc2VydmljZU5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihzZXJ2aWNlTmFtZTogc3RyaW5nKSB7XG4gICAgdGhpcy5zZXJ2aWNlTmFtZSA9IHNlcnZpY2VOYW1lO1xuICB9XG5cbiAgaGFzTW9kZWxzKCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiByZWdpc3RyeVxuICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgLnNvbWUobyA9PiBvLmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIik7XG4gIH1cblxuICBoYXNTZXJ2aWNlTWV0aG9kcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gKFxuICAgICAgcmVnaXN0cnlcbiAgICAgICAgLmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSh0aGlzLnNlcnZpY2VOYW1lKVxuICAgICAgICAuZmxhdE1hcChvID0+IG8ubWV0aG9kcy5hdHRycykubGVuZ3RoID4gMFxuICAgICk7XG4gIH1cblxuICBoYXNFbnRpdHlJbnRlZ3JhdGlvblNlcnZjaWNlcygpOiBib29sZWFuIHtcbiAgICBjb25zdCBpbnRlZ3JhdGlvblNlcnZpY2VzID0gbmV3IFNldChcbiAgICAgIHRoaXMuZW50aXRpZXMoKS5mbGF0TWFwKGVudGl0eSA9PlxuICAgICAgICB0aGlzLmVudGl0eWludGVncmF0aW9uU2VydmljZXNGb3IoZW50aXR5KSxcbiAgICAgICksXG4gICAgKTtcbiAgICByZXR1cm4gaW50ZWdyYXRpb25TZXJ2aWNlcy5zaXplID4gMDtcbiAgfVxuXG4gIGVudGl0aWVzKCk6IEVudGl0eU9iamVjdFtdIHtcbiAgICByZXR1cm4gcmVnaXN0cnlcbiAgICAgIC5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUodGhpcy5zZXJ2aWNlTmFtZSlcbiAgICAgIC5maWx0ZXIobyA9PiBvIGluc3RhbmNlb2YgRW50aXR5T2JqZWN0KSBhcyBFbnRpdHlPYmplY3RbXTtcbiAgfVxuXG4gIGVudGl0eUFjdGlvbnMoZW50aXR5OiBFbnRpdHlPYmplY3QpOiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uW10ge1xuICAgIHJldHVybiBlbnRpdHkubWV0aG9kcy5hdHRycy5maWx0ZXIoXG4gICAgICBtID0+IG0gaW5zdGFuY2VvZiBQcm9wUHJlbHVkZS5Qcm9wQWN0aW9uLFxuICAgICkgYXMgUHJvcFByZWx1ZGUuUHJvcEFjdGlvbltdO1xuICB9XG5cbiAgZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHk6IEVudGl0eU9iamVjdCk6IEludGVncmF0aW9uU2VydmljZVtdIHtcbiAgICBjb25zdCByZXN1bHQ6IFNldDxJbnRlZ3JhdGlvblNlcnZpY2U+ID0gbmV3IFNldCgpO1xuICAgIGZvciAoY29uc3QgaW50ZWdyYXRpb25TZXJ2aWNlIG9mIGVudGl0eS5pbnRlZ3JhdGlvblNlcnZpY2VzKSB7XG4gICAgICByZXN1bHQuYWRkKGludGVncmF0aW9uU2VydmljZSk7XG4gICAgfVxuICAgIGZvciAoY29uc3QgYWN0aW9uIG9mIHRoaXMuZW50aXR5QWN0aW9ucyhlbnRpdHkpKSB7XG4gICAgICBmb3IgKGNvbnN0IGludGVncmF0aW9uU2VydmljZSBvZiBhY3Rpb24uaW50ZWdyYXRpb25TZXJ2aWNlcykge1xuICAgICAgICByZXN1bHQuYWRkKGludGVncmF0aW9uU2VydmljZSk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiBBcnJheS5mcm9tKHJlc3VsdCk7XG4gIH1cblxuICBlbnRpdHlJbnRlZ3JhdGlvblNlcnZpY2VzKCk6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlW10ge1xuICAgIHJldHVybiB0aGlzLmVudGl0aWVzKCkuZmxhdE1hcChlbnRpdHkgPT5cbiAgICAgIHRoaXMuZW50aXR5aW50ZWdyYXRpb25TZXJ2aWNlc0ZvcihlbnRpdHkpLm1hcChpbnRlZ3JhdGlvblNlcnZpY2UgPT4gKHtcbiAgICAgICAgaW50ZWdyYXRpb25OYW1lOiBpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25OYW1lLFxuICAgICAgICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBpbnRlZ3JhdGlvblNlcnZpY2UuaW50ZWdyYXRpb25TZXJ2aWNlTmFtZSxcbiAgICAgICAgZW50aXR5OiBlbnRpdHksXG4gICAgICAgIGFnZW50TmFtZTogYCR7c25ha2VDYXNlKFxuICAgICAgICAgIGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvbk5hbWUsXG4gICAgICAgICl9XyR7c25ha2VDYXNlKGludGVncmF0aW9uU2VydmljZS5pbnRlZ3JhdGlvblNlcnZpY2VOYW1lKX1fJHtzbmFrZUNhc2UoXG4gICAgICAgICAgZW50aXR5LmJhc2VUeXBlTmFtZSxcbiAgICAgICAgKX1gLFxuICAgICAgfSkpLFxuICAgICk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kKCk6IFByb21pc2U8dm9pZD4ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXCIvLyBBdXRvLWdlbmVyYXRlZCBjb2RlIVwiLCBcIi8vIE5vIHRvdWNoeSFcIiwgXCJcIl07XG4gICAgaWYgKHRoaXMuaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKFwicHViIG1vZCBhZ2VudDtcIik7XG4gICAgfVxuICAgIGlmICh0aGlzLmhhc01vZGVscygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goXCJwdWIgbW9kIG1vZGVsO1wiKTtcbiAgICB9XG4gICAgaWYgKHRoaXMuaGFzU2VydmljZU1ldGhvZHMoKSkge1xuICAgICAgcmVzdWx0cy5wdXNoKFwicHViIG1vZCBzZXJ2aWNlO1wiKTtcbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9tb2RlbC9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuTW9kZWxNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBmb3IgKGNvbnN0IHN5c3RlbU9iamVjdCBvZiByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoXG4gICAgICB0aGlzLnNlcnZpY2VOYW1lLFxuICAgICkpIHtcbiAgICAgIGlmIChzeXN0ZW1PYmplY3Qua2luZCgpICE9IFwiYmFzZU9iamVjdFwiKSB7XG4gICAgICAgIHJlc3VsdHMucHVzaChgcHViIG1vZCAke3NuYWtlQ2FzZShzeXN0ZW1PYmplY3QudHlwZU5hbWUpfTtgKTtcbiAgICAgIH1cbiAgICB9XG4gICAgYXdhaXQgdGhpcy53cml0ZUNvZGUoXCJnZW4vbW9kZWwvbW9kLnJzXCIsIHJlc3VsdHMuam9pbihcIlxcblwiKSk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlblNlcnZpY2UoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3Qvc2VydmljZS5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXJTZXJ2aWNlKHRoaXMuc2VydmljZU5hbWUpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vc2VydmljZS5yc2AsIG91dHB1dCk7XG4gIH1cblxuICBhc3luYyBnZW5lcmF0ZUdlbk1vZGVsKHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBvdXRwdXQgPSBlanMucmVuZGVyKFxuICAgICAgXCI8JS0gaW5jbHVkZSgnc3JjL2NvZGVnZW4vcnVzdC9tb2RlbC5ycy5lanMnLCB7IGZtdDogZm10IH0pICU+XCIsXG4gICAgICB7XG4gICAgICAgIGZtdDogbmV3IFJ1c3RGb3JtYXR0ZXIoc3lzdGVtT2JqZWN0KSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIGZpbGVuYW1lOiBcIi5cIixcbiAgICAgIH0sXG4gICAgKTtcbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcbiAgICAgIGBnZW4vbW9kZWwvJHtzbmFrZUNhc2Uoc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ucnNgLFxuICAgICAgb3V0cHV0LFxuICAgICk7XG4gIH1cblxuICAvLyBHZW5lcmF0ZSB0aGUgJ2dlbi9hZ2VudC9tb2QucnMnXG4gIGFzeW5jIGdlbmVyYXRlR2VuQWdlbnRNb2QoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtcIi8vIEF1dG8tZ2VuZXJhdGVkIGNvZGUhXCIsIFwiLy8gTm8gdG91Y2h5IVwiLCBcIlwiXTtcbiAgICBmb3IgKGNvbnN0IGFnZW50IG9mIHRoaXMuZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcygpKSB7XG4gICAgICByZXN1bHRzLnB1c2goYHB1YiBtb2QgJHthZ2VudC5hZ2VudE5hbWV9O2ApO1xuICAgIH1cbiAgICByZXN1bHRzLnB1c2goXCJcIik7XG4gICAgZm9yIChjb25zdCBhZ2VudCBvZiB0aGlzLmVudGl0eUludGVncmF0aW9uU2VydmljZXMoKSkge1xuICAgICAgY29uc3QgZm10ID0gbmV3IFJ1c3RGb3JtYXR0ZXJBZ2VudCh0aGlzLnNlcnZpY2VOYW1lLCBhZ2VudCk7XG4gICAgICByZXN1bHRzLnB1c2goXG4gICAgICAgIGBwdWIgdXNlICR7XG4gICAgICAgICAgYWdlbnQuYWdlbnROYW1lXG4gICAgICAgIH06Onske2ZtdC5kaXNwYXRjaEZ1bmN0aW9uVHJhaXROYW1lKCl9LCAke2ZtdC5kaXNwYXRjaGVyVHlwZU5hbWUoKX19O2AsXG4gICAgICApO1xuICAgIH1cbiAgICBhd2FpdCB0aGlzLndyaXRlQ29kZShcImdlbi9hZ2VudC9tb2QucnNcIiwgcmVzdWx0cy5qb2luKFwiXFxuXCIpKTtcbiAgfVxuXG4gIGFzeW5jIGdlbmVyYXRlR2VuQWdlbnQoYWdlbnQ6IEFnZW50SW50ZWdyYXRpb25TZXJ2aWNlKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgY29uc3Qgb3V0cHV0ID0gZWpzLnJlbmRlcihcbiAgICAgIFwiPCUtIGluY2x1ZGUoJ3NyYy9jb2RlZ2VuL3J1c3QvYWdlbnQucnMuZWpzJywgeyBmbXQ6IGZtdCB9KSAlPlwiLFxuICAgICAge1xuICAgICAgICBmbXQ6IG5ldyBSdXN0Rm9ybWF0dGVyQWdlbnQodGhpcy5zZXJ2aWNlTmFtZSwgYWdlbnQpLFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgZmlsZW5hbWU6IFwiLlwiLFxuICAgICAgfSxcbiAgICApO1xuICAgIGF3YWl0IHRoaXMud3JpdGVDb2RlKGBnZW4vYWdlbnQvJHtzbmFrZUNhc2UoYWdlbnQuYWdlbnROYW1lKX0ucnNgLCBvdXRwdXQpO1xuICB9XG5cbiAgLy9hc3luYyBtYWtlUGF0aChwYXRoUGFydDogc3RyaW5nKTogUHJvbWlzZTxzdHJpbmc+IHtcbiAgLy8gIGNvbnN0IHBhdGhOYW1lID0gcGF0aC5qb2luKFwiLi5cIiwgYHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gLCBcInNyY1wiLCBwYXRoUGFydCk7XG4gIC8vICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhOYW1lKTtcbiAgLy8gIGF3YWl0IGZzLnByb21pc2VzLm1rZGlyKHBhdGgucmVzb2x2ZShwYXRoTmFtZSksIHsgcmVjdXJzaXZlOiB0cnVlIH0pO1xuICAvLyAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG4gIC8vfVxuXG4gIGFzeW5jIGZvcm1hdENvZGUoKTogUHJvbWlzZTx2b2lkPiB7XG4gICAgYXdhaXQgZXhlY0NtZChgY2FyZ28gZm10IC1wIHNpLSR7dGhpcy5zZXJ2aWNlTmFtZX1gKTtcbiAgfVxuXG4gIGFzeW5jIHdyaXRlQ29kZShmaWxlbmFtZTogc3RyaW5nLCBjb2RlOiBzdHJpbmcpOiBQcm9taXNlPHZvaWQ+IHtcbiAgICBjb25zdCBmdWxsUGF0aE5hbWUgPSBwYXRoLmpvaW4oXG4gICAgICBcIi4uXCIsXG4gICAgICBgc2ktJHt0aGlzLnNlcnZpY2VOYW1lfWAsXG4gICAgICBcInNyY1wiLFxuICAgICAgZmlsZW5hbWUsXG4gICAgKTtcbiAgICBhd2FpdCBjb2RlRnMud3JpdGVDb2RlKGZ1bGxQYXRoTmFtZSwgY29kZSk7XG4gIH1cbn1cbiJdfQ==