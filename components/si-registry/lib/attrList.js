"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropAction = exports.PropMethod = exports.PropObject = exports.AttrList = void 0;

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _prop = require("./prop");

var _text = require("./prop/text");

var _code = require("./prop/code");

var _number = require("./prop/number");

var _map = require("./prop/map");

var _enum = require("./prop/enum");

var _bool = require("./prop/bool");

var _link = require("./prop/link");

var _password = require("./prop/password");

var _changeCase = require("change-case");

var _registry = require("./registry");

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

var AttrList = /*#__PURE__*/function () {
  function AttrList(_ref) {
    var parentName = _ref.parentName,
        readOnly = _ref.readOnly,
        componentTypeName = _ref.componentTypeName,
        autoCreateEdits = _ref.autoCreateEdits;
    (0, _classCallCheck2["default"])(this, AttrList);
    (0, _defineProperty2["default"])(this, "attrs", void 0);
    (0, _defineProperty2["default"])(this, "readOnly", void 0);
    (0, _defineProperty2["default"])(this, "parentName", void 0);
    (0, _defineProperty2["default"])(this, "autoCreateEdits", void 0);
    (0, _defineProperty2["default"])(this, "componentTypeName", void 0);
    this.parentName = parentName || "";
    this.attrs = [];
    this.componentTypeName = componentTypeName || "";
    this.readOnly = readOnly || false;
    this.autoCreateEdits = autoCreateEdits || false;
  }

  (0, _createClass2["default"])(AttrList, [{
    key: "hasEntries",
    value: function hasEntries() {
      return this.attrs.length > 0;
    }
  }, {
    key: "entries",
    value: function entries() {
      return this.attrs;
    }
  }, {
    key: "getEntry",
    value: function getEntry(name) {
      var result = this.attrs.find(function (e) {
        return e.name == name;
      });

      if (result == undefined) {
        throw "Cannot find property ".concat(name, " for ").concat(this.componentTypeName);
      }

      return result;
    }
  }, {
    key: "createValueObject",
    value: function createValueObject(defaultValues) {
      var resultValues = defaultValues || {};

      var _iterator = _createForOfIteratorHelper(this.entries()),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var item = _step.value;

          if (resultValues[item.name]) {
            continue;
          } else {
            resultValues[item.name] = item.defaultValue();
          }
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      return resultValues;
    }
  }, {
    key: "realValues",
    value: function realValues(values) {
      var resultValues = {};

      var _iterator2 = _createForOfIteratorHelper(this.entries()),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var item = _step2.value;

          if (item.kind() == "code" && item instanceof _code.PropCode) {
            if (values[item.name]) {
              resultValues[item.name] = item.realValue(values[item.name]);
            }
          } else {
            resultValues[item.name] = values[item.name];
          }
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return resultValues;
    }
  }, {
    key: "addExisting",
    value: function addExisting(p) {
      p.reference = true;
      this.attrs.push(p);
    }
  }, {
    key: "addProp",
    value: function addProp(p, addArgs) {
      if (addArgs.options) {
        addArgs.options(p);
      }

      if (this.readOnly) {
        p.readOnly = this.readOnly;
      }

      if (this.autoCreateEdits) {
        this.autoCreateEditAction(p);
      }

      this.attrs.push(p);
    }
  }, {
    key: "addBool",
    value: function addBool(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _bool.PropBool(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addText",
    value: function addText(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _text.PropText(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addPassword",
    value: function addPassword(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _password.PropPassword(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addEnum",
    value: function addEnum(addArgs) {
      addArgs.parentName = (0, _changeCase.pascalCase)(this.parentName);
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _enum.PropEnum(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addNumber",
    value: function addNumber(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _number.PropNumber(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addLink",
    value: function addLink(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _link.PropLink(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addObject",
    value: function addObject(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      addArgs.parentName = (0, _changeCase.pascalCase)(this.parentName);
      var p = new PropObject(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addAction",
    value: function addAction(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      addArgs.parentName = (0, _changeCase.pascalCase)(this.parentName);
      var p = new PropAction(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addMethod",
    value: function addMethod(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      addArgs.parentName = (0, _changeCase.pascalCase)(this.parentName);
      var p = new PropMethod(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addMap",
    value: function addMap(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _map.PropMap(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "addCode",
    value: function addCode(addArgs) {
      addArgs.componentTypeName = this.componentTypeName;
      var p = new _code.PropCode(addArgs);
      this.addProp(p, addArgs);
    }
  }, {
    key: "autoCreateEditAction",
    value: function autoCreateEditAction(p) {
      var notAllowedKinds = ["method", "action"];

      if (notAllowedKinds.includes(p.kind())) {
        return;
      }

      var systemObject = _registry.registry.get(p.componentTypeName);

      systemObject.methods.addAction({
        name: "".concat((0, _changeCase.camelCase)(p.name), "Edit"),
        label: "Edit ".concat((0, _changeCase.camelCase)(p.parentName)).concat((0, _changeCase.pascalCase)(p.name), " Property"),
        options: function options(pa) {
          pa.universal = true;
          pa.mutation = true;
          pa.request.properties.addLink({
            name: "property",
            label: "The ".concat(p.label, " property value"),
            options: function options(pl) {
              pl.lookup = {
                typeName: p.componentTypeName,
                names: ["properties", p.name]
              };
            }
          });
        }
      });
    }
  }, {
    key: "length",
    get: function get() {
      return this.attrs.length;
    }
  }]);
  return AttrList;
}();

exports.AttrList = AttrList;

var PropObject = /*#__PURE__*/function (_Prop) {
  (0, _inherits2["default"])(PropObject, _Prop);

  var _super = _createSuper(PropObject);

  function PropObject(_ref2) {
    var _this;

    var name = _ref2.name,
        label = _ref2.label,
        componentTypeName = _ref2.componentTypeName,
        parentName = _ref2.parentName,
        defaultValue = _ref2.defaultValue;
    (0, _classCallCheck2["default"])(this, PropObject);
    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "baseDefaultValue", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "properties", void 0);
    _this.baseDefaultValue = defaultValue || {};
    _this.parentName = parentName || "";
    _this.properties = new AttrList({
      parentName: "".concat((0, _changeCase.pascalCase)(_this.parentName)).concat((0, _changeCase.pascalCase)(name)),
      componentTypeName: _this.componentTypeName
    });
    return _this;
  }

  (0, _createClass2["default"])(PropObject, [{
    key: "kind",
    value: function kind() {
      return "object";
    }
  }, {
    key: "protobufType",
    value: function protobufType() {
      var suffix = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : "";
      return "".concat((0, _changeCase.pascalCase)(this.parentName)).concat((0, _changeCase.pascalCase)(this.name)).concat((0, _changeCase.pascalCase)(suffix));
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }, {
    key: "bagNames",
    value: function bagNames() {
      return ["properties"];
    }
  }]);
  return PropObject;
}(_prop.Prop);

exports.PropObject = PropObject;

var PropMethod = /*#__PURE__*/function (_Prop2) {
  (0, _inherits2["default"])(PropMethod, _Prop2);

  var _super2 = _createSuper(PropMethod);

  // Methods have a Request and a Response
  //
  // The Request is made up of properties!
  // The Reply is made up of properties!
  function PropMethod(_ref3) {
    var _this2;

    var name = _ref3.name,
        label = _ref3.label,
        componentTypeName = _ref3.componentTypeName,
        parentName = _ref3.parentName,
        defaultValue = _ref3.defaultValue;
    (0, _classCallCheck2["default"])(this, PropMethod);
    _this2 = _super2.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "baseDefaultValue", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "request", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "reply", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "mutation", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "skipAuth", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "isPrivate", void 0);
    _this2.baseDefaultValue = defaultValue || {};
    _this2.parentName = parentName || "";
    _this2.request = new PropObject({
      name: "".concat((0, _changeCase.pascalCase)(name), "Request"),
      label: "".concat(label, " Request"),
      parentName: _this2.parentName,
      componentTypeName: _this2.componentTypeName
    });
    _this2.reply = new PropObject({
      name: "".concat((0, _changeCase.pascalCase)(name), "Reply"),
      label: "".concat(label, " Reply"),
      parentName: _this2.parentName,
      componentTypeName: _this2.componentTypeName
    });
    _this2.mutation = false;
    _this2.skipAuth = false;
    _this2.isPrivate = false;
    return _this2;
  }

  (0, _createClass2["default"])(PropMethod, [{
    key: "kind",
    value: function kind() {
      return "method";
    }
  }, {
    key: "protobufType",
    value: function protobufType() {
      var suffix = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : "";
      return "".concat((0, _changeCase.pascalCase)(this.parentName)).concat((0, _changeCase.pascalCase)(this.name)).concat((0, _changeCase.pascalCase)(suffix));
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }, {
    key: "bagNames",
    value: function bagNames() {
      return ["request", "reply"];
    }
  }]);
  return PropMethod;
}(_prop.Prop);

exports.PropMethod = PropMethod;

var PropAction = /*#__PURE__*/function (_PropMethod) {
  (0, _inherits2["default"])(PropAction, _PropMethod);

  var _super3 = _createSuper(PropAction);

  // Actions have a Request and a Response
  //
  // The Response is always `{ entityEvent: EntityEvent }`;
  //
  // The Request is made up of properties!
  function PropAction(_ref4) {
    var _this3;

    var name = _ref4.name,
        label = _ref4.label,
        componentTypeName = _ref4.componentTypeName,
        parentName = _ref4.parentName,
        defaultValue = _ref4.defaultValue;
    (0, _classCallCheck2["default"])(this, PropAction);
    _this3 = _super3.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      parentName: parentName,
      defaultValue: defaultValue
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this3), "integrationServices", void 0);
    _this3.integrationServices = [];

    _this3.request.properties.addText({
      name: "id",
      label: "Entity ID",
      options: function options(p) {
        p.universal = true;
        p.required = true;
      }
    });

    _this3.reply.properties.addLink({
      name: "item",
      label: "Entity Event",
      options: function options(p) {
        p.universal = true;
        p.readOnly = true;
        p.lookup = {
          typeName: "".concat(this.componentTypeName, "Event")
        };
      }
    });

    return _this3;
  }

  (0, _createClass2["default"])(PropAction, [{
    key: "kind",
    value: function kind() {
      return "action";
    }
  }, {
    key: "protobufType",
    value: function protobufType() {
      var suffix = arguments.length > 0 && arguments[0] !== undefined ? arguments[0] : "";
      return "".concat((0, _changeCase.pascalCase)(this.parentName)).concat((0, _changeCase.pascalCase)(this.name)).concat((0, _changeCase.pascalCase)(suffix));
    }
  }]);
  return PropAction;
}(PropMethod);

exports.PropAction = PropAction;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9hdHRyTGlzdC50cyJdLCJuYW1lcyI6WyJBdHRyTGlzdCIsInBhcmVudE5hbWUiLCJyZWFkT25seSIsImNvbXBvbmVudFR5cGVOYW1lIiwiYXV0b0NyZWF0ZUVkaXRzIiwiYXR0cnMiLCJsZW5ndGgiLCJuYW1lIiwicmVzdWx0IiwiZmluZCIsImUiLCJ1bmRlZmluZWQiLCJkZWZhdWx0VmFsdWVzIiwicmVzdWx0VmFsdWVzIiwiZW50cmllcyIsIml0ZW0iLCJkZWZhdWx0VmFsdWUiLCJ2YWx1ZXMiLCJraW5kIiwiUHJvcENvZGUiLCJyZWFsVmFsdWUiLCJwIiwicmVmZXJlbmNlIiwicHVzaCIsImFkZEFyZ3MiLCJvcHRpb25zIiwiYXV0b0NyZWF0ZUVkaXRBY3Rpb24iLCJQcm9wQm9vbCIsImFkZFByb3AiLCJQcm9wVGV4dCIsIlByb3BQYXNzd29yZCIsIlByb3BFbnVtIiwiUHJvcE51bWJlciIsIlByb3BMaW5rIiwiUHJvcE9iamVjdCIsIlByb3BBY3Rpb24iLCJQcm9wTWV0aG9kIiwiUHJvcE1hcCIsIm5vdEFsbG93ZWRLaW5kcyIsImluY2x1ZGVzIiwic3lzdGVtT2JqZWN0IiwicmVnaXN0cnkiLCJnZXQiLCJtZXRob2RzIiwiYWRkQWN0aW9uIiwibGFiZWwiLCJwYSIsInVuaXZlcnNhbCIsIm11dGF0aW9uIiwicmVxdWVzdCIsInByb3BlcnRpZXMiLCJhZGRMaW5rIiwicGwiLCJsb29rdXAiLCJ0eXBlTmFtZSIsIm5hbWVzIiwiYmFzZURlZmF1bHRWYWx1ZSIsInN1ZmZpeCIsIlByb3AiLCJyZXBseSIsInNraXBBdXRoIiwiaXNQcml2YXRlIiwiaW50ZWdyYXRpb25TZXJ2aWNlcyIsImFkZFRleHQiLCJyZXF1aXJlZCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7QUFDQTs7QUFDQTs7QUFFQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFFQTs7QUFFQTs7Ozs7Ozs7Ozs7O0lBa0NhQSxRO0FB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YWx1ZXMsIFByb3BDb25zdHJ1Y3RvciB9IGZyb20gXCIuL3Byb3BcIjtcbmltcG9ydCB7IFByb3BUZXh0IH0gZnJvbSBcIi4vcHJvcC90ZXh0XCI7XG5pbXBvcnQgeyBQcm9wQ29kZSB9IGZyb20gXCIuL3Byb3AvY29kZVwiO1xuaW1wb3J0IHsgUHJvcFNlbGVjdCB9IGZyb20gXCIuL3Byb3Avc2VsZWN0XCI7XG5pbXBvcnQgeyBQcm9wTnVtYmVyIH0gZnJvbSBcIi4vcHJvcC9udW1iZXJcIjtcbmltcG9ydCB7IFByb3BNYXAgfSBmcm9tIFwiLi9wcm9wL21hcFwiO1xuaW1wb3J0IHsgUHJvcEVudW0gfSBmcm9tIFwiLi9wcm9wL2VudW1cIjtcbmltcG9ydCB7IFByb3BCb29sIH0gZnJvbSBcIi4vcHJvcC9ib29sXCI7XG5pbXBvcnQgeyBQcm9wTGluayB9IGZyb20gXCIuL3Byb3AvbGlua1wiO1xuaW1wb3J0IHsgUHJvcFBhc3N3b3JkIH0gZnJvbSBcIi4vcHJvcC9wYXNzd29yZFwiO1xuXG5pbXBvcnQgeyBwYXNjYWxDYXNlLCBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcblxuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi9yZWdpc3RyeVwiO1xuXG5leHBvcnQgdHlwZSBQcm9wcyA9XG4gIHwgUHJvcFRleHRcbiAgfCBQcm9wUGFzc3dvcmRcbiAgfCBQcm9wU2VsZWN0XG4gIHwgUHJvcENvZGVcbiAgfCBQcm9wTnVtYmVyXG4gIHwgUHJvcE9iamVjdFxuICB8IFByb3BNYXBcbiAgfCBQcm9wRW51bVxuICB8IFByb3BCb29sXG4gIHwgUHJvcExpbms7XG5cbmludGVyZmFjZSBBZGRBcmd1bWVudHMge1xuICBuYW1lOiBzdHJpbmc7XG4gIGxhYmVsOiBzdHJpbmc7XG4gIGNvbXBvbmVudFR5cGVOYW1lPzogc3RyaW5nO1xuICBwYXJlbnROYW1lPzogc3RyaW5nO1xuICBvcHRpb25zPyhwOiBQcm9wcyk6IHZvaWQ7XG59XG5cbmludGVyZmFjZSBBdHRyTGlzdENvbnN0cnVjdG9yIHtcbiAgY29tcG9uZW50VHlwZU5hbWU/OiBzdHJpbmc7XG4gIHBhcmVudE5hbWU/OiBzdHJpbmc7XG4gIHJlYWRPbmx5PzogYm9vbGVhbjtcbiAgYXV0b0NyZWF0ZUVkaXRzPzogYm9vbGVhbjtcbn1cblxuZXhwb3J0IGludGVyZmFjZSBJbnRlZ3JhdGlvblNlcnZpY2Uge1xuICBpbnRlZ3JhdGlvbk5hbWU6IHN0cmluZztcbiAgaW50ZWdyYXRpb25TZXJ2aWNlTmFtZTogc3RyaW5nO1xufVxuXG5leHBvcnQgY2xhc3MgQXR0ckxpc3Qge1xuICBhdHRyczogUHJvcHNbXTtcbiAgcmVhZE9ubHk6IGJvb2xlYW47XG4gIHBhcmVudE5hbWU6IHN0cmluZztcbiAgYXV0b0NyZWF0ZUVkaXRzOiBib29sZWFuO1xuICBjb21wb25lbnRUeXBlTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBwYXJlbnROYW1lLFxuICAgIHJlYWRPbmx5LFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIGF1dG9DcmVhdGVFZGl0cyxcbiAgfTogQXR0ckxpc3RDb25zdHJ1Y3Rvcikge1xuICAgIHRoaXMucGFyZW50TmFtZSA9IHBhcmVudE5hbWUgfHwgXCJcIjtcbiAgICB0aGlzLmF0dHJzID0gW107XG4gICAgdGhpcy5jb21wb25lbnRUeXBlTmFtZSA9IGNvbXBvbmVudFR5cGVOYW1lIHx8IFwiXCI7XG4gICAgdGhpcy5yZWFkT25seSA9IHJlYWRPbmx5IHx8IGZhbHNlO1xuICAgIHRoaXMuYXV0b0NyZWF0ZUVkaXRzID0gYXV0b0NyZWF0ZUVkaXRzIHx8IGZhbHNlO1xuICB9XG5cbiAgZ2V0IGxlbmd0aCgpOiBudW1iZXIge1xuICAgIHJldHVybiB0aGlzLmF0dHJzLmxlbmd0aDtcbiAgfVxuXG4gIGhhc0VudHJpZXMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuYXR0cnMubGVuZ3RoID4gMDtcbiAgfVxuXG4gIGVudHJpZXMoKTogdGhpc1tcImF0dHJzXCJdIHtcbiAgICByZXR1cm4gdGhpcy5hdHRycztcbiAgfVxuXG4gIGdldEVudHJ5KG5hbWU6IHN0cmluZyk6IFByb3BzIHtcbiAgICBjb25zdCByZXN1bHQgPSB0aGlzLmF0dHJzLmZpbmQoZSA9PiBlLm5hbWUgPT0gbmFtZSk7XG4gICAgaWYgKHJlc3VsdCA9PSB1bmRlZmluZWQpIHtcbiAgICAgIHRocm93IGBDYW5ub3QgZmluZCBwcm9wZXJ0eSAke25hbWV9IGZvciAke3RoaXMuY29tcG9uZW50VHlwZU5hbWV9YDtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIGNyZWF0ZVZhbHVlT2JqZWN0KGRlZmF1bHRWYWx1ZXM/OiBQcm9wRGVmYXVsdFZhbHVlcyk6IFByb3BEZWZhdWx0VmFsdWVzIHtcbiAgICBjb25zdCByZXN1bHRWYWx1ZXMgPSBkZWZhdWx0VmFsdWVzIHx8IHt9O1xuICAgIGZvciAoY29uc3QgaXRlbSBvZiB0aGlzLmVudHJpZXMoKSkge1xuICAgICAgaWYgKHJlc3VsdFZhbHVlc1tpdGVtLm5hbWVdKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0gPSBpdGVtLmRlZmF1bHRWYWx1ZSgpO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0VmFsdWVzO1xuICB9XG5cbiAgcmVhbFZhbHVlcyh2YWx1ZXM6IFByb3BEZWZhdWx0VmFsdWVzKTogUHJvcERlZmF1bHRWYWx1ZXMge1xuICAgIGNvbnN0IHJlc3VsdFZhbHVlczogUHJvcERlZmF1bHRWYWx1ZXMgPSB7fTtcbiAgICBmb3IgKGNvbnN0IGl0ZW0gb2YgdGhpcy5lbnRyaWVzKCkpIHtcbiAgICAgIGlmIChpdGVtLmtpbmQoKSA9PSBcImNvZGVcIiAmJiBpdGVtIGluc3RhbmNlb2YgUHJvcENvZGUpIHtcbiAgICAgICAgaWYgKHZhbHVlc1tpdGVtLm5hbWVdKSB7XG4gICAgICAgICAgcmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0gPSBpdGVtLnJlYWxWYWx1ZSh2YWx1ZXNbaXRlbS5uYW1lXSk7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdFZhbHVlc1tpdGVtLm5hbWVdID0gdmFsdWVzW2l0ZW0ubmFtZV07XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRWYWx1ZXM7XG4gIH1cblxuICBhZGRFeGlzdGluZyhwOiBQcm9wcyk6IHZvaWQge1xuICAgIHAucmVmZXJlbmNlID0gdHJ1ZTtcbiAgICB0aGlzLmF0dHJzLnB1c2gocCk7XG4gIH1cblxuICBhZGRQcm9wKHA6IFByb3BzLCBhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBpZiAoYWRkQXJncy5vcHRpb25zKSB7XG4gICAgICBhZGRBcmdzLm9wdGlvbnMocCk7XG4gICAgfVxuICAgIGlmICh0aGlzLnJlYWRPbmx5KSB7XG4gICAgICBwLnJlYWRPbmx5ID0gdGhpcy5yZWFkT25seTtcbiAgICB9XG4gICAgaWYgKHRoaXMuYXV0b0NyZWF0ZUVkaXRzKSB7XG4gICAgICB0aGlzLmF1dG9DcmVhdGVFZGl0QWN0aW9uKHApO1xuICAgIH1cbiAgICB0aGlzLmF0dHJzLnB1c2gocCk7XG4gIH1cblxuICBhZGRCb29sKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcEJvb2woYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZFRleHQoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wVGV4dChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkUGFzc3dvcmQoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wUGFzc3dvcmQoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZEVudW0oYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5wYXJlbnROYW1lID0gcGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpO1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcEVudW0oYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZE51bWJlcihhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BOdW1iZXIoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZExpbmsoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wTGluayhhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkT2JqZWN0KGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGFkZEFyZ3MucGFyZW50TmFtZSA9IHBhc2NhbENhc2UodGhpcy5wYXJlbnROYW1lKTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BPYmplY3QoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZEFjdGlvbihhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBhZGRBcmdzLnBhcmVudE5hbWUgPSBwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSk7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wQWN0aW9uKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRNZXRob2QoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgYWRkQXJncy5wYXJlbnROYW1lID0gcGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE1ldGhvZChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkTWFwKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE1hcChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkQ29kZShhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BDb2RlKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhdXRvQ3JlYXRlRWRpdEFjdGlvbihwOiBQcm9wcyk6IHZvaWQge1xuICAgIGNvbnN0IG5vdEFsbG93ZWRLaW5kcyA9IFtcIm1ldGhvZFwiLCBcImFjdGlvblwiXTtcbiAgICBpZiAobm90QWxsb3dlZEtpbmRzLmluY2x1ZGVzKHAua2luZCgpKSkge1xuICAgICAgcmV0dXJuO1xuICAgIH1cbiAgICBjb25zdCBzeXN0ZW1PYmplY3QgPSByZWdpc3RyeS5nZXQocC5jb21wb25lbnRUeXBlTmFtZSk7XG5cbiAgICBzeXN0ZW1PYmplY3QubWV0aG9kcy5hZGRBY3Rpb24oe1xuICAgICAgbmFtZTogYCR7Y2FtZWxDYXNlKHAubmFtZSl9RWRpdGAsXG4gICAgICBsYWJlbDogYEVkaXQgJHtjYW1lbENhc2UocC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocC5uYW1lKX0gUHJvcGVydHlgLFxuICAgICAgb3B0aW9ucyhwYTogUHJvcEFjdGlvbikge1xuICAgICAgICBwYS51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwYS5tdXRhdGlvbiA9IHRydWU7XG4gICAgICAgIHBhLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInByb3BlcnR5XCIsXG4gICAgICAgICAgbGFiZWw6IGBUaGUgJHtwLmxhYmVsfSBwcm9wZXJ0eSB2YWx1ZWAsXG4gICAgICAgICAgb3B0aW9ucyhwbDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHBsLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IHAuY29tcG9uZW50VHlwZU5hbWUsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJwcm9wZXJ0aWVzXCIsIHAubmFtZV0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUHJvcE9iamVjdCBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICBwcm9wZXJ0aWVzOiBBdHRyTGlzdDtcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBwYXJlbnROYW1lLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBwYXJlbnROYW1lPzogUHJvcFtcInBhcmVudE5hbWVcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogUHJvcE9iamVjdFtcImJhc2VEZWZhdWx0VmFsdWVcIl07XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSB9KTtcbiAgICB0aGlzLmJhc2VEZWZhdWx0VmFsdWUgPSBkZWZhdWx0VmFsdWUgfHwge307XG4gICAgdGhpcy5wYXJlbnROYW1lID0gcGFyZW50TmFtZSB8fCBcIlwiO1xuICAgIHRoaXMucHJvcGVydGllcyA9IG5ldyBBdHRyTGlzdCh7XG4gICAgICBwYXJlbnROYW1lOiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKG5hbWUpfWAsXG4gICAgICBjb21wb25lbnRUeXBlTmFtZTogdGhpcy5jb21wb25lbnRUeXBlTmFtZSxcbiAgICB9KTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJvYmplY3RcIjtcbiAgfVxuXG4gIHByb3RvYnVmVHlwZShzdWZmaXggPSBcIlwiKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7cGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZSh0aGlzLm5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgIHN1ZmZpeCxcbiAgICApfWA7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcE9iamVjdFtcImJhc2VEZWZhdWx0VmFsdWVcIl0ge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cblxuICBiYWdOYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgcmV0dXJuIFtcInByb3BlcnRpZXNcIl07XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFByb3BNZXRob2QgZXh0ZW5kcyBQcm9wIHtcbiAgYmFzZURlZmF1bHRWYWx1ZTogUmVjb3JkPHN0cmluZywgYW55PjtcbiAgcmVxdWVzdDogUHJvcE9iamVjdDtcbiAgcmVwbHk6IFByb3BPYmplY3Q7XG4gIG11dGF0aW9uOiBib29sZWFuO1xuICBza2lwQXV0aDogYm9vbGVhbjtcbiAgaXNQcml2YXRlOiBib29sZWFuO1xuXG4gIC8vIE1ldGhvZHMgaGF2ZSBhIFJlcXVlc3QgYW5kIGEgUmVzcG9uc2VcbiAgLy9cbiAgLy8gVGhlIFJlcXVlc3QgaXMgbWFkZSB1cCBvZiBwcm9wZXJ0aWVzIVxuICAvLyBUaGUgUmVwbHkgaXMgbWFkZSB1cCBvZiBwcm9wZXJ0aWVzIVxuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHBhcmVudE5hbWUsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHBhcmVudE5hbWU/OiBQcm9wW1wicGFyZW50TmFtZVwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBQcm9wQWN0aW9uW1wiYmFzZURlZmF1bHRWYWx1ZVwiXTtcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCB7fTtcbiAgICB0aGlzLnBhcmVudE5hbWUgPSBwYXJlbnROYW1lIHx8IFwiXCI7XG4gICAgdGhpcy5yZXF1ZXN0ID0gbmV3IFByb3BPYmplY3Qoe1xuICAgICAgbmFtZTogYCR7cGFzY2FsQ2FzZShuYW1lKX1SZXF1ZXN0YCxcbiAgICAgIGxhYmVsOiBgJHtsYWJlbH0gUmVxdWVzdGAsXG4gICAgICBwYXJlbnROYW1lOiB0aGlzLnBhcmVudE5hbWUsXG4gICAgICBjb21wb25lbnRUeXBlTmFtZTogdGhpcy5jb21wb25lbnRUeXBlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLnJlcGx5ID0gbmV3IFByb3BPYmplY3Qoe1xuICAgICAgbmFtZTogYCR7cGFzY2FsQ2FzZShuYW1lKX1SZXBseWAsXG4gICAgICBsYWJlbDogYCR7bGFiZWx9IFJlcGx5YCxcbiAgICAgIHBhcmVudE5hbWU6IHRoaXMucGFyZW50TmFtZSxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0aGlzLmNvbXBvbmVudFR5cGVOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMubXV0YXRpb24gPSBmYWxzZTtcbiAgICB0aGlzLnNraXBBdXRoID0gZmFsc2U7XG4gICAgdGhpcy5pc1ByaXZhdGUgPSBmYWxzZTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJtZXRob2RcIjtcbiAgfVxuXG4gIHByb3RvYnVmVHlwZShzdWZmaXggPSBcIlwiKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7cGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZSh0aGlzLm5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgIHN1ZmZpeCxcbiAgICApfWA7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcE9iamVjdFtcImJhc2VEZWZhdWx0VmFsdWVcIl0ge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cblxuICBiYWdOYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgcmV0dXJuIFtcInJlcXVlc3RcIiwgXCJyZXBseVwiXTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUHJvcEFjdGlvbiBleHRlbmRzIFByb3BNZXRob2Qge1xuICBpbnRlZ3JhdGlvblNlcnZpY2VzOiBJbnRlZ3JhdGlvblNlcnZpY2VbXTtcblxuICAvLyBBY3Rpb25zIGhhdmUgYSBSZXF1ZXN0IGFuZCBhIFJlc3BvbnNlXG4gIC8vXG4gIC8vIFRoZSBSZXNwb25zZSBpcyBhbHdheXMgYHsgZW50aXR5RXZlbnQ6IEVudGl0eUV2ZW50IH1gO1xuICAvL1xuICAvLyBUaGUgUmVxdWVzdCBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcGFyZW50TmFtZSxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcGFyZW50TmFtZT86IFByb3BbXCJwYXJlbnROYW1lXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IFByb3BBY3Rpb25bXCJiYXNlRGVmYXVsdFZhbHVlXCJdO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUsIHBhcmVudE5hbWUsIGRlZmF1bHRWYWx1ZSB9KTtcbiAgICB0aGlzLmludGVncmF0aW9uU2VydmljZXMgPSBbXTtcbiAgICB0aGlzLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiaWRcIixcbiAgICAgIGxhYmVsOiBcIkVudGl0eSBJRFwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwiaXRlbVwiLFxuICAgICAgbGFiZWw6IGBFbnRpdHkgRXZlbnRgLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7dGhpcy5jb21wb25lbnRUeXBlTmFtZX1FdmVudGAsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImFjdGlvblwiO1xuICB9XG5cbiAgcHJvdG9idWZUeXBlKHN1ZmZpeCA9IFwiXCIpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHRoaXMubmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgc3VmZml4LFxuICAgICl9YDtcbiAgfVxufVxuIl19