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
        throw new Error("Cannot find property ".concat(name, " for ").concat(this.componentTypeName));
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9hdHRyTGlzdC50cyJdLCJuYW1lcyI6WyJBdHRyTGlzdCIsInBhcmVudE5hbWUiLCJyZWFkT25seSIsImNvbXBvbmVudFR5cGVOYW1lIiwiYXV0b0NyZWF0ZUVkaXRzIiwiYXR0cnMiLCJsZW5ndGgiLCJuYW1lIiwicmVzdWx0IiwiZmluZCIsImUiLCJ1bmRlZmluZWQiLCJFcnJvciIsImRlZmF1bHRWYWx1ZXMiLCJyZXN1bHRWYWx1ZXMiLCJlbnRyaWVzIiwiaXRlbSIsImRlZmF1bHRWYWx1ZSIsInZhbHVlcyIsImtpbmQiLCJQcm9wQ29kZSIsInJlYWxWYWx1ZSIsInAiLCJyZWZlcmVuY2UiLCJwdXNoIiwiYWRkQXJncyIsIm9wdGlvbnMiLCJhdXRvQ3JlYXRlRWRpdEFjdGlvbiIsIlByb3BCb29sIiwiYWRkUHJvcCIsIlByb3BUZXh0IiwiUHJvcFBhc3N3b3JkIiwiUHJvcEVudW0iLCJQcm9wTnVtYmVyIiwiUHJvcExpbmsiLCJQcm9wT2JqZWN0IiwiUHJvcEFjdGlvbiIsIlByb3BNZXRob2QiLCJQcm9wTWFwIiwibm90QWxsb3dlZEtpbmRzIiwiaW5jbHVkZXMiLCJzeXN0ZW1PYmplY3QiLCJyZWdpc3RyeSIsImdldCIsIm1ldGhvZHMiLCJhZGRBY3Rpb24iLCJsYWJlbCIsInBhIiwidW5pdmVyc2FsIiwibXV0YXRpb24iLCJyZXF1ZXN0IiwicHJvcGVydGllcyIsImFkZExpbmsiLCJwbCIsImxvb2t1cCIsInR5cGVOYW1lIiwibmFtZXMiLCJiYXNlRGVmYXVsdFZhbHVlIiwic3VmZml4IiwiUHJvcCIsInJlcGx5Iiwic2tpcEF1dGgiLCJpc1ByaXZhdGUiLCJpbnRlZ3JhdGlvblNlcnZpY2VzIiwiYWRkVGV4dCIsInJlcXVpcmVkIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFBOztBQUNBOztBQUNBOztBQUVBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUNBOztBQUVBOztBQUVBOzs7Ozs7Ozs7Ozs7SUFr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dGVudCI6WyJpbXBvcnQgeyBQcm9wLCBQcm9wRGVmYXVsdFZhbHVlcywgUHJvcENvbnN0cnVjdG9yIH0gZnJvbSBcIi4vcHJvcFwiO1xuaW1wb3J0IHsgUHJvcFRleHQgfSBmcm9tIFwiLi9wcm9wL3RleHRcIjtcbmltcG9ydCB7IFByb3BDb2RlIH0gZnJvbSBcIi4vcHJvcC9jb2RlXCI7XG5pbXBvcnQgeyBQcm9wU2VsZWN0IH0gZnJvbSBcIi4vcHJvcC9zZWxlY3RcIjtcbmltcG9ydCB7IFByb3BOdW1iZXIgfSBmcm9tIFwiLi9wcm9wL251bWJlclwiO1xuaW1wb3J0IHsgUHJvcE1hcCB9IGZyb20gXCIuL3Byb3AvbWFwXCI7XG5pbXBvcnQgeyBQcm9wRW51bSB9IGZyb20gXCIuL3Byb3AvZW51bVwiO1xuaW1wb3J0IHsgUHJvcEJvb2wgfSBmcm9tIFwiLi9wcm9wL2Jvb2xcIjtcbmltcG9ydCB7IFByb3BMaW5rIH0gZnJvbSBcIi4vcHJvcC9saW5rXCI7XG5pbXBvcnQgeyBQcm9wUGFzc3dvcmQgfSBmcm9tIFwiLi9wcm9wL3Bhc3N3b3JkXCI7XG5cbmltcG9ydCB7IHBhc2NhbENhc2UsIGNhbWVsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuXG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuL3JlZ2lzdHJ5XCI7XG5cbmV4cG9ydCB0eXBlIFByb3BzID1cbiAgfCBQcm9wVGV4dFxuICB8IFByb3BQYXNzd29yZFxuICB8IFByb3BTZWxlY3RcbiAgfCBQcm9wQ29kZVxuICB8IFByb3BOdW1iZXJcbiAgfCBQcm9wT2JqZWN0XG4gIHwgUHJvcE1hcFxuICB8IFByb3BFbnVtXG4gIHwgUHJvcEJvb2xcbiAgfCBQcm9wTGluaztcblxuaW50ZXJmYWNlIEFkZEFyZ3VtZW50cyB7XG4gIG5hbWU6IHN0cmluZztcbiAgbGFiZWw6IHN0cmluZztcbiAgY29tcG9uZW50VHlwZU5hbWU/OiBzdHJpbmc7XG4gIHBhcmVudE5hbWU/OiBzdHJpbmc7XG4gIG9wdGlvbnM/KHA6IFByb3BzKTogdm9pZDtcbn1cblxuaW50ZXJmYWNlIEF0dHJMaXN0Q29uc3RydWN0b3Ige1xuICBjb21wb25lbnRUeXBlTmFtZT86IHN0cmluZztcbiAgcGFyZW50TmFtZT86IHN0cmluZztcbiAgcmVhZE9ubHk/OiBib29sZWFuO1xuICBhdXRvQ3JlYXRlRWRpdHM/OiBib29sZWFuO1xufVxuXG5leHBvcnQgaW50ZXJmYWNlIEludGVncmF0aW9uU2VydmljZSB7XG4gIGludGVncmF0aW9uTmFtZTogc3RyaW5nO1xuICBpbnRlZ3JhdGlvblNlcnZpY2VOYW1lOiBzdHJpbmc7XG59XG5cbmV4cG9ydCBjbGFzcyBBdHRyTGlzdCB7XG4gIGF0dHJzOiBQcm9wc1tdO1xuICByZWFkT25seTogYm9vbGVhbjtcbiAgcGFyZW50TmFtZTogc3RyaW5nO1xuICBhdXRvQ3JlYXRlRWRpdHM6IGJvb2xlYW47XG4gIGNvbXBvbmVudFR5cGVOYW1lOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIHBhcmVudE5hbWUsXG4gICAgcmVhZE9ubHksXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgYXV0b0NyZWF0ZUVkaXRzLFxuICB9OiBBdHRyTGlzdENvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy5wYXJlbnROYW1lID0gcGFyZW50TmFtZSB8fCBcIlwiO1xuICAgIHRoaXMuYXR0cnMgPSBbXTtcbiAgICB0aGlzLmNvbXBvbmVudFR5cGVOYW1lID0gY29tcG9uZW50VHlwZU5hbWUgfHwgXCJcIjtcbiAgICB0aGlzLnJlYWRPbmx5ID0gcmVhZE9ubHkgfHwgZmFsc2U7XG4gICAgdGhpcy5hdXRvQ3JlYXRlRWRpdHMgPSBhdXRvQ3JlYXRlRWRpdHMgfHwgZmFsc2U7XG4gIH1cblxuICBnZXQgbGVuZ3RoKCk6IG51bWJlciB7XG4gICAgcmV0dXJuIHRoaXMuYXR0cnMubGVuZ3RoO1xuICB9XG5cbiAgaGFzRW50cmllcygpOiBib29sZWFuIHtcbiAgICByZXR1cm4gdGhpcy5hdHRycy5sZW5ndGggPiAwO1xuICB9XG5cbiAgZW50cmllcygpOiB0aGlzW1wiYXR0cnNcIl0ge1xuICAgIHJldHVybiB0aGlzLmF0dHJzO1xuICB9XG5cbiAgZ2V0RW50cnkobmFtZTogc3RyaW5nKTogUHJvcHMge1xuICAgIGNvbnN0IHJlc3VsdCA9IHRoaXMuYXR0cnMuZmluZChlID0+IGUubmFtZSA9PSBuYW1lKTtcbiAgICBpZiAocmVzdWx0ID09IHVuZGVmaW5lZCkge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBgQ2Fubm90IGZpbmQgcHJvcGVydHkgJHtuYW1lfSBmb3IgJHt0aGlzLmNvbXBvbmVudFR5cGVOYW1lfWAsXG4gICAgICApO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgY3JlYXRlVmFsdWVPYmplY3QoZGVmYXVsdFZhbHVlcz86IFByb3BEZWZhdWx0VmFsdWVzKTogUHJvcERlZmF1bHRWYWx1ZXMge1xuICAgIGNvbnN0IHJlc3VsdFZhbHVlcyA9IGRlZmF1bHRWYWx1ZXMgfHwge307XG4gICAgZm9yIChjb25zdCBpdGVtIG9mIHRoaXMuZW50cmllcygpKSB7XG4gICAgICBpZiAocmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0pIHtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHRWYWx1ZXNbaXRlbS5uYW1lXSA9IGl0ZW0uZGVmYXVsdFZhbHVlKCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRWYWx1ZXM7XG4gIH1cblxuICByZWFsVmFsdWVzKHZhbHVlczogUHJvcERlZmF1bHRWYWx1ZXMpOiBQcm9wRGVmYXVsdFZhbHVlcyB7XG4gICAgY29uc3QgcmVzdWx0VmFsdWVzOiBQcm9wRGVmYXVsdFZhbHVlcyA9IHt9O1xuICAgIGZvciAoY29uc3QgaXRlbSBvZiB0aGlzLmVudHJpZXMoKSkge1xuICAgICAgaWYgKGl0ZW0ua2luZCgpID09IFwiY29kZVwiICYmIGl0ZW0gaW5zdGFuY2VvZiBQcm9wQ29kZSkge1xuICAgICAgICBpZiAodmFsdWVzW2l0ZW0ubmFtZV0pIHtcbiAgICAgICAgICByZXN1bHRWYWx1ZXNbaXRlbS5uYW1lXSA9IGl0ZW0ucmVhbFZhbHVlKHZhbHVlc1tpdGVtLm5hbWVdKTtcbiAgICAgICAgfVxuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0VmFsdWVzW2l0ZW0ubmFtZV0gPSB2YWx1ZXNbaXRlbS5uYW1lXTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdFZhbHVlcztcbiAgfVxuXG4gIGFkZEV4aXN0aW5nKHA6IFByb3BzKTogdm9pZCB7XG4gICAgcC5yZWZlcmVuY2UgPSB0cnVlO1xuICAgIHRoaXMuYXR0cnMucHVzaChwKTtcbiAgfVxuXG4gIGFkZFByb3AocDogUHJvcHMsIGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGlmIChhZGRBcmdzLm9wdGlvbnMpIHtcbiAgICAgIGFkZEFyZ3Mub3B0aW9ucyhwKTtcbiAgICB9XG4gICAgaWYgKHRoaXMucmVhZE9ubHkpIHtcbiAgICAgIHAucmVhZE9ubHkgPSB0aGlzLnJlYWRPbmx5O1xuICAgIH1cbiAgICBpZiAodGhpcy5hdXRvQ3JlYXRlRWRpdHMpIHtcbiAgICAgIHRoaXMuYXV0b0NyZWF0ZUVkaXRBY3Rpb24ocCk7XG4gICAgfVxuICAgIHRoaXMuYXR0cnMucHVzaChwKTtcbiAgfVxuXG4gIGFkZEJvb2woYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wQm9vbChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkVGV4dChhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BUZXh0KGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRQYXNzd29yZChhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BQYXNzd29yZChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkRW51bShhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLnBhcmVudE5hbWUgPSBwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSk7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wRW51bShhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkTnVtYmVyKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE51bWJlcihhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkTGluayhhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BMaW5rKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRPYmplY3QoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgYWRkQXJncy5wYXJlbnROYW1lID0gcGFzY2FsQ2FzZSh0aGlzLnBhcmVudE5hbWUpO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcE9iamVjdChhZGRBcmdzIGFzIFByb3BDb25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5hZGRQcm9wKHAsIGFkZEFyZ3MpO1xuICB9XG5cbiAgYWRkQWN0aW9uKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGFkZEFyZ3MucGFyZW50TmFtZSA9IHBhc2NhbENhc2UodGhpcy5wYXJlbnROYW1lKTtcbiAgICBjb25zdCBwID0gbmV3IFByb3BBY3Rpb24oYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGFkZE1ldGhvZChhZGRBcmdzOiBBZGRBcmd1bWVudHMpOiB2b2lkIHtcbiAgICBhZGRBcmdzLmNvbXBvbmVudFR5cGVOYW1lID0gdGhpcy5jb21wb25lbnRUeXBlTmFtZTtcbiAgICBhZGRBcmdzLnBhcmVudE5hbWUgPSBwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSk7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wTWV0aG9kKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRNYXAoYWRkQXJnczogQWRkQXJndW1lbnRzKTogdm9pZCB7XG4gICAgYWRkQXJncy5jb21wb25lbnRUeXBlTmFtZSA9IHRoaXMuY29tcG9uZW50VHlwZU5hbWU7XG4gICAgY29uc3QgcCA9IG5ldyBQcm9wTWFwKGFkZEFyZ3MgYXMgUHJvcENvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmFkZFByb3AocCwgYWRkQXJncyk7XG4gIH1cblxuICBhZGRDb2RlKGFkZEFyZ3M6IEFkZEFyZ3VtZW50cyk6IHZvaWQge1xuICAgIGFkZEFyZ3MuY29tcG9uZW50VHlwZU5hbWUgPSB0aGlzLmNvbXBvbmVudFR5cGVOYW1lO1xuICAgIGNvbnN0IHAgPSBuZXcgUHJvcENvZGUoYWRkQXJncyBhcyBQcm9wQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuYWRkUHJvcChwLCBhZGRBcmdzKTtcbiAgfVxuXG4gIGF1dG9DcmVhdGVFZGl0QWN0aW9uKHA6IFByb3BzKTogdm9pZCB7XG4gICAgY29uc3Qgbm90QWxsb3dlZEtpbmRzID0gW1wibWV0aG9kXCIsIFwiYWN0aW9uXCJdO1xuICAgIGlmIChub3RBbGxvd2VkS2luZHMuaW5jbHVkZXMocC5raW5kKCkpKSB7XG4gICAgICByZXR1cm47XG4gICAgfVxuICAgIGNvbnN0IHN5c3RlbU9iamVjdCA9IHJlZ2lzdHJ5LmdldChwLmNvbXBvbmVudFR5cGVOYW1lKTtcblxuICAgIHN5c3RlbU9iamVjdC5tZXRob2RzLmFkZEFjdGlvbih7XG4gICAgICBuYW1lOiBgJHtjYW1lbENhc2UocC5uYW1lKX1FZGl0YCxcbiAgICAgIGxhYmVsOiBgRWRpdCAke2NhbWVsQ2FzZShwLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShwLm5hbWUpfSBQcm9wZXJ0eWAsXG4gICAgICBvcHRpb25zKHBhOiBQcm9wQWN0aW9uKSB7XG4gICAgICAgIHBhLnVuaXZlcnNhbCA9IHRydWU7XG4gICAgICAgIHBhLm11dGF0aW9uID0gdHJ1ZTtcbiAgICAgICAgcGEucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgICAgIG5hbWU6IFwicHJvcGVydHlcIixcbiAgICAgICAgICBsYWJlbDogYFRoZSAke3AubGFiZWx9IHByb3BlcnR5IHZhbHVlYCxcbiAgICAgICAgICBvcHRpb25zKHBsOiBQcm9wTGluaykge1xuICAgICAgICAgICAgcGwubG9va3VwID0ge1xuICAgICAgICAgICAgICB0eXBlTmFtZTogcC5jb21wb25lbnRUeXBlTmFtZSxcbiAgICAgICAgICAgICAgbmFtZXM6IFtcInByb3BlcnRpZXNcIiwgcC5uYW1lXSxcbiAgICAgICAgICAgIH07XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9LFxuICAgIH0pO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBQcm9wT2JqZWN0IGV4dGVuZHMgUHJvcCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IFJlY29yZDxzdHJpbmcsIGFueT47XG4gIHByb3BlcnRpZXM6IEF0dHJMaXN0O1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHBhcmVudE5hbWUsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHBhcmVudE5hbWU/OiBQcm9wW1wicGFyZW50TmFtZVwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBQcm9wT2JqZWN0W1wiYmFzZURlZmF1bHRWYWx1ZVwiXTtcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCB7fTtcbiAgICB0aGlzLnBhcmVudE5hbWUgPSBwYXJlbnROYW1lIHx8IFwiXCI7XG4gICAgdGhpcy5wcm9wZXJ0aWVzID0gbmV3IEF0dHJMaXN0KHtcbiAgICAgIHBhcmVudE5hbWU6IGAke3Bhc2NhbENhc2UodGhpcy5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UobmFtZSl9YCxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0aGlzLmNvbXBvbmVudFR5cGVOYW1lLFxuICAgIH0pO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcIm9iamVjdFwiO1xuICB9XG5cbiAgcHJvdG9idWZUeXBlKHN1ZmZpeCA9IFwiXCIpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHRoaXMubmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgc3VmZml4LFxuICAgICl9YDtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wT2JqZWN0W1wiYmFzZURlZmF1bHRWYWx1ZVwiXSB7XG4gICAgcmV0dXJuIHRoaXMuYmFzZURlZmF1bHRWYWx1ZTtcbiAgfVxuXG4gIGJhZ05hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICByZXR1cm4gW1wicHJvcGVydGllc1wiXTtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgUHJvcE1ldGhvZCBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICByZXF1ZXN0OiBQcm9wT2JqZWN0O1xuICByZXBseTogUHJvcE9iamVjdDtcbiAgbXV0YXRpb246IGJvb2xlYW47XG4gIHNraXBBdXRoOiBib29sZWFuO1xuICBpc1ByaXZhdGU6IGJvb2xlYW47XG5cbiAgLy8gTWV0aG9kcyBoYXZlIGEgUmVxdWVzdCBhbmQgYSBSZXNwb25zZVxuICAvL1xuICAvLyBUaGUgUmVxdWVzdCBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG4gIC8vIFRoZSBSZXBseSBpcyBtYWRlIHVwIG9mIHByb3BlcnRpZXMhXG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcGFyZW50TmFtZSxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcGFyZW50TmFtZT86IFByb3BbXCJwYXJlbnROYW1lXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IFByb3BBY3Rpb25bXCJiYXNlRGVmYXVsdFZhbHVlXCJdO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUgfSk7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IHt9O1xuICAgIHRoaXMucGFyZW50TmFtZSA9IHBhcmVudE5hbWUgfHwgXCJcIjtcbiAgICB0aGlzLnJlcXVlc3QgPSBuZXcgUHJvcE9iamVjdCh7XG4gICAgICBuYW1lOiBgJHtwYXNjYWxDYXNlKG5hbWUpfVJlcXVlc3RgLFxuICAgICAgbGFiZWw6IGAke2xhYmVsfSBSZXF1ZXN0YCxcbiAgICAgIHBhcmVudE5hbWU6IHRoaXMucGFyZW50TmFtZSxcbiAgICAgIGNvbXBvbmVudFR5cGVOYW1lOiB0aGlzLmNvbXBvbmVudFR5cGVOYW1lLFxuICAgIH0pO1xuICAgIHRoaXMucmVwbHkgPSBuZXcgUHJvcE9iamVjdCh7XG4gICAgICBuYW1lOiBgJHtwYXNjYWxDYXNlKG5hbWUpfVJlcGx5YCxcbiAgICAgIGxhYmVsOiBgJHtsYWJlbH0gUmVwbHlgLFxuICAgICAgcGFyZW50TmFtZTogdGhpcy5wYXJlbnROYW1lLFxuICAgICAgY29tcG9uZW50VHlwZU5hbWU6IHRoaXMuY29tcG9uZW50VHlwZU5hbWUsXG4gICAgfSk7XG4gICAgdGhpcy5tdXRhdGlvbiA9IGZhbHNlO1xuICAgIHRoaXMuc2tpcEF1dGggPSBmYWxzZTtcbiAgICB0aGlzLmlzUHJpdmF0ZSA9IGZhbHNlO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcIm1ldGhvZFwiO1xuICB9XG5cbiAgcHJvdG9idWZUeXBlKHN1ZmZpeCA9IFwiXCIpOiBzdHJpbmcge1xuICAgIHJldHVybiBgJHtwYXNjYWxDYXNlKHRoaXMucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKHRoaXMubmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgc3VmZml4LFxuICAgICl9YDtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wT2JqZWN0W1wiYmFzZURlZmF1bHRWYWx1ZVwiXSB7XG4gICAgcmV0dXJuIHRoaXMuYmFzZURlZmF1bHRWYWx1ZTtcbiAgfVxuXG4gIGJhZ05hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICByZXR1cm4gW1wicmVxdWVzdFwiLCBcInJlcGx5XCJdO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBQcm9wQWN0aW9uIGV4dGVuZHMgUHJvcE1ldGhvZCB7XG4gIGludGVncmF0aW9uU2VydmljZXM6IEludGVncmF0aW9uU2VydmljZVtdO1xuXG4gIC8vIEFjdGlvbnMgaGF2ZSBhIFJlcXVlc3QgYW5kIGEgUmVzcG9uc2VcbiAgLy9cbiAgLy8gVGhlIFJlc3BvbnNlIGlzIGFsd2F5cyBgeyBlbnRpdHlFdmVudDogRW50aXR5RXZlbnQgfWA7XG4gIC8vXG4gIC8vIFRoZSBSZXF1ZXN0IGlzIG1hZGUgdXAgb2YgcHJvcGVydGllcyFcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBwYXJlbnROYW1lLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBwYXJlbnROYW1lPzogUHJvcFtcInBhcmVudE5hbWVcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogUHJvcEFjdGlvbltcImJhc2VEZWZhdWx0VmFsdWVcIl07XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSwgcGFyZW50TmFtZSwgZGVmYXVsdFZhbHVlIH0pO1xuICAgIHRoaXMuaW50ZWdyYXRpb25TZXJ2aWNlcyA9IFtdO1xuICAgIHRoaXMucmVxdWVzdC5wcm9wZXJ0aWVzLmFkZFRleHQoe1xuICAgICAgbmFtZTogXCJpZFwiLFxuICAgICAgbGFiZWw6IFwiRW50aXR5IElEXCIsXG4gICAgICBvcHRpb25zKHApIHtcbiAgICAgICAgcC51bml2ZXJzYWwgPSB0cnVlO1xuICAgICAgICBwLnJlcXVpcmVkID0gdHJ1ZTtcbiAgICAgIH0sXG4gICAgfSk7XG4gICAgdGhpcy5yZXBseS5wcm9wZXJ0aWVzLmFkZExpbmsoe1xuICAgICAgbmFtZTogXCJpdGVtXCIsXG4gICAgICBsYWJlbDogYEVudGl0eSBFdmVudGAsXG4gICAgICBvcHRpb25zKHA6IFByb3BMaW5rKSB7XG4gICAgICAgIHAudW5pdmVyc2FsID0gdHJ1ZTtcbiAgICAgICAgcC5yZWFkT25seSA9IHRydWU7XG4gICAgICAgIHAubG9va3VwID0ge1xuICAgICAgICAgIHR5cGVOYW1lOiBgJHt0aGlzLmNvbXBvbmVudFR5cGVOYW1lfUV2ZW50YCxcbiAgICAgICAgfTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiYWN0aW9uXCI7XG4gIH1cblxuICBwcm90b2J1ZlR5cGUoc3VmZml4ID0gXCJcIik6IHN0cmluZyB7XG4gICAgcmV0dXJuIGAke3Bhc2NhbENhc2UodGhpcy5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UodGhpcy5uYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICBzdWZmaXgsXG4gICAgKX1gO1xuICB9XG59XG4iXX0=