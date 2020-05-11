"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.RelationshipList = exports.Either = exports.Updates = exports.Relationship = void 0;

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _registry = require("../registry");

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var Relationship = /*#__PURE__*/function () {
  function Relationship(args) {
    (0, _classCallCheck2["default"])(this, Relationship);
    (0, _defineProperty2["default"])(this, "partner", void 0);
    this.partner = args.partner;
  }

  (0, _createClass2["default"])(Relationship, [{
    key: "partnerObject",
    value: function partnerObject() {
      return _registry.registry.get(this.partner.typeName);
    }
  }, {
    key: "partnerProp",
    value: function partnerProp() {
      return _registry.registry.lookupProp(this.partner);
    }
  }]);
  return Relationship;
}(); // An updates relationship ensures that when one method changes,
// another one gets notified.


exports.Relationship = Relationship;

var Updates = /*#__PURE__*/function (_Relationship) {
  (0, _inherits2["default"])(Updates, _Relationship);

  var _super = _createSuper(Updates);

  function Updates() {
    (0, _classCallCheck2["default"])(this, Updates);
    return _super.apply(this, arguments);
  }

  (0, _createClass2["default"])(Updates, [{
    key: "kind",
    value: function kind() {
      return "updates";
    }
  }]);
  return Updates;
}(Relationship);

exports.Updates = Updates;

var Either = /*#__PURE__*/function (_Relationship2) {
  (0, _inherits2["default"])(Either, _Relationship2);

  var _super2 = _createSuper(Either);

  function Either() {
    (0, _classCallCheck2["default"])(this, Either);
    return _super2.apply(this, arguments);
  }

  (0, _createClass2["default"])(Either, [{
    key: "kind",
    value: function kind() {
      return "either";
    }
  }]);
  return Either;
}(Relationship);

exports.Either = Either;

var RelationshipList = /*#__PURE__*/function () {
  function RelationshipList() {
    (0, _classCallCheck2["default"])(this, RelationshipList);
    (0, _defineProperty2["default"])(this, "relationships", []);
  }

  (0, _createClass2["default"])(RelationshipList, [{
    key: "all",
    value: function all() {
      return this.relationships;
    }
  }, {
    key: "updates",
    value: function updates(args) {
      return new Updates(args);
    }
  }, {
    key: "either",
    value: function either(args) {
      return new Either(args);
    }
  }]);
  return RelationshipList;
}();

exports.RelationshipList = RelationshipList;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL3JlbGF0aW9uc2hpcHMudHMiXSwibmFtZXMiOlsiUmVsYXRpb25zaGlwIiwiYXJncyIsInBhcnRuZXIiLCJyZWdpc3RyeSIsImdldCIsInR5cGVOYW1lIiwibG9va3VwUHJvcCIsIlVwZGF0ZXMiLCJFaXRoZXIiLCJSZWxhdGlvbnNoaXBMaXN0IiwicmVsYXRpb25zaGlwcyJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBR0E7Ozs7OztJQVFzQkEsWTtBQUdwQix3QkFBWUMsSUFBWixFQUEyQztBQUFBO0FBQUE7QUFDekMsU0FBS0MsT0FBTCxHQUFlRCxJQUFJLENBQUNDLE9BQXBCO0FBQ0Q7Ozs7b0NBRTRCO0FBQzNCLGFBQU9DLG1CQUFTQyxHQUFULENBQWEsS0FBS0YsT0FBTCxDQUFhRyxRQUExQixDQUFQO0FBQ0Q7OztrQ0FFb0I7QUFDbkIsYUFBT0YsbUJBQVNHLFVBQVQsQ0FBb0IsS0FBS0osT0FBekIsQ0FBUDtBQUNEOzs7S0FLSDtBQUNBOzs7OztJQUNhSyxPOzs7Ozs7Ozs7Ozs7MkJBQ0k7QUFDYixhQUFPLFNBQVA7QUFDRDs7O0VBSDBCUCxZOzs7O0lBTWhCUSxNOzs7Ozs7Ozs7Ozs7MkJBQ0k7QUFDYixhQUFPLFFBQVA7QUFDRDs7O0VBSHlCUixZOzs7O0lBTWZTLGdCOzs7NERBQ3NCLEU7Ozs7OzBCQUVRO0FBQ3ZDLGFBQU8sS0FBS0MsYUFBWjtBQUNEOzs7NEJBRU9ULEksRUFBd0M7QUFDOUMsYUFBTyxJQUFJTSxPQUFKLENBQVlOLElBQVosQ0FBUDtBQUNEOzs7MkJBRU1BLEksRUFBdUM7QUFDNUMsYUFBTyxJQUFJTyxNQUFKLENBQVdQLElBQVgsQ0FBUDtBQUNEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcExvb2t1cCB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuaW1wb3J0IHsgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcbmltcG9ydCB7IE9iamVjdFR5cGVzIH0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcblxuZXhwb3J0IHR5cGUgUmVsYXRpb25zaGlwcyA9IFVwZGF0ZXMgfCBFaXRoZXI7XG5cbmludGVyZmFjZSBSZWxhdGlvbnNoaXBDb25zdHJ1Y3RvciB7XG4gIHBhcnRuZXI6IFByb3BMb29rdXA7XG59XG5cbmV4cG9ydCBhYnN0cmFjdCBjbGFzcyBSZWxhdGlvbnNoaXAge1xuICBwYXJ0bmVyOiBQcm9wTG9va3VwO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IFJlbGF0aW9uc2hpcENvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy5wYXJ0bmVyID0gYXJncy5wYXJ0bmVyO1xuICB9XG5cbiAgcGFydG5lck9iamVjdCgpOiBPYmplY3RUeXBlcyB7XG4gICAgcmV0dXJuIHJlZ2lzdHJ5LmdldCh0aGlzLnBhcnRuZXIudHlwZU5hbWUpO1xuICB9XG5cbiAgcGFydG5lclByb3AoKTogUHJvcHMge1xuICAgIHJldHVybiByZWdpc3RyeS5sb29rdXBQcm9wKHRoaXMucGFydG5lcik7XG4gIH1cblxuICBhYnN0cmFjdCBraW5kKCk6IHN0cmluZztcbn1cblxuLy8gQW4gdXBkYXRlcyByZWxhdGlvbnNoaXAgZW5zdXJlcyB0aGF0IHdoZW4gb25lIG1ldGhvZCBjaGFuZ2VzLFxuLy8gYW5vdGhlciBvbmUgZ2V0cyBub3RpZmllZC5cbmV4cG9ydCBjbGFzcyBVcGRhdGVzIGV4dGVuZHMgUmVsYXRpb25zaGlwIHtcbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcInVwZGF0ZXNcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgRWl0aGVyIGV4dGVuZHMgUmVsYXRpb25zaGlwIHtcbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImVpdGhlclwiO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBSZWxhdGlvbnNoaXBMaXN0IHtcbiAgcmVsYXRpb25zaGlwczogUmVsYXRpb25zaGlwc1tdID0gW107XG5cbiAgYWxsKCk6IFJlbGF0aW9uc2hpcExpc3RbXCJyZWxhdGlvbnNoaXBzXCJdIHtcbiAgICByZXR1cm4gdGhpcy5yZWxhdGlvbnNoaXBzO1xuICB9XG5cbiAgdXBkYXRlcyhhcmdzOiBSZWxhdGlvbnNoaXBDb25zdHJ1Y3Rvcik6IFVwZGF0ZXMge1xuICAgIHJldHVybiBuZXcgVXBkYXRlcyhhcmdzKTtcbiAgfVxuXG4gIGVpdGhlcihhcmdzOiBSZWxhdGlvbnNoaXBDb25zdHJ1Y3Rvcik6IEVpdGhlciB7XG4gICAgcmV0dXJuIG5ldyBFaXRoZXIoYXJncyk7XG4gIH1cbn1cbiJdfQ==