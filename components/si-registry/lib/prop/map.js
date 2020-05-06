"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropMap = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _prop = require("../prop");

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var PropMap = /*#__PURE__*/function (_Prop) {
  (0, _inherits2["default"])(PropMap, _Prop);

  var _super = _createSuper(PropMap);

  function PropMap(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        defaultValue = _ref.defaultValue;
    (0, _classCallCheck2["default"])(this, PropMap);
    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "baseDefaultValue", void 0);
    _this.baseDefaultValue = defaultValue || {};
    return _this;
  }

  (0, _createClass2["default"])(PropMap, [{
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL21hcC50cyJdLCJuYW1lcyI6WyJQcm9wTWFwIiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwiUHJvcCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7Ozs7O0lBRWFBLE87Ozs7O0FBR1gseUJBVUc7QUFBQTs7QUFBQSxRQVREQyxJQVNDLFFBVERBLElBU0M7QUFBQSxRQVJEQyxLQVFDLFFBUkRBLEtBUUM7QUFBQSxRQVBEQyxpQkFPQyxRQVBEQSxpQkFPQztBQUFBLFFBTkRDLFlBTUMsUUFOREEsWUFNQztBQUFBO0FBQ0QsOEJBQU07QUFBRUgsTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBO0FBQWYsS0FBTjtBQURDO0FBRUQsVUFBS0UsZ0JBQUwsR0FBd0JELFlBQVksSUFBSSxFQUF4QztBQUZDO0FBR0Y7Ozs7MkJBRWM7QUFDYixhQUFPLEtBQVA7QUFDRDs7O21DQUV5QjtBQUN4QixhQUFPLEtBQUtDLGdCQUFaO0FBQ0Q7OztFQXhCMEJDLFUiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wLCBQcm9wVmFsdWUgfSBmcm9tIFwiLi4vcHJvcFwiO1xuXG5leHBvcnQgY2xhc3MgUHJvcE1hcCBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBSZWNvcmQ8c3RyaW5nLCBzdHJpbmc+O1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBQcm9wTWFwW1wiYmFzZURlZmF1bHRWYWx1ZVwiXTtcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCB7fTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJtYXBcIjtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wVmFsdWUge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cbn1cbiJdfQ==