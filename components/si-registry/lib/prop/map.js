"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropMap = void 0;

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

var PropMap = /*#__PURE__*/function (_Prop) {
  _inherits(PropMap, _Prop);

  var _super = _createSuper(PropMap);

  function PropMap(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        defaultValue = _ref.defaultValue;

    _classCallCheck(this, PropMap);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _this.baseDefaultValue = defaultValue || {};
    return _this;
  }

  _createClass(PropMap, [{
    key: "kind",
    value: function kind() {
      return "map";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }]);

  return PropMap;
}(_prop.Prop);

exports.PropMap = PropMap;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL21hcC50cyJdLCJuYW1lcyI6WyJQcm9wTWFwIiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwiUHJvcCJdLCJtYXBwaW5ncyI6Ijs7Ozs7OztBQUFBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztJQUVhQSxPOzs7OztBQUdYLHlCQVVHO0FBQUE7O0FBQUEsUUFUREMsSUFTQyxRQVREQSxJQVNDO0FBQUEsUUFSREMsS0FRQyxRQVJEQSxLQVFDO0FBQUEsUUFQREMsaUJBT0MsUUFQREEsaUJBT0M7QUFBQSxRQU5EQyxZQU1DLFFBTkRBLFlBTUM7O0FBQUE7O0FBQ0QsOEJBQU07QUFBRUgsTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBO0FBQWYsS0FBTjs7QUFEQzs7QUFFRCxVQUFLRSxnQkFBTCxHQUF3QkQsWUFBWSxJQUFJLEVBQXhDO0FBRkM7QUFHRjs7OzsyQkFFYztBQUNiLGFBQU8sS0FBUDtBQUNEOzs7bUNBRXlCO0FBQ3hCLGFBQU8sS0FBS0MsZ0JBQVo7QUFDRDs7OztFQXhCMEJDLFUiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wLCBQcm9wVmFsdWUgfSBmcm9tIFwiLi4vcHJvcFwiO1xuXG5leHBvcnQgY2xhc3MgUHJvcE1hcCBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBSZWNvcmQ8c3RyaW5nLCBzdHJpbmc+O1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBQcm9wTWFwW1wiYmFzZURlZmF1bHRWYWx1ZVwiXTtcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCB7fTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJtYXBcIjtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wVmFsdWUge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cbn1cbiJdfQ==