"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropBool = void 0;

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

var PropBool = /*#__PURE__*/function (_Prop) {
  _inherits(PropBool, _Prop);

  var _super = _createSuper(PropBool);

  function PropBool(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;

    _classCallCheck(this, PropBool);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _this.baseDefaultValue = defaultValue || false;
    return _this;
  }

  _createClass(PropBool, [{
    key: "kind",
    value: function kind() {
      return "bool";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }]);

  return PropBool;
}(_prop.Prop);

exports.PropBool = PropBool;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2Jvb2wudHMiXSwibmFtZXMiOlsiUHJvcEJvb2wiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwiUHJvcCJdLCJtYXBwaW5ncyI6Ijs7Ozs7OztBQUFBOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztJQUVhQSxROzs7OztBQUdYLDBCQWNHO0FBQUE7O0FBQUEsUUFiREMsSUFhQyxRQWJEQSxJQWFDO0FBQUEsUUFaREMsS0FZQyxRQVpEQSxLQVlDO0FBQUEsUUFYREMsaUJBV0MsUUFYREEsaUJBV0M7QUFBQSxRQVZEQyxLQVVDLFFBVkRBLEtBVUM7QUFBQSxRQVREQyxRQVNDLFFBVERBLFFBU0M7QUFBQSxRQVJEQyxZQVFDLFFBUkRBLFlBUUM7O0FBQUE7O0FBQ0QsOEJBQU07QUFBRUwsTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBLGlCQUFmO0FBQWtDQyxNQUFBQSxLQUFLLEVBQUxBLEtBQWxDO0FBQXlDQyxNQUFBQSxRQUFRLEVBQVJBO0FBQXpDLEtBQU47O0FBREM7O0FBRUQsVUFBS0UsZ0JBQUwsR0FBd0JELFlBQVksSUFBSSxLQUF4QztBQUZDO0FBR0Y7Ozs7MkJBRWM7QUFDYixhQUFPLE1BQVA7QUFDRDs7O21DQUV5QjtBQUN4QixhQUFPLEtBQUtDLGdCQUFaO0FBQ0Q7Ozs7RUE1QjJCQyxVIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcCwgUHJvcFZhbHVlIH0gZnJvbSBcIi4uL3Byb3BcIjtcblxuZXhwb3J0IGNsYXNzIFByb3BCb29sIGV4dGVuZHMgUHJvcCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IGJvb2xlYW47XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcnVsZXMsXG4gICAgcmVxdWlyZWQsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHJ1bGVzPzogUHJvcFtcInJ1bGVzXCJdO1xuICAgIHJlcXVpcmVkPzogUHJvcFtcInJlcXVpcmVkXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IGJvb2xlYW47XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSwgcnVsZXMsIHJlcXVpcmVkIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCBmYWxzZTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJib29sXCI7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlIHtcbiAgICByZXR1cm4gdGhpcy5iYXNlRGVmYXVsdFZhbHVlO1xuICB9XG59XG4iXX0=