"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.Prop = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _relationships = require("./prop/relationships");

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
    (0, _classCallCheck2["default"])(this, Prop);
    (0, _defineProperty2["default"])(this, "name", void 0);
    (0, _defineProperty2["default"])(this, "label", void 0);
    (0, _defineProperty2["default"])(this, "rules", void 0);
    (0, _defineProperty2["default"])(this, "required", void 0);
    (0, _defineProperty2["default"])(this, "readOnly", void 0);
    (0, _defineProperty2["default"])(this, "relationships", void 0);
    (0, _defineProperty2["default"])(this, "hidden", void 0);
    (0, _defineProperty2["default"])(this, "repeated", void 0);
    (0, _defineProperty2["default"])(this, "universal", void 0);
    (0, _defineProperty2["default"])(this, "lookupTag", void 0);
    (0, _defineProperty2["default"])(this, "parentName", void 0);
    (0, _defineProperty2["default"])(this, "reference", void 0);
    (0, _defineProperty2["default"])(this, "componentTypeName", void 0);
    (0, _defineProperty2["default"])(this, "skip", void 0);
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
    this.relationships = new _relationships.RelationshipList();
  }

  (0, _createClass2["default"])(Prop, [{
    key: "bagNames",
    value: function bagNames() {
      return [];
    }
  }]);
  return Prop;
}();

exports.Prop = Prop;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9wcm9wLnRzIl0sIm5hbWVzIjpbIlByb3AiLCJuYW1lIiwibGFiZWwiLCJjb21wb25lbnRUeXBlTmFtZSIsInJ1bGVzIiwicmVxdWlyZWQiLCJyZWFkT25seSIsImhpZGRlbiIsInJlcGVhdGVkIiwidW5pdmVyc2FsIiwibG9va3VwVGFnIiwicGFyZW50TmFtZSIsInJlZmVyZW5jZSIsInNraXAiLCJyZWxhdGlvbnNoaXBzIiwiUmVsYXRpb25zaGlwTGlzdCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0lBbUJzQkEsSTtBQVFwQjtBQVFBO0FBR0Esc0JBa0JHO0FBQUEsUUFqQkRDLElBaUJDLFFBakJEQSxJQWlCQztBQUFBLFFBaEJEQyxLQWdCQyxRQWhCREEsS0FnQkM7QUFBQSxRQWZEQyxpQkFlQyxRQWZEQSxpQkFlQztBQUFBLFFBZERDLEtBY0MsUUFkREEsS0FjQztBQUFBLFFBYkRDLFFBYUMsUUFiREEsUUFhQztBQUFBLFFBWkRDLFFBWUMsUUFaREEsUUFZQztBQUFBLFFBWERDLE1BV0MsUUFYREEsTUFXQztBQUFBLFFBVkRDLFFBVUMsUUFWREEsUUFVQztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUNELFNBQUtQLElBQUwsR0FBWUEsSUFBWjtBQUNBLFNBQUtDLEtBQUwsR0FBYUEsS0FBYjtBQUNBLFNBQUtDLGlCQUFMLEdBQXlCQSxpQkFBekI7QUFDQSxTQUFLQyxLQUFMLEdBQWFBLEtBQUssSUFBSSxFQUF0QjtBQUNBLFNBQUtDLFFBQUwsR0FBZ0JBLFFBQVEsSUFBSSxLQUE1QjtBQUNBLFNBQUtDLFFBQUwsR0FBZ0JBLFFBQVEsSUFBSSxLQUE1QjtBQUNBLFNBQUtDLE1BQUwsR0FBY0EsTUFBTSxJQUFJLEtBQXhCO0FBQ0EsU0FBS0MsUUFBTCxHQUFnQkEsUUFBUSxJQUFJLEtBQTVCO0FBQ0EsU0FBS0MsU0FBTCxHQUFpQixLQUFqQjtBQUNBLFNBQUtDLFNBQUwsR0FBaUIsSUFBakI7QUFDQSxTQUFLQyxVQUFMLEdBQWtCLEVBQWxCO0FBQ0EsU0FBS0MsU0FBTCxHQUFpQixLQUFqQjtBQUNBLFNBQUtDLElBQUwsR0FBWSxLQUFaO0FBQ0EsU0FBS0MsYUFBTCxHQUFxQixJQUFJQywrQkFBSixFQUFyQjtBQUNEOzs7OytCQUtvQjtBQUNuQixhQUFPLEVBQVA7QUFDRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFJlbGF0aW9uc2hpcExpc3QgfSBmcm9tIFwiLi9wcm9wL3JlbGF0aW9uc2hpcHNcIjtcblxuZXhwb3J0IGludGVyZmFjZSBQcm9wQ29uc3RydWN0b3Ige1xuICBuYW1lOiBzdHJpbmc7XG4gIGxhYmVsOiBzdHJpbmc7XG4gIGNvbXBvbmVudFR5cGVOYW1lOiBzdHJpbmc7XG59XG5cbmV4cG9ydCB0eXBlIFByb3BWYWx1ZSA9XG4gIHwgbnVsbFxuICB8IHN0cmluZ1xuICB8IHN0cmluZ1tdXG4gIHwgUmVjb3JkPHN0cmluZywgYW55PlxuICB8IGJvb2xlYW47XG5cbmV4cG9ydCB0eXBlIFByb3BEZWZhdWx0VmFsdWVzID0ge1xuICBba2V5OiBzdHJpbmddOiBQcm9wVmFsdWU7XG59O1xuXG5leHBvcnQgYWJzdHJhY3QgY2xhc3MgUHJvcCB7XG4gIG5hbWU6IHN0cmluZztcbiAgbGFiZWw6IHN0cmluZztcbiAgcnVsZXM6ICgodjogYW55KSA9PiBib29sZWFuIHwgc3RyaW5nKVtdO1xuICByZXF1aXJlZDogYm9vbGVhbjtcbiAgcmVhZE9ubHk6IGJvb2xlYW47XG4gIHJlbGF0aW9uc2hpcHM6IFJlbGF0aW9uc2hpcExpc3Q7XG5cbiAgLy8gSGlkZGVuIGZyb20gdGhlIFVJXG4gIGhpZGRlbjogYm9vbGVhbjtcbiAgcmVwZWF0ZWQ6IGJvb2xlYW47XG4gIHVuaXZlcnNhbDogYm9vbGVhbjtcbiAgbG9va3VwVGFnOiBudWxsIHwgc3RyaW5nO1xuICBwYXJlbnROYW1lOiBzdHJpbmc7XG4gIHJlZmVyZW5jZTogYm9vbGVhbjtcbiAgY29tcG9uZW50VHlwZU5hbWU6IHN0cmluZztcbiAgLy8gSGlkZGVuIGZyb20gdGhlIEFQSVxuICBza2lwOiBib29sZWFuO1xuXG4gIGNvbnN0cnVjdG9yKHtcbiAgICBuYW1lLFxuICAgIGxhYmVsLFxuICAgIGNvbXBvbmVudFR5cGVOYW1lLFxuICAgIHJ1bGVzLFxuICAgIHJlcXVpcmVkLFxuICAgIHJlYWRPbmx5LFxuICAgIGhpZGRlbixcbiAgICByZXBlYXRlZCxcbiAgfToge1xuICAgIG5hbWU6IFByb3BbXCJuYW1lXCJdO1xuICAgIGxhYmVsOiBQcm9wW1wibGFiZWxcIl07XG4gICAgY29tcG9uZW50VHlwZU5hbWU6IFByb3BbXCJjb21wb25lbnRUeXBlTmFtZVwiXTtcbiAgICBydWxlcz86IFByb3BbXCJydWxlc1wiXTtcbiAgICByZXF1aXJlZD86IFByb3BbXCJyZXF1aXJlZFwiXTtcbiAgICByZWFkT25seT86IFByb3BbXCJyZWFkT25seVwiXTtcbiAgICBoaWRkZW4/OiBQcm9wW1wiaGlkZGVuXCJdO1xuICAgIHJlcGVhdGVkPzogUHJvcFtcInJlcGVhdGVkXCJdO1xuICB9KSB7XG4gICAgdGhpcy5uYW1lID0gbmFtZTtcbiAgICB0aGlzLmxhYmVsID0gbGFiZWw7XG4gICAgdGhpcy5jb21wb25lbnRUeXBlTmFtZSA9IGNvbXBvbmVudFR5cGVOYW1lO1xuICAgIHRoaXMucnVsZXMgPSBydWxlcyB8fCBbXTtcbiAgICB0aGlzLnJlcXVpcmVkID0gcmVxdWlyZWQgfHwgZmFsc2U7XG4gICAgdGhpcy5yZWFkT25seSA9IHJlYWRPbmx5IHx8IGZhbHNlO1xuICAgIHRoaXMuaGlkZGVuID0gaGlkZGVuIHx8IGZhbHNlO1xuICAgIHRoaXMucmVwZWF0ZWQgPSByZXBlYXRlZCB8fCBmYWxzZTtcbiAgICB0aGlzLnVuaXZlcnNhbCA9IGZhbHNlO1xuICAgIHRoaXMubG9va3VwVGFnID0gbnVsbDtcbiAgICB0aGlzLnBhcmVudE5hbWUgPSBcIlwiO1xuICAgIHRoaXMucmVmZXJlbmNlID0gZmFsc2U7XG4gICAgdGhpcy5za2lwID0gZmFsc2U7XG4gICAgdGhpcy5yZWxhdGlvbnNoaXBzID0gbmV3IFJlbGF0aW9uc2hpcExpc3QoKTtcbiAgfVxuXG4gIGFic3RyYWN0IGtpbmQoKTogc3RyaW5nO1xuICBhYnN0cmFjdCBkZWZhdWx0VmFsdWUoKTogUHJvcFZhbHVlO1xuXG4gIGJhZ05hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICByZXR1cm4gW107XG4gIH1cbn1cbiJdfQ==