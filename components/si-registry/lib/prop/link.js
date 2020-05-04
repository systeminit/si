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

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

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
        throw "Link must have a lookup object defined on `p.lookup`";
      }

      return _registry.registry.get(this.lookup.typeName);
    }
  }, {
    key: "lookupMyself",
    value: function lookupMyself() {
      if (this.lookup == undefined) {
        throw "Link must have a lookup object defined on `p.lookup`";
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2xpbmsudHMiXSwibmFtZXMiOlsiUHJvcExpbmsiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwibG9va3VwIiwidW5kZWZpbmVkIiwicmVnaXN0cnkiLCJnZXQiLCJ0eXBlTmFtZSIsImxvb2t1cFByb3AiLCJsb29rdXBNeXNlbGYiLCJiYWdOYW1lcyIsIlByb3AiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0FBQ0E7Ozs7OztJQUthQSxROzs7OztBQUlYLDBCQWNHO0FBQUE7O0FBQUEsUUFiREMsSUFhQyxRQWJEQSxJQWFDO0FBQUEsUUFaREMsS0FZQyxRQVpEQSxLQVlDO0FBQUEsUUFYREMsaUJBV0MsUUFYREEsaUJBV0M7QUFBQSxRQVZEQyxLQVVDLFFBVkRBLEtBVUM7QUFBQSxRQVREQyxRQVNDLFFBVERBLFFBU0M7QUFBQSxRQVJEQyxZQVFDLFFBUkRBLFlBUUM7QUFBQTtBQUNELDhCQUFNO0FBQUVMLE1BQUFBLElBQUksRUFBSkEsSUFBRjtBQUFRQyxNQUFBQSxLQUFLLEVBQUxBLEtBQVI7QUFBZUMsTUFBQUEsaUJBQWlCLEVBQWpCQSxpQkFBZjtBQUFrQ0MsTUFBQUEsS0FBSyxFQUFMQSxLQUFsQztBQUF5Q0MsTUFBQUEsUUFBUSxFQUFSQTtBQUF6QyxLQUFOO0FBREM7QUFBQTtBQUVELFVBQUtFLGdCQUFMLEdBQXdCRCxZQUFZLElBQUksRUFBeEM7QUFGQztBQUdGOzs7O21DQUUyQjtBQUMxQixVQUFJLEtBQUtFLE1BQUwsSUFBZUMsU0FBbkIsRUFBOEI7QUFDNUIsY0FBTSxzREFBTjtBQUNEOztBQUNELGFBQU9DLG1CQUFTQyxHQUFULENBQWEsS0FBS0gsTUFBTCxDQUFZSSxRQUF6QixDQUFQO0FBQ0Q7OzttQ0FFcUI7QUFDcEIsVUFBSSxLQUFLSixNQUFMLElBQWVDLFNBQW5CLEVBQThCO0FBQzVCLGNBQU0sc0RBQU47QUFDRDs7QUFDRCxhQUFPQyxtQkFBU0csVUFBVCxDQUFvQixLQUFLTCxNQUF6QixDQUFQO0FBQ0Q7OzsyQkFFYztBQUNiLGFBQU8sTUFBUDtBQUNEOzs7bUNBRXlCO0FBQ3hCLGFBQU8sS0FBS00sWUFBTCxHQUFvQlAsZ0JBQTNCO0FBQ0Q7OzsrQkFFb0I7QUFDbkIsYUFBTyxLQUFLTyxZQUFMLEdBQW9CQyxRQUFwQixFQUFQO0FBQ0Q7OztFQS9DMkJDLFUiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wLCBQcm9wVmFsdWUgfSBmcm9tIFwiLi4vcHJvcFwiO1xuaW1wb3J0IHsgUHJvcExvb2t1cCwgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3BzIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5cbmltcG9ydCB7IE9iamVjdFR5cGVzIH0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuXG5leHBvcnQgY2xhc3MgUHJvcExpbmsgZXh0ZW5kcyBQcm9wIHtcbiAgYmFzZURlZmF1bHRWYWx1ZTogc3RyaW5nO1xuICBsb29rdXA6IHVuZGVmaW5lZCB8IFByb3BMb29rdXA7XG5cbiAgY29uc3RydWN0b3Ioe1xuICAgIG5hbWUsXG4gICAgbGFiZWwsXG4gICAgY29tcG9uZW50VHlwZU5hbWUsXG4gICAgcnVsZXMsXG4gICAgcmVxdWlyZWQsXG4gICAgZGVmYXVsdFZhbHVlLFxuICB9OiB7XG4gICAgbmFtZTogUHJvcFtcIm5hbWVcIl07XG4gICAgbGFiZWw6IFByb3BbXCJsYWJlbFwiXTtcbiAgICBjb21wb25lbnRUeXBlTmFtZTogUHJvcFtcImNvbXBvbmVudFR5cGVOYW1lXCJdO1xuICAgIHJ1bGVzPzogUHJvcFtcInJ1bGVzXCJdO1xuICAgIHJlcXVpcmVkPzogUHJvcFtcInJlcXVpcmVkXCJdO1xuICAgIGRlZmF1bHRWYWx1ZT86IHN0cmluZztcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lLCBydWxlcywgcmVxdWlyZWQgfSk7XG4gICAgdGhpcy5iYXNlRGVmYXVsdFZhbHVlID0gZGVmYXVsdFZhbHVlIHx8IFwiXCI7XG4gIH1cblxuICBsb29rdXBPYmplY3QoKTogT2JqZWN0VHlwZXMge1xuICAgIGlmICh0aGlzLmxvb2t1cCA9PSB1bmRlZmluZWQpIHtcbiAgICAgIHRocm93IFwiTGluayBtdXN0IGhhdmUgYSBsb29rdXAgb2JqZWN0IGRlZmluZWQgb24gYHAubG9va3VwYFwiO1xuICAgIH1cbiAgICByZXR1cm4gcmVnaXN0cnkuZ2V0KHRoaXMubG9va3VwLnR5cGVOYW1lKTtcbiAgfVxuXG4gIGxvb2t1cE15c2VsZigpOiBQcm9wcyB7XG4gICAgaWYgKHRoaXMubG9va3VwID09IHVuZGVmaW5lZCkge1xuICAgICAgdGhyb3cgXCJMaW5rIG11c3QgaGF2ZSBhIGxvb2t1cCBvYmplY3QgZGVmaW5lZCBvbiBgcC5sb29rdXBgXCI7XG4gICAgfVxuICAgIHJldHVybiByZWdpc3RyeS5sb29rdXBQcm9wKHRoaXMubG9va3VwKTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJsaW5rXCI7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlIHtcbiAgICByZXR1cm4gdGhpcy5sb29rdXBNeXNlbGYoKS5iYXNlRGVmYXVsdFZhbHVlO1xuICB9XG5cbiAgYmFnTmFtZXMoKTogc3RyaW5nW10ge1xuICAgIHJldHVybiB0aGlzLmxvb2t1cE15c2VsZigpLmJhZ05hbWVzKCk7XG4gIH1cbn1cbiJdfQ==