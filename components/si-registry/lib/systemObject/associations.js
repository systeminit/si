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

function _createSuper(Derived) { return function () { var Super = (0, _getPrototypeOf2["default"])(Derived), result; if (_isNativeReflectConstruct()) { var NewTarget = (0, _getPrototypeOf2["default"])(this).constructor; result = Reflect.construct(Super, arguments, NewTarget); } else { result = Super.apply(this, arguments); } return (0, _possibleConstructorReturn2["default"])(this, result); }; }

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
        throw "Cannot get association field ".concat(fieldName, "; it does not exist on the object");
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
        throw "Cannot get association type ".concat(typeName, "; it does not exist on the object");
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvYXNzb2NpYXRpb25zLnRzIl0sIm5hbWVzIjpbIkFzc29jaWF0aW9uIiwiYXJncyIsInR5cGVOYW1lIiwibWV0aG9kTmFtZSIsIm1ldGhvZEFyZ3VtZW50TmFtZSIsImZpZWxkTmFtZSIsInVuZGVmaW5lZCIsIkJlbG9uZ3NUbyIsImZyb21GaWVsZFBhdGgiLCJIYXNNYW55IiwicXVlcnlGaWVsZCIsIkhhc0xpc3QiLCJJbkxpc3QiLCJ0b0ZpZWxkUGF0aCIsIkFzc29jaWF0aW9uTGlzdCIsImFzc29jaWF0aW9ucyIsInJlc3VsdCIsImZpbmQiLCJhIiwiYXNzb2MiLCJwdXNoIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7SUFTYUEsVyxHQU1YLHFCQUFZQyxJQUFaLEVBQTBDO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUN4QyxPQUFLQyxRQUFMLEdBQWdCRCxJQUFJLENBQUNDLFFBQXJCO0FBQ0EsT0FBS0MsVUFBTCxHQUFrQkYsSUFBSSxDQUFDRSxVQUF2QjtBQUNBLE9BQUtDLGtCQUFMLEdBQTBCSCxJQUFJLENBQUNHLGtCQUEvQjs7QUFDQSxNQUFJSCxJQUFJLENBQUNJLFNBQUwsSUFBa0JDLFNBQXRCLEVBQWlDO0FBQy9CLFNBQUtELFNBQUwsR0FBaUJKLElBQUksQ0FBQ0MsUUFBdEI7QUFDRCxHQUZELE1BRU87QUFDTCxTQUFLRyxTQUFMLEdBQWlCSixJQUFJLENBQUNJLFNBQXRCO0FBQ0Q7QUFDRixDOzs7O0lBVVVFLFM7Ozs7O0FBR1gscUJBQVlOLElBQVosRUFBd0M7QUFBQTs7QUFBQTs7QUFDdEMsUUFBSUEsSUFBSSxDQUFDRSxVQUFMLElBQW1CRyxTQUF2QixFQUFrQztBQUNoQ0wsTUFBQUEsSUFBSSxDQUFDRSxVQUFMLEdBQWtCLEtBQWxCO0FBQ0Q7O0FBQ0QsUUFBSUYsSUFBSSxDQUFDRyxrQkFBTCxJQUEyQkUsU0FBL0IsRUFBMEM7QUFDeENMLE1BQUFBLElBQUks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T1csTUFBUDtBQUNEOzs7a0NBRWFkLFEsRUFBZ0M7QUFDNUMsVUFBTWMsTUFBTSxHQUFHLEtBQUtELFlBQUwsQ0FBa0JFLElBQWxCLENBQXVCLFVBQUFDLENBQUM7QUFBQSxlQUFJQSxDQUFDLENBQUNoQixRQUFGLElBQWNBLFFBQWxCO0FBQUEsT0FBeEIsQ0FBZjs7QUFDQSxVQUFJYyxNQUFNLElBQUlWLFNBQWQsRUFBeUI7QUFDdkIsb0RBQXFDSixRQUFyQztBQUNEOztBQUNELGFBQU9jLE1BQVA7QUFDRDs7OzhCQUVTZixJLEVBQXVDO0FBQy9DLFVBQU1rQixLQUFLLEdBQUcsSUFBSVosU0FBSixDQUFjTixJQUFkLENBQWQ7QUFDQSxXQUFLYyxZQUFMLENBQWtCSyxJQUFsQixDQUF1QkQsS0FBdkI7QUFDQSxhQUFPQSxLQUFQO0FBQ0Q7Ozs0QkFFT2xCLEksRUFBbUM7QUFDekMsVUFBTWtCLEtBQUssR0FBRyxJQUFJVixPQUFKLENBQVlSLElBQVosQ0FBZDtBQUNBLFdBQUtjLFlBQUwsQ0FBa0JLLElBQWxCLENBQXVCRCxLQUF2QjtBQUNBLGFBQU9BLEtBQVA7QUFDRDs7OzRCQUVPbEIsSSxFQUFtQztBQUN6QyxVQUFNa0IsS0FBSyxHQUFHLElBQUlSLE9BQUosQ0FBWVYsSUFBWixDQUFkO0FBQ0EsV0FBS2MsWUFBTCxDQUFrQkssSUFBbEIsQ0FBdUJELEtBQXZCO0FBQ0EsYUFBT0EsS0FBUDtBQUNEOzs7MkJBRU1sQixJLEVBQWlDO0FBQ3RDLFVBQU1rQixLQUFLLEdBQUcsSUFBSVAsTUFBSixDQUFXWCxJQUFYLENBQWQ7QUFDQSxXQUFLYyxZQUFMLENBQWtCSyxJQUFsQixDQUF1QkQsS0FBdkI7QUFDQSxhQUFPQSxLQUFQO0FBQ0QiLCJzb3VyY2VzQ29udGVudCI6WyJleHBvcnQgdHlwZSBBc3NvY2lhdGlvbnMgPSBCZWxvbmdzVG87XG5cbmludGVyZmFjZSBBc3NvY2lhdGlvbkNvbnN0cnVjdG9yIHtcbiAgdHlwZU5hbWU6IEFzc29jaWF0aW9uW1widHlwZU5hbWVcIl07XG4gIG1ldGhvZE5hbWU6IEFzc29jaWF0aW9uW1wibWV0aG9kTmFtZVwiXTtcbiAgbWV0aG9kQXJndW1lbnROYW1lOiBCZWxvbmdzVG9bXCJtZXRob2RBcmd1bWVudE5hbWVcIl07XG4gIGZpZWxkTmFtZT86IEFzc29jaWF0aW9uW1wiZmllbGROYW1lXCJdO1xufVxuXG5leHBvcnQgY2xhc3MgQXNzb2NpYXRpb24ge1xuICB0eXBlTmFtZTogc3RyaW5nO1xuICBtZXRob2ROYW1lOiBzdHJpbmc7XG4gIG1ldGhvZEFyZ3VtZW50TmFtZTogc3RyaW5nO1xuICBmaWVsZE5hbWU6IHN0cmluZztcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBBc3NvY2lhdGlvbkNvbnN0cnVjdG9yKSB7XG4gICAgdGhpcy50eXBlTmFtZSA9IGFyZ3MudHlwZU5hbWU7XG4gICAgdGhpcy5tZXRob2ROYW1lID0gYXJncy5tZXRob2ROYW1lO1xuICAgIHRoaXMubWV0aG9kQXJndW1lbnROYW1lID0gYXJncy5tZXRob2RBcmd1bWVudE5hbWU7XG4gICAgaWYgKGFyZ3MuZmllbGROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgdGhpcy5maWVsZE5hbWUgPSBhcmdzLnR5cGVOYW1lO1xuICAgIH0gZWxzZSB7XG4gICAgICB0aGlzLmZpZWxkTmFtZSA9IGFyZ3MuZmllbGROYW1lO1xuICAgIH1cbiAgfVxufVxuXG5pbnRlcmZhY2UgQmVsb25nc1RvQ29uc3RydWN0b3JcbiAgZXh0ZW5kcyBPbWl0PEFzc29jaWF0aW9uQ29uc3RydWN0b3IsIFwibWV0aG9kTmFtZVwiIHwgXCJtZXRob2RBcmd1bWVudE5hbWVcIj4ge1xuICBmcm9tRmllbGRQYXRoOiBCZWxvbmdzVG9bXCJmcm9tRmllbGRQYXRoXCJdO1xuICBtZXRob2ROYW1lPzogQXNzb2NpYXRpb25bXCJtZXRob2ROYW1lXCJdO1xuICBtZXRob2RBcmd1bWVudE5hbWU/OiBBc3NvY2lhdGlvbltcIm1ldGhvZEFyZ3VtZW50TmFtZVwiXTtcbn1cblxuZXhwb3J0IGNsYXNzIEJlbG9uZ3NUbyBleHRlbmRzIEFzc29jaWF0aW9uIHtcbiAgZnJvbUZpZWxkUGF0aDogc3RyaW5nW107XG5cbiAgY29uc3RydWN0b3IoYXJnczogQmVsb25nc1RvQ29uc3RydWN0b3IpIHtcbiAgICBpZiAoYXJncy5tZXRob2ROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgYXJncy5tZXRob2ROYW1lID0gXCJnZXRcIjtcbiAgICB9XG4gICAgaWYgKGFyZ3MubWV0aG9kQXJndW1lbnROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgYXJncy5tZXRob2RBcmd1bWVudE5hbWUgPSBgaWRgO1xuICAgIH1cbiAgICBzdXBlcihhcmdzIGFzIEFzc29jaWF0aW9uQ29uc3RydWN0b3IpO1xuICAgIHRoaXMuZnJvbUZpZWxkUGF0aCA9IGFyZ3MuZnJvbUZpZWxkUGF0aDtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJiZWxvbmdzVG9cIjtcbiAgfVxufVxuXG5pbnRlcmZhY2UgSGFzTWFueUNvbnN0cnVjdG9yXG4gIGV4dGVuZHMgT21pdDxcbiAgICBBc3NvY2lhdGlvbkNvbnN0cnVjdG9yLFxuICAgIFwibWV0aG9kTmFtZVwiIHwgXCJtZXRob2RBcmd1bWVudE5hbWVcIiB8IFwiZnJvbUZpZWxkUGF0aFwiXG4gID4ge1xuICBmcm9tRmllbGRQYXRoPzogSGFzTWFueVtcImZyb21GaWVsZFBhdGhcIl07XG4gIG1ldGhvZE5hbWU/OiBBc3NvY2lhdGlvbltcIm1ldGhvZE5hbWVcIl07XG4gIG1ldGhvZEFyZ3VtZW50TmFtZT86IEFzc29jaWF0aW9uW1wibWV0aG9kQXJndW1lbnROYW1lXCJdO1xuICBxdWVyeUZpZWxkPzogSGFzTWFueVtcInF1ZXJ5RmllbGRcIl07XG59XG5cbmV4cG9ydCBjbGFzcyBIYXNNYW55IGV4dGVuZHMgQXNzb2NpYXRpb24ge1xuICBmcm9tRmllbGRQYXRoOiBzdHJpbmdbXTtcbiAgcXVlcnlGaWVsZDogc3RyaW5nO1xuXG4gIGNvbnN0cnVjdG9yKGFyZ3M6IEhhc01hbnlDb25zdHJ1Y3Rvcikge1xuICAgIGlmIChhcmdzLm1ldGhvZE5hbWUgPT0gdW5kZWZpbmVkKSB7XG4gICAgICBhcmdzLm1ldGhvZE5hbWUgPSBcImxpc3RcIjtcbiAgICB9XG4gICAgaWYgKGFyZ3MubWV0aG9kQXJndW1lbnROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgYXJncy5tZXRob2RBcmd1bWVudE5hbWUgPSBgaW5wdXRgO1xuICAgIH1cbiAgICBzdXBlcihhcmdzIGFzIEFzc29jaWF0aW9uQ29uc3RydWN0b3IpO1xuICAgIGlmIChhcmdzLmZyb21GaWVsZFBhdGgpIHtcbiAgICAgIHRoaXMuZnJvbUZpZWxkUGF0aCA9IGFyZ3MuZnJvbUZpZWxkUGF0aDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhpcy5mcm9tRmllbGRQYXRoID0gW1wiaWRcIl07XG4gICAgfVxuICAgIGlmIChhcmdzLnF1ZXJ5RmllbGQpIHtcbiAgICAgIHRoaXMucXVlcnlGaWVsZCA9IGFyZ3MucXVlcnlGaWVsZDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhpcy5xdWVyeUZpZWxkID0gXCJzY29wZUJ5VGVuYW50SWRcIjtcbiAgICB9XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiaGFzTWFueVwiO1xuICB9XG59XG5cbmludGVyZmFjZSBIYXNMaXN0Q29uc3RydWN0b3JcbiAgZXh0ZW5kcyBPbWl0PEFzc29jaWF0aW9uQ29uc3RydWN0b3IsIFwibWV0aG9kTmFtZVwiIHwgXCJtZXRob2RBcmd1bWVudE5hbWVcIj4ge1xuICBmcm9tRmllbGRQYXRoOiBIYXNMaXN0W1wiZnJvbUZpZWxkUGF0aFwiXTtcbiAgbWV0aG9kTmFtZT86IEFzc29jaWF0aW9uW1wibWV0aG9kTmFtZVwiXTtcbiAgbWV0aG9kQXJndW1lbnROYW1lPzogQXNzb2NpYXRpb25bXCJtZXRob2RBcmd1bWVudE5hbWVcIl07XG59XG5cbmV4cG9ydCBjbGFzcyBIYXNMaXN0IGV4dGVuZHMgQXNzb2NpYXRpb24ge1xuICBmcm9tRmllbGRQYXRoOiBzdHJpbmdbXTtcblxuICBjb25zdHJ1Y3RvcihhcmdzOiBIYXNMaXN0Q29uc3RydWN0b3IpIHtcbiAgICBpZiAoYXJncy5tZXRob2ROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgYXJncy5tZXRob2ROYW1lID0gXCJsaXN0XCI7XG4gICAgfVxuICAgIGlmIChhcmdzLm1ldGhvZEFyZ3VtZW50TmFtZSA9PSB1bmRlZmluZWQpIHtcbiAgICAgIGFyZ3MubWV0aG9kQXJndW1lbnROYW1lID0gYGlucHV0YDtcbiAgICB9XG4gICAgc3VwZXIoYXJncyBhcyBBc3NvY2lhdGlvbkNvbnN0cnVjdG9yKTtcbiAgICB0aGlzLmZyb21GaWVsZFBhdGggPSBhcmdzLmZyb21GaWVsZFBhdGg7XG4gIH1cblxuICBraW5kKCk6IHN0cmluZyB7XG4gICAgcmV0dXJuIFwiaGFzTGlzdFwiO1xuICB9XG59XG5cbmludGVyZmFjZSBJbkxpc3RDb25zdHJ1Y3RvclxuICBleHRlbmRzIE9taXQ8XG4gICAgQXNzb2NpYXRpb25Db25zdHJ1Y3RvcixcbiAgICBcIm1ldGhvZE5hbWVcIiB8IFwibWV0aG9kQXJndW1lbnROYW1lXCIgfCBcImZyb21GaWVsZFBhdGhcIlxuICA+IHtcbiAgdG9GaWVsZFBhdGg6IEluTGlzdFtcInRvRmllbGRQYXRoXCJdO1xuICBmcm9tRmllbGRQYXRoPzogSW5MaXN0W1wiZnJvbUZpZWxkUGF0aFwiXTtcbiAgbWV0aG9kTmFtZT86IEFzc29jaWF0aW9uW1wibWV0aG9kTmFtZVwiXTtcbiAgbWV0aG9kQXJndW1lbnROYW1lPzogQXNzb2NpYXRpb25bXCJtZXRob2RBcmd1bWVudE5hbWVcIl07XG59XG5cbmV4cG9ydCBjbGFzcyBJbkxpc3QgZXh0ZW5kcyBBc3NvY2lhdGlvbiB7XG4gIGZyb21GaWVsZFBhdGg6IHN0cmluZ1tdO1xuICB0b0ZpZWxkUGF0aDogc3RyaW5nW107XG5cbiAgY29uc3RydWN0b3IoYXJnczogSW5MaXN0Q29uc3RydWN0b3IpIHtcbiAgICBpZiAoYXJncy5tZXRob2ROYW1lID09IHVuZGVmaW5lZCkge1xuICAgICAgYXJncy5tZXRob2ROYW1lID0gXCJsaXN0XCI7XG4gICAgfVxuICAgIGlmIChhcmdzLm1ldGhvZEFyZ3VtZW50TmFtZSA9PSB1bmRlZmluZWQpIHtcbiAgICAgIGFyZ3MubWV0aG9kQXJndW1lbnROYW1lID0gYGlucHV0YDtcbiAgICB9XG4gICAgc3VwZXIoYXJncyBhcyBBc3NvY2lhdGlvbkNvbnN0cnVjdG9yKTtcbiAgICBpZiAoYXJncy5mcm9tRmllbGRQYXRoKSB7XG4gICAgICB0aGlzLmZyb21GaWVsZFBhdGggPSBhcmdzLmZyb21GaWVsZFBhdGg7XG4gICAgfSBlbHNlIHtcbiAgICAgIHRoaXMuZnJvbUZpZWxkUGF0aCA9IFtcImlkXCJdO1xuICAgIH1cbiAgICB0aGlzLnRvRmllbGRQYXRoID0gYXJncy50b0ZpZWxkUGF0aDtcbiAgfVxuXG4gIGtpbmQoKTogc3RyaW5nIHtcbiAgICByZXR1cm4gXCJpbkxpc3RcIjtcbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgQXNzb2NpYXRpb25MaXN0IHtcbiAgYXNzb2NpYXRpb25zOiBBc3NvY2lhdGlvbnNbXSA9IFtdO1xuXG4gIGFsbCgpOiBBc3NvY2lhdGlvbkxpc3RbXCJhc3NvY2lhdGlvbnNcIl0ge1xuICAgIHJldHVybiB0aGlzLmFzc29jaWF0aW9ucztcbiAgfVxuXG4gIGdldEJ5RmllbGROYW1lKGZpZWxkTmFtZTogc3RyaW5nKTogQXNzb2NpYXRpb25zIHtcbiAgICBjb25zdCByZXN1bHQgPSB0aGlzLmFzc29jaWF0aW9ucy5maW5kKGEgPT4gYS5maWVsZE5hbWUgPT0gZmllbGROYW1lKTtcbiAgICBpZiAocmVzdWx0ID09IHVuZGVmaW5lZCkge1xuICAgICAgdGhyb3cgYENhbm5vdCBnZXQgYXNzb2NpYXRpb24gZmllbGQgJHtmaWVsZE5hbWV9OyBpdCBkb2VzIG5vdCBleGlzdCBvbiB0aGUgb2JqZWN0YDtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIGdldEJ5VHlwZU5hbWUodHlwZU5hbWU6IHN0cmluZyk6IEFzc29jaWF0aW9ucyB7XG4gICAgY29uc3QgcmVzdWx0ID0gdGhpcy5hc3NvY2lhdGlvbnMuZmluZChhID0+IGEudHlwZU5hbWUgPT0gdHlwZU5hbWUpO1xuICAgIGlmIChyZXN1bHQgPT0gdW5kZWZpbmVkKSB7XG4gICAgICB0aHJvdyBgQ2Fubm90IGdldCBhc3NvY2lhdGlvbiB0eXBlICR7dHlwZU5hbWV9OyBpdCBkb2VzIG5vdCBleGlzdCBvbiB0aGUgb2JqZWN0YDtcbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIGJlbG9uZ3NUbyhhcmdzOiBCZWxvbmdzVG9Db25zdHJ1Y3Rvcik6IEJlbG9uZ3NUbyB7XG4gICAgY29uc3QgYXNzb2MgPSBuZXcgQmVsb25nc1RvKGFyZ3MpO1xuICAgIHRoaXMuYXNzb2NpYXRpb25zLnB1c2goYXNzb2MpO1xuICAgIHJldHVybiBhc3NvYztcbiAgfVxuXG4gIGhhc01hbnkoYXJnczogSGFzTWFueUNvbnN0cnVjdG9yKTogSGFzTWFueSB7XG4gICAgY29uc3QgYXNzb2MgPSBuZXcgSGFzTWFueShhcmdzKTtcbiAgICB0aGlzLmFzc29jaWF0aW9ucy5wdXNoKGFzc29jKTtcbiAgICByZXR1cm4gYXNzb2M7XG4gIH1cblxuICBoYXNMaXN0KGFyZ3M6IEhhc0xpc3RDb25zdHJ1Y3Rvcik6IEhhc0xpc3Qge1xuICAgIGNvbnN0IGFzc29jID0gbmV3IEhhc0xpc3QoYXJncyk7XG4gICAgdGhpcy5hc3NvY2lhdGlvbnMucHVzaChhc3NvYyk7XG4gICAgcmV0dXJuIGFzc29jO1xuICB9XG5cbiAgaW5MaXN0KGFyZ3M6IEluTGlzdENvbnN0cnVjdG9yKTogSW5MaXN0IHtcbiAgICBjb25zdCBhc3NvYyA9IG5ldyBJbkxpc3QoYXJncyk7XG4gICAgdGhpcy5hc3NvY2lhdGlvbnMucHVzaChhc3NvYyk7XG4gICAgcmV0dXJuIGFzc29jO1xuICB9XG59XG4iXX0=