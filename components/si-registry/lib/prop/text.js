"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropText = void 0;

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

var PropText = /*#__PURE__*/function (_Prop) {
  _inherits(PropText, _Prop);

  var _super = _createSuper(PropText);

  function PropText(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;

    _classCallCheck(this, PropText);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _this.baseDefaultValue = defaultValue || "";
    return _this;
  }

  _createClass(PropText, [{
    key: "kind",
    value: function kind() {
      return "text";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }]);

  return PropText;
}(_prop.Prop);

exports.PropText = PropText;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL3RleHQudHMiXSwibmFtZXMiOlsiUHJvcFRleHQiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwiUHJvcCJdLCJtYXBwaW5ncyI6Ijs7Ozs7OztBQUFBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztJQUVhQSxROzs7OztBQUdYLDBCQWNHO0FBQUE7O0FBQUEsUUFiREMsSUFhQyxRQWJEQSxJQWFDO0FBQUEsUUFaREMsS0FZQyxRQVpEQSxLQVlDO0FBQUEsUUFYREMsaUJBV0MsUUFYREEsaUJBV0M7QUFBQSxRQVZEQyxLQVVDLFFBVkRBLEtBVUM7QUFBQSxRQVREQyxRQVNDLFFBVERBLFFBU0M7QUFBQSxRQVJEQyxZQVFDLFFBUkRBLFlBUUM7O0FBQUE7O0FBQ0QsOEJBQU07QUFBRUwsTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBLGlCQUFmO0FBQWtDQyxNQUFBQSxLQUFLLEVBQUxBLEtBQWxDO0FBQXlDQyxNQUFBQSxRQUFRLEVBQVJBO0FBQXpDLEtBQU47O0FBREM7O0FBRUQsVUFBS0UsZ0JBQUwsR0FBd0JELFlBQVksSUFBSSxFQUF4QztBQUZDO0FBR0Y7Ozs7MkJBRWM7QUFDYixhQUFPLE1BQVA7QUFDRDs7O21DQUV5QjtBQUN4QixhQUFPLEtBQUtDLGdCQUFaO0FBQ0Q7Ozs7RUE1QjJCQyxVIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcCwgUHJvcFZhbHVlIH0gZnJvbSBcIi4uL3Byb3BcIjtcblxuZXhwb3J0IGNsYXNzIFByb3BUZXh0IGV4dGVuZHMgUHJvcCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBydWxlcyxcbiAgICByZXF1aXJlZCxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcnVsZXM/OiBQcm9wW1wicnVsZXNcIl07XG4gICAgcmVxdWlyZWQ/OiBQcm9wW1wicmVxdWlyZWRcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogc3RyaW5nO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUsIHJ1bGVzLCByZXF1aXJlZCB9KTtcbiAgICB0aGlzLmJhc2VEZWZhdWx0VmFsdWUgPSBkZWZhdWx0VmFsdWUgfHwgXCJcIjtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJ0ZXh0XCI7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlIHtcbiAgICByZXR1cm4gdGhpcy5iYXNlRGVmYXVsdFZhbHVlO1xuICB9XG59XG4iXX0=