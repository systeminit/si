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
      this.relationships.push(new Updates(args));
    }
  }, {
    key: "either",
    value: function either(args) {
      this.relationships.push(new Either(args));
    }
  }]);
  return RelationshipList;
}();

exports.RelationshipList = RelationshipList;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9wcm9wL3JlbGF0aW9uc2hpcHMudHMiXSwibmFtZXMiOlsiUmVsYXRpb25zaGlwIiwiYXJncyIsInBhcnRuZXIiLCJyZWdpc3RyeSIsImdldCIsInR5cGVOYW1lIiwibG9va3VwUHJvcCIsIlVwZGF0ZXMiLCJFaXRoZXIiLCJSZWxhdGlvbnNoaXBMaXN0IiwicmVsYXRpb25zaGlwcyIsInB1c2giXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUdBOzs7Ozs7SUFRc0JBLFk7QUFHcEIsd0JBQVlDLElBQVosRUFBMkM7QUFBQTtBQUFBO0FBQ3pDLFNBQUtDLE9BQUwsR0FBZUQsSUFBSSxDQUFDQyxPQUFwQjtBQUNEOzs7O29DQUU0QjtBQUMzQixhQUFPQyxtQkFBU0MsR0FBVCxDQUFhLEtBQUtGLE9BQUwsQ0FBYUcsUUFBMUIsQ0FBUDtBQUNEOzs7a0NBRW9CO0FBQ25CLGFBQU9GLG1CQUFTRyxVQUFULENBQW9CLEtBQUtKLE9BQXpCLENBQVA7QUFDRDs7O0tBS0g7QUFDQTs7Ozs7SUFDYUssTzs7Ozs7Ozs7Ozs7OzJCQUNJO0FBQ2IsYUFBTyxTQUFQO0FBQ0Q7OztFQUgwQlAsWTs7OztJQU1oQlEsTTs7Ozs7Ozs7Ozs7OzJCQUNJO0FBQ2IsYUFBTyxRQUFQO0FBQ0Q7OztFQUh5QlIsWTs7OztJQU1mUyxnQjs7OzREQUNzQixFOzs7OzswQkFFUTtBQUN2QyxhQUFPLEtBQUtDLGFBQVo7QUFDRDs7OzRCQUVPVCxJLEVBQXFDO0FBQzNDLFdBQUtTLGFBQUwsQ0FBbUJDLElBQW5CLENBQXdCLElBQUlKLE9BQUosQ0FBWU4sSUFBWixDQUF4QjtBQUNEOzs7MkJBRU1BLEksRUFBcUM7QUFDMUMsV0FBS1MsYUFBTCxDQUFtQkMsSUFBbkIsQ0FBd0IsSUFBSUgsTUFBSixDQUFXUCxJQUFYLENBQXhCO0FBQ0QiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wTG9va3VwIH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5pbXBvcnQgeyBQcm9wcyB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuXG5leHBvcnQgdHlwZSBSZWxhdGlvbnNoaXBzID0gVXBkYXRlcyB8IEVpdGhlcjtcblxuaW50ZXJmYWNlIFJlbGF0aW9uc2hpcENvbnN0cnVjdG9yIHtcbiAgcGFydG5lcjogUHJvcExvb2t1cDtcbn1cblxuZXhwb3J0IGFic3RyYWN0IGNsYXNzIFJlbGF0aW9uc2hpcCB7XG4gIHBhcnRuZXI6IFByb3BMb29rdXA7XG5cbiAgY29uc3RydWN0b3IoYXJnczogUmVsYXRpb25zaGlwQ29uc3RydWN0b3IpIHtcbiAgICB0aGlzLnBhcnRuZXIgPSBhcmdzLnBhcnRuZXI7XG4gIH1cblxuICBwYXJ0bmVyT2JqZWN0KCk6IE9iamVjdFR5cGVzIHtcbiAgICByZXR1cm4gcmVnaXN0cnkuZ2V0KHRoaXMucGFydG5lci50eXBlTmFtZSk7XG4gIH1cblxuICBwYXJ0bmVyUHJvcCgpOiBQcm9wcyB7XG4gICAgcmV0dXJuIHJlZ2lzdHJ5Lmxvb2t1cFByb3AodGhpcy5wYXJ0bmVyKTtcbiAgfVxuXG4gIGFic3RyYWN0IGtpbmQoKTogc3RyaW5nO1xufVxuXG4vLyBBbiB1cGRhdGVzIHJlbGF0aW9uc2hpcCBlbnN1cmVzIHRoYXQgd2hlbiBvbmUgbWV0aG9kIGNoYW5nZXMsXG4vLyBhbm90aGVyIG9uZSBnZXRzIG5vdGlmaWVkLlxuZXhwb3J0IGNsYXNzIFVwZGF0ZXMgZXh0ZW5kcyBSZWxhdGlvbnNoaXAge1xuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwidXBkYXRlc1wiO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBFaXRoZXIgZXh0ZW5kcyBSZWxhdGlvbnNoaXAge1xuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiZWl0aGVyXCI7XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFJlbGF0aW9uc2hpcExpc3Qge1xuICByZWxhdGlvbnNoaXBzOiBSZWxhdGlvbnNoaXBzW10gPSBbXTtcblxuICBhbGwoKTogUmVsYXRpb25zaGlwTGlzdFtcInJlbGF0aW9uc2hpcHNcIl0ge1xuICAgIHJldHVybiB0aGlzLnJlbGF0aW9uc2hpcHM7XG4gIH1cblxuICB1cGRhdGVzKGFyZ3M6IFJlbGF0aW9uc2hpcENvbnN0cnVjdG9yKTogdm9pZCB7XG4gICAgdGhpcy5yZWxhdGlvbnNoaXBzLnB1c2gobmV3IFVwZGF0ZXMoYXJncykpO1xuICB9XG5cbiAgZWl0aGVyKGFyZ3M6IFJlbGF0aW9uc2hpcENvbnN0cnVjdG9yKTogdm9pZCB7XG4gICAgdGhpcy5yZWxhdGlvbnNoaXBzLnB1c2gobmV3IEVpdGhlcihhcmdzKSk7XG4gIH1cbn1cbiJdfQ==