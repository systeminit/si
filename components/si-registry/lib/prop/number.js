"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropNumber = void 0;

var _text = require("../prop/text");

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

var PropNumber = /*#__PURE__*/function (_PropText) {
  _inherits(PropNumber, _PropText);

  var _super = _createSuper(PropNumber);

  function PropNumber(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        defaultValue = _ref.defaultValue;

    _classCallCheck(this, PropNumber);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _defineProperty(_assertThisInitialized(_this), "numberKind", void 0);

    _this.baseDefaultValue = defaultValue || "";
    _this.numberKind = "int64";
    return _this;
  }

  _createClass(PropNumber, [{
    key: "kind",
    value: function kind() {
      return "number";
    }
  }]);

  return PropNumber;
}(_text.PropText);

exports.PropNumber = PropNumber;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL251bWJlci50cyJdLCJuYW1lcyI6WyJQcm9wTnVtYmVyIiwibmFtZSIsImxhYmVsIiwiY29tcG9uZW50VHlwZU5hbWUiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwibnVtYmVyS2luZCIsIlByb3BUZXh0Il0sIm1hcHBpbmdzIjoiOzs7Ozs7O0FBQ0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0lBRWFBLFU7Ozs7O0FBSVgsNEJBVUc7QUFBQTs7QUFBQSxRQVREQyxJQVNDLFFBVERBLElBU0M7QUFBQSxRQVJEQyxLQVFDLFFBUkRBLEtBUUM7QUFBQSxRQVBEQyxpQkFPQyxRQVBEQSxpQkFPQztBQUFBLFFBTkRDLFlBTUMsUUFOREEsWUFNQzs7QUFBQTs7QUFDRCw4QkFBTTtBQUFFSCxNQUFBQSxJQUFJLEVBQUpBLElBQUY7QUFBUUMsTUFBQUEsS0FBSyxFQUFMQSxLQUFSO0FBQWVDLE1BQUFBLGlCQUFpQixFQUFqQkE7QUFBZixLQUFOOztBQURDOztBQUFBOztBQUVELFVBQUtFLGdCQUFMLEdBQXdCRCxZQUFZLElBQUksRUFBeEM7QUFDQSxVQUFLRSxVQUFMLEdBQWtCLE9BQWxCO0FBSEM7QUFJRjs7OzsyQkFFYztBQUNiLGFBQU8sUUFBUDtBQUNEOzs7O0VBdEI2QkMsYyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3AgfSBmcm9tIFwiLi4vcHJvcFwiO1xuaW1wb3J0IHsgUHJvcFRleHQgfSBmcm9tIFwiLi4vcHJvcC90ZXh0XCI7XG5cbmV4cG9ydCBjbGFzcyBQcm9wTnVtYmVyIGV4dGVuZHMgUHJvcFRleHQge1xuICBiYXNlRGVmYXVsdFZhbHVlOiBzdHJpbmc7XG4gIG51bWJlcktpbmQ6IFwiaW50MzJcIiB8IFwidWludDMyXCIgfCBcImludDY0XCIgfCBcInVpbnQ2NFwiO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIGRlZmF1bHRWYWx1ZSxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBkZWZhdWx0VmFsdWU/OiBQcm9wTnVtYmVyW1wiYmFzZURlZmF1bHRWYWx1ZVwiXTtcbiAgfSkge1xuICAgIHN1cGVyKHsgbmFtZSwgbGFiZWwsIGNvbXBvbmVudFR5cGVOYW1lIH0pO1xuICAgIHRoaXMuYmFzZURlZmF1bHRWYWx1ZSA9IGRlZmF1bHRWYWx1ZSB8fCBcIlwiO1xuICAgIHRoaXMubnVtYmVyS2luZCA9IFwiaW50NjRcIjtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJudW1iZXJcIjtcbiAgfVxufVxuIl19