"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropLink = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _prop = require("../prop");

var _registry = require("../registry");

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var PropLink = /*#__PURE__*/function (_Prop) {
  (0, _inherits2["default"])(PropLink, _Prop);

  var _super = _createSuper(PropLink);

  function PropLink(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;
    (0, _classCallCheck2["default"])(this, PropLink);
    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "baseDefaultValue", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "lookup", void 0);
    _this.baseDefaultValue = defaultValue || "";
    return _this;
  }

  (0, _createClass2["default"])(PropLink, [{
    key: "lookupObject",
    value: function lookupObject() {
      if (this.lookup == undefined) {
        throw new Error("Link must have a lookup object defined on `p.lookup`");
      }

      return _registry.registry.get(this.lookup.typeName);
    }
  }, {
    key: "lookupMyself",
    value: function lookupMyself() {
      if (this.lookup == undefined) {
        throw new Error("Link must have a lookup object defined on `p.lookup`");
      }

      return _registry.registry.lookupProp(this.lookup);
    }
  }, {
    key: "kind",
    value: function kind() {
      return "link";
    }
  }, {
    key: "defaultValue",
    value: function defaultValue() {
      return this.lookupMyself().baseDefaultValue;
    }
  }, {
    key: "bagNames",
    value: function bagNames() {
      return this.lookupMyself().bagNames();
    }
  }]);
  return PropLink;
}(_prop.Prop);

exports.PropLink = PropLink;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2xpbmsudHMiXSwibmFtZXMiOlsiUHJvcExpbmsiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwibG9va3VwIiwidW5kZWZpbmVkIiwiRXJyb3IiLCJyZWdpc3RyeSIsImdldCIsInR5cGVOYW1lIiwibG9va3VwUHJvcCIsImxvb2t1cE15c2VsZiIsImJhZ05hbWVzIiwiUHJvcCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7QUFDQTs7Ozs7O0lBS2FBLFE7Ozs7O0FBSVgsMEJBY0c7QUFBQTs7QUFBQSxRQWJEQyxJQWFDLFFBYkRBLElBYUM7QUFBQSxRQVpEQyxLQVlDLFFBWkRBLEtBWUM7QUFBQSxRQVhEQyxpQkFXQyxRQVhEQSxpQkFXQztBQUFBLFFBVkRDLEtBVUMsUUFWREEsS0FVQztBQUFBLFFBVERDLFFBU0MsUUFUREEsUUFTQztBQUFBLFFBUkRDLFlBUUMsUUFSREEsWUFRQztBQUFBO0FBQ0QsOEJBQU07QUFBRUwsTUFBQUEsSUFBSSxFQUFKQSxJQUFGO0FBQVFDLE1BQUFBLEtBQUssRUFBTEEsS0FBUjtBQUFlQyxNQUFBQSxpQkFBaUIsRUFBakJBLGlCQUFmO0FBQWtDQyxNQUFBQSxLQUFLLEVBQUxBLEtBQWxDO0FBQXlDQyxNQUFBQSxRQUFRLEVBQVJBO0FBQXpDLEtBQU47QUFEQztBQUFBO0FBRUQsVUFBS0UsZ0JBQUwsR0FBd0JELFlBQVksSUFBSSxFQUF4QztBQUZDO0FBR0Y7Ozs7bUNBRTJCO0FBQzFCLFVBQUksS0FBS0UsTUFBTCxJQUFlQyxTQUFuQixFQUE4QjtBQUM1QixjQUFNLElBQUlDLEtBQUosQ0FBVSxzREFBVixDQUFOO0FBQ0Q7O0FBQ0QsYUFBT0MsbUJBQVNDLEdBQVQsQ0FBYSxLQUFLSixNQUFMLENBQVlLLFFBQXpCLENBQVA7QUFDRDs7O21DQUVxQjtBQUNwQixVQUFJLEtBQUtMLE1BQUwsSUFBZUMsU0FBbkIsRUFBOEI7QUFDNUIsY0FBTSxJQUFJQyxLQUFKLENBQVUsc0RBQVYsQ0FBTjtBQUNEOztBQUNELGFBQU9DLG1CQUFTRyxVQUFULENBQW9CLEtBQUtOLE1BQXpCLENBQVA7QUFDRDs7OzJCQUVjO0FBQ2IsYUFBTyxNQUFQO0FBQ0Q7OzttQ0FFeUI7QUFDeEIsYUFBTyxLQUFLTyxZQUFMLEdBQW9CUixnQkFBM0I7QUFDRDs7OytCQUVvQjtBQUNuQixhQUFPLEtBQUtRLFlBQUwsR0FBb0JDLFFBQXBCLEVBQVA7QUFDRDs7O0VBL0MyQkMsVSIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3AsIFByb3BWYWx1ZSB9IGZyb20gXCIuLi9wcm9wXCI7XG5pbXBvcnQgeyBQcm9wTG9va3VwLCByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuaW1wb3J0IHsgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcblxuaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5cbmV4cG9ydCBjbGFzcyBQcm9wTGluayBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBzdHJpbmc7XG4gIGxvb2t1cDogdW5kZWZpbmVkIHwgUHJvcExvb2t1cDtcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBydWxlcyxcbiAgICByZXF1aXJlZCxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcnVsZXM/OiBQcm9wW1wicnVsZXNcIl07XG4gICAgcmVxdWlyZWQ/OiBQcm9wW1wicmVxdWlyZWRcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogc3RyaW5nO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUsIHJ1bGVzLCByZXF1aXJlZCB9KTtcbiAgICB0aGlzLmJhc2VEZWZhdWx0VmFsdWUgPSBkZWZhdWx0VmFsdWUgfHwgXCJcIjtcbiAgfVxuXG4gIGxvb2t1cE9iamVjdCgpOiBPYmplY3RUeXBlcyB7XG4gICAgaWYgKHRoaXMubG9va3VwID09IHVuZGVmaW5lZCkge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFwiTGluayBtdXN0IGhhdmUgYSBsb29rdXAgb2JqZWN0IGRlZmluZWQgb24gYHAubG9va3VwYFwiKTtcbiAgICB9XG4gICAgcmV0dXJuIHJlZ2lzdHJ5LmdldCh0aGlzLmxvb2t1cC50eXBlTmFtZSk7XG4gIH1cblxuICBsb29rdXBNeXNlbGYoKTogUHJvcHMge1xuICAgIGlmICh0aGlzLmxvb2t1cCA9PSB1bmRlZmluZWQpIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcIkxpbmsgbXVzdCBoYXZlIGEgbG9va3VwIG9iamVjdCBkZWZpbmVkIG9uIGBwLmxvb2t1cGBcIik7XG4gICAgfVxuICAgIHJldHVybiByZWdpc3RyeS5sb29rdXBQcm9wKHRoaXMubG9va3VwKTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJsaW5rXCI7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlIHtcbiAgICByZXR1cm4gdGhpcy5sb29rdXBNeXNlbGYoKS5iYXNlRGVmYXVsdFZhbHVlO1xuICB9XG5cbiAgYmFnTmFtZXMoKTogc3RyaW5nW10ge1xuICAgIHJldHVybiB0aGlzLmxvb2t1cE15c2VsZigpLmJhZ05hbWVzKCk7XG4gIH1cbn1cbiJdfQ==