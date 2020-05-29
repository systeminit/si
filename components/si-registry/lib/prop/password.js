"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropPassword = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _text = require("./text");

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var PropPassword = /*#__PURE__*/function (_PropText) {
  (0, _inherits2["default"])(PropPassword, _PropText);

  var _super = _createSuper(PropPassword);

  function PropPassword(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;
    (0, _classCallCheck2["default"])(this, PropPassword);
    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "baseDefaultValue", void 0);
    _this.baseDefaultValue = defaultValue || "";
    return _this;
  }

  (0, _createClass2["default"])(PropPassword, [{
    key: "kind",
    value: function kind() {
      return "password";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.baseDefaultValue;
    }
  }]);
  return PropPassword;
}(_text.PropText);

exports.PropPassword = PropPassword;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL3Bhc3N3b3JkLnRzIl0sIm5hbWVzIjpbIlByb3BQYXNzd29yZCIsIm5hbWUiLCJsYWJlbCIsImNvbXBvbmVudFR5cGVOYW1lIiwicnVsZXMiLCJyZXF1aXJlZCIsImRlZmF1bHRWYWx1ZSIsImJhc2VEZWZhdWx0VmFsdWUiLCJQcm9wVGV4dCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFDQTs7Ozs7O0lBRWFBLFk7Ozs7O0FBR1gsOEJBY0c7QUFBQTs7QUFBQSxRQWJEQyxJQWFDLFFBYkRBLElBYUM7QUFBQSxRQVpEQyxLQVlDLFFBWkRBLEtBWUM7QUFBQSxRQVhEQyxpQkFXQyxRQVhEQSxpQkFXQztBQUFBLFFBVkRDLEtBVUMsUUFWREEsS0FVQztBQUFBLFFBVERDLFFBU0MsUUFUREEsUUFTQztBQUFBLFFBUkRDLFlBUUMsUUFSREEsWUFRQztBQUFBO0FBQ0QsOEJBQU07QUFBRUwsTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBLGlCQUFmO0FBQWtDQyxNQUFBQSxLQUFLLEVBQUxBLEtBQWxDO0FBQXlDQyxNQUFBQSxRQUFRLEVBQVJBO0FBQXpDLEtBQU47QUFEQztBQUVELFVBQUtFLGdCQUFMLEdBQXdCRCxZQUFZLElBQUksRUFBeEM7QUFGQztBQUdGOzs7OzJCQUVjO0FBQ2IsYUFBTyxVQUFQO0FBQ0Q7OzttQ0FFeUI7QUFDeEIsYUFBTyxLQUFLQyxnQkFBWjtBQUNEOzs7RUE1QitCQyxjIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcCwgUHJvcFZhbHVlIH0gZnJvbSBcIi4uL3Byb3BcIjtcbmltcG9ydCB7IFByb3BUZXh0IH0gZnJvbSBcIi4vdGV4dFwiO1xuXG5leHBvcnQgY2xhc3MgUHJvcFBhc3N3b3JkIGV4dGVuZHMgUHJvcFRleHQge1xuICBiYXNlRGVmYXVsdFZhbHVlOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcnVsZXMsXG4gICAgcmVxdWlyZWQsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHJ1bGVzPzogUHJvcFtcInJ1bGVzXCJdO1xuICAgIHJlcXVpcmVkPzogUHJvcFtcInJlcXVpcmVkXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IHN0cmluZztcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lLCBydWxlcywgcmVxdWlyZWQgfSk7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IFwiXCI7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwicGFzc3dvcmRcIjtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wVmFsdWUge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cbn1cbiJdfQ==