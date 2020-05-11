"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropEnum = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _prop = require("../prop");

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var PropEnum = /*#__PURE__*/function (_Prop) {
  (0, _inherits2["default"])(PropEnum, _Prop);

  var _super = _createSuper(PropEnum);

  function PropEnum(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        parentName = _ref.parentName,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;
    (0, _classCallCheck2["default"])(this, PropEnum);
    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "baseDefaultValue", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "variants", void 0);
    _this.variants = [];
    _this.parentName = parentName || "";
    _this.baseDefaultValue = defaultValue || "";
    return _this;
  }

  (0, _createClass2["default"])(PropEnum, [{
    key: "kind",
    value: function kind() {
      return "enum";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }]);
  return PropEnum;
}(_prop.Prop);

exports.PropEnum = PropEnum;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2VudW0udHMiXSwibmFtZXMiOlsiUHJvcEVudW0iLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInBhcmVudE5hbWUiLCJydWxlcyIsInJlcXVpcmVkIiwiZGVmYXVsdFZhbHVlIiwidmFyaWFudHMiLCJiYXNlRGVmYXVsdFZhbHVlIiwiUHJvcCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7Ozs7O0lBR2FBLFE7Ozs7O0FBSVgsMEJBZ0JHO0FBQUE7O0FBQUEsUUFmREMsSUFlQyxRQWZEQSxJQWVDO0FBQUEsUUFkREMsS0FjQyxRQWREQSxLQWNDO0FBQUEsUUFiREMsaUJBYUMsUUFiREEsaUJBYUM7QUFBQSxRQVpEQyxVQVlDLFFBWkRBLFVBWUM7QUFBQSxRQVhEQyxLQVdDLFFBWERBLEtBV0M7QUFBQSxRQVZEQyxRQVVDLFFBVkRBLFFBVUM7QUFBQSxRQVREQyxZQVNDLFFBVERBLFlBU0M7QUFBQTtBQUNELDhCQUFNO0FBQUVOLE1BQUFBLElBQUksRUFBSkEsSUFBRjtBQUFRQyxNQUFBQSxLQUFLLEVBQUxBLEtBQVI7QUFBZUMsTUFBQUEsaUJBQWlCLEVBQWpCQSxpQkFBZjtBQUFrQ0UsTUFBQUEsS0FBSyxFQUFMQSxLQUFsQztBQUF5Q0MsTUFBQUEsUUFBUSxFQUFSQTtBQUF6QyxLQUFOO0FBREM7QUFBQTtBQUVELFVBQUtFLFFBQUwsR0FBZ0IsRUFBaEI7QUFDQSxVQUFLSixVQUFMLEdBQWtCQSxVQUFVLElBQUksRUFBaEM7QUFDQSxVQUFLSyxnQkFBTCxHQUF3QkYsWUFBWSxJQUFJLEVBQXhDO0FBSkM7QUFLRjs7OzsyQkFFYztBQUNiLGFBQU8sTUFBUDtBQUNEOzs7bUNBRXlCO0FBQ3hCLGFBQU8sS0FBS0UsZ0JBQVo7QUFDRDs7O0VBakMyQkMsVSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3AsIFByb3BWYWx1ZSB9IGZyb20gXCIuLi9wcm9wXCI7XG5pbXBvcnQgeyBwYXNjYWxDYXNlLCBjb25zdGFudENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcblxuZXhwb3J0IGNsYXNzIFByb3BFbnVtIGV4dGVuZHMgUHJvcCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IHN0cmluZztcbiAgdmFyaWFudHM6IHN0cmluZ1tdO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHBhcmVudE5hbWUsXG4gICAgcnVsZXMsXG4gICAgcmVxdWlyZWQsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHBhcmVudE5hbWU/OiBQcm9wW1wicGFyZW50TmFtZVwiXTtcbiAgICBydWxlcz86IFByb3BbXCJydWxlc1wiXTtcbiAgICByZXF1aXJlZD86IFByb3BbXCJyZXF1aXJlZFwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBzdHJpbmc7XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSwgcnVsZXMsIHJlcXVpcmVkIH0pO1xuICAgIHRoaXMudmFyaWFudHMgPSBbXTtcbiAgICB0aGlzLnBhcmVudE5hbWUgPSBwYXJlbnROYW1lIHx8IFwiXCI7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IFwiXCI7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiZW51bVwiO1xuICB9XG5cbiAgZGVmYXVsdFZhbHVlKCk6IFByb3BWYWx1ZSB7XG4gICAgcmV0dXJuIHRoaXMuYmFzZURlZmF1bHRWYWx1ZTtcbiAgfVxufVxuIl19