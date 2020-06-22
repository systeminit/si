"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropCode = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _toml = _interopRequireDefault(require("@iarna/toml"));

var _prop = require("../prop");

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var PropCode = /*#__PURE__*/function (_Prop) {
  (0, _inherits2["default"])(PropCode, _Prop);

  var _super = _createSuper(PropCode);

  function PropCode(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        parsed = _ref.parsed,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;
    (0, _classCallCheck2["default"])(this, PropCode);
    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "baseDefaultValue", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "language", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "parsed", void 0);
    _this.baseDefaultValue = defaultValue || "";
    _this.parsed = parsed || false;
    _this.language = "autodetect";
    return _this;
  }

  (0, _createClass2["default"])(PropCode, [{
    key: "kind",
    value: function kind() {
      return "code";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }, {
    key: "realValue",
    value: function realValue(value) {
      if (value === null) {
        return null;
      }

      if (this.parsed) {
        if (this.language == "toml" && typeof value == "string") {
          var objectData = _toml["default"].parse(value);

          return objectData;
        } else {
          throw new Error("Do not know how to parse this thing");
        }
      } else {
        return value;
      }
    }
  }]);
  return PropCode;
}(_prop.Prop);

exports.PropCode = PropCode;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2NvZGUudHMiXSwibmFtZXMiOlsiUHJvcENvZGUiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInBhcnNlZCIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwibGFuZ3VhZ2UiLCJ2YWx1ZSIsIm9iamVjdERhdGEiLCJUT01MIiwicGFyc2UiLCJFcnJvciIsIlByb3AiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0FBRUE7Ozs7OztJQU9hQSxROzs7OztBQUtYLDBCQWlCRztBQUFBOztBQUFBLFFBaEJEQyxJQWdCQyxRQWhCREEsSUFnQkM7QUFBQSxRQWZEQyxLQWVDLFFBZkRBLEtBZUM7QUFBQSxRQWREQyxpQkFjQyxRQWREQSxpQkFjQztBQUFBLFFBYkRDLE1BYUMsUUFiREEsTUFhQztBQUFBLFFBWkRDLEtBWUMsUUFaREEsS0FZQztBQUFBLFFBWERDLFFBV0MsUUFYREEsUUFXQztBQUFBLFFBVkRDLFlBVUMsUUFWREEsWUFVQztBQUFBO0FBQ0QsOEJBQU07QUFBRU4sTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBLGlCQUFmO0FBQWtDRSxNQUFBQSxLQUFLLEVBQUxBLEtBQWxDO0FBQXlDQyxNQUFBQSxRQUFRLEVBQVJBO0FBQXpDLEtBQU47QUFEQztBQUFBO0FBQUE7QUFFRCxVQUFLRSxnQkFBTCxHQUF3QkQsWUFBWSxJQUFJLEVBQXhDO0FBQ0EsVUFBS0gsTUFBTCxHQUFjQSxNQUFNLElBQUksS0FBeEI7QUFDQSxVQUFLSyxRQUFMLEdBQWdCLFlBQWhCO0FBSkM7QUFLRjs7OzsyQkFFYztBQUNiLGFBQU8sTUFBUDtBQUNEOzs7bUNBRXlCO0FBQ3hCLGFBQU8sS0FBS0QsZ0JBQVo7QUFDRDs7OzhCQUVTRSxLLEVBQTZCO0FBQ3JDLFVBQUlBLEtBQUssS0FBSyxJQUFkLEVBQW9CO0FBQ2xCLGVBQU8sSUFBUDtBQUNEOztBQUNELFVBQUksS0FBS04sTUFBVCxFQUFpQjtBQUNmLFlBQUksS0FBS0ssUUFBTCxJQUFpQixNQUFqQixJQUEyQixPQUFPQyxLQUFQLElBQWdCLFFBQS9DLEVBQXlEO0FBQ3ZELGNBQU1DLFVBQVUsR0FBR0MsaUJBQUtDLEtBQUwsQ0FBV0gsS0FBWCxDQUFuQjs7QUFDQSxpQkFBT0MsVUFBUDtBQUNELFNBSEQsTUFHTztBQUNMLGdCQUFNLElBQUlHLEtBQUosQ0FBVSxxQ0FBVixDQUFOO0FBQ0Q7QUFDRixPQVBELE1BT087QUFDTCxlQUFPSixLQUFQO0FBQ0Q7QUFDRjs7O0VBbkQyQkssVSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCBUT01MIGZyb20gXCJAaWFybmEvdG9tbFwiO1xuXG5pbXBvcnQgeyBQcm9wLCBQcm9wVmFsdWUgfSBmcm9tIFwiLi4vcHJvcFwiO1xuXG5pbnRlcmZhY2UgUGFyc2VkVmFsdWUge1xuICBwYXJzZWQ6IFJlY29yZDxzdHJpbmcsIGFueT4gfCBudWxsO1xuICBlcnJvcjogc3RyaW5nO1xufVxuXG5leHBvcnQgY2xhc3MgUHJvcENvZGUgZXh0ZW5kcyBQcm9wIHtcbiAgYmFzZURlZmF1bHRWYWx1ZTogc3RyaW5nO1xuICBsYW5ndWFnZTogc3RyaW5nO1xuICBwYXJzZWQ6IGJvb2xlYW47XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcGFyc2VkLFxuICAgIHJ1bGVzLFxuICAgIHJlcXVpcmVkLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBsYW5ndWFnZT86IFByb3BDb2RlW1wibGFuZ3VhZ2VcIl07XG4gICAgcGFyc2VkPzogUHJvcENvZGVbXCJwYXJzZWRcIl07XG4gICAgcnVsZXM/OiBQcm9wW1wicnVsZXNcIl07XG4gICAgcmVxdWlyZWQ/OiBQcm9wW1wicmVxdWlyZWRcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogc3RyaW5nO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUsIHJ1bGVzLCByZXF1aXJlZCB9KTtcbiAgICB0aGlzLmJhc2VEZWZhdWx0VmFsdWUgPSBkZWZhdWx0VmFsdWUgfHwgXCJcIjtcbiAgICB0aGlzLnBhcnNlZCA9IHBhcnNlZCB8fCBmYWxzZTtcbiAgICB0aGlzLmxhbmd1YWdlID0gXCJhdXRvZGV0ZWN0XCI7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiY29kZVwiO1xuICB9XG5cbiAgZGVmYXVsdFZhbHVlKCk6IFByb3BWYWx1ZSB7XG4gICAgcmV0dXJuIHRoaXMuYmFzZURlZmF1bHRWYWx1ZTtcbiAgfVxuXG4gIHJlYWxWYWx1ZSh2YWx1ZTogUHJvcFZhbHVlKTogUHJvcFZhbHVlIHtcbiAgICBpZiAodmFsdWUgPT09IG51bGwpIHtcbiAgICAgIHJldHVybiBudWxsO1xuICAgIH1cbiAgICBpZiAodGhpcy5wYXJzZWQpIHtcbiAgICAgIGlmICh0aGlzLmxhbmd1YWdlID09IFwidG9tbFwiICYmIHR5cGVvZiB2YWx1ZSA9PSBcInN0cmluZ1wiKSB7XG4gICAgICAgIGNvbnN0IG9iamVjdERhdGEgPSBUT01MLnBhcnNlKHZhbHVlKTtcbiAgICAgICAgcmV0dXJuIG9iamVjdERhdGE7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJEbyBub3Qga25vdyBob3cgdG8gcGFyc2UgdGhpcyB0aGluZ1wiKTtcbiAgICAgIH1cbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHZhbHVlO1xuICAgIH1cbiAgfVxufVxuXG4iXX0=