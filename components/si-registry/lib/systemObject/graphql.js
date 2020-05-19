"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.SiGraphql = void 0;

var _taggedTemplateLiteral2 = _interopRequireDefault(require("@babel/runtime/helpers/taggedTemplateLiteral"));

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _registry = require("../registry");

var _changeCase = require("change-case");

var _graphqlTag = _interopRequireDefault(require("graphql-tag"));

function _templateObject2() {
  var data = (0, _taggedTemplateLiteral2["default"])(["\n      ", "\n    "]);

  _templateObject2 = function _templateObject2() {
    return data;
  };

  return data;
}

function _templateObject() {
  var data = (0, _taggedTemplateLiteral2["default"])(["\n      ", "\n    "]);

  _templateObject = function _templateObject() {
    return data;
  };

  return data;
}

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

var SiGraphql = /*#__PURE__*/function () {
  function SiGraphql(systemObject) {
    (0, _classCallCheck2["default"])(this, SiGraphql);
    (0, _defineProperty2["default"])(this, "systemObject", void 0);
    this.systemObject = systemObject;
  }

  (0, _createClass2["default"])(SiGraphql, [{
    key: "validateResult",
    value: function validateResult(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var reply = method.reply;
      var lookupName = args.overrideName || "".concat((0, _changeCase.camelCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(args.methodName));
      var result = args.data.data[lookupName];

      var _iterator = _createForOfIteratorHelper(reply.properties.attrs),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var field = _step.value;

          if (field.required && result[field.name] == undefined) {
            throw "response incomplete; missing required field ".concat(field);
          }
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      return result;
    }
  }, {
    key: "variablesObject",
    value: function variablesObject(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var request = method.request;
      var result = {};

      var _iterator2 = _createForOfIteratorHelper(request.properties.attrs),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var field = _step2.value;
          result[field.name] = field.defaultValue();
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return result;
    }
  }, {
    key: "graphqlTypeName",
    value: function graphqlTypeName(prop, inputType) {
      var result = "";

      if (prop.kind() == "object" || prop.kind() == "enum") {
        var request = "";

        if (inputType) {
          request = "Request";
        }

        result = "".concat((0, _changeCase.pascalCase)(prop.parentName)).concat((0, _changeCase.pascalCase)(prop.name)).concat(request);
      } else if (prop.kind() == "text" || prop.kind() == "password") {
        if (prop.name == "id") {
          result = "ID";
        } else {
          result = "String";
        }
      } else if (prop.kind() == "number") {
        result = "Int";
      } else if (prop.kind() == "link") {
        var linkProp = prop;
        var realProp = linkProp.lookupMyself();
        return this.graphqlTypeName(realProp, inputType);
      }

      if (prop.required) {
        return "".concat(result, "!");
      } else {
        return result;
      }
    }
  }, {
    key: "associationFieldList",
    value: function associationFieldList(associations, systemObject) {
      var associationList = associations && associations[systemObject.typeName];

      if (associationList) {
        var result = [];
        result.push("associations {");

        var _iterator3 = _createForOfIteratorHelper(associationList),
            _step3;

        try {
          for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
            var fieldName = _step3.value;
            var assocObj = systemObject.associations.getByFieldName(fieldName);

            var assocSystem = _registry.registry.get(assocObj.typeName);

            var assocMethod = assocSystem.methods.getEntry(assocObj.methodName);
            result.push("".concat(fieldName, " {"));
            result.push(this.fieldList(assocMethod.reply, associations, assocSystem));
            result.push("}");
          }
        } catch (err) {
          _iterator3.e(err);
        } finally {
          _iterator3.f();
        }

        result.push("}");
        return result.join(" ");
      } else {
        return "";
      }
    }
  }, {
    key: "fieldList",
    value: function fieldList(propObject, associations, systemObjectMemo) {
      var systemObject;

      if (systemObjectMemo) {
        systemObject = systemObjectMemo;
      } else {
        systemObject = this.systemObject;
      }

      var result = [];

      var _iterator4 = _createForOfIteratorHelper(propObject.properties.attrs),
          _step4;

      try {
        for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
          var prop = _step4.value;

          if (prop.hidden || prop.skip) {
            continue;
          }

          result.push("".concat(prop.name));

          if (prop.kind() == "object") {
            result.push("{");
            result.push(this.fieldList(prop, undefined, systemObject));
            result.push(this.associationFieldList(associations, systemObject));
            result.push("}");
          }

          if (prop.kind() == "map") {
            result.push("{ key value }");
          } else if (prop.kind() == "link") {
            // @ts-ignore
            var realObj = prop.lookupMyself();

            if (realObj.kind() == "object") {
              result.push("{");
            }

            result.push(this.fieldList(realObj, undefined, systemObject));

            if (realObj.kind() == "object") {
              result.push(this.associationFieldList(associations, systemObject));
              result.push("}");
            }
          }
        }
      } catch (err) {
        _iterator4.e(err);
      } finally {
        _iterator4.f();
      }

      return "".concat(result.join(" "));
    }
  }, {
    key: "query",
    value: function query(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var methodName = args.overrideName || "".concat((0, _changeCase.camelCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(args.methodName));
      var request = method.request;
      var requestVariables = [];
      var inputVariables = [];

      var _iterator5 = _createForOfIteratorHelper(request.properties.attrs),
          _step5;

      try {
        for (_iterator5.s(); !(_step5 = _iterator5.n()).done;) {
          var prop = _step5.value;
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop)));
          inputVariables.push("".concat(prop.name, ": $").concat(prop.name));
        }
      } catch (err) {
        _iterator5.e(err);
      } finally {
        _iterator5.f();
      }

      var reply = method.reply;
      var fieldList;

      if (args.overrideFields) {
        fieldList = "".concat(args.overrideFields);
      } else {
        fieldList = this.fieldList(reply, args.associations, this.systemObject);
      }

      var resultString = "query ".concat(methodName, "(").concat(requestVariables.join(", "), ") { ").concat(methodName, "(input: { ").concat(inputVariables.join(", "), " }) { ").concat(fieldList, " } }");
      return (0, _graphqlTag["default"])(_templateObject(), resultString);
    }
  }, {
    key: "mutation",
    value: function mutation(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var methodName = args.overrideName || "".concat((0, _changeCase.camelCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(args.methodName));
      var request = method.request;
      var requestVariables = [];
      var inputVariables = [];

      var _iterator6 = _createForOfIteratorHelper(request.properties.attrs),
          _step6;

      try {
        for (_iterator6.s(); !(_step6 = _iterator6.n()).done;) {
          var prop = _step6.value;
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop, true)));
          inputVariables.push("".concat(prop.name, ": $").concat(prop.name));
        }
      } catch (err) {
        _iterator6.e(err);
      } finally {
        _iterator6.f();
      }

      var reply = method.reply;
      var fieldList;

      if (args.overrideFields) {
        fieldList = "".concat(args.overrideFields);
      } else {
        fieldList = this.fieldList(reply, args.associations, this.systemObject);
      }

      var resultString = "mutation ".concat(methodName, "(").concat(requestVariables.join(", "), ") { ").concat(methodName, "(input: { ").concat(inputVariables.join(", "), " }) { ").concat(fieldList, " } }");
      console.log(resultString);
      return (0, _graphqlTag["default"])(_templateObject2(), resultString);
    }
  }]);
  return SiGraphql;
}();

exports.SiGraphql = SiGraphql;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJTaUdyYXBocWwiLCJzeXN0ZW1PYmplY3QiLCJhcmdzIiwibWV0aG9kIiwibWV0aG9kcyIsImdldEVudHJ5IiwibWV0aG9kTmFtZSIsInJlcGx5IiwibG9va3VwTmFtZSIsIm92ZXJyaWRlTmFtZSIsInR5cGVOYW1lIiwicmVzdWx0IiwiZGF0YSIsInByb3BlcnRpZXMiLCJhdHRycyIsImZpZWxkIiwicmVxdWlyZWQiLCJuYW1lIiwidW5kZWZpbmVkIiwicmVxdWVzdCIsImRlZmF1bHRWYWx1ZSIsInByb3AiLCJpbnB1dFR5cGUiLCJraW5kIiwicGFyZW50TmFtZSIsImxpbmtQcm9wIiwicmVhbFByb3AiLCJsb29rdXBNeXNlbGYiLCJncmFwaHFsVHlwZU5hbWUiLCJhc3NvY2lhdGlvbnMiLCJhc3NvY2lhdGlvbkxpc3QiLCJwdXNoIiwiZmllbGROYW1lIiwiYXNzb2NPYmoiLCJnZXRCeUZpZWxkTmFtZSIsImFzc29jU3lzdGVtIiwicmVnaXN0cnkiLCJnZXQiLCJhc3NvY01ldGhvZCIsImZpZWxkTGlzdCIsImpvaW4iLCJwcm9wT2JqZWN0Iiwic3lzdGVtT2JqZWN0TWVtbyIsImhpZGRlbiIsInNraXAiLCJhc3NvY2lhdGlvbkZpZWxkTGlzdCIsInJlYWxPYmoiLCJyZXF1ZXN0VmFyaWFibGVzIiwiaW5wdXRWYXJpYWJsZXMiLCJvdmVycmlkZUZpZWxkcyIsInJlc3VsdFN0cmluZyIsImdxbCIsImNvbnNvbGUiLCJsb2ciXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBR0E7O0FBRUE7O0FBQ0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7SUF1QmFBLFM7QUFHWCxxQkFBWUMsWUFBWixFQUFxRDtBQUFBO0FBQUE7QUFDbkQsU0FBS0EsWUFBTCxHQUFvQkEsWUFBcEI7QUFDRDs7OzttQ0FFY0MsSSxFQUErQztBQUM1RCxVQUFNQyxNQUFNLEdBQUcsS0FBS0Ys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QWtDO0FBQzNDMEIsVUFBQUEsZ0JBQWdCLENBQUNoQixJQUFqQixZQUNNVixJQUFJLENBQUNKLElBRFgsZUFDb0IsS0FBS1csZUFBTCxDQUFxQlAsSUFBckIsRUFBMkIsSUFBM0IsQ0FEcEI7QUFHQTJCLFVBQUFBLGNBQWMsQ0FBQ2pCLElBQWYsV0FBdUJWLElBQUksQ0FBQ0osSUFBNUIsZ0JBQXNDSSxJQUFJLENBQUNKLElBQTNDO0FBQ0Q7QUFoQnFDO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBa0J0QyxVQUFNVixLQUFLLEdBQUdKLE1BQU0sQ0FBQ0ksS0FBckI7QUFDQSxVQUFJZ0MsU0FBSjs7QUFDQSxVQUFJckMsSUFBSSxDQUFDK0MsY0FBVCxFQUF5QjtBQUN2QlYsUUFBQUEsU0FBUyxhQUFNckMsSUFBSSxDQUFDK0MsY0FBWCxDQUFUO0FBQ0QsT0FGRCxNQUVPO0FBQ0xWLFFBQUFBLFNBQVMsR0FBRyxLQUFLQSxTQUFMLENBQWVoQyxLQUFmLEVBQXNCTCxJQUFJLENBQUMyQixZQUEzQixFQUF5QyxLQUFLNUIsWUFBOUMsQ0FBWjtBQUNEOztBQUVELFVBQU1pRCxZQUFZLHNCQUFlNUMsVUFBZixjQUE2QnlDLGdCQUFnQixDQUFDUCxJQUFqQixDQUM3QyxJQUQ2QyxDQUE3QixpQkFFVmxDLFVBRlUsdUJBRWEwQyxjQUFjLENBQUNSLElBQWYsQ0FDN0IsSUFENkIsQ0FGYixtQkFJUkQsU0FKUSxTQUFsQjtBQUtBYSxNQUFBQSxPQUFPLENBQUNDLEdBQVIsQ0FBWUgsWUFBWjtBQUNBLGlCQUFPQyxzQkFBUCxzQkFDSUQsWUFESjtBQUdEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcE1ldGhvZCwgUHJvcE9iamVjdCwgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcbmltcG9ydCB7IFByb3BMaW5rIH0gZnJvbSBcIi4uL3Byb3AvbGlua1wiO1xuaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuXG5pbXBvcnQgeyBwYXNjYWxDYXNlLCBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBncWwgZnJvbSBcImdyYXBocWwtdGFnXCI7XG5pbXBvcnQgeyBEb2N1bWVudE5vZGUgfSBmcm9tIFwiZ3JhcGhxbFwiO1xuaW1wb3J0IHsgQXNzb2NpYXRpb24gfSBmcm9tIFwiLi9hc3NvY2lhdGlvbnNcIjtcblxuaW50ZXJmYWNlIFF1ZXJ5QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xuICBvdmVycmlkZUZpZWxkcz86IHN0cmluZztcbiAgYXNzb2NpYXRpb25zPzoge1xuICAgIFtrZXk6IHN0cmluZ106IHN0cmluZ1tdO1xuICB9O1xufVxuXG5pbnRlcmZhY2UgVmFyaWFibGVzT2JqZWN0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbn1cblxuaW50ZXJmYWNlIFZhbGlkYXRlUmVzdWx0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgZGF0YTogUmVjb3JkPHN0cmluZywgYW55PjtcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xufVxuXG5leHBvcnQgY2xhc3MgU2lHcmFwaHFsIHtcbiAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcblxuICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFNpR3JhcGhxbFtcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgdmFsaWRhdGVSZXN1bHQoYXJnczogVmFsaWRhdGVSZXN1bHRBcmdzKTogUmVjb3JkPHN0cmluZywgYW55PiB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgY29uc3QgbG9va3VwTmFtZSA9XG4gICAgICBhcmdzLm92ZXJyaWRlTmFtZSB8fFxuICAgICAgYCR7Y2FtZWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ke3Bhc2NhbENhc2UoYXJncy5tZXRob2ROYW1lKX1gO1xuICAgIGNvbnN0IHJlc3VsdCA9IGFyZ3MuZGF0YS5kYXRhW2xvb2t1cE5hbWVdO1xuICAgIGZvciAoY29uc3QgZmllbGQgb2YgcmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKGZpZWxkLnJlcXVpcmVkICYmIHJlc3VsdFtmaWVsZC5uYW1lXSA9PSB1bmRlZmluZWQpIHtcbiAgICAgICAgdGhyb3cgYHJlc3BvbnNlIGluY29tcGxldGU7IG1pc3NpbmcgcmVxdWlyZWQgZmllbGQgJHtmaWVsZH1gO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgdmFyaWFibGVzT2JqZWN0KGFyZ3M6IFZhcmlhYmxlc09iamVjdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVzdWx0OiBSZWNvcmQ8c3RyaW5nLCBhbnk+ID0ge307XG4gICAgZm9yIChjb25zdCBmaWVsZCBvZiByZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIHJlc3VsdFtmaWVsZC5uYW1lXSA9IGZpZWxkLmRlZmF1bHRWYWx1ZSgpO1xuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgZ3JhcGhxbFR5cGVOYW1lKHByb3A6IFByb3BzLCBpbnB1dFR5cGU/OiBib29sZWFuKTogc3RyaW5nIHtcbiAgICBsZXQgcmVzdWx0ID0gXCJcIjtcbiAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIiB8fCBwcm9wLmtpbmQoKSA9PSBcImVudW1cIikge1xuICAgICAgbGV0IHJlcXVlc3QgPSBcIlwiO1xuICAgICAgaWYgKGlucHV0VHlwZSkge1xuICAgICAgICByZXF1ZXN0ID0gXCJSZXF1ZXN0XCI7XG4gICAgICB9XG4gICAgICByZXN1bHQgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLm5hbWUsXG4gICAgICApfSR7cmVxdWVzdH1gO1xuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJ0ZXh0XCIgfHwgcHJvcC5raW5kKCkgPT0gXCJwYXNzd29yZFwiKSB7XG4gICAgICBpZiAocHJvcC5uYW1lID09IFwiaWRcIikge1xuICAgICAgICByZXN1bHQgPSBcIklEXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHQgPSBcIlN0cmluZ1wiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJudW1iZXJcIikge1xuICAgICAgcmVzdWx0ID0gXCJJbnRcIjtcbiAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICBjb25zdCBsaW5rUHJvcCA9IHByb3AgYXMgUHJvcExpbms7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IGxpbmtQcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgcmV0dXJuIHRoaXMuZ3JhcGhxbFR5cGVOYW1lKHJlYWxQcm9wLCBpbnB1dFR5cGUpO1xuICAgIH1cbiAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuICAgICAgcmV0dXJuIGAke3Jlc3VsdH0hYDtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHJlc3VsdDtcbiAgICB9XG4gIH1cblxuICBhc3NvY2lhdGlvbkZpZWxkTGlzdChcbiAgICBhc3NvY2lhdGlvbnM6IFF1ZXJ5QXJnc1tcImFzc29jaWF0aW9uc1wiXSxcbiAgICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzLFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IGFzc29jaWF0aW9uTGlzdCA9IGFzc29jaWF0aW9ucyAmJiBhc3NvY2lhdGlvbnNbc3lzdGVtT2JqZWN0LnR5cGVOYW1lXTtcbiAgICBpZiAoYXNzb2NpYXRpb25MaXN0KSB7XG4gICAgICBjb25zdCByZXN1bHQ6IHN0cmluZ1tdID0gW107XG4gICAgICByZXN1bHQucHVzaChcImFzc29jaWF0aW9ucyB7XCIpO1xuICAgICAgZm9yIChjb25zdCBmaWVsZE5hbWUgb2YgYXNzb2NpYXRpb25MaXN0KSB7XG4gICAgICAgIGNvbnN0IGFzc29jT2JqID0gc3lzdGVtT2JqZWN0LmFzc29jaWF0aW9ucy5nZXRCeUZpZWxkTmFtZShmaWVsZE5hbWUpO1xuICAgICAgICBjb25zdCBhc3NvY1N5c3RlbSA9IHJlZ2lzdHJ5LmdldChhc3NvY09iai50eXBlTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jTWV0aG9kID0gYXNzb2NTeXN0ZW0ubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgICAgICBhc3NvY09iai5tZXRob2ROYW1lLFxuICAgICAgICApIGFzIFByb3BNZXRob2Q7XG5cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfSB7YCk7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KGFzc29jTWV0aG9kLnJlcGx5LCBhc3NvY2lhdGlvbnMsIGFzc29jU3lzdGVtKSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goYH1gKTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgIHJldHVybiByZXN1bHQuam9pbihcIiBcIik7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlwiO1xuICAgIH1cbiAgfVxuXG4gIGZpZWxkTGlzdChcbiAgICBwcm9wT2JqZWN0OiBQcm9wT2JqZWN0LFxuICAgIGFzc29jaWF0aW9uczogUXVlcnlBcmdzW1wiYXNzb2NpYXRpb25zXCJdLFxuICAgIHN5c3RlbU9iamVjdE1lbW86IE9iamVjdFR5cGVzLFxuICApOiBzdHJpbmcge1xuICAgIGxldCBzeXN0ZW1PYmplY3Q7XG4gICAgaWYgKHN5c3RlbU9iamVjdE1lbW8pIHtcbiAgICAgIHN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdE1lbW87XG4gICAgfSBlbHNlIHtcbiAgICAgIHN5c3RlbU9iamVjdCA9IHRoaXMuc3lzdGVtT2JqZWN0O1xuICAgIH1cbiAgICBjb25zdCByZXN1bHQ6IHN0cmluZ1tdID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKHByb3AuaGlkZGVuIHx8IHByb3Auc2tpcCkge1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdC5wdXNoKGAke3Byb3AubmFtZX1gKTtcbiAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKFwie1wiKTtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QocHJvcCBhcyBQcm9wT2JqZWN0LCB1bmRlZmluZWQsIHN5c3RlbU9iamVjdCksXG4gICAgICAgICk7XG4gICAgICAgIHJlc3VsdC5wdXNoKHRoaXMuYXNzb2NpYXRpb25GaWVsZExpc3QoYXNzb2NpYXRpb25zLCBzeXN0ZW1PYmplY3QpKTtcbiAgICAgICAgcmVzdWx0LnB1c2goXCJ9XCIpO1xuICAgICAgfVxuICAgICAgaWYgKHByb3Aua2luZCgpID09IFwibWFwXCIpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXCJ7IGtleSB2YWx1ZSB9XCIpO1xuICAgICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgICAgICAvLyBAdHMtaWdub3JlXG4gICAgICAgIGNvbnN0IHJlYWxPYmogPSBwcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgICBpZiAocmVhbE9iai5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFwie1wiKTtcbiAgICAgICAgfVxuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICB0aGlzLmZpZWxkTGlzdChyZWFsT2JqIGFzIFByb3BPYmplY3QsIHVuZGVmaW5lZCwgc3lzdGVtT2JqZWN0KSxcbiAgICAgICAgKTtcbiAgICAgICAgaWYgKHJlYWxPYmoua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgICByZXN1bHQucHVzaCh0aGlzLmFzc29jaWF0aW9uRmllbGRMaXN0KGFzc29jaWF0aW9ucywgc3lzdGVtT2JqZWN0KSk7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXCJ9XCIpO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiBgJHtyZXN1bHQuam9pbihcIiBcIil9YDtcbiAgfVxuXG4gIHF1ZXJ5KGFyZ3M6IFF1ZXJ5QXJncyk6IERvY3VtZW50Tm9kZSB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgbWV0aG9kTmFtZSA9XG4gICAgICBhcmdzLm92ZXJyaWRlTmFtZSB8fFxuICAgICAgYCR7Y2FtZWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ke3Bhc2NhbENhc2UoYXJncy5tZXRob2ROYW1lKX1gO1xuXG4gICAgY29uc3QgcmVxdWVzdCA9IG1ldGhvZC5yZXF1ZXN0O1xuICAgIGNvbnN0IHJlcXVlc3RWYXJpYWJsZXMgPSBbXTtcbiAgICBjb25zdCBpbnB1dFZhcmlhYmxlcyA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiByZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIHJlcXVlc3RWYXJpYWJsZXMucHVzaChgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wKX1gKTtcbiAgICAgIGlucHV0VmFyaWFibGVzLnB1c2goYCR7cHJvcC5uYW1lfTogJCR7cHJvcC5uYW1lfWApO1xuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgcXVlcnkgJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIHJldHVybiBncWxgXG4gICAgICAke3Jlc3VsdFN0cmluZ31cbiAgICBgO1xuICB9XG5cbiAgbXV0YXRpb24oYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLFxuICAgICAgKTtcbiAgICAgIGlucHV0VmFyaWFibGVzLnB1c2goYCR7cHJvcC5uYW1lfTogJCR7cHJvcC5uYW1lfWApO1xuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgbXV0YXRpb24gJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIGNvbnNvbGUubG9nKHJlc3VsdFN0cmluZyk7XG4gICAgcmV0dXJuIGdxbGBcbiAgICAgICR7cmVzdWx0U3RyaW5nfVxuICAgIGA7XG4gIH1cbn1cbiJdfQ==