"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropCode = void 0;

var _toml = _interopRequireDefault(require("@iarna/toml"));

var _prop = require("../prop");

function _interopRequireDefault(obj) { return obj && obj.__esModule ? obj : { "default": obj }; }

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

var PropCode = /*#__PURE__*/function (_Prop) {
  _inherits(PropCode, _Prop);

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

    _classCallCheck(this, PropCode);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _defineProperty(_assertThisInitialized(_this), "language", void 0);

    _defineProperty(_assertThisInitialized(_this), "parsed", void 0);

    _this.baseDefaultValue = defaultValue || "";
    _this.parsed = parsed || false;
    _this.language = "autodetect";
    return _this;
  }

  _createClass(PropCode, [{
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2NvZGUudHMiXSwibmFtZXMiOlsiUHJvcENvZGUiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInBhcnNlZCIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwibGFuZ3VhZ2UiLCJ2YWx1ZSIsIm9iamVjdERhdGEiLCJUT01MIiwicGFyc2UiLCJQcm9wIl0sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQUE7O0FBRUE7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7SUFPYUEsUTs7Ozs7QUFLWCwwQkFpQkc7QUFBQTs7QUFBQSxRQWhCREMsSUFnQkMsUUFoQkRBLElBZ0JDO0FBQUEsUUFmREMsS0FlQyxRQWZEQSxLQWVDO0FBQUEsUUFkREMsaUJBY0MsUUFkREEsaUJBY0M7QUFBQSxRQWJEQyxNQWFDLFFBYkRBLE1BYUM7QUFBQSxRQVpEQyxLQVlDLFFBWkRBLEtBWUM7QUFBQSxRQVhEQyxRQVdDLFFBWERBLFFBV0M7QUFBQSxRQVZEQyxZQVVDLFFBVkRBLFlBVUM7O0FBQUE7O0FBQ0QsOEJBQU07QUFBRU4sTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBLGlCQUFmO0FBQWtDRSxNQUFBQSxLQUFLLEVBQUxBLEtBQWxDO0FBQXlDQyxNQUFBQSxRQUFRLEVBQVJBO0FBQXpDLEtBQU47O0FBREM7O0FBQUE7O0FBQUE7O0FBRUQsVUFBS0UsZ0JBQUwsR0FBd0JELFlBQVksSUFBSSxFQUF4QztBQUNBLFVBQUtILE1BQUwsR0FBY0EsTUFBTSxJQUFJLEtBQXhCO0FBQ0EsVUFBS0ssUUFBTCxHQUFnQixZQUFoQjtBQUpDO0FBS0Y7Ozs7MkJBRWM7QUFDYixhQUFPLE1BQVA7QUFDRDs7O21DQUV5QjtBQUN4QixhQUFPLEtBQUtELGdCQUFaO0FBQ0Q7Ozs4QkFFU0UsSyxFQUE2QjtBQUNyQyxVQUFJQSxLQUFLLEtBQUssSUFBZCxFQUFvQjtBQUNsQixlQUFPLElBQVA7QUFDRDs7QUFDRCxVQUFJLEtBQUtOLE1BQVQsRUFBaUI7QUFDZixZQUFJLEtBQUtLLFFBQUwsSUFBaUIsTUFBakIsSUFBMkIsT0FBT0MsS0FBUCxJQUFnQixRQUEvQyxFQUF5RDtBQUN2RCxjQUFNQyxVQUFVLEdBQUdDLGlCQUFLQyxLQUFMLENBQVdILEtBQVgsQ0FBbkI7O0FBQ0EsaUJBQU9DLFVBQVA7QUFDRCxTQUhELE1BR087QUFDTCxnQkFBTSxxQ0FBTjtBQUNEO0FBQ0YsT0FQRCxNQU9PO0FBQ0wsZUFBT0QsS0FBUDtBQUNEO0FBQ0Y7Ozs7RUFuRDJCSSxVIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IFRPTUwgZnJvbSBcIkBpYXJuYS90b21sXCI7XG5cbmltcG9ydCB7IFByb3AsIFByb3BWYWx1ZSB9IGZyb20gXCIuLi9wcm9wXCI7XG5cbmludGVyZmFjZSBQYXJzZWRWYWx1ZSB7XG4gIHBhcnNlZDogUmVjb3JkPHN0cmluZywgYW55PiB8IG51bGw7XG4gIGVycm9yOiBzdHJpbmc7XG59XG5cbmV4cG9ydCBjbGFzcyBQcm9wQ29kZSBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBzdHJpbmc7XG4gIGxhbmd1YWdlOiBzdHJpbmc7XG4gIHBhcnNlZDogYm9vbGVhbjtcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBwYXJzZWQsXG4gICAgcnVsZXMsXG4gICAgcmVxdWlyZWQsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIGxhbmd1YWdlPzogUHJvcENvZGVbXCJsYW5ndWFnZVwiXTtcbiAgICBwYXJzZWQ/OiBQcm9wQ29kZVtcInBhcnNlZFwiXTtcbiAgICBydWxlcz86IFByb3BbXCJydWxlc1wiXTtcbiAgICByZXF1aXJlZD86IFByb3BbXCJyZXF1aXJlZFwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBzdHJpbmc7XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSwgcnVsZXMsIHJlcXVpcmVkIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCBcIlwiO1xuICAgIHRoaXMucGFyc2VkID0gcGFyc2VkIHx8IGZhbHNlO1xuICAgIHRoaXMubGFuZ3VhZ2UgPSBcImF1dG9kZXRlY3RcIjtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJjb2RlXCI7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlIHtcbiAgICByZXR1cm4gdGhpcy5iYXNlRGVmYXVsdFZhbHVlO1xuICB9XG5cbiAgcmVhbFZhbHVlKHZhbHVlOiBQcm9wVmFsdWUpOiBQcm9wVmFsdWUge1xuICAgIGlmICh2YWx1ZSA9PT0gbnVsbCkge1xuICAgICAgcmV0dXJuIG51bGw7XG4gICAgfVxuICAgIGlmICh0aGlzLnBhcnNlZCkge1xuICAgICAgaWYgKHRoaXMubGFuZ3VhZ2UgPT0gXCJ0b21sXCIgJiYgdHlwZW9mIHZhbHVlID09IFwic3RyaW5nXCIpIHtcbiAgICAgICAgY29uc3Qgb2JqZWN0RGF0YSA9IFRPTUwucGFyc2UodmFsdWUpO1xuICAgICAgICByZXR1cm4gb2JqZWN0RGF0YTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHRocm93IFwiRG8gbm90IGtub3cgaG93IHRvIHBhcnNlIHRoaXMgdGhpbmdcIjtcbiAgICAgIH1cbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHZhbHVlO1xuICAgIH1cbiAgfVxufVxuXG4iXX0=