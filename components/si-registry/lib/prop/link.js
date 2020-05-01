"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.PropLink = void 0;

var _prop = require("../prop");

var _registry = require("../registry");

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

var PropLink = /*#__PURE__*/function (_Prop) {
  _inherits(PropLink, _Prop);

  var _super = _createSuper(PropLink);

  function PropLink(_ref) {
    var _this;

    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        rules = _ref.rules,
        required = _ref.required,
        defaultValue = _ref.defaultValue;

    _classCallCheck(this, PropLink);

    _this = _super.call(this, {
      name: name,
      label: label,
      componentTypeName: componentTypeName,
      rules: rules,
      required: required
    });

    _defineProperty(_assertThisInitialized(_this), "baseDefaultValue", void 0);

    _defineProperty(_assertThisInitialized(_this), "lookup", void 0);

    _this.baseDefaultValue = defaultValue || "";
    return _this;
  }

  _createClass(PropLink, [{
    key: "lookupObject",
    value: function lookupObject() {
      return _registry.registry.get(this.lookup.typeName);
    }
  }, {
    key: "lookupMyself",
    value: function lookupMyself() {
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL2xpbmsudHMiXSwibmFtZXMiOlsiUHJvcExpbmsiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInJ1bGVzIiwicmVxdWlyZWQiLCJkZWZhdWx0VmFsdWUiLCJiYXNlRGVmYXVsdFZhbHVlIiwicmVnaXN0cnkiLCJnZXQiLCJsb29rdXAiLCJ0eXBlTmFtZSIsImxvb2t1cFByb3AiLCJsb29rdXBNeXNlbGYiLCJiYWdOYW1lcyIsIlByb3AiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7QUFBQTs7QUFDQTs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7SUFNYUEsUTs7Ozs7QUFJWCwwQkFjRztBQUFBOztBQUFBLFFBYkRDLElBYUMsUUFiREEsSUFhQztBQUFBLFFBWkRDLEtBWUMsUUFaREEsS0FZQztBQUFBLFFBWERDLGlCQVdDLFFBWERBLGlCQVdDO0FBQUEsUUFWREMsS0FVQyxRQVZEQSxLQVVDO0FBQUEsUUFUREMsUUFTQyxRQVREQSxRQVNDO0FBQUEsUUFSREMsWUFRQyxRQVJEQSxZQVFDOztBQUFBOztBQUNELDhCQUFNO0FBQUVMLE1BQUFBLElBQUksRUFBSkEsSUFBRjtBQUFRQyxNQUFBQSxLQUFLLEVBQUxBLEtBQVI7QUFBZUMsTUFBQUEsaUJBQWlCLEVBQWpCQSxpQkFBZjtBQUFrQ0MsTUFBQUEsS0FBSyxFQUFMQSxLQUFsQztBQUF5Q0MsTUFBQUEsUUFBUSxFQUFSQTtBQUF6QyxLQUFOOztBQURDOztBQUFBOztBQUVELFVBQUtFLGdCQUFMLEdBQXdCRCxZQUFZLElBQUksRUFBeEM7QUFGQztBQUdGOzs7O21DQUUyQjtBQUMxQixhQUFPRSxtQkFBU0MsR0FBVCxDQUFhLEtBQUtDLE1BQUwsQ0FBWUMsUUFBekIsQ0FBUDtBQUNEOzs7bUNBRXFCO0FBQ3BCLGFBQU9ILG1CQUFTSSxVQUFULENBQW9CLEtBQUtGLE1BQXpCLENBQVA7QUFDRDs7OzJCQUVjO0FBQ2IsYUFBTyxNQUFQO0FBQ0Q7OzttQ0FFeUI7QUFDeEIsYUFBTyxLQUFLRyxZQUFMLEdBQW9CTixnQkFBM0I7QUFDRDs7OytCQUVvQjtBQUNuQixhQUFPLEtBQUtNLFlBQUwsR0FBb0JDLFFBQXBCLEVBQVA7QUFDRDs7OztFQXpDMkJDLFUiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wLCBQcm9wVmFsdWUgfSBmcm9tIFwiLi4vcHJvcFwiO1xuaW1wb3J0IHsgUHJvcExvb2t1cCwgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3BzIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5cbmltcG9ydCB7IHNuYWtlQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5cbmV4cG9ydCBjbGFzcyBQcm9wTGluayBleHRlbmRzIFByb3Age1xuICBiYXNlRGVmYXVsdFZhbHVlOiBzdHJpbmc7XG4gIGxvb2t1cDogUHJvcExvb2t1cDtcblxuICBjb25zdHJ1Y3Rvcih7XG4gICAgbmFtZSxcbiAgICBsYWJlbCxcbiAgICBjb21wb25lbnRUeXBlTmFtZSxcbiAgICBydWxlcyxcbiAgICByZXF1aXJlZCxcbiAgICBkZWZhdWx0VmFsdWUsXG4gIH06IHtcbiAgICBuYW1lOiBQcm9wW1wibmFtZVwiXTtcbiAgICBsYWJlbDogUHJvcFtcImxhYmVsXCJdO1xuICAgIGNvbXBvbmVudFR5cGVOYW1lOiBQcm9wW1wiY29tcG9uZW50VHlwZU5hbWVcIl07XG4gICAgcnVsZXM/OiBQcm9wW1wicnVsZXNcIl07XG4gICAgcmVxdWlyZWQ/OiBQcm9wW1wicmVxdWlyZWRcIl07XG4gICAgZGVmYXVsdFZhbHVlPzogc3RyaW5nO1xuICB9KSB7XG4gICAgc3VwZXIoeyBuYW1lLCBsYWJlbCwgY29tcG9uZW50VHlwZU5hbWUsIHJ1bGVzLCByZXF1aXJlZCB9KTtcbiAgICB0aGlzLmJhc2VEZWZhdWx0VmFsdWUgPSBkZWZhdWx0VmFsdWUgfHwgXCJcIjtcbiAgfVxuXG4gIGxvb2t1cE9iamVjdCgpOiBPYmplY3RUeXBlcyB7XG4gICAgcmV0dXJuIHJlZ2lzdHJ5LmdldCh0aGlzLmxvb2t1cC50eXBlTmFtZSk7XG4gIH1cblxuICBsb29rdXBNeXNlbGYoKTogUHJvcHMge1xuICAgIHJldHVybiByZWdpc3RyeS5sb29rdXBQcm9wKHRoaXMubG9va3VwKTtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJsaW5rXCI7XG4gIH1cblxuICBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlIHtcbiAgICByZXR1cm4gdGhpcy5sb29rdXBNeXNlbGYoKS5iYXNlRGVmYXVsdFZhbHVlO1xuICB9XG5cbiAgYmFnTmFtZXMoKTogc3RyaW5nW10ge1xuICAgIHJldHVybiB0aGlzLmxvb2t1cE15c2VsZigpLmJhZ05hbWVzKCk7XG4gIH1cbn1cbiJdfQ==