"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropSelect = void 0;

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

var PropSelect = /*#__PURE__*/function (_Prop) {
  _inherits(PropSelect, _Prop);

  var _super = _createSuper(PropSelect);

  function PropSelect(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        options = _ref.options,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;

    _classCallCheck(this, PropSelect);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _defineProperty(_assertThisInitialized(_this), "options", void 0);

    _this.options = options;
    _this.baseDefaultValue = defaultValue || "";
    return _this;
  }

  _createClass(PropSelect, [{
    key: "kind",
    value: function kind() {
      return "select";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }]);

  return PropSelect;
}(_prop.Prop);

exports.PropSelect = PropSelect;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL3NlbGVjdC50cyJdLCJuYW1lcyI6WyJQcm9wU2VsZWN0IiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJvcHRpb25zIiwicnVsZXMiLCJyZXF1aXJlZCIsImRlZmF1bHRWYWx1ZSIsImJhc2VEZWZhdWx0VmFsdWUiLCJQcm9wIl0sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUE7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0lBRWFBLFU7Ozs7O0FBSVgsNEJBZ0JHO0FBQUE7O0FBQUEsUUFmREMsSUFlQyxRQWZEQSxJQWVDO0FBQUEsUUFkREMsS0FjQyxRQWREQSxLQWNDO0FBQUEsUUFiREMsaUJBYUMsUUFiREEsaUJBYUM7QUFBQSxRQVpEQyxPQVlDLFFBWkRBLE9BWUM7QUFBQSxRQVhEQyxLQVdDLFFBWERBLEtBV0M7QUFBQSxRQVZEQyxRQVVDLFFBVkRBLFFBVUM7QUFBQSxRQVREQyxZQVNDLFFBVERBLFlBU0M7O0FBQUE7O0FBQ0QsOEJBQU07QUFBRU4sTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBLGlCQUFmO0FBQWtDRSxNQUFBQSxLQUFLLEVBQUxBLEtBQWxDO0FBQXlDQyxNQUFBQSxRQUFRLEVBQVJBO0FBQXpDLEtBQU47O0FBREM7O0FBQUE7O0FBRUQsVUFBS0YsT0FBTCxHQUFlQSxPQUFmO0FBQ0EsVUFBS0ksZ0JBQUwsR0FBd0JELFlBQVksSUFBSSxFQUF4QztBQUhDO0FBSUY7Ozs7MkJBRWM7QUFDYixhQUFPLFFBQVA7QUFDRDs7O21DQUV5QjtBQUN4QixhQUFPLEtBQUtDLGdCQUFaO0FBQ0Q7Ozs7RUFoQzZCQyxVIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcCwgUHJvcFZhbHVlIH0gZnJvbSBcIi4uL3Byb3BcIjtcblxuZXhwb3J0IGNsYXNzIFByb3BTZWxlY3QgZXh0ZW5kcyBQcm9wIHtcbiAgYmFzZURlZmF1bHRWYWx1ZTogc3RyaW5nO1xuICBvcHRpb25zOiBzdHJpbmdbXTtcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBvcHRpb25zLFxuICAgIHJ1bGVzLFxuICAgIHJlcXVpcmVkLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBvcHRpb25zOiBQcm9wU2VsZWN0W1wib3B0aW9uc1wiXTtcbiAgICBydWxlcz86IFByb3BbXCJydWxlc1wiXTtcbiAgICByZXF1aXJlZD86IFByb3BbXCJyZXF1aXJlZFwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBzdHJpbmc7XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSwgcnVsZXMsIHJlcXVpcmVkIH0pO1xuICAgIHRoaXMub3B0aW9ucyA9IG9wdGlvbnM7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IFwiXCI7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwic2VsZWN0XCI7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlIHtcbiAgICByZXR1cm4gdGhpcy5iYXNlRGVmYXVsdFZhbHVlO1xuICB9XG59XG4iXX0=