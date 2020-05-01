"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropAction = exports.PropMethod = exports.PropObject = exports.AttrList = void 0;

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

function _typeof(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

function _inherits(subClass, superClass) { if (typeof superClass !== "function" && superClass !== null) { throw new TypeError("Super expression must either be null or a function"); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, writable: true, configurable: true } }); if (superClass) _setPrototypeOf(subClass, superClass); }

function _setPrototypeOf(o, p) { _setPrototypeOf = Object.setPrototypeOf || function _setPrototypeOf(o, p) { o.__proto__ = p; return o; }; return _setPrototypeOf(o, p); }

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = _getPrototypeOf(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = _getPrototypeOf(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return _possibleConstructorReturn(this, result); }; }

function _possibleConstructorReturn(self, call) { if (call && (_typeof(call) === "object" || typeof call === "function")) { return call; } return _assertThisInitialized(self); }

function _assertThisInitialized(self) { if (self === void 0) { throw new ReferenceError("this hasn't been initialised - super() hasn't been called"); } return self; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

function _getPrototypeOf(o) { _getPrototypeOf = Object.setPrototypeOf ? Object.getPrototypeOf : function _getPrototypeOf(o) { return o.__proto__ || Object.getPrototypeOf(o); }; return _getPrototypeOf(o); }

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

var AttrList = /*#__PURE__*/function () {
  function AttrList(_ref) {
    var parentName = _ref.parentName,
        readOnly = _ref.readOnly,
        componentTypeName = _ref.componentTypeName,
        autoCreateEdits = _ref.autoCreateEdits;

    _classCallCheck(this, AttrList);

    _defineProperty(this, "attrs", void 0);

    _defineProperty(this, "readOnly", void 0);

    _defineProperty(this, "parentName", void 0);

    _defineProperty(this, "autoCreateEdits", void 0);

    _defineProperty(this, "componentTypeName", void 0);

    this.parentName = parentName || "";
    this.attrs = [];
    this.componentTypeName = componentTypeName;
    this.readOnly = readOnly || false;
    this.autoCreateEdits = autoCreateEdits || false;
  }

  _createClass(AttrList, [{
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
  _inherits(PropObject, _Prop);

  var _super = _createSuper(PropObject);

  function PropObject(_ref2) {
    var _this;

    var name = _ref2.name,
        label = _ref2.label,
        componentTypeName = _ref2.componentTypeName,
        parentName = _ref2.parentName,
        defaultValue = _ref2.defaultValue;

    _classCallCheck(this, PropObject);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _defineProperty(_assertThisInitialized(_this), "properties", void 0);

    _defineProperty(_assertThisInitialized(_this), "realParentName", void 0);

    _this.baseDefaultValue = defaultValue || {};
    _this.parentName = parentName;
    _this.properties = new AttrList({
      parentName: "".concat((0, _changeCase.pascalCase)(parentName)).concat((0, _changeCase.pascalCase)(name)),
      componentTypeName: _this.componentTypeName
    });
    return _this;
  }

  _createClass(PropObject, [{
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
  _inherits(PropMethod, _Prop2);

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

    _classCallCheck(this, PropMethod);

    _this2 = _super2.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });

    _defineProperty(_assertThisInitialized(_this2), "baseDefaultValue", void 0);

    _defineProperty(_assertThisInitialized(_this2), "request", void 0);

    _defineProperty(_assertThisInitialized(_this2), "reply", void 0);

    _defineProperty(_assertThisInitialized(_this2), "realParentName", void 0);

    _defineProperty(_assertThisInitialized(_this2), "mutation", void 0);

    _defineProperty(_assertThisInitialized(_this2), "skipAuth", void 0);

    _defineProperty(_assertThisInitialized(_this2), "isPrivate", void 0);

    _this2.baseDefaultValue = defaultValue || {};
    _this2.parentName = parentName;
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

  _createClass(PropMethod, [{
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
  _inherits(PropAction, _PropMethod);

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

    _classCallCheck(this, PropAction);

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

  _createClass(PropAction, [{
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9hdHRyTGlzdC50cyJdLCJuYW1lcyI6WyJBdHRyTGlzdCIsInBhcmVudE5hbWUiLCJyZWFkT25seSIsImNvbXBvbmVudFR5cGVOYW1lIiwiYXV0b0NyZWF0ZUVkaXRzIiwiYXR0cnMiLCJsZW5ndGgiLCJuYW1lIiwicmVzdWx0IiwiZmluZCIsImUiLCJ1bmRlZmluZWQiLCJkZWZhdWx0VmFsdWVzIiwicmVzdWx0VmFsdWVzIiwiZW50cmllcyIsIml0ZW0iLCJkZWZhdWx0VmFsdWUiLCJ2YWx1ZXMiLCJraW5kIiwiUHJvcENvZGUiLCJyZWFsVmFsdWUiLCJwIiwicmVmZXJlbmNlIiwicHVzaCIsImFkZEFyZ3MiLCJvcHRpb25zIiwiYXV0b0NyZWF0ZUVkaXRBY3Rpb24iLCJQcm9wQm9vbCIsImFkZFByb3AiLCJQcm9wVGV4dCIsIlByb3BQYXNzd29yZCIsIlByb3BFbnVtIiwiUHJvcE51bWJlciIsIlByb3BMaW5rIiwiUHJvcE9iamVjdCIsIlByb3BBY3Rpb24iLCJQcm9wTWV0aG9kIiwiUHJvcE1hcCIsIm5vdEFsbG93ZWRLaW5kcyIsImluY2x1ZGVzIiwic3lzdGVtT2JqZWN0IiwicmVnaXN0cnkiLCJnZXQiLCJtZXRob2RzIiwiYWRkQWN0aW9uIiwibGFiZWwiLCJwYSIsInVuaXZlcnNhbCIsIm11dGF0aW9uIiwicmVxdWVzdCIsInByb3BlcnRpZXMiLCJhZGRMaW5rIiwicGwiLCJsb29rdXAiLCJ0eXBlTmFtZSIsIm5hbWVzIiwiYmFzZURlZmF1bHRWYWx1ZSIsInN1ZmZpeCIsIlByb3AiLCJyZXBseSIsInNraXBBdXRoIiwiaXNQcml2YXRlIiwiYWRkVGV4dCIsInJlcXVpcmVkIl0sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUE7O0FBQ0E7O0FBQ0E7O0FBRUE7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBQ0E7O0FBRUE7O0FBRUE7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0lBNkJhQSxRO0FBT1gsMEJBS3dCO0FBQUEsUUFK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LEVBQVZBLFVBQWxDO0FBQThDZSxNQUFBQSxZQUFZLEVBQVpBO0FBQTlDLEtBQU47O0FBQ0EsV0FBS2lDLE9BQUwsQ0FBYUMsVUFBYixDQUF3QlksT0FBeEIsQ0FBZ0M7QUFDOUJ2RCxNQUFBQSxJQUFJLEVBQUUsVUFEd0I7QUFFOUJzQyxNQUFBQSxLQUFLLEVBQUUsV0FGdUI7QUFHOUJwQixNQUFBQSxPQUg4QixtQkFHdEJKLENBSHNCLEVBR25CO0FBQ1RBLFFBQUFBLENBQUMsQ0FBQzBCLFNBQUYsR0FBYyxJQUFkO0FBQ0ExQixRQUFBQSxDQUFDLENBQUMwQyxRQUFGLEdBQWEsSUFBYjtBQUNEO0FBTjZCLEtBQWhDOztBQVFBLFdBQUtKLEtBQUwsQ0FBV1QsVUFBWCxDQUFzQkMsT0FBdEIsQ0FBOEI7QUFDNUI1QyxNQUFBQSxJQUFJLEVBQUUsYUFEc0I7QUFFNUJzQyxNQUFBQSxLQUFLLGdCQUZ1QjtBQUc1QnBCLE1BQUFBLE9BSDRCLG1CQUdwQkosQ0FIb0IsRUFHUDtBQUNuQkEsUUFBQUEsQ0FBQyxDQUFDMEIsU0FBRixHQUFjLElBQWQ7QUFDQTFCLFFBQUFBLENBQUMsQ0FBQ25CLFFBQUYsR0FBYSxJQUFiO0FBQ0FtQixRQUFBQSxDQUFDLENBQUNnQyxNQUFGLEdBQVc7QUFDVEMsVUFBQUEsUUFBUSxZQUFLLEtBQUtuRCxpQkFBVjtBQURDLFNBQVg7QUFHRDtBQVQyQixLQUE5Qjs7QUFWQztBQXFCRjs7OzsyQkFFYztBQUNiLGFBQU8sUUFBUDtBQUNEOzs7bUNBRWlDO0FBQUEsVUFBckJzRCxNQUFxQix1RUFBWixFQUFZO0FBQ2hDLHVCQUFVLDRCQUFXLEtBQUt4RCxVQUFoQixDQUFWLFNBQXdDLDRCQUFXLEtBQUtNLElBQWhCLENBQXhDLFNBQWdFLDRCQUM5RGtELE1BRDhELENBQWhFO0FBR0Q7Ozs7RUFsRDZCckIsVSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3AsIFByb3BEZWZhdWx0VmFsdWVzLCBQcm9wQ29uc3RydWN0b3IgfSBmcm9tIFwiLi9wcm9wXCI7XG5pbXBvcnQgeyBQcm9wVGV4dCB9IGZyb20gXCIuL3Byb3AvdGV4dFwiO1xuaW1wb3J0IHsgUHJvcENvZGUgfSBmcm9tIFwiLi9wcm9wL2NvZGVcIjtcbmltcG9ydCB7IFByb3BTZWxlY3QgfSBmcm9tIFwiLi9wcm9wL3NlbGVjdFwiO1xuaW1wb3J0IHsgUHJvcE51bWJlciB9IGZyb20gXCIuL3Byb3AvbnVtYmVyXCI7XG5pbXBvcnQgeyBQcm9wTWFwIH0gZnJvbSBcIi4vcHJvcC9tYXBcIjtcbmltcG9ydCB7IFByb3BFbnVtIH0gZnJvbSBcIi4vcHJvcC9lbnVtXCI7XG5pbXBvcnQgeyBQcm9wQm9vbCB9IGZyb20gXCIuL3Byb3AvYm9vbFwiO1xuaW1wb3J0IHsgUHJvcExpbmsgfSBmcm9tIFwiLi9wcm9wL2xpbmtcIjtcbmltcG9ydCB7IFByb3BQYXNzd29yZCB9IGZyb20gXCIuL3Byb3AvcGFzc3dvcmRcIjtcblxuaW1wb3J0IHsgcGFzY2FsQ2FzZSwgY2FtZWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5cbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4vcmVnaXN0cnlcIjtcblxuZXhwb3J0IHR5cGUgUHJvcHMgPVxuICB8IFByb3BUZXh0XG4gIHwgUHJvcFBhc3N3b3JkXG4gIHwgUHJvcFNlbGVjdFxuICB8IFByb3BDb2RlXG4gIHwgUHJvcE51bWJlclxuICB8IFByb3BPYmplY3RcbiAgfCBQcm9wTWFwXG4gIHwgUHJvcEVudW1cbiAgfCBQcm9wQm9vbFxuICB8IFByb3BMaW5rO1xuXG5pbnRlcmZhY2UgQWRkQXJndW1lbnRzIHtcbiAgbmFtZTogc3RyaW5nO1xuICBsYWJlbDogc3RyaW5nO1xuICBjb21wb25lbnRUeXBlTmFtZT86IHN0cmluZztcbiAgcGFyZW50TmFtZT86IHN0cmluZztcbiAgb3B0aW9ucz8ocDogUHJvcHMpOiB2b2lkO1xufVxuXG5pbnRlcmZhY2UgQXR0ckxpc3RDb25zdHJ1Y3RvciB7XG4gIGNvbXBvbmVudFR5cGVOYW1lPzogc3RyaW5nO1xuICBwYXJlbnROYW1lPzogc3RyaW5nO1xuICByZWFkT25seT86IGJvb2xlYW47XG4gIGF1dG9DcmVhdGVFZGl0cz86IGJvb2xlYW47XG59XG5cbmV4cG9ydCBjbGFzcyBBdHRyTGlzdCB7XG4gIGF0dHJzOiBQcm9wc1tdO1xuICByZWFkT25seTogYm9vbGVhbjtcbiAgcGFyZW50TmFtZTogc3RyaW5nO1xuICBhdXRvQ3JlYXRlRWRpdHM6IGJvb2xlYW47XG4gIGNvbXBvbmVudFR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIHBhcmVudE5hbWUsXG4gICAgcmVhZE9ubHksXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgYXV0b0NyZWF0ZUVkaXRzLFxuICB9OiBBdHRyTGlzdENvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy5wYXJlbnROYW1lID0gcGFyZW50TmFtZSB8fCBcIlwiO1xuICAgIHRoaXMuYXR0cnMgPSBbXTtcbiAgICB0aGlzLmNvbXBvbmVudFR5cGVOYW1lID0gY29tcG9uZW50VHlwZU5hbWU7XG4gICAgdGhpcy5yZWFkT25seSA9IHJlYWRPbmx5IHx8IGZhbHNlO1xuICAgIHRoaXMuYXV0b0NyZWF0ZUVkaXRzID0gYXV0b0NyZWF0ZUVkaXRzIHx8IGZhbHNlO1xuICB9XG5cbiAgZ2V0IGxlbmd0aCgpOiBudW1iZXIge1xuICAgIHJldHVybiB0aGlzLmF0dHJzLmxlbmd0aDtcbiAgfVxuXG4gIGhhc0VudHJpZXMoKTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIHRoaXMuYXR0cnMubGVuZ3RoID4gMDtcbiAgfVxuXG4gIGVudHJpZXMoKTogdGhpc1tcImF0dHJzXCJdIHtcbiAgICByZXR1cm4gdGhpcy5hdHRycztcbiAgfVxuXG4gIGdldEVudHJ5KG5hbWU6IHN0cmluZyk6IFByb3BzIHtcbiAgICBjb25zdCByZXN1bHQgPSB0aGlzLmF0dHJzLmZpbmQoZSA9PiBlLm5hbWUgPT0gbmFtZSk7XG4gICAgaWYgKHJlc3VsdCA9PSB1bmRlZmluZWQpIHtcbiAgICAgIHRocm93IGBDYW5ub3QgZmluZCBwcm9wZXJ0eSAke25hbWV9IGZvciAke3RoaXMuY29tcG9uZW50VHlwZU5hbWV9YDtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIGNyZWF0ZVZhbHVlT2JqZWN0KGRlZmF1bHRWYWx1ZXM/OiBQcm9wRGVmYXVsdFZhbHVlcyk6IFByb3BEZWZhdWx0VmFsdWVzIHtcbiAgICBjb25zdCByZXN1bHRWYWx1ZXMgPSBkZWZhdWx0VmFsdWVzIHx8IHt9O1xuICAgIGZvciAoY29uc3QgaXRlbSBvZiB0aGlzLmVudHJpZXMoKSkge1xuICAgICAgaWYgKHJlc3VsdFZhbHVlc1tpdGVtLm5hbWVdKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0gPSBpdGVtLmRlZmF1bHRWYWx1ZSgpO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0VmFsdWVzO1xuICB9XG5cbiAgcmVhbFZhbHVlcyh2YWx1ZXM6IFByb3BEZWZhdWx0VmFsdWVzKTogUHJvcERlZmF1bHRWYWx1ZXMge1xuICAgIGNvbnN0IHJlc3VsdFZhbHVlczogUHJvcERlZmF1bHRWYWx1ZXMgPSB7fTtcbiAgICBmb3IgKGNvbnN0IGl0ZW0gb2YgdGhpcy5lbnRyaWVzKCkpIHtcbiAgICAgIGlmIChpdGVtLmtpbmQoKSA9PSBcImNvZGVcIiAmJiBpdGVtIGluc3RhbmNlb2YgUHJvcENvZGUpIHtcbiAgICAgICAgaWYgKHZhbHVlc1tpdGVtLm5hbWVdKSB7XG4gICAgICAgICAgcmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0gPSBpdGVtLnJlYWxWYWx1ZSh2YWx1ZXNbaXRlbS5uYW1lXSk7XG4gICAgICAgIH1cbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdFZhbHVlc1tpdGVtLm5hbWVdID0gdmFsdWVzW2l0ZW0ubmFtZV07XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRWYWx1ZXM7XG4gIH1cblxuICBhZGRFeGlzdGluZyhwOiBQcm9wcyk6IHZvaWQge1xuICAgIHAucmVmZXJlbmNlID0gdHJ1ZTtcbiAgICB0aGlzLmF0dHJzLnB1c2gocCk7XG4gIH1cblxuICBhZGRQcm9wKHA6IFByb3BzLCBhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBpZiAoYWRkQXJncy5vcHRpb25zKSB7XG4gICAgICBhZGRBcmdzLm9wdGlvbnMocCk7XG4gICAgfVxuICAgIGlmICh0aGlzLnJlYWRPbmx5KSB7XG4gICAgICBwLnJlYWRPbmx5ID0gdGhpcy5yZWFkT25seTtcbiAgICB9XG4gICAgaWYgKHRoaXMuYXV0b0NyZWF0ZUVkaXRzKSB7XG4gICAgICB0aGlzLmF1dG9DcmVhdGVFZGl0QWN0aW9uKHApO1xuICAgIH1cbiAgICB0aGlzLmF0dHJzLnB1c2gocCk7XG4gIH1cblxuICBhZGRCb29sKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcEJvb2woYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZFRleHQoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wVGV4dChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkUGFzc3dvcmQoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wUGFzc3dvcmQoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZEVudW0oYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5wYXJlbnROYW1lID0gcGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpO1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcEVudW0oYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZE51bWJlcihhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BOdW1iZXIoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZExpbmsoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wTGluayhhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkT2JqZWN0KGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGFkZEFyZ3MucGFyZW50TmFtZSA9IHBhc2NhbENhc2UodGhpcy5wYXJlbnROYW1lKTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BPYmplY3QoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZEFjdGlvbihhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBhZGRBcmdzLnBhcmVudE5hbWUgPSBwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSk7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wQWN0aW9uKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRNZXRob2QoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgYWRkQXJncy5wYXJlbnROYW1lID0gcGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE1ldGhvZChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkTWFwKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE1hcChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkQ29kZShhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BDb2RlKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhdXRvQ3JlYXRlRWRpdEFjdGlvbihwOiBQcm9wcyk6IHZvaWQge1xuICAgIGNvbnN0IG5vdEFsbG93ZWRLaW5kcyA9IFtcIm1ldGhvZFwiLCBcImFjdGlvblwiXTtcbiAgICBpZiAobm90QWxsb3dlZEtpbmRzLmluY2x1ZGVzKHAua2luZCgpKSkge1xuICAgICAgcmV0dXJuO1xuICAgIH1cbiAgICBjb25zdCBzeXN0ZW1PYmplY3QgPSByZWdpc3RyeS5nZXQocC5jb21wb25lbnRUeXBlTmFtZSk7XG5cbiAgICBzeXN0ZW1PYmplY3QubWV0aG9kcy5hZGRBY3Rpb24oe1xuICAgICAgbmFtZTogYCR7Y2FtZWxDYXNlKHAubmFtZSl9RWRpdGAsXG4gICAgICBsYWJlbDogYEVkaXQgJHtjYW1lbENhc2UocC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UocC5uYW1lKX0gUHJvcGVydHlgLFxuICAgICAgb3B0aW9ucyhwYTogUHJvcEFjdGlvbikge1xuICAgICAgICBwYS51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwYS5tdXRhdGlvbiA9IHRydWU7XG4gICAgICAgIHBhLnJlcXVlc3QucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgICAgICBuYW1lOiBcInByb3BlcnR5XCIsXG4gICAgICAgICAgbGFiZWw6IGBUaGUgJHtwLmxhYmVsfSBwcm9wZXJ0eSB2YWx1ZWAsXG4gICAgICAgICAgb3B0aW9ucyhwbDogUHJvcExpbmspIHtcbiAgICAgICAgICAgIHBsLmxvb2t1cCA9IHtcbiAgICAgICAgICAgICAgdHlwZU5hbWU6IHAuY29tcG9uZW50VHlwZU5hbWUsXG4gICAgICAgICAgICAgIG5hbWVzOiBbXCJwcm9wZXJ0aWVzXCIsIHAubmFtZV0sXG4gICAgICAgICAgICB9O1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUHJvcE9iamVjdCBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICBwcm9wZXJ0aWVzOiBBdHRyTGlzdDtcbiAgcmVhbFBhcmVudE5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBwYXJlbnROYW1lLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBwYXJlbnROYW1lPzogUHJvcFtcInBhcmVudE5hbWVcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogUHJvcE9iamVjdFtcImJhc2VEZWZhdWx0VmFsdWVcIl07XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSB9KTtcbiAgICB0aGlzLmJhc2VEZWZhdWx0VmFsdWUgPSBkZWZhdWx0VmFsdWUgfHwge307XG4gICAgdGhpcy5wYXJlbnROYW1lID0gcGFyZW50TmFtZTtcbiAgICB0aGlzLnByb3BlcnRpZXMgPSBuZXcgQXR0ckxpc3Qoe1xuICAgICAgcGFyZW50TmFtZTogYCR7cGFzY2FsQ2FzZShwYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UobmFtZSl9YCxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0aGlzLmNvbXBvbmVudFR5cGVOYW1lLFxuICAgIH0pO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcIm9iamVjdFwiO1xuICB9XG5cbiAgcHJvdG9idWZUeXBlKHN1ZmZpeCA9IFwiXCIpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHRoaXMubmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgc3VmZml4LFxuICAgICl9YDtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wT2JqZWN0W1wiYmFzZURlZmF1bHRWYWx1ZVwiXSB7XG4gICAgcmV0dXJuIHRoaXMuYmFzZURlZmF1bHRWYWx1ZTtcbiAgfVxuXG4gIGJhZ05hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICByZXR1cm4gW1wicHJvcGVydGllc1wiXTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUHJvcE1ldGhvZCBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICByZXF1ZXN0OiBQcm9wT2JqZWN0O1xuICByZXBseTogUHJvcE9iamVjdDtcbiAgcmVhbFBhcmVudE5hbWU6IHN0cmluZztcbiAgbXV0YXRpb246IGJvb2xlYW47XG4gIHNraXBBdXRoOiBib29sZWFuO1xuICBpc1ByaXZhdGU6IGJvb2xlYW47XG5cbiAgLy8gTWV0aG9kcyBoYXZlIGEgUmVxdWVzdCBhbmQgYSBSZXNwb25zZVxuICAvL1xuICAvLyBUaGUgUmVxdWVzdCBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG4gIC8vIFRoZSBSZXBseSBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcGFyZW50TmFtZSxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcGFyZW50TmFtZT86IFByb3BbXCJwYXJlbnROYW1lXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IFByb3BBY3Rpb25bXCJiYXNlRGVmYXVsdFZhbHVlXCJdO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUgfSk7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IHt9O1xuICAgIHRoaXMucGFyZW50TmFtZSA9IHBhcmVudE5hbWU7XG4gICAgdGhpcy5yZXF1ZXN0ID0gbmV3IFByb3BPYmplY3Qoe1xuICAgICAgbmFtZTogYCR7cGFzY2FsQ2FzZShuYW1lKX1SZXF1ZXN0YCxcbiAgICAgIGxhYmVsOiBgJHtsYWJlbH0gUmVxdWVzdGAsXG4gICAgICBwYXJlbnROYW1lOiB0aGlzLnBhcmVudE5hbWUsXG4gICAgICBjb21wb25lbnRUeXBlTmFtZTogdGhpcy5jb21wb25lbnRUeXBlTmFtZSxcbiAgICB9KTtcbiAgICB0aGlzLnJlcGx5ID0gbmV3IFByb3BPYmplY3Qoe1xuICAgICAgbmFtZTogYCR7cGFzY2FsQ2FzZShuYW1lKX1SZXBseWAsXG4gICAgICBsYWJlbDogYCR7bGFiZWx9IFJlcGx5YCxcbiAgICAgIHBhcmVudE5hbWU6IHRoaXMucGFyZW50TmFtZSxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0aGlzLmNvbXBvbmVudFR5cGVOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMubXV0YXRpb24gPSBmYWxzZTtcbiAgICB0aGlzLnNraXBBdXRoID0gZmFsc2U7XG4gICAgdGhpcy5pc1ByaXZhdGUgPSBmYWxzZTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJtZXRob2RcIjtcbiAgfVxuXG4gIHByb3RvYnVmVHlwZShzdWZmaXggPSBcIlwiKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7cGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZSh0aGlzLm5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgIHN1ZmZpeCxcbiAgICApfWA7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcE9iamVjdFtcImJhc2VEZWZhdWx0VmFsdWVcIl0ge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cblxuICBiYWdOYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgcmV0dXJuIFtcInJlcXVlc3RcIiwgXCJyZXBseVwiXTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUHJvcEFjdGlvbiBleHRlbmRzIFByb3BNZXRob2Qge1xuICAvLyBBY3Rpb25zIGhhdmUgYSBSZXF1ZXN0IGFuZCBhIFJlc3BvbnNlXG4gIC8vXG4gIC8vIFRoZSBSZXNwb25zZSBpcyBhbHdheXMgYHsgZW50aXR5RXZlbnQ6IEVudGl0eUV2ZW50IH1gO1xuICAvL1xuICAvLyBUaGUgUmVxdWVzdCBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcGFyZW50TmFtZSxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcGFyZW50TmFtZT86IFByb3BbXCJwYXJlbnROYW1lXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IFByb3BBY3Rpb25bXCJiYXNlRGVmYXVsdFZhbHVlXCJdO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUsIHBhcmVudE5hbWUsIGRlZmF1bHRWYWx1ZSB9KTtcbiAgICB0aGlzLnJlcXVlc3QucHJvcGVydGllcy5hZGRUZXh0KHtcbiAgICAgIG5hbWU6IFwiZW50aXR5SWRcIixcbiAgICAgIGxhYmVsOiBcIkVudGl0eSBJRFwiLFxuICAgICAgb3B0aW9ucyhwKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZXF1aXJlZCA9IHRydWU7XG4gICAgICB9LFxuICAgIH0pO1xuICAgIHRoaXMucmVwbHkucHJvcGVydGllcy5hZGRMaW5rKHtcbiAgICAgIG5hbWU6IFwiZW50aXR5RXZlbnRcIixcbiAgICAgIGxhYmVsOiBgRW50aXR5IEV2ZW50YCxcbiAgICAgIG9wdGlvbnMocDogUHJvcExpbmspIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlYWRPbmx5ID0gdHJ1ZTtcbiAgICAgICAgcC5sb29rdXAgPSB7XG4gICAgICAgICAgdHlwZU5hbWU6IGAke3RoaXMuY29tcG9uZW50VHlwZU5hbWV9RXZlbnRgLFxuICAgICAgICB9O1xuICAgICAgfSxcbiAgICB9KTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJhY3Rpb25cIjtcbiAgfVxuXG4gIHByb3RvYnVmVHlwZShzdWZmaXggPSBcIlwiKTogc3RyaW5nIHtcbiAgICByZXR1cm4gYCR7cGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZSh0aGlzLm5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgIHN1ZmZpeCxcbiAgICApfWA7XG4gIH1cbn1cbiJdfQ==