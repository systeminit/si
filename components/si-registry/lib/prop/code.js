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

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

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
          throw "Do not know how to parse this thing";
        }
      } else {
        return value;
      }
    }
  }]);
  return PropCode;
}(_prop.Prop);

exports.PropCode = PropCode;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2NvZGUudHMiXSwibmFtZXMiOlsiUHJvcENvZGUiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInBhcnNlZCIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwibGFuZ3VhZ2UiLCJ2YWx1ZSIsIm9iamVjdERhdGEiLCJUT01MIiwicGFyc2UiLCJQcm9wIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFBOztBQUVBOzs7Ozs7SUFPYUEsUTs7Ozs7QUFLWCwwQkFpQkc7QUFBQTs7QUFBQSxRQWhCREMsSUFnQkMsUUFoQkRBLElBZ0JDO0FBQUEsUUFmREMsS0FlQyxRQWZEQSxLQWVDO0FBQUEsUUFkREMsaUJBY0MsUUFkREEsaUJBY0M7QUFBQSxRQWJEQyxNQWFDLFFBYkRBLE1BYUM7QUFBQSxRQVpEQyxLQVlDLFFBWkRBLEtBWUM7QUFBQSxRQVhEQyxRQVdDLFFBWERBLFFBV0M7QUFBQSxRQVZEQyxZQVVDLFFBVkRBLFlBVUM7QUFBQTtBQUNELDhCQUFNO0FBQUVOLE1BQUFBLElBQUksRUFBSkEsSUFBRjtBQUFRQyxNQUFBQSxLQUFLLEVBQUxBLEtBQVI7QUFBZUMsTUFBQUEsaUJBQWlCLEVBQWpCQSxpQkFBZjtBQUFrQ0UsTUFBQUEsS0FBSyxFQUFMQSxLQUFsQztBQUF5Q0MsTUFBQUEsUUFBUSxFQUFSQTtBQUF6QyxLQUFOO0FBREM7QUFBQTtBQUFBO0FBRUQsVUFBS0UsZ0JBQUwsR0FBd0JELFlBQVksSUFBSSxFQUF4QztBQUNBLFVBQUtILE1BQUwsR0FBY0EsTUFBTSxJQUFJLEtBQXhCO0FBQ0EsVUFBS0ssUUFBTCxHQUFnQixZQUFoQjtBQUpDO0FBS0Y7Ozs7MkJBRWM7QUFDYixhQUFPLE1BQVA7QUFDRDs7O21DQUV5QjtBQUN4QixhQUFPLEtBQUtELGdCQUFaO0FBQ0Q7Ozs4QkFFU0UsSyxFQUE2QjtBQUNyQyxVQUFJQSxLQUFLLEtBQUssSUFBZCxFQUFvQjtBQUNsQixlQUFPLElBQVA7QUFDRDs7QUFDRCxVQUFJLEtBQUtOLE1BQVQsRUFBaUI7QUFDZixZQUFJLEtBQUtLLFFBQUwsSUFBaUIsTUFBakIsSUFBMkIsT0FBT0MsS0FBUCxJQUFnQixRQUEvQyxFQUF5RDtBQUN2RCxjQUFNQyxVQUFVLEdBQUdDLGlCQUFLQyxLQUFMLENBQVdILEtBQVgsQ0FBbkI7O0FBQ0EsaUJBQU9DLFVBQVA7QUFDRCxTQUhELE1BR087QUFDTCxnQkFBTSxxQ0FBTjtBQUNEO0FBQ0YsT0FQRCxNQU9PO0FBQ0wsZUFBT0QsS0FBUDtBQUNEO0FBQ0Y7OztFQW5EMkJJLFUiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgVE9NTCBmcm9tIFwiQGlhcm5hL3RvbWxcIjtcblxuaW1wb3J0IHsgUHJvcCwgUHJvcFZhbHVlIH0gZnJvbSBcIi4uL3Byb3BcIjtcblxuaW50ZXJmYWNlIFBhcnNlZFZhbHVlIHtcbiAgcGFyc2VkOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHwgbnVsbDtcbiAgZXJyb3I6IHN0cmluZztcbn1cblxuZXhwb3J0IGNsYXNzIFByb3BDb2RlIGV4dGVuZHMgUHJvcCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IHN0cmluZztcbiAgbGFuZ3VhZ2U6IHN0cmluZztcbiAgcGFyc2VkOiBib29sZWFuO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHBhcnNlZCxcbiAgICBydWxlcyxcbiAgICByZXF1aXJlZCxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgbGFuZ3VhZ2U/OiBQcm9wQ29kZVtcImxhbmd1YWdlXCJdO1xuICAgIHBhcnNlZD86IFByb3BDb2RlW1wicGFyc2VkXCJdO1xuICAgIHJ1bGVzPzogUHJvcFtcInJ1bGVzXCJdO1xuICAgIHJlcXVpcmVkPzogUHJvcFtcInJlcXVpcmVkXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IHN0cmluZztcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lLCBydWxlcywgcmVxdWlyZWQgfSk7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IFwiXCI7XG4gICAgdGhpcy5wYXJzZWQgPSBwYXJzZWQgfHwgZmFsc2U7XG4gICAgdGhpcy5sYW5ndWFnZSA9IFwiYXV0b2RldGVjdFwiO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImNvZGVcIjtcbiAgfVxuXG4gIGRlZmF1bHRWYWx1ZSgpOiBQcm9wVmFsdWUge1xuICAgIHJldHVybiB0aGlzLmJhc2VEZWZhdWx0VmFsdWU7XG4gIH1cblxuICByZWFsVmFsdWUodmFsdWU6IFByb3BWYWx1ZSk6IFByb3BWYWx1ZSB7XG4gICAgaWYgKHZhbHVlID09PSBudWxsKSB7XG4gICAgICByZXR1cm4gbnVsbDtcbiAgICB9XG4gICAgaWYgKHRoaXMucGFyc2VkKSB7XG4gICAgICBpZiAodGhpcy5sYW5ndWFnZSA9PSBcInRvbWxcIiAmJiB0eXBlb2YgdmFsdWUgPT0gXCJzdHJpbmdcIikge1xuICAgICAgICBjb25zdCBvYmplY3REYXRhID0gVE9NTC5wYXJzZSh2YWx1ZSk7XG4gICAgICAgIHJldHVybiBvYmplY3REYXRhO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgdGhyb3cgXCJEbyBub3Qga25vdyBob3cgdG8gcGFyc2UgdGhpcyB0aGluZ1wiO1xuICAgICAgfVxuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gdmFsdWU7XG4gICAgfVxuICB9XG59XG5cbiJdfQ==