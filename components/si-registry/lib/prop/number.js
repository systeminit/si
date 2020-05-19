"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropNumber = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _text = require("../prop/text");

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var PropNumber = /*#__PURE__*/function (_PropText) {
  (0, _inherits2["default"])(PropNumber, _PropText);

  var _super = _createSuper(PropNumber);

  function PropNumber(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        defaultValue = _ref.defaultValue;
    (0, _classCallCheck2["default"])(this, PropNumber);
    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "baseDefaultValue", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "numberKind", void 0);
    _this.baseDefaultValue = defaultValue || "";
    _this.numberKind = "int64";
    return _this;
  }

  (0, _createClass2["default"])(PropNumber, [{
    key: "kind",
    value: function kind() {
      return "number";
    }
  }]);
  return PropNumber;
}(_text.PropText);

exports.PropNumber = PropNumber;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL251bWJlci50cyJdLCJuYW1lcyI6WyJQcm9wTnVtYmVyIiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwibnVtYmVyS2luZCIsIlByb3BUZXh0Il0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUNBOzs7Ozs7SUFFYUEsVTs7Ozs7QUFJWCw0QkFVRztBQUFBOztBQUFBLFFBVERDLElBU0MsUUFUREEsSUFTQztBQUFBLFFBUkRDLEtBUUMsUUFSREEsS0FRQztBQUFBLFFBUERDLGlCQU9DLFFBUERBLGlCQU9DO0FBQUEsUUFOREMsWUFNQyxRQU5EQSxZQU1DO0FBQUE7QUFDRCw4QkFBTTtBQUFFSCxNQUFBQSxJQUFJLEVBQUpBLElBQUY7QUFBUUMsTUFBQUEsS0FBSyxFQUFMQSxLQUFSO0FBQWVDLE1BQUFBLGlCQUFpQixFQUFqQkE7QUFBZixLQUFOO0FBREM7QUFBQTtBQUVELFVBQUtFLGdCQUFMLEdBQXdCRCxZQUFZLElBQUksRUFBeEM7QUFDQSxVQUFLRSxVQUFMLEdBQWtCLE9BQWxCO0FBSEM7QUFJRjs7OzsyQkFFYztBQUNiLGFBQU8sUUFBUDtBQUNEOzs7RUF0QjZCQyxjIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcCB9IGZyb20gXCIuLi9wcm9wXCI7XG5pbXBvcnQgeyBQcm9wVGV4dCB9IGZyb20gXCIuLi9wcm9wL3RleHRcIjtcblxuZXhwb3J0IGNsYXNzIFByb3BOdW1iZXIgZXh0ZW5kcyBQcm9wVGV4dCB7XG4gIGJhc2VEZWZhdWx0VmFsdWU6IHN0cmluZztcbiAgbnVtYmVyS2luZDogXCJpbnQzMlwiIHwgXCJ1aW50MzJcIiB8IFwiaW50NjRcIiB8IFwidWludDY0XCIgfCBcInUxMjhcIjtcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogUHJvcE51bWJlcltcImJhc2VEZWZhdWx0VmFsdWVcIl07XG4gIH0pIHtcbiAgICBzdXBlcih7IG5hbWUsIGxhYmVsLCBjb21wb25lbnRUeXBlTmFtZSB9KTtcbiAgICB0aGlzLmJhc2VEZWZhdWx0VmFsdWUgPSBkZWZhdWx0VmFsdWUgfHwgXCJcIjtcbiAgICB0aGlzLm51bWJlcktpbmQgPSBcImludDY0XCI7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwibnVtYmVyXCI7XG4gIH1cbn1cbiJdfQ==