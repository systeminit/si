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
    } // Second argument is if you want a repeated field

  }, {
    key: "variablesObjectForProperty",
    value: function variablesObjectForProperty(prop) {
      var repeated = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : false;

      if (prop.kind() == "text" || prop.kind() == "number" || prop.kind() == "code" || prop.kind() == "enum") {
        if (prop.repeated && repeated) {
          return [];
        } else {
          return "";
        }
      } else if (prop.kind() == "map") {
        if (prop.repeated && repeated) {
          return [];
        } else {
          return {};
        }
      } else if (prop.kind() == "link") {
        var propLink = prop;

        if (prop.repeated && repeated) {
          return [];
        } else {
          // TODO: There might be a bug here, where the name of the prop itself
          // and the name of the linked prop don't match, and so we get the
          // wrong field name if the prop is an object.
          return this.variablesObjectForProperty(propLink.lookupMyself(), repeated);
        }
      } else if (prop.kind() == "object" || prop.kind() == "method") {
        var propObject = prop;
        var result = {};

        var _iterator2 = _createForOfIteratorHelper(propObject.properties.attrs),
            _step2;

        try {
          for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
            var field = _step2.value;
            var fieldVariables = this.variablesObjectForProperty(field, repeated);
            result["".concat(field.name)] = fieldVariables;
          }
        } catch (err) {
          _iterator2.e(err);
        } finally {
          _iterator2.f();
        }

        if (prop.repeated && repeated) {
          return [];
        } else {
          return result;
        }
      }
    }
  }, {
    key: "variablesObject",
    value: function variablesObject(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var request = method.request;
      return this.variablesObjectForProperty(request, true);
    }
  }, {
    key: "graphqlTypeName",
    value: function graphqlTypeName(prop, inputType) {
      var result = "";

      if (prop.kind() == "object" || prop.kind() == "enum") {
        var request = "";

        if (inputType && prop.kind() != "enum") {
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
        result = "String";
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
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop, true)));
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJTaUdyYXBocWwiLCJzeXN0ZW1PYmplY3QiLCJhcmdzIiwibWV0aG9kIiwibWV0aG9kcyIsImdldEVudHJ5IiwibWV0aG9kTmFtZSIsInJlcGx5IiwibG9va3VwTmFtZSIsIm92ZXJyaWRlTmFtZSIsInR5cGVOYW1lIiwicmVzdWx0IiwiZGF0YSIsInByb3BlcnRpZXMiLCJhdHRycyIsImZpZWxkIiwicmVxdWlyZWQiLCJuYW1lIiwidW5kZWZpbmVkIiwicHJvcCIsInJlcGVhdGVkIiwia2luZCIsInByb3BMaW5rIiwidmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkiLCJsb29rdXBNeXNlbGYiLCJwcm9wT2JqZWN0IiwiZmllbGRWYXJpYWJsZXMiLCJyZXF1ZXN0IiwiaW5wdXRUeXBlIiwicGFyZW50TmFtZSIsImxpbmtQcm9wIiwicmVhbFByb3AiLCJncmFwaHFsVHlwZU5hbWUiLCJhc3NvY2lhdGlvbnMiLCJhc3NvY2lhdGlvbkxpc3QiLCJwdXNoIiwiZmllbGROYW1lIiwiYXNzb2NPYmoiLCJnZXRCeUZpZWxkTmFtZSIsImFzc29jU3lzdGVtIiwicmVnaXN0cnkiLCJnZXQiLCJhc3NvY01ldGhvZCIsImZpZWxkTGlzdCIsImpvaW4iLCJzeXN0ZW1PYmplY3RNZW1vIiwiaGlkZGVuIiwic2tpcCIsImFzc29jaWF0aW9uRmllbGRMaXN0IiwicmVhbE9iaiIsInJlcXVlc3RWYXJpYWJsZXMiLCJpbnB1dFZhcmlhYmxlcyIsIm92ZXJyaWRlRmllbGRzIiwicmVzdWx0U3RyaW5nIiwiZ3FsIiwiY29uc29sZSIsImxvZyJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7QUFHQTs7QUFFQTs7QUFDQTs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztJQXVCYUEsUztBQUdYLHFCQUFZQyxZQUFaLEVBQXFEO0FBQUE7QUFBQTtBQUNuRCxTQUFLQSxZQUFMLEdBQW9CQSxZQUFw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SUQsWUFESjtBQUdEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcE1ldGhvZCwgUHJvcE9iamVjdCwgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcbmltcG9ydCB7IFByb3BMaW5rIH0gZnJvbSBcIi4uL3Byb3AvbGlua1wiO1xuaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuXG5pbXBvcnQgeyBwYXNjYWxDYXNlLCBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBncWwgZnJvbSBcImdyYXBocWwtdGFnXCI7XG5pbXBvcnQgeyBEb2N1bWVudE5vZGUgfSBmcm9tIFwiZ3JhcGhxbFwiO1xuaW1wb3J0IHsgQXNzb2NpYXRpb24gfSBmcm9tIFwiLi9hc3NvY2lhdGlvbnNcIjtcblxuaW50ZXJmYWNlIFF1ZXJ5QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xuICBvdmVycmlkZUZpZWxkcz86IHN0cmluZztcbiAgYXNzb2NpYXRpb25zPzoge1xuICAgIFtrZXk6IHN0cmluZ106IHN0cmluZ1tdO1xuICB9O1xufVxuXG5pbnRlcmZhY2UgVmFyaWFibGVzT2JqZWN0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbn1cblxuaW50ZXJmYWNlIFZhbGlkYXRlUmVzdWx0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgZGF0YTogUmVjb3JkPHN0cmluZywgYW55PjtcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xufVxuXG5leHBvcnQgY2xhc3MgU2lHcmFwaHFsIHtcbiAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcblxuICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFNpR3JhcGhxbFtcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgdmFsaWRhdGVSZXN1bHQoYXJnczogVmFsaWRhdGVSZXN1bHRBcmdzKTogUmVjb3JkPHN0cmluZywgYW55PiB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgY29uc3QgbG9va3VwTmFtZSA9XG4gICAgICBhcmdzLm92ZXJyaWRlTmFtZSB8fFxuICAgICAgYCR7Y2FtZWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ke3Bhc2NhbENhc2UoYXJncy5tZXRob2ROYW1lKX1gO1xuICAgIGNvbnN0IHJlc3VsdCA9IGFyZ3MuZGF0YS5kYXRhW2xvb2t1cE5hbWVdO1xuICAgIGZvciAoY29uc3QgZmllbGQgb2YgcmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKGZpZWxkLnJlcXVpcmVkICYmIHJlc3VsdFtmaWVsZC5uYW1lXSA9PSB1bmRlZmluZWQpIHtcbiAgICAgICAgdGhyb3cgYHJlc3BvbnNlIGluY29tcGxldGU7IG1pc3NpbmcgcmVxdWlyZWQgZmllbGQgJHtmaWVsZH1gO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgLy8gU2Vjb25kIGFyZ3VtZW50IGlzIGlmIHlvdSB3YW50IGEgcmVwZWF0ZWQgZmllbGRcbiAgdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkocHJvcDogUHJvcHMsIHJlcGVhdGVkID0gZmFsc2UpOiBhbnkge1xuICAgIGlmIChcbiAgICAgIHByb3Aua2luZCgpID09IFwidGV4dFwiIHx8XG4gICAgICBwcm9wLmtpbmQoKSA9PSBcIm51bWJlclwiIHx8XG4gICAgICBwcm9wLmtpbmQoKSA9PSBcImNvZGVcIiB8fFxuICAgICAgcHJvcC5raW5kKCkgPT0gXCJlbnVtXCJcbiAgICApIHtcbiAgICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICAgIHJldHVybiBbXTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiBcIlwiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJtYXBcIikge1xuICAgICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgICAgcmV0dXJuIFtdO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmV0dXJuIHt9O1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgIGNvbnN0IHByb3BMaW5rID0gcHJvcCBhcyBQcm9wTGluaztcbiAgICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICAgIHJldHVybiBbXTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIC8vIFRPRE86IFRoZXJlIG1pZ2h0IGJlIGEgYnVnIGhlcmUsIHdoZXJlIHRoZSBuYW1lIG9mIHRoZSBwcm9wIGl0c2VsZlxuICAgICAgICAvLyBhbmQgdGhlIG5hbWUgb2YgdGhlIGxpbmtlZCBwcm9wIGRvbid0IG1hdGNoLCBhbmQgc28gd2UgZ2V0IHRoZVxuICAgICAgICAvLyB3cm9uZyBmaWVsZCBuYW1lIGlmIHRoZSBwcm9wIGlzIGFuIG9iamVjdC5cbiAgICAgICAgcmV0dXJuIHRoaXMudmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkoXG4gICAgICAgICAgcHJvcExpbmsubG9va3VwTXlzZWxmKCksXG4gICAgICAgICAgcmVwZWF0ZWQsXG4gICAgICAgICk7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiIHx8IHByb3Aua2luZCgpID09IFwibWV0aG9kXCIpIHtcbiAgICAgIGNvbnN0IHByb3BPYmplY3QgPSBwcm9wIGFzIFByb3BPYmplY3Q7XG4gICAgICBjb25zdCByZXN1bHQ6IFJlY29yZDxzdHJpbmcsIHVua25vd24+ID0ge307XG4gICAgICBmb3IgKGNvbnN0IGZpZWxkIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgICBjb25zdCBmaWVsZFZhcmlhYmxlcyA9IHRoaXMudmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkoZmllbGQsIHJlcGVhdGVkKTtcbiAgICAgICAgcmVzdWx0W2Ake2ZpZWxkLm5hbWV9YF0gPSBmaWVsZFZhcmlhYmxlcztcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICAgIHJldHVybiBbXTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiByZXN1bHQ7XG4gICAgICB9XG4gICAgfVxuICB9XG5cbiAgdmFyaWFibGVzT2JqZWN0KGFyZ3M6IFZhcmlhYmxlc09iamVjdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgcmV0dXJuIHRoaXMudmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkocmVxdWVzdCwgdHJ1ZSk7XG4gIH1cblxuICBncmFwaHFsVHlwZU5hbWUocHJvcDogUHJvcHMsIGlucHV0VHlwZT86IGJvb2xlYW4pOiBzdHJpbmcge1xuICAgIGxldCByZXN1bHQgPSBcIlwiO1xuICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiIHx8IHByb3Aua2luZCgpID09IFwiZW51bVwiKSB7XG4gICAgICBsZXQgcmVxdWVzdCA9IFwiXCI7XG4gICAgICBpZiAoaW5wdXRUeXBlICYmIHByb3Aua2luZCgpICE9IFwiZW51bVwiKSB7XG4gICAgICAgIHJlcXVlc3QgPSBcIlJlcXVlc3RcIjtcbiAgICAgIH1cbiAgICAgIHJlc3VsdCA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9JHtyZXF1ZXN0fWA7XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcInRleHRcIiB8fCBwcm9wLmtpbmQoKSA9PSBcInBhc3N3b3JkXCIpIHtcbiAgICAgIGlmIChwcm9wLm5hbWUgPT0gXCJpZFwiKSB7XG4gICAgICAgIHJlc3VsdCA9IFwiSURcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdCA9IFwiU3RyaW5nXCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm51bWJlclwiKSB7XG4gICAgICByZXN1bHQgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgIGNvbnN0IGxpbmtQcm9wID0gcHJvcCBhcyBQcm9wTGluaztcbiAgICAgIGNvbnN0IHJlYWxQcm9wID0gbGlua1Byb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICByZXR1cm4gdGhpcy5ncmFwaHFsVHlwZU5hbWUocmVhbFByb3AsIGlucHV0VHlwZSk7XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICByZXR1cm4gYCR7cmVzdWx0fSFgO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH1cbiAgfVxuXG4gIGFzc29jaWF0aW9uRmllbGRMaXN0KFxuICAgIGFzc29jaWF0aW9uczogUXVlcnlBcmdzW1wiYXNzb2NpYXRpb25zXCJdLFxuICAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgYXNzb2NpYXRpb25MaXN0ID0gYXNzb2NpYXRpb25zICYmIGFzc29jaWF0aW9uc1tzeXN0ZW1PYmplY3QudHlwZU5hbWVdO1xuICAgIGlmIChhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYXNzb2NpYXRpb25zIHtcIik7XG4gICAgICBmb3IgKGNvbnN0IGZpZWxkTmFtZSBvZiBhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgICAgY29uc3QgYXNzb2NPYmogPSBzeXN0ZW1PYmplY3QuYXNzb2NpYXRpb25zLmdldEJ5RmllbGROYW1lKGZpZWxkTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jU3lzdGVtID0gcmVnaXN0cnkuZ2V0KGFzc29jT2JqLnR5cGVOYW1lKTtcbiAgICAgICAgY29uc3QgYXNzb2NNZXRob2QgPSBhc3NvY1N5c3RlbS5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgICAgIGFzc29jT2JqLm1ldGhvZE5hbWUsXG4gICAgICAgICkgYXMgUHJvcE1ldGhvZDtcblxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9IHtgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QoYXNzb2NNZXRob2QucmVwbHksIGFzc29jaWF0aW9ucywgYXNzb2NTeXN0ZW0pLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaChgfWApO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goXCJ9XCIpO1xuICAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiIFwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9XG5cbiAgZmllbGRMaXN0KFxuICAgIHByb3BPYmplY3Q6IFByb3BPYmplY3QsXG4gICAgYXNzb2NpYXRpb25zOiBRdWVyeUFyZ3NbXCJhc3NvY2lhdGlvbnNcIl0sXG4gICAgc3lzdGVtT2JqZWN0TWVtbzogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgbGV0IHN5c3RlbU9iamVjdDtcbiAgICBpZiAoc3lzdGVtT2JqZWN0TWVtbykge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0TWVtbztcbiAgICB9IGVsc2Uge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gdGhpcy5zeXN0ZW1PYmplY3Q7XG4gICAgfVxuICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4gfHwgcHJvcC5za2lwKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goYCR7cHJvcC5uYW1lfWApO1xuICAgICAgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICB0aGlzLmZpZWxkTGlzdChwcm9wIGFzIFByb3BPYmplY3QsIHVuZGVmaW5lZCwgc3lzdGVtT2JqZWN0KSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICB9XG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJtYXBcIikge1xuICAgICAgICByZXN1bHQucHVzaChcInsga2V5IHZhbHVlIH1cIik7XG4gICAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgY29uc3QgcmVhbE9iaiA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHJlYWxPYmogYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICBpZiAocmVhbE9iai5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKHRoaXMuYXNzb2NpYXRpb25GaWVsZExpc3QoYXNzb2NpYXRpb25zLCBzeXN0ZW1PYmplY3QpKTtcbiAgICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIGAke3Jlc3VsdC5qb2luKFwiIFwiKX1gO1xuICB9XG5cbiAgcXVlcnkoYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLFxuICAgICAgKTtcbiAgICAgIGlucHV0VmFyaWFibGVzLnB1c2goYCR7cHJvcC5uYW1lfTogJCR7cHJvcC5uYW1lfWApO1xuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgcXVlcnkgJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIHJldHVybiBncWxgXG4gICAgICAke3Jlc3VsdFN0cmluZ31cbiAgICBgO1xuICB9XG5cbiAgbXV0YXRpb24oYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLFxuICAgICAgKTtcbiAgICAgIGlucHV0VmFyaWFibGVzLnB1c2goYCR7cHJvcC5uYW1lfTogJCR7cHJvcC5uYW1lfWApO1xuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgbXV0YXRpb24gJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIGNvbnNvbGUubG9nKHJlc3VsdFN0cmluZyk7XG4gICAgcmV0dXJuIGdxbGBcbiAgICAgICR7cmVzdWx0U3RyaW5nfVxuICAgIGA7XG4gIH1cbn1cbiJdfQ==