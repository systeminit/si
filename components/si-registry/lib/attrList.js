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

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(n); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

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

    _this3.request.properties.addText({
      name: "entityId",
      label: "Entity ID",
      options: function options(p) {
        p.universal = true;
        p.required = true;
      }
    });

    _this3.reply.properties.addLink({
      name: "entityEvent",
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9hdHRyTGlzdC50cyJdLCJuYW1lcyI6WyJBdHRyTGlzdCIsInBhcmVudE5hbWUiLCJyZWFkT25seSIsImNvbXBvbmVudFR5cGVOYW1lIiwiYXV0b0NyZWF0ZUVkaXRzIiwiYXR0cnMiLCJsZW5ndGgiLCJuYW1lIiwicmVzdWx0IiwiZmluZCIsImUiLCJ1bmRlZmluZWQiLCJkZWZhdWx0VmFsdWVzIiwicmVzdWx0VmFsdWVzIiwiZW50cmllcyIsIml0ZW0iLCJkZWZhdWx0VmFsdWUiLCJ2YWx1ZXMiLCJraW5kIiwiUHJvcENvZGUiLCJyZWFsVmFsdWUiLCJwIiwicmVmZXJlbmNlIiwicHVzaCIsImFkZEFyZ3MiLCJvcHRpb25zIiwiYXV0b0NyZWF0ZUVkaXRBY3Rpb24iLCJQcm9wQm9vbCIsImFkZFByb3AiLCJQcm9wVGV4dCIsIlByb3BQYXNzd29yZCIsIlByb3BFbnVtIiwiUHJvcE51bWJlciIsIlByb3BMaW5rIiwiUHJvcE9iamVjdCIsIlByb3BBY3Rpb24iLCJQcm9wTWV0aG9kIiwiUHJvcE1hcCIsIm5vdEFsbG93ZWRLaW5kcyIsImluY2x1ZGVzIiwic3lzdGVtT2JqZWN0IiwicmVnaXN0cnkiLCJnZXQiLCJtZXRob2RzIiwiYWRkQWN0aW9uIiwibGFiZWwiLCJwYSIsInVuaXZlcnNhbCIsIm11dGF0aW9uIiwicmVxdWVzdCIsInByb3BlcnRpZXMiLCJhZGRMaW5rIiwicGwiLCJsb29rdXAiLCJ0eXBlTmFtZSIsIm5hbWVzIiwiYmFzZURlZmF1bHRWYWx1ZSIsInN1ZmZpeCIsIlByb3AiLCJyZXBseSIsInNraXBBdXRoIiwiaXNQcml2YXRlIiwiYWRkVGV4dCIsInJlcXVpcmVkIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFBOztBQUNBOztBQUNBOztBQUVBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUVBOztBQUVBOzs7Ozs7Ozs7Ozs7SUE2QmFBLFE7QUFPWCwwQkFLd0I7QUFBQSxRQUp0QkMs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QSxVQUFsQztBQUE4Q2UsTUFBQUEsWUFBWSxFQUFaQTtBQUE5QyxLQUFOOztBQUNBLFdBQUtpQyxPQUFMLENBQWFDLFVBQWIsQ0FBd0JZLE9BQXhCLENBQWdDO0FBQzlCdkQsTUFBQUEsSUFBSSxFQUFFLFVBRHdCO0FBRTlCc0MsTUFBQUEsS0FBSyxFQUFFLFdBRnVCO0FBRzlCcEIsTUFBQUEsT0FIOEIsbUJBR3RCSixDQUhzQixFQUduQjtBQUNUQSxRQUFBQSxDQUFDLENBQUMwQixTQUFGLEdBQWMsSUFBZDtBQUNBMUIsUUFBQUEsQ0FBQyxDQUFDMEMsUUFBRixHQUFhLElBQWI7QUFDRDtBQU42QixLQUFoQzs7QUFRQSxXQUFLSixLQUFMLENBQVdULFVBQVgsQ0FBc0JDLE9BQXRCLENBQThCO0FBQzVCNUMsTUFBQUEsSUFBSSxFQUFFLGFBRHNCO0FBRTVCc0MsTUFBQUEsS0FBSyxnQkFGdUI7QUFHNUJwQixNQUFBQSxPQUg0QixtQkFHcEJKLENBSG9CLEVBR1A7QUFDbkJBLFFBQUFBLENBQUMsQ0FBQzBCLFNBQUYsR0FBYyxJQUFkO0FBQ0ExQixRQUFBQSxDQUFDLENBQUNuQixRQUFGLEdBQWEsSUFBYjtBQUNBbUIsUUFBQUEsQ0FBQyxDQUFDZ0MsTUFBRixHQUFXO0FBQ1RDLFVBQUFBLFFBQVEsWUFBSyxLQUFLbkQsaUJBQVY7QUFEQyxTQUFYO0FBR0Q7QUFUMkIsS0FBOUI7O0FBVkM7QUFxQkY7Ozs7MkJBRWM7QUFDYixhQUFPLFFBQVA7QUFDRDs7O21DQUVpQztBQUFBLFVBQXJCc0QsTUFBcUIsdUVBQVosRUFBWTtBQUNoQyx1QkFBVSw0QkFBVyxLQUFLeEQsVUFBaEIsQ0FBVixTQUF3Qyw0QkFBVyxLQUFLTSxJQUFoQixDQUF4QyxTQUFnRSw0QkFDOURrRCxNQUQ4RCxDQUFoRTtBQUdEOzs7RUFsRDZCckIsVSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3AsIFByb3BEZWZhdWx0VmFsdWVzLCBQcm9wQ29uc3RydWN0b3IgfSBmcm9tIFwiLi9wcm9wXCI7XG5pbXBvcnQgeyBQcm9wVGV4dCB9IGZyb20gXCIuL3Byb3AvdGV4dFwiO1xuaW1wb3J0IHsgUHJvcENvZGUgfSBmcm9tIFwiLi9wcm9wL2NvZGVcIjtcbmltcG9ydCB7IFByb3BTZWxlY3QgfSBmcm9tIFwiLi9wcm9wL3NlbGVjdFwiO1xuaW1wb3J0IHsgUHJvcE51bWJlciB9IGZyb20gXCIuL3Byb3AvbnVtYmVyXCI7XG5pbXBvcnQgeyBQcm9wTWFwIH0gZnJvbSBcIi4vcHJvcC9tYXBcIjtcbmltcG9ydCB7IFByb3BFbnVtIH0gZnJvbSBcIi4vcHJvcC9lbnVtXCI7XG5pbXBvcnQgeyBQcm9wQm9vbCB9IGZyb20gXCIuL3Byb3AvYm9vbFwiO1xuaW1wb3J0IHsgUHJvcExpbmsgfSBmcm9tIFwiLi9wcm9wL2xpbmtcIjtcbmltcG9ydCB7IFByb3BQYXNzd29yZCB9IGZyb20gXCIuL3Byb3AvcGFzc3dvcmRcIjtcblxuaW1wb3J0IHsgcGFzY2FsQ2FzZSwgY2FtZWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5cbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4vcmVnaXN0cnlcIjtcblxuZXhwb3J0IHR5cGUgUHJvcHMgPVxuICB8IFByb3BUZXh0XG4gIHwgUHJvcFBhc3N3b3JkXG4gIHwgUHJvcFNlbGVjdFxuICB8IFByb3BDb2RlXG4gIHwgUHJvcE51bWJlclxuICB8IFByb3BPYmplY3RcbiAgfCBQcm9wTWFwXG4gIHwgUHJvcEVudW1cbiAgfCBQcm9wQm9vbFxuICB8IFByb3BMaW5rO1xuXG5pbnRlcmZhY2UgQWRkQXJndW1lbnRzIHtcbiAgbmFtZTogc3RyaW5nO1xuICBsYWJlbDogc3RyaW5nO1xuICBjb21wb25lbnRUeXBlTmFtZT86IHN0cmluZztcbiAgcGFyZW50TmFtZT86IHN0cmluZztcbiAgb3B0aW9ucz8ocDogUHJvcHMpOiB2b2lkO1xufVxuXG5pbnRlcmZhY2UgQXR0ckxpc3RDb25zdHJ1Y3RvciB7XG4gIGNvbXBvbmVudFR5cGVOYW1lPzogc3RyaW5nO1xuICBwYXJlbnROYW1lPzogc3RyaW5nO1xuICByZWFkT25seT86IGJvb2xlYW47XG4gIGF1dG9DcmVhdGVFZGl0cz86IGJvb2xlYW47XG59XG5cbmV4cG9ydCBjbGFzcyBBdHRyTGlzdCB7XG4gIGF0dHJzOiBQcm9wc1tdO1xuICByZWFkT25seTogYm9vbGVhbjtcbiAgcGFyZW50TmFtZTogc3RyaW5nO1xuICBhdXRvQ3JlYXRlRWRpdHM6IGJvb2xlYW47XG4gIGNvbXBvbmVudFR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIHBhcmVudE5hbWUsXG4gICAgcmVhZE9ubHksXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgYXV0b0NyZWF0ZUVkaXRzLFxuICB9OiBBdHRyTGlzdENvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy5wYXJlbnROYW1lID0gcGFyZW50TmFtZSB8fCBcIlwiO1xuICAgIHRoaXMuYXR0cnMgPSBbXTtcbiAgICB0aGlzLmNvbXBvbmVudFR5cGVOYW1lID0gY29tcG9uZW50VHlwZU5hbWUgfHwgXCJcIjtcbiAgICB0aGlzLnJlYWRPbmx5ID0gcmVhZE9ubHkgfHwgZmFsc2U7XG4gICAgdGhpcy5hdXRvQ3JlYXRlRWRpdHMgPSBhdXRvQ3JlYXRlRWRpdHMgfHwgZmFsc2U7XG4gIH1cblxuICBnZXQgbGVuZ3RoKCk6IG51bWJlciB7XG4gICAgcmV0dXJuIHRoaXMuYXR0cnMubGVuZ3RoO1xuICB9XG5cbiAgaGFzRW50cmllcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5hdHRycy5sZW5ndGggPiAwO1xuICB9XG5cbiAgZW50cmllcygpOiB0aGlzW1wiYXR0cnNcIl0ge1xuICAgIHJldHVybiB0aGlzLmF0dHJzO1xuICB9XG5cbiAgZ2V0RW50cnkobmFtZTogc3RyaW5nKTogUHJvcHMge1xuICAgIGNvbnN0IHJlc3VsdCA9IHRoaXMuYXR0cnMuZmluZChlID0+IGUubmFtZSA9PSBuYW1lKTtcbiAgICBpZiAocmVzdWx0ID09IHVuZGVmaW5lZCkge1xuICAgICAgdGhyb3cgYENhbm5vdCBmaW5kIHByb3BlcnR5ICR7bmFtZX0gZm9yICR7dGhpcy5jb21wb25lbnRUeXBlTmFtZX1gO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgY3JlYXRlVmFsdWVPYmplY3QoZGVmYXVsdFZhbHVlcz86IFByb3BEZWZhdWx0VmFsdWVzKTogUHJvcERlZmF1bHRWYWx1ZXMge1xuICAgIGNvbnN0IHJlc3VsdFZhbHVlcyA9IGRlZmF1bHRWYWx1ZXMgfHwge307XG4gICAgZm9yIChjb25zdCBpdGVtIG9mIHRoaXMuZW50cmllcygpKSB7XG4gICAgICBpZiAocmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0pIHtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHRWYWx1ZXNbaXRlbS5uYW1lXSA9IGl0ZW0uZGVmYXVsdFZhbHVlKCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRWYWx1ZXM7XG4gIH1cblxuICByZWFsVmFsdWVzKHZhbHVlczogUHJvcERlZmF1bHRWYWx1ZXMpOiBQcm9wRGVmYXVsdFZhbHVlcyB7XG4gICAgY29uc3QgcmVzdWx0VmFsdWVzOiBQcm9wRGVmYXVsdFZhbHVlcyA9IHt9O1xuICAgIGZvciAoY29uc3QgaXRlbSBvZiB0aGlzLmVudHJpZXMoKSkge1xuICAgICAgaWYgKGl0ZW0ua2luZCgpID09IFwiY29kZVwiICYmIGl0ZW0gaW5zdGFuY2VvZiBQcm9wQ29kZSkge1xuICAgICAgICBpZiAodmFsdWVzW2l0ZW0ubmFtZV0pIHtcbiAgICAgICAgICByZXN1bHRWYWx1ZXNbaXRlbS5uYW1lXSA9IGl0ZW0ucmVhbFZhbHVlKHZhbHVlc1tpdGVtLm5hbWVdKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0gPSB2YWx1ZXNbaXRlbS5uYW1lXTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdFZhbHVlcztcbiAgfVxuXG4gIGFkZEV4aXN0aW5nKHA6IFByb3BzKTogdm9pZCB7XG4gICAgcC5yZWZlcmVuY2UgPSB0cnVlO1xuICAgIHRoaXMuYXR0cnMucHVzaChwKTtcbiAgfVxuXG4gIGFkZFByb3AocDogUHJvcHMsIGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGlmIChhZGRBcmdzLm9wdGlvbnMpIHtcbiAgICAgIGFkZEFyZ3Mub3B0aW9ucyhwKTtcbiAgICB9XG4gICAgaWYgKHRoaXMucmVhZE9ubHkpIHtcbiAgICAgIHAucmVhZE9ubHkgPSB0aGlzLnJlYWRPbmx5O1xuICAgIH1cbiAgICBpZiAodGhpcy5hdXRvQ3JlYXRlRWRpdHMpIHtcbiAgICAgIHRoaXMuYXV0b0NyZWF0ZUVkaXRBY3Rpb24ocCk7XG4gICAgfVxuICAgIHRoaXMuYXR0cnMucHVzaChwKTtcbiAgfVxuXG4gIGFkZEJvb2woYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wQm9vbChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkVGV4dChhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BUZXh0KGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRQYXNzd29yZChhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BQYXNzd29yZChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkRW51bShhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLnBhcmVudE5hbWUgPSBwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSk7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wRW51bShhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkTnVtYmVyKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE51bWJlcihhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkTGluayhhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BMaW5rKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRPYmplY3QoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgYWRkQXJncy5wYXJlbnROYW1lID0gcGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE9iamVjdChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkQWN0aW9uKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGFkZEFyZ3MucGFyZW50TmFtZSA9IHBhc2NhbENhc2UodGhpcy5wYXJlbnROYW1lKTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BBY3Rpb24oYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZE1ldGhvZChhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBhZGRBcmdzLnBhcmVudE5hbWUgPSBwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSk7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wTWV0aG9kKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRNYXAoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wTWFwKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRDb2RlKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcENvZGUoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGF1dG9DcmVhdGVFZGl0QWN0aW9uKHA6IFByb3BzKTogdm9pZCB7XG4gICAgY29uc3Qgbm90QWxsb3dlZEtpbmRzID0gW1wibWV0aG9kXCIsIFwiYWN0aW9uXCJdO1xuICAgIGlmIChub3RBbGxvd2VkS2luZHMuaW5jbHVkZXMocC5raW5kKCkpKSB7XG4gICAgICByZXR1cm47XG4gICAgfVxuICAgIGNvbnN0IHN5c3RlbU9iamVjdCA9IHJlZ2lzdHJ5LmdldChwLmNvbXBvbmVudFR5cGVOYW1lKTtcblxuICAgIHN5c3RlbU9iamVjdC5tZXRob2RzLmFkZEFjdGlvbih7XG4gICAgICBuYW1lOiBgJHtjYW1lbENhc2UocC5uYW1lKX1FZGl0YCxcbiAgICAgIGxhYmVsOiBgRWRpdCAke2NhbWVsQ2FzZShwLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwLm5hbWUpfSBQcm9wZXJ0eWAsXG4gICAgICBvcHRpb25zKHBhOiBQcm9wQWN0aW9uKSB7XG4gICAgICAgIHBhLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHBhLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcGEucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwicHJvcGVydHlcIixcbiAgICAgICAgICBsYWJlbDogYFRoZSAke3AubGFiZWx9IHByb3BlcnR5IHZhbHVlYCxcbiAgICAgICAgICBvcHRpb25zKHBsOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcGwubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogcC5jb21wb25lbnRUeXBlTmFtZSxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcInByb3BlcnRpZXNcIiwgcC5uYW1lXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBQcm9wT2JqZWN0IGV4dGVuZHMgUHJvcCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IFJlY29yZDxzdHJpbmcsIGFueT47XG4gIHByb3BlcnRpZXM6IEF0dHJMaXN0O1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHBhcmVudE5hbWUsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHBhcmVudE5hbWU/OiBQcm9wW1wicGFyZW50TmFtZVwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBQcm9wT2JqZWN0W1wiYmFzZURlZmF1bHRWYWx1ZVwiXTtcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCB7fTtcbiAgICB0aGlzLnBhcmVudE5hbWUgPSBwYXJlbnROYW1lIHx8IFwiXCI7XG4gICAgdGhpcy5wcm9wZXJ0aWVzID0gbmV3IEF0dHJMaXN0KHtcbiAgICAgIHBhcmVudE5hbWU6IGAke3Bhc2NhbENhc2UodGhpcy5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UobmFtZSl9YCxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0aGlzLmNvbXBvbmVudFR5cGVOYW1lLFxuICAgIH0pO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcIm9iamVjdFwiO1xuICB9XG5cbiAgcHJvdG9idWZUeXBlKHN1ZmZpeCA9IFwiXCIpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHRoaXMubmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgc3VmZml4LFxuICAgICl9YDtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wT2JqZWN0W1wiYmFzZURlZmF1bHRWYWx1ZVwiXSB7XG4gICAgcmV0dXJuIHRoaXMuYmFzZURlZmF1bHRWYWx1ZTtcbiAgfVxuXG4gIGJhZ05hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICByZXR1cm4gW1wicHJvcGVydGllc1wiXTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUHJvcE1ldGhvZCBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICByZXF1ZXN0OiBQcm9wT2JqZWN0O1xuICByZXBseTogUHJvcE9iamVjdDtcbiAgbXV0YXRpb246IGJvb2xlYW47XG4gIHNraXBBdXRoOiBib29sZWFuO1xuICBpc1ByaXZhdGU6IGJvb2xlYW47XG5cbiAgLy8gTWV0aG9kcyBoYXZlIGEgUmVxdWVzdCBhbmQgYSBSZXNwb25zZVxuICAvL1xuICAvLyBUaGUgUmVxdWVzdCBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG4gIC8vIFRoZSBSZXBseSBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcGFyZW50TmFtZSxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcGFyZW50TmFtZT86IFByb3BbXCJwYXJlbnROYW1lXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IFByb3BBY3Rpb25bXCJiYXNlRGVmYXVsdFZhbHVlXCJdO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUgfSk7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IHt9O1xuICAgIHRoaXMucGFyZW50TmFtZSA9IHBhcmVudE5hbWUgfHwgXCJcIjtcbiAgICB0aGlzLnJlcXVlc3QgPSBuZXcgUHJvcE9iamVjdCh7XG4gICAgICBuYW1lOiBgJHtwYXNjYWxDYXNlKG5hbWUpfVJlcXVlc3RgLFxuICAgICAgbGFiZWw6IGAke2xhYmVsfSBSZXF1ZXN0YCxcbiAgICAgIHBhcmVudE5hbWU6IHRoaXMucGFyZW50TmFtZSxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0aGlzLmNvbXBvbmVudFR5cGVOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMucmVwbHkgPSBuZXcgUHJvcE9iamVjdCh7XG4gICAgICBuYW1lOiBgJHtwYXNjYWxDYXNlKG5hbWUpfVJlcGx5YCxcbiAgICAgIGxhYmVsOiBgJHtsYWJlbH0gUmVwbHlgLFxuICAgICAgcGFyZW50TmFtZTogdGhpcy5wYXJlbnROYW1lLFxuICAgICAgY29tcG9uZW50VHlwZU5hbWU6IHRoaXMuY29tcG9uZW50VHlwZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5tdXRhdGlvbiA9IGZhbHNlO1xuICAgIHRoaXMuc2tpcEF1dGggPSBmYWxzZTtcbiAgICB0aGlzLmlzUHJpdmF0ZSA9IGZhbHNlO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcIm1ldGhvZFwiO1xuICB9XG5cbiAgcHJvdG9idWZUeXBlKHN1ZmZpeCA9IFwiXCIpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHRoaXMubmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgc3VmZml4LFxuICAgICl9YDtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wT2JqZWN0W1wiYmFzZURlZmF1bHRWYWx1ZVwiXSB7XG4gICAgcmV0dXJuIHRoaXMuYmFzZURlZmF1bHRWYWx1ZTtcbiAgfVxuXG4gIGJhZ05hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICByZXR1cm4gW1wicmVxdWVzdFwiLCBcInJlcGx5XCJdO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBQcm9wQWN0aW9uIGV4dGVuZHMgUHJvcE1ldGhvZCB7XG4gIC8vIEFjdGlvbnMgaGF2ZSBhIFJlcXVlc3QgYW5kIGEgUmVzcG9uc2VcbiAgLy9cbiAgLy8gVGhlIFJlc3BvbnNlIGlzIGFsd2F5cyBgeyBlbnRpdHlFdmVudDogRW50aXR5RXZlbnQgfWA7XG4gIC8vXG4gIC8vIFRoZSBSZXF1ZXN0IGlzIG1hZGUgdXAgb2YgcHJvcGVydGllcyFcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBwYXJlbnROYW1lLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBwYXJlbnROYW1lPzogUHJvcFtcInBhcmVudE5hbWVcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogUHJvcEFjdGlvbltcImJhc2VEZWZhdWx0VmFsdWVcIl07XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSwgcGFyZW50TmFtZSwgZGVmYXVsdFZhbHVlIH0pO1xuICAgIHRoaXMucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJlbnRpdHlJZFwiLFxuICAgICAgbGFiZWw6IFwiRW50aXR5IElEXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJlbnRpdHlFdmVudFwiLFxuICAgICAgbGFiZWw6IGBFbnRpdHkgRXZlbnRgLFxuICAgICAgb3B0aW9ucyhwOiBQcm9wTGluaykge1xuICAgICAgICBwLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHAucmVhZE9ubHkgPSB0cnVlO1xuICAgICAgICBwLmxvb2t1cCA9IHtcbiAgICAgICAgICB0eXBlTmFtZTogYCR7dGhpcy5jb21wb25lbnRUeXBlTmFtZX1FdmVudGAsXG4gICAgICAgIH07XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImFjdGlvblwiO1xuICB9XG5cbiAgcHJvdG9idWZUeXBlKHN1ZmZpeCA9IFwiXCIpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHRoaXMubmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgc3VmZml4LFxuICAgICl9YDtcbiAgfVxufVxuIl19