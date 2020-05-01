"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropEnum = void 0;

var _prop = require("../prop");

function _typeof(obj) { "@babel/helpers - typeof"; if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") { _typeof = function _typeof(obj) { return typeof obj; }; } else { _typeof = function _typeof(obj) { return obj && typeof Symbol === "function" && obj.constructor === Symbol && obj !== Symbol.prototype ? "symbol" : typeof obj; }; } return _typeof(obj); }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _inherits(subClass, superClass) { if (typeof superClass !== "function" && superClass !== null) { throw new TypeError("Super expression must either be null or a function"); } subClass.prototype = Object.create(superClass && superClass.prototype, { constructor: { value: subClass, writable: true, configurable: true } }); if (superClass) _setPrototypeOf(subClass, superClass); }

function _setPrototypeOf(o, p) { _setPrototypeOf = Object.setPrototypeOf || function _setPrototypeOf(o, p) { o.__proto__ = p; return o; }; return _setPrototypeOf(o, p); }

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = _getPrototypeOf(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = _getPrototypeOf(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return _possibleConstructorReturn(this, result); }; }

function _possibleConstructorReturn(self, call) { if (call && (_typeof(call) === "object" || typeof call === "function")) { return call; } return _assertThisInitialized(self); }

function _assertThisInitialized(self) { if (self === void 0) { throw new ReferenceError("this hasn't been initialised - super() hasn't been called"); } return self; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

function _getPrototypeOf(o) { _getPrototypeOf = Object.setPrototypeOf ? Object.getPrototypeOf : function _getPrototypeOf(o) { return o.__proto__ || Object.getPrototypeOf(o); }; return _getPrototypeOf(o); }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

var PropEnum = /*#__PURE__*/function (_Prop) {
  _inherits(PropEnum, _Prop);

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

    _classCallCheck(this, PropEnum);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _defineProperty(_assertThisInitialized(_this), "variants", void 0);

    _this.variants = [];
    _this.parentName = parentName;
    _this.baseDefaultValue = defaultValue || "";
    return _this;
  }

  _createClass(PropEnum, [{
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2VudW0udHMiXSwibmFtZXMiOlsiUHJvcEVudW0iLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInBhcmVudE5hbWUiLCJydWxlcyIsInJlcXVpcmVkIiwiZGVmYXVsdFZhbHVlIiwidmFyaWFudHMiLCJiYXNlRGVmYXVsdFZhbHVlIiwiUHJvcCJdLCJtYXBwaW5ncyI6Ijs7Ozs7OztBQUFBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztJQUdhQSxROzs7OztBQUlYLDBCQWdCRztBQUFBOztBQUFBLFFBZkRDLElBZUMsUUFmREEsSUFlQztBQUFBLFFBZERDLEtBY0MsUUFkREEsS0FjQztBQUFBLFFBYkRDLGlCQWFDLFFBYkRBLGlCQWFDO0FBQUEsUUFaREMsVUFZQyxRQVpEQSxVQVlDO0FBQUEsUUFYREMsS0FXQyxRQVhEQSxLQVdDO0FBQUEsUUFWREMsUUFVQyxRQVZEQSxRQVVDO0FBQUEsUUFUREMsWUFTQyxRQVREQSxZQVNDOztBQUFBOztBQUNELDhCQUFNO0FBQUVOLE1BQUFBLElBQUksRUFBSkEsSUFBRjtBQUFRQyxNQUFBQSxLQUFLLEVBQUxBLEtBQVI7QUFBZUMsTUFBQUEsaUJBQWlCLEVBQWpCQSxpQkFBZjtBQUFrQ0UsTUFBQUEsS0FBSyxFQUFMQSxLQUFsQztBQUF5Q0MsTUFBQUEsUUFBUSxFQUFSQTtBQUF6QyxLQUFOOztBQURDOztBQUFBOztBQUVELFVBQUtFLFFBQUwsR0FBZ0IsRUFBaEI7QUFDQSxVQUFLSixVQUFMLEdBQWtCQSxVQUFsQjtBQUNBLFVBQUtLLGdCQUFMLEdBQXdCRixZQUFZLElBQUksRUFBeEM7QUFKQztBQUtGOzs7OzJCQUVjO0FBQ2IsYUFBTyxNQUFQO0FBQ0Q7OzttQ0FFeUI7QUFDeEIsYUFBTyxLQUFLRSxnQkFBWjtBQUNEOzs7O0VBakMyQkMsVSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3AsIFByb3BWYWx1ZSB9IGZyb20gXCIuLi9wcm9wXCI7XG5pbXBvcnQgeyBwYXNjYWxDYXNlLCBjb25zdGFudENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcblxuZXhwb3J0IGNsYXNzIFByb3BFbnVtIGV4dGVuZHMgUHJvcCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IHN0cmluZztcbiAgdmFyaWFudHM6IHN0cmluZ1tdO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHBhcmVudE5hbWUsXG4gICAgcnVsZXMsXG4gICAgcmVxdWlyZWQsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHBhcmVudE5hbWU/OiBQcm9wW1wicGFyZW50TmFtZVwiXTtcbiAgICBydWxlcz86IFByb3BbXCJydWxlc1wiXTtcbiAgICByZXF1aXJlZD86IFByb3BbXCJyZXF1aXJlZFwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBzdHJpbmc7XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSwgcnVsZXMsIHJlcXVpcmVkIH0pO1xuICAgIHRoaXMudmFyaWFudHMgPSBbXTtcbiAgICB0aGlzLnBhcmVudE5hbWUgPSBwYXJlbnROYW1lO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCBcIlwiO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImVudW1cIjtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wVmFsdWUge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cbn1cbiJdfQ==