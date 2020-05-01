"use strict";

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.Prop = void 0;

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

function _defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } }

function _createClass(Constructor, protoProps, staticProps) { if (protoProps) _defineProperties(Constructor.prototype, protoProps); if (staticProps) _defineProperties(Constructor, staticProps); return Constructor; }

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

var Prop = /*#__PURE__*/function () {
  // Hidden from the UI
  // Hidden from the API
  function Prop(_ref) {
    var name = _ref.name,
        label = _ref.label,
        componentTypeName = _ref.componentTypeName,
        rules = _ref.rules,
        required = _ref.required,
        readOnly = _ref.readOnly,
        hidden = _ref.hidden,
        repeated = _ref.repeated;

    _classCallCheck(this, Prop);

    _defineProperty(this, "name", void 0);

    _defineProperty(this, "label", void 0);

    _defineProperty(this, "rules", void 0);

    _defineProperty(this, "required", void 0);

    _defineProperty(this, "readOnly", void 0);

    _defineProperty(this, "hidden", void 0);

    _defineProperty(this, "repeated", void 0);

    _defineProperty(this, "universal", void 0);

    _defineProperty(this, "lookupTag", void 0);

    _defineProperty(this, "parentName", void 0);

    _defineProperty(this, "reference", void 0);

    _defineProperty(this, "componentTypeName", void 0);

    _defineProperty(this, "skip", void 0);

    this.name = name;
    this.label = label;
    this.componentTypeName = componentTypeName;
    this.rules = rules || [];
    this.required = required || false;
    this.readOnly = readOnly || false;
    this.hidden = hidden || false;
    this.repeated = repeated || false;
    this.universal = false;
    this.lookupTag = null;
    this.parentName = "";
    this.reference = false;
    this.skip = false;
  }

  _createClass(Prop, [{
    key: "bagNames",
    value: function bagNames() {
      return [];
    }
  }]);

  return Prop;
}();

exports.Prop = Prop;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9wcm9wLnRzIl0sIm5hbWVzIjpbIlByb3AiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInJ1bGVzIiwicmVxdWlyZWQiLCJyZWFkT25seSIsImhpZGRlbiIsInJlcGVhdGVkIiwidW5pdmVyc2FsIiwibG9va3VwVGFnIiwicGFyZW50TmFtZSIsInJlZmVyZW5jZSIsInNraXAiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7OztJQWlCc0JBLEk7QUFNcEI7QUFRQTtBQUdBLHNCQWtCRztBQUFBLFFBakJEQyxJQWlCQyxRQWpCREEsSUFpQkM7QUFBQSxRQWhCREMsS0FnQkMsUUFoQkRBLEtBZ0JDO0FBQUEsUUFmREMsaUJBZUMsUUFmREEsaUJBZUM7QUFBQSxRQWREQyxLQWNDLFFBZERBLEtBY0M7QUFBQSxRQWJEQyxRQWFDLFFBYkRBLFFBYUM7QUFBQSxRQVpEQyxRQVlDLFFBWkRBLFFBWUM7QUFBQSxRQVhEQyxNQVdDLFFBWERBLE1BV0M7QUFBQSxRQVZEQyxRQVVDLFFBVkRBLFFBVUM7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQUE7O0FBQ0QsU0FBS1AsSUFBTCxHQUFZQSxJQUFaO0FBQ0EsU0FBS0MsS0FBTCxHQUFhQSxLQUFiO0FBQ0EsU0FBS0MsaUJBQUwsR0FBeUJBLGlCQUF6QjtBQUNBLFNBQUtDLEtBQUwsR0FBYUEsS0FBSyxJQUFJLEVBQXRCO0FBQ0EsU0FBS0MsUUFBTCxHQUFnQkEsUUFBUSxJQUFJLEtBQTVCO0FBQ0EsU0FBS0MsUUFBTCxHQUFnQkEsUUFBUSxJQUFJLEtBQTVCO0FBQ0EsU0FBS0MsTUFBTCxHQUFjQSxNQUFNLElBQUksS0FBeEI7QUFDQSxTQUFLQyxRQUFMLEdBQWdCQSxRQUFRLElBQUksS0FBNUI7QUFDQSxTQUFLQyxTQUFMLEdBQWlCLEtBQWpCO0FBQ0EsU0FBS0MsU0FBTCxHQUFpQixJQUFqQjtBQUNBLFNBQUtDLFVBQUwsR0FBa0IsRUFBbEI7QUFDQSxTQUFLQyxTQUFMLEdBQWlCLEtBQWpCO0FBQ0EsU0FBS0MsSUFBTCxHQUFZLEtBQVo7QUFDRDs7OzsrQkFLb0I7QUFDbkIsYUFBTyxFQUFQO0FBQ0QiLCJzb3VyY2VzQ29udGVudCI6WyJleHBvcnQgaW50ZXJmYWNlIFByb3BDb25zdHJ1Y3RvciB7XG4gIG5hbWU6IHN0cmluZztcbiAgbGFiZWw6IHN0cmluZztcbiAgY29tcG9uZW50VHlwZU5hbWU6IHN0cmluZztcbn1cblxuZXhwb3J0IHR5cGUgUHJvcFZhbHVlID1cbiAgfCBudWxsXG4gIHwgc3RyaW5nXG4gIHwgc3RyaW5nW11cbiAgfCBSZWNvcmQ8c3RyaW5nLCBhbnk+XG4gIHwgYm9vbGVhbjtcblxuZXhwb3J0IHR5cGUgUHJvcERlZmF1bHRWYWx1ZXMgPSB7XG4gIFtrZXk6IHN0cmluZ106IFByb3BWYWx1ZTtcbn07XG5cbmV4cG9ydCBhYnN0cmFjdCBjbGFzcyBQcm9wIHtcbiAgbmFtZTogc3RyaW5nO1xuICBsYWJlbDogc3RyaW5nO1xuICBydWxlczogKCh2OiBhbnkpID0+IGJvb2xlYW4gfCBzdHJpbmcpW107XG4gIHJlcXVpcmVkOiBib29sZWFuO1xuICByZWFkT25seTogYm9vbGVhbjtcbiAgLy8gSGlkZGVuIGZyb20gdGhlIFVJXG4gIGhpZGRlbjogYm9vbGVhbjtcbiAgcmVwZWF0ZWQ6IGJvb2xlYW47XG4gIHVuaXZlcnNhbDogYm9vbGVhbjtcbiAgbG9va3VwVGFnOiBudWxsIHwgc3RyaW5nO1xuICBwYXJlbnROYW1lOiBzdHJpbmc7XG4gIHJlZmVyZW5jZTogYm9vbGVhbjtcbiAgY29tcG9uZW50VHlwZU5hbWU6IHN0cmluZztcbiAgLy8gSGlkZGVuIGZyb20gdGhlIEFQSVxuICBza2lwOiBib29sZWFuO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHJ1bGVzLFxuICAgIHJlcXVpcmVkLFxuICAgIHJlYWRPbmx5LFxuICAgIGhpZGRlbixcbiAgICByZXBlYXRlZCxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBydWxlcz86IFByb3BbXCJydWxlc1wiXTtcbiAgICByZXF1aXJlZD86IFByb3BbXCJyZXF1aXJlZFwiXTtcbiAgICByZWFkT25seT86IFByb3BbXCJyZWFkT25seVwiXTtcbiAgICBoaWRkZW4/OiBQcm9wW1wiaGlkZGVuXCJdO1xuICAgIHJlcGVhdGVkPzogUHJvcFtcInJlcGVhdGVkXCJdO1xuICB9KSB7XG4gICAgdGhpcy5uYW1lID0gbmFtZTtcbiAgICB0aGlzLmxhYmVsID0gbGFiZWw7XG4gICAgdGhpcy5jb21wb25lbnRUeXBlTmFtZSA9IGNvbXBvbmVudFR5cGVOYW1lO1xuICAgIHRoaXMucnVsZXMgPSBydWxlcyB8fCBbXTtcbiAgICB0aGlzLnJlcXVpcmVkID0gcmVxdWlyZWQgfHwgZmFsc2U7XG4gICAgdGhpcy5yZWFkT25seSA9IHJlYWRPbmx5IHx8IGZhbHNlO1xuICAgIHRoaXMuaGlkZGVuID0gaGlkZGVuIHx8IGZhbHNlO1xuICAgIHRoaXMucmVwZWF0ZWQgPSByZXBlYXRlZCB8fCBmYWxzZTtcbiAgICB0aGlzLnVuaXZlcnNhbCA9IGZhbHNlO1xuICAgIHRoaXMubG9va3VwVGFnID0gbnVsbDtcbiAgICB0aGlzLnBhcmVudE5hbWUgPSBcIlwiO1xuICAgIHRoaXMucmVmZXJlbmNlID0gZmFsc2U7XG4gICAgdGhpcy5za2lwID0gZmFsc2U7XG4gIH1cblxuICBhYnN0cmFjdCBraW5kKCk6IHN0cmluZztcbiAgYWJzdHJhY3QgZGVmYXVsdFZhbHVlKCk6IFByb3BWYWx1ZTtcblxuICBiYWdOYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgcmV0dXJuIFtdO1xuICB9XG59XG4iXX0=