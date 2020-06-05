"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.AssociationList = exports.InList = exports.HasList = exports.HasMany = exports.BelongsTo = exports.Association = void 0;

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _assertThisInitialized2 = _interopRequireDefault(require("@babel/runtime/helpers/assertThisInitialized"));

var _inherits2 = _interopRequireDefault(require("@babel/runtime/helpers/inherits"));

var _possibleConstructorReturn2 = _interopRequireDefault(require("@babel/runtime/helpers/possibleConstructorReturn"));

var _getPrototypeOf2 = _interopRequireDefault(require("@babel/runtime/helpers/getPrototypeOf"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

function _createSuper(Derived) { var hasNativeReflectConstruct = _isNativeReflectConstruct(); return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (hasNativeReflectConstruct) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

function _isNativeReflectConstruct() { if (typeof Reflect === "undefined" || !Reflect.construct) return false; if (Reflect.construct.sham) return false; if (typeof Proxy === "function") return true; try { Date.prototype.toString.call(Reflect.construct(Date, [], function () {})); return true; } catch (e) { return false; } }

var Association = function Association(args) {
  (0, _classCallCheck2["default"])(this, Association);
  (0, _defineProperty2["default"])(this, "typeName", void 0);
  (0, _defineProperty2["default"])(this, "methodName", void 0);
  (0, _defineProperty2["default"])(this, "methodArgumentName", void 0);
  (0, _defineProperty2["default"])(this, "fieldName", void 0);
  this.typeName = args.typeName;
  this.methodName = args.methodName;
  this.methodArgumentName = args.methodArgumentName;

  if (args.fieldName == undefined) {
    this.fieldName = args.typeName;
  } else {
    this.fieldName = args.fieldName;
  }
};

exports.Association = Association;

var BelongsTo = /*#__PURE__*/function (_Association) {
  (0, _inherits2["default"])(BelongsTo, _Association);

  var _super = _createSuper(BelongsTo);

  function BelongsTo(args) {
    var _this;

    (0, _classCallCheck2["default"])(this, BelongsTo);

    if (args.methodName == undefined) {
      args.methodName = "get";
    }

    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = "id";
    }

    _this = _super.call(this, args);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this), "fromFieldPath", void 0);
    _this.fromFieldPath = args.fromFieldPath;
    return _this;
  }

  (0, _createClass2["default"])(BelongsTo, [{
    key: "kind",
    value: function kind() {
      return "belongsTo";
    }
  }]);
  return BelongsTo;
}(Association);

exports.BelongsTo = BelongsTo;

var HasMany = /*#__PURE__*/function (_Association2) {
  (0, _inherits2["default"])(HasMany, _Association2);

  var _super2 = _createSuper(HasMany);

  function HasMany(args) {
    var _this2;

    (0, _classCallCheck2["default"])(this, HasMany);

    if (args.methodName == undefined) {
      args.methodName = "list";
    }

    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = "input";
    }

    _this2 = _super2.call(this, args);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "fromFieldPath", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this2), "queryField", void 0);

    if (args.fromFieldPath) {
      _this2.fromFieldPath = args.fromFieldPath;
    } else {
      _this2.fromFieldPath = ["id"];
    }

    if (args.queryField) {
      _this2.queryField = args.queryField;
    } else {
      _this2.queryField = "scopeByTenantId";
    }

    return _this2;
  }

  (0, _createClass2["default"])(HasMany, [{
    key: "kind",
    value: function kind() {
      return "hasMany";
    }
  }]);
  return HasMany;
}(Association);

exports.HasMany = HasMany;

var HasList = /*#__PURE__*/function (_Association3) {
  (0, _inherits2["default"])(HasList, _Association3);

  var _super3 = _createSuper(HasList);

  function HasList(args) {
    var _this3;

    (0, _classCallCheck2["default"])(this, HasList);

    if (args.methodName == undefined) {
      args.methodName = "list";
    }

    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = "input";
    }

    _this3 = _super3.call(this, args);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this3), "fromFieldPath", void 0);
    _this3.fromFieldPath = args.fromFieldPath;
    return _this3;
  }

  (0, _createClass2["default"])(HasList, [{
    key: "kind",
    value: function kind() {
      return "hasList";
    }
  }]);
  return HasList;
}(Association);

exports.HasList = HasList;

var InList = /*#__PURE__*/function (_Association4) {
  (0, _inherits2["default"])(InList, _Association4);

  var _super4 = _createSuper(InList);

  function InList(args) {
    var _this4;

    (0, _classCallCheck2["default"])(this, InList);

    if (args.methodName == undefined) {
      args.methodName = "list";
    }

    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = "input";
    }

    _this4 = _super4.call(this, args);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this4), "fromFieldPath", void 0);
    (0, _defineProperty2["default"])((0, _assertThisInitialized2["default"])(_this4), "toFieldPath", void 0);

    if (args.fromFieldPath) {
      _this4.fromFieldPath = args.fromFieldPath;
    } else {
      _this4.fromFieldPath = ["id"];
    }

    _this4.toFieldPath = args.toFieldPath;
    return _this4;
  }

  (0, _createClass2["default"])(InList, [{
    key: "kind",
    value: function kind() {
      return "inList";
    }
  }]);
  return InList;
}(Association);

exports.InList = InList;

var AssociationList = /*#__PURE__*/function () {
  function AssociationList() {
    (0, _classCallCheck2["default"])(this, AssociationList);
    (0, _defineProperty2["default"])(this, "associations", []);
  }

  (0, _createClass2["default"])(AssociationList, [{
    key: "all",
    value: function all() {
      return this.associations;
    }
  }, {
    key: "getByFieldName",
    value: function getByFieldName(fieldName) {
      var result = this.associations.find(function (a) {
        return a.fieldName == fieldName;
      });

      if (result == undefined) {
        throw new Error("Cannot get association field ".concat(fieldName, "; it does not exist on the object"));
      }

      return result;
    }
  }, {
    key: "getByTypeName",
    value: function getByTypeName(typeName) {
      var result = this.associations.find(function (a) {
        return a.typeName == typeName;
      });

      if (result == undefined) {
        throw new Error("Cannot get association type ".concat(typeName, "; it does not exist on the object"));
      }

      return result;
    }
  }, {
    key: "belongsTo",
    value: function belongsTo(args) {
      var assoc = new BelongsTo(args);
      this.associations.push(assoc);
      return assoc;
    }
  }, {
    key: "hasMany",
    value: function hasMany(args) {
      var assoc = new HasMany(args);
      this.associations.push(assoc);
      return assoc;
    }
  }, {
    key: "hasList",
    value: function hasList(args) {
      var assoc = new HasList(args);
      this.associations.push(assoc);
      return assoc;
    }
  }, {
    key: "inList",
    value: function inList(args) {
      var assoc = new InList(args);
      this.associations.push(assoc);
      return assoc;
    }
  }]);
  return AssociationList;
}();

exports.AssociationList = AssociationList;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvYXNzb2NpYXRpb25zLnRzIl0sIm5hbWVzIjpbIkFzc29jaWF0aW9uIiwiYXJncyIsInR5cGVOYW1lIiwibWV0aG9kTmFtZSIsIm1ldGhvZEFyZ3VtZW50TmFtZSIsImZpZWxkTmFtZSIsInVuZGVmaW5lZCIsIkJlbG9uZ3NUbyIsImZyb21GaWVsZFBhdGgiLCJIYXNNYW55IiwicXVlcnlGaWVsZCIsIkhhc0xpc3QiLCJJbkxpc3QiLCJ0b0ZpZWxkUGF0aCIsIkFzc29jaWF0aW9uTGlzdCIsImFzc29jaWF0aW9ucyIsInJlc3VsdCIsImZpbmQiLCJhIiwiRXJyb3IiLCJhc3NvYyIsInB1c2giXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztJQVNhQSxXLEdBTVgscUJBQVlDLElBQVosRUFBMEM7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQ3hDLE9BQUtDLFFBQUwsR0FBZ0JELElBQUksQ0FBQ0MsUUFBckI7QUFDQSxPQUFLQyxVQUFMLEdBQWtCRixJQUFJLENBQUNFLFVBQXZCO0FBQ0EsT0FBS0Msa0JBQUwsR0FBMEJILElBQUksQ0FBQ0csa0JBQS9COztBQUNBLE1BQUlILElBQUksQ0FBQ0ksU0FBTCxJQUFrQkMsU0FBdEIsRUFBaUM7QUFDL0IsU0FBS0QsU0FBTCxHQUFpQkosSUFBSSxDQUFDQyxRQUF0QjtBQUNELEdBRkQsTUFFTztBQUNMLFNBQUtHLFNBQUwsR0FBaUJKLElBQUksQ0FBQ0ksU0FBdEI7QUFDRDtBQUNGLEM7Ozs7SUFVVUUsUzs7Ozs7QUFHWCxxQkFBWU4sSUFBWixFQUF3QztBQUFBOztBQUFBOztBQUN0QyxRQUFJQSxJQUFJLENBQUNFLFVBQUwsSUFBbUJHLFNBQXZCLEVBQWtDO0FBQ2hDTCxNQUFBQSxJQUFJLENBQUNFLFVBQUwsR0FBa0IsS0FBbEI7QUFDRDs7QUFDRCxRQUFJRixJQUFJLENBQUNHLGtCQUFMLElBQTJCRSxTQUEvQixFQUEwQztBQUN4Q0wsTUFBQUEsSUFBSSxDQUFDRyxrQkFBTDtBQUNEOztBQUNELDhCQUFNSCxJQUFOO0FBUHNDO0FBUXRDLFVBQUtPLGFBQUwsR0FBcUJQLElBQUksQ0FBQ08sYUFBMUI7QUFSc0M7QUFTdkM7Ozs7MkJBRWM7QUFDYixhQUFPLFdBQVA7QUFDRDs7O0VBaEI0QlIsVzs7OztJQThCbEJTLE87Ozs7O0FBSVgsbUJBQVlSLElBQVosRUFBc0M7QUFBQTs7QUFBQTs7QUFDcEMsUUFBSUEsSUFBSSxDQUFDRSxVQUFMLElBQW1CRyxTQUF2QixFQUFrQztBQUNoQ0wsTUFBQUEsSUFBSSxDQUFDRSxVQUFMLEdBQWtCLE1BQWxCO0FBQ0Q7O0FBQ0QsUUFBSUYsSUFBSSxDQUFDRyxrQkFBTCxJQUEyQkUsU0FBL0IsRUFBMEM7QUFDeENMLE1BQUFBLElBQUksQ0FBQ0csa0JBQUw7QUFDRDs7QUFDRCxnQ0FBTUgsSUFBTjtBQVBvQztBQUFBOztBQVFwQyxRQUFJQSxJQUFJLENBQUNPLGFBQVQsRUFBd0I7QUFDdEIsYUFBS0EsYUFBTCxHQUFxQlAsSUFBSSxDQUFDTyxhQUExQjtBQUNELEtBRkQsTUFFTztBQUNMLGFBQUtBLGFBQUwsR0FBcUIsQ0FBQyxJQUFELENBQXJCO0FBQ0Q7O0FBQ0QsUUFBSVAsSUFBSSxDQUFDUyxVQUFULEVBQXFCO0FBQ25CLGFBQUtBLFVBQUwsR0FBa0JULElBQUksQ0FBQ1MsVUFBdkI7QUFDRCxLQUZELE1BRU87QUFDTCxhQUFLQSxVQUFMLEdBQWtCLGlCQUFsQjtBQUNEOztBQWpCbUM7QUFrQnJDOzs7OzJCQUVjO0FBQ2IsYUFBTyxTQUFQO0FBQ0Q7OztFQTFCMEJWLFc7Ozs7SUFvQ2hCVyxPOzs7OztBQUdYLG1CQUFZVixJQUFaLEVBQXNDO0FBQUE7O0FBQUE7O0FBQ3BDLFFBQUlBLElBQUksQ0FBQ0UsVUFBTCxJQUFtQkcsU0FBdkIsRUFBa0M7QUFDaENMLE1BQUFBLElBQUksQ0FBQ0UsVUFBTCxHQUFrQixNQUFsQjtBQUNEOztBQUNELFFBQUlGLElBQUksQ0FBQ0csa0JBQUwsSUFBMkJFLFNBQS9CLEVBQTBDO0FBQ3hDTCxNQUFBQSxJQUFJLENBQUNHLGtCQUFMO0FBQ0Q7O0FBQ0QsZ0NBQU1ILElBQU47QUFQb0M7QUFRcEMsV0FBS08sYUFBTCxHQUFxQlAsSUFBSSxDQUFDTyxhQUExQjtBQVJvQztBQVNyQzs7OzsyQkFFYztBQUNiLGFBQU8sU0FBUDtBQUNEOzs7RUFoQjBCUixXOzs7O0lBOEJoQlksTTs7Ozs7QUFJWCxrQkFBWVgsSUFBWixFQUFxQztBQUFBOztBQUFBOztBQUNuQyxRQUFJQSxJQUFJLENBQUNFLFVBQUwsSUFBbUJHLFNBQXZCLEVBQWtDO0FBQ2hDTCxNQUFBQSxJQUFJLENBQUNFLFVBQUwsR0FBa0IsTUFBbEI7QUFDRDs7QUFDRCxRQUFJRixJQUFJLENBQUNHLGtCQUFMLElBQTJCRSxTQUEvQixFQUEwQztBQUN4Q0wsTUFBQUEsSUFBSSxDQUFDRyxrQkFBTDtBQUNEOztBQUNELGdDQUFNSCxJQUFOO0FBUG1DO0FBQUE7O0FBUW5DLFFBQUlBLElBQUksQ0FBQ08sYUFBVCxFQUF3QjtBQUN0QixhQUFLQSxhQUFMLEdBQXFCUCxJQUFJLENBQUNPLGFBQTFCO0FBQ0QsS0FGRCxNQUVPO0FBQ0wsYUFBS0EsYUFBTCxHQUFxQixDQUFDLElBQUQsQ0FBckI7QUFDRDs7QUFDRCxXQUFLSyxXQUFMLEdBQW1CWixJQUFJLENBQUNZLFdBQXhCO0FBYm1DO0FBY3BDOzs7OzJCQUVjO0FBQ2IsYUFBTyxRQUFQO0FBQ0Q7OztFQXRCeUJiLFc7Ozs7SUF5QmZjLGU7OzsyREFDb0IsRTs7Ozs7MEJBRVE7QUFDckMsYUFBTyxLQUFLQyxZQUFaO0FBQ0Q7OzttQ0FFY1YsUyxFQUFpQztBQUM5QyxVQUFNVyxNQUFNLEdBQUcsS0FBS0QsWUFBTCxDQUFrQkUsSUFBbEIsQ0FBdUIsVUFBQUMsQ0FBQztBQUFBLGVBQUlBLENBQUMsQ0FBQ2IsU0FBRixJQUFlQSxTQUFuQjtBQUFBLE9BQXhCLENBQWY7O0FBQ0EsVUFBSVcsTUFBTSxJQUFJVixTQUFkLEVBQXlCO0FBQ3ZCLGNBQU0sSUFBSWEsS0FBSix3Q0FDNEJkLFNBRDVCLHVDQUFOO0FBR0Q7O0FBQ0QsYUFBT1csTUFBUDtBQUNEOzs7a0NBRWFkLFEsRUFBZ0M7QUFDNUMsVUFBTWMsTUFBTSxHQUFHLEtBQUtELFlBQUwsQ0FBa0JFLElBQWxCLENBQXVCLFVBQUFDLENBQUM7QUFBQSxlQUFJQSxDQUFDLENBQUNoQixRQUFGLElBQWNBLFFBQWxCO0FBQUEsT0FBeEIsQ0FBZjs7QUFDQSxVQUFJYyxNQUFNLElBQUlWLFNBQWQsRUFBeUI7QUFDdkIsY0FBTSxJQUFJYSxLQUFKLHVDQUMyQmpCLFFBRDNCLHVDQUFOO0FBR0Q7O0FBQ0QsYUFBT2MsTUFBUDtBQUNEOzs7OEJBRVNmLEksRUFBdUM7QUFDL0MsVUFBTW1CLEtBQUssR0FBRyxJQUFJYixTQUFKLENBQWNOLElBQWQsQ0FBZDtBQUNBLFdBQUtjLFlBQUwsQ0FBa0JNLElBQWxCLENBQXVCRCxLQUF2QjtBQUNBLGFBQU9BLEtBQVA7QUFDRDs7OzRCQUVPbkIsSSxFQUFtQztBQUN6QyxVQUFNbUIsS0FBSyxHQUFHLElBQUlYLE9BQUosQ0FBWVIsSUFBWixDQUFkO0FBQ0EsV0FBS2MsWUFBTCxDQUFrQk0sSUFBbEIsQ0FBdUJELEtBQXZCO0FBQ0EsYUFBT0EsS0FBUDtBQUNEOzs7NEJBRU9uQixJLEVBQW1DO0FBQ3pDLFVBQU1tQixLQUFLLEdBQUcsSUFBSVQsT0FBSixDQUFZVixJQUFaLENBQWQ7QUFDQSxXQUFLYyxZQUFMLENBQWtCTSxJQUFsQixDQUF1QkQsS0FBdkI7QUFDQSxhQUFPQSxLQUFQO0FBQ0Q7OzsyQkFFTW5CLEksRUFBaUM7QUFDdEMsVUFBTW1CLEtBQUssR0FBRyxJQUFJUixNQUFKLENBQVdYLElBQVgsQ0FBZDtBQUNBLFdBQUtjLFlBQUwsQ0FBa0JNLElBQWxCLENBQXVCRCxLQUF2QjtBQUNBLGFBQU9BLEtBQVA7QUFDRCIsInNvdXJjZXNDb250ZW50IjpbImV4cG9ydCB0eXBlIEFzc29jaWF0aW9ucyA9IEJlbG9uZ3NUbztcblxuaW50ZXJmYWNlIEFzc29jaWF0aW9uQ29uc3RydWN0b3Ige1xuICB0eXBlTmFtZTogQXNzb2NpYXRpb25bXCJ0eXBlTmFtZVwiXTtcbiAgbWV0aG9kTmFtZTogQXNzb2NpYXRpb25bXCJtZXRob2ROYW1lXCJdO1xuICBtZXRob2RBcmd1bWVudE5hbWU6IEJlbG9uZ3NUb1tcIm1ldGhvZEFyZ3VtZW50TmFtZVwiXTtcbiAgZmllbGROYW1lPzogQXNzb2NpYXRpb25bXCJmaWVsZE5hbWVcIl07XG59XG5cbmV4cG9ydCBjbGFzcyBBc3NvY2lhdGlvbiB7XG4gIHR5cGVOYW1lOiBzdHJpbmc7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgbWV0aG9kQXJndW1lbnROYW1lOiBzdHJpbmc7XG4gIGZpZWxkTmFtZTogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEFzc29jaWF0aW9uQ29uc3RydWN0b3IpIHtcbiAgICB0aGlzLnR5cGVOYW1lID0gYXJncy50eXBlTmFtZTtcbiAgICB0aGlzLm1ldGhvZE5hbWUgPSBhcmdzLm1ldGhvZE5hbWU7XG4gICAgdGhpcy5tZXRob2RBcmd1bWVudE5hbWUgPSBhcmdzLm1ldGhvZEFyZ3VtZW50TmFtZTtcbiAgICBpZiAoYXJncy5maWVsZE5hbWUgPT0gdW5kZWZpbmVkKSB7XG4gICAgICB0aGlzLmZpZWxkTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRoaXMuZmllbGROYW1lID0gYXJncy5maWVsZE5hbWU7XG4gICAgfVxuICB9XG59XG5cbmludGVyZmFjZSBCZWxvbmdzVG9Db25zdHJ1Y3RvclxuICBleHRlbmRzIE9taXQ8QXNzb2NpYXRpb25Db25zdHJ1Y3RvciwgXCJtZXRob2ROYW1lXCIgfCBcIm1ldGhvZEFyZ3VtZW50TmFtZVwiPiB7XG4gIGZyb21GaWVsZFBhdGg6IEJlbG9uZ3NUb1tcImZyb21GaWVsZFBhdGhcIl07XG4gIG1ldGhvZE5hbWU/OiBBc3NvY2lhdGlvbltcIm1ldGhvZE5hbWVcIl07XG4gIG1ldGhvZEFyZ3VtZW50TmFtZT86IEFzc29jaWF0aW9uW1wibWV0aG9kQXJndW1lbnROYW1lXCJdO1xufVxuXG5leHBvcnQgY2xhc3MgQmVsb25nc1RvIGV4dGVuZHMgQXNzb2NpYXRpb24ge1xuICBmcm9tRmllbGRQYXRoOiBzdHJpbmdbXTtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBCZWxvbmdzVG9Db25zdHJ1Y3Rvcikge1xuICAgIGlmIChhcmdzLm1ldGhvZE5hbWUgPT0gdW5kZWZpbmVkKSB7XG4gICAgICBhcmdzLm1ldGhvZE5hbWUgPSBcImdldFwiO1xuICAgIH1cbiAgICBpZiAoYXJncy5tZXRob2RBcmd1bWVudE5hbWUgPT0gdW5kZWZpbmVkKSB7XG4gICAgICBhcmdzLm1ldGhvZEFyZ3VtZW50TmFtZSA9IGBpZGA7XG4gICAgfVxuICAgIHN1cGVyKGFyZ3MgYXMgQXNzb2NpYXRpb25Db25zdHJ1Y3Rvcik7XG4gICAgdGhpcy5mcm9tRmllbGRQYXRoID0gYXJncy5mcm9tRmllbGRQYXRoO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImJlbG9uZ3NUb1wiO1xuICB9XG59XG5cbmludGVyZmFjZSBIYXNNYW55Q29uc3RydWN0b3JcbiAgZXh0ZW5kcyBPbWl0PFxuICAgIEFzc29jaWF0aW9uQ29uc3RydWN0b3IsXG4gICAgXCJtZXRob2ROYW1lXCIgfCBcIm1ldGhvZEFyZ3VtZW50TmFtZVwiIHwgXCJmcm9tRmllbGRQYXRoXCJcbiAgPiB7XG4gIGZyb21GaWVsZFBhdGg/OiBIYXNNYW55W1wiZnJvbUZpZWxkUGF0aFwiXTtcbiAgbWV0aG9kTmFtZT86IEFzc29jaWF0aW9uW1wibWV0aG9kTmFtZVwiXTtcbiAgbWV0aG9kQXJndW1lbnROYW1lPzogQXNzb2NpYXRpb25bXCJtZXRob2RBcmd1bWVudE5hbWVcIl07XG4gIHF1ZXJ5RmllbGQ/OiBIYXNNYW55W1wicXVlcnlGaWVsZFwiXTtcbn1cblxuZXhwb3J0IGNsYXNzIEhhc01hbnkgZXh0ZW5kcyBBc3NvY2lhdGlvbiB7XG4gIGZyb21GaWVsZFBhdGg6IHN0cmluZ1tdO1xuICBxdWVyeUZpZWxkOiBzdHJpbmc7XG5cbiAgY29uc3RydWN0b3IoYXJnczogSGFzTWFueUNvbnN0cnVjdG9yKSB7XG4gICAgaWYgKGFyZ3MubWV0aG9kTmFtZSA9PSB1bmRlZmluZWQpIHtcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSA9IFwibGlzdFwiO1xuICAgIH1cbiAgICBpZiAoYXJncy5tZXRob2RBcmd1bWVudE5hbWUgPT0gdW5kZWZpbmVkKSB7XG4gICAgICBhcmdzLm1ldGhvZEFyZ3VtZW50TmFtZSA9IGBpbnB1dGA7XG4gICAgfVxuICAgIHN1cGVyKGFyZ3MgYXMgQXNzb2NpYXRpb25Db25zdHJ1Y3Rvcik7XG4gICAgaWYgKGFyZ3MuZnJvbUZpZWxkUGF0aCkge1xuICAgICAgdGhpcy5mcm9tRmllbGRQYXRoID0gYXJncy5mcm9tRmllbGRQYXRoO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aGlzLmZyb21GaWVsZFBhdGggPSBbXCJpZFwiXTtcbiAgICB9XG4gICAgaWYgKGFyZ3MucXVlcnlGaWVsZCkge1xuICAgICAgdGhpcy5xdWVyeUZpZWxkID0gYXJncy5xdWVyeUZpZWxkO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aGlzLnF1ZXJ5RmllbGQgPSBcInNjb3BlQnlUZW5hbnRJZFwiO1xuICAgIH1cbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJoYXNNYW55XCI7XG4gIH1cbn1cblxuaW50ZXJmYWNlIEhhc0xpc3RDb25zdHJ1Y3RvclxuICBleHRlbmRzIE9taXQ8QXNzb2NpYXRpb25Db25zdHJ1Y3RvciwgXCJtZXRob2ROYW1lXCIgfCBcIm1ldGhvZEFyZ3VtZW50TmFtZVwiPiB7XG4gIGZyb21GaWVsZFBhdGg6IEhhc0xpc3RbXCJmcm9tRmllbGRQYXRoXCJdO1xuICBtZXRob2ROYW1lPzogQXNzb2NpYXRpb25bXCJtZXRob2ROYW1lXCJdO1xuICBtZXRob2RBcmd1bWVudE5hbWU/OiBBc3NvY2lhdGlvbltcIm1ldGhvZEFyZ3VtZW50TmFtZVwiXTtcbn1cblxuZXhwb3J0IGNsYXNzIEhhc0xpc3QgZXh0ZW5kcyBBc3NvY2lhdGlvbiB7XG4gIGZyb21GaWVsZFBhdGg6IHN0cmluZ1tdO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEhhc0xpc3RDb25zdHJ1Y3Rvcikge1xuICAgIGlmIChhcmdzLm1ldGhvZE5hbWUgPT0gdW5kZWZpbmVkKSB7XG4gICAgICBhcmdzLm1ldGhvZE5hbWUgPSBcImxpc3RcIjtcbiAgICB9XG4gICAgaWYgKGFyZ3MubWV0aG9kQXJndW1lbnROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgYXJncy5tZXRob2RBcmd1bWVudE5hbWUgPSBgaW5wdXRgO1xuICAgIH1cbiAgICBzdXBlcihhcmdzIGFzIEFzc29jaWF0aW9uQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuZnJvbUZpZWxkUGF0aCA9IGFyZ3MuZnJvbUZpZWxkUGF0aDtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJoYXNMaXN0XCI7XG4gIH1cbn1cblxuaW50ZXJmYWNlIEluTGlzdENvbnN0cnVjdG9yXG4gIGV4dGVuZHMgT21pdDxcbiAgICBBc3NvY2lhdGlvbkNvbnN0cnVjdG9yLFxuICAgIFwibWV0aG9kTmFtZVwiIHwgXCJtZXRob2RBcmd1bWVudE5hbWVcIiB8IFwiZnJvbUZpZWxkUGF0aFwiXG4gID4ge1xuICB0b0ZpZWxkUGF0aDogSW5MaXN0W1widG9GaWVsZFBhdGhcIl07XG4gIGZyb21GaWVsZFBhdGg/OiBJbkxpc3RbXCJmcm9tRmllbGRQYXRoXCJdO1xuICBtZXRob2ROYW1lPzogQXNzb2NpYXRpb25bXCJtZXRob2ROYW1lXCJdO1xuICBtZXRob2RBcmd1bWVudE5hbWU/OiBBc3NvY2lhdGlvbltcIm1ldGhvZEFyZ3VtZW50TmFtZVwiXTtcbn1cblxuZXhwb3J0IGNsYXNzIEluTGlzdCBleHRlbmRzIEFzc29jaWF0aW9uIHtcbiAgZnJvbUZpZWxkUGF0aDogc3RyaW5nW107XG4gIHRvRmllbGRQYXRoOiBzdHJpbmdbXTtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBJbkxpc3RDb25zdHJ1Y3Rvcikge1xuICAgIGlmIChhcmdzLm1ldGhvZE5hbWUgPT0gdW5kZWZpbmVkKSB7XG4gICAgICBhcmdzLm1ldGhvZE5hbWUgPSBcImxpc3RcIjtcbiAgICB9XG4gICAgaWYgKGFyZ3MubWV0aG9kQXJndW1lbnROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgYXJncy5tZXRob2RBcmd1bWVudE5hbWUgPSBgaW5wdXRgO1xuICAgIH1cbiAgICBzdXBlcihhcmdzIGFzIEFzc29jaWF0aW9uQ29uc3RydWN0b3IpO1xuICAgIGlmIChhcmdzLmZyb21GaWVsZFBhdGgpIHtcbiAgICAgIHRoaXMuZnJvbUZpZWxkUGF0aCA9IGFyZ3MuZnJvbUZpZWxkUGF0aDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhpcy5mcm9tRmllbGRQYXRoID0gW1wiaWRcIl07XG4gICAgfVxuICAgIHRoaXMudG9GaWVsZFBhdGggPSBhcmdzLnRvRmllbGRQYXRoO1xuICB9XG5cbiAga2luZCgpOiBzdHJpbmcge1xuICAgIHJldHVybiBcImluTGlzdFwiO1xuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBBc3NvY2lhdGlvbkxpc3Qge1xuICBhc3NvY2lhdGlvbnM6IEFzc29jaWF0aW9uc1tdID0gW107XG5cbiAgYWxsKCk6IEFzc29jaWF0aW9uTGlzdFtcImFzc29jaWF0aW9uc1wiXSB7XG4gICAgcmV0dXJuIHRoaXMuYXNzb2NpYXRpb25zO1xuICB9XG5cbiAgZ2V0QnlGaWVsZE5hbWUoZmllbGROYW1lOiBzdHJpbmcpOiBBc3NvY2lhdGlvbnMge1xuICAgIGNvbnN0IHJlc3VsdCA9IHRoaXMuYXNzb2NpYXRpb25zLmZpbmQoYSA9PiBhLmZpZWxkTmFtZSA9PSBmaWVsZE5hbWUpO1xuICAgIGlmIChyZXN1bHQgPT0gdW5kZWZpbmVkKSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoXG4gICAgICAgIGBDYW5ub3QgZ2V0IGFzc29jaWF0aW9uIGZpZWxkICR7ZmllbGROYW1lfTsgaXQgZG9lcyBub3QgZXhpc3Qgb24gdGhlIG9iamVjdGAsXG4gICAgICApO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgZ2V0QnlUeXBlTmFtZSh0eXBlTmFtZTogc3RyaW5nKTogQXNzb2NpYXRpb25zIHtcbiAgICBjb25zdCByZXN1bHQgPSB0aGlzLmFzc29jaWF0aW9ucy5maW5kKGEgPT4gYS50eXBlTmFtZSA9PSB0eXBlTmFtZSk7XG4gICAgaWYgKHJlc3VsdCA9PSB1bmRlZmluZWQpIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgYENhbm5vdCBnZXQgYXNzb2NpYXRpb24gdHlwZSAke3R5cGVOYW1lfTsgaXQgZG9lcyBub3QgZXhpc3Qgb24gdGhlIG9iamVjdGAsXG4gICAgICApO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgYmVsb25nc1RvKGFyZ3M6IEJlbG9uZ3NUb0NvbnN0cnVjdG9yKTogQmVsb25nc1RvIHtcbiAgICBjb25zdCBhc3NvYyA9IG5ldyBCZWxvbmdzVG8oYXJncyk7XG4gICAgdGhpcy5hc3NvY2lhdGlvbnMucHVzaChhc3NvYyk7XG4gICAgcmV0dXJuIGFzc29jO1xuICB9XG5cbiAgaGFzTWFueShhcmdzOiBIYXNNYW55Q29uc3RydWN0b3IpOiBIYXNNYW55IHtcbiAgICBjb25zdCBhc3NvYyA9IG5ldyBIYXNNYW55KGFyZ3MpO1xuICAgIHRoaXMuYXNzb2NpYXRpb25zLnB1c2goYXNzb2MpO1xuICAgIHJldHVybiBhc3NvYztcbiAgfVxuXG4gIGhhc0xpc3QoYXJnczogSGFzTGlzdENvbnN0cnVjdG9yKTogSGFzTGlzdCB7XG4gICAgY29uc3QgYXNzb2MgPSBuZXcgSGFzTGlzdChhcmdzKTtcbiAgICB0aGlzLmFzc29jaWF0aW9ucy5wdXNoKGFzc29jKTtcbiAgICByZXR1cm4gYXNzb2M7XG4gIH1cblxuICBpbkxpc3QoYXJnczogSW5MaXN0Q29uc3RydWN0b3IpOiBJbkxpc3Qge1xuICAgIGNvbnN0IGFzc29jID0gbmV3IEluTGlzdChhcmdzKTtcbiAgICB0aGlzLmFzc29jaWF0aW9ucy5wdXNoKGFzc29jKTtcbiAgICByZXR1cm4gYXNzb2M7XG4gIH1cbn1cbiJdfQ==