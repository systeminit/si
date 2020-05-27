"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.variablesObjectForProperty = variablesObjectForProperty;
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

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(n); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

// Second argument is if you want a repeated field
// AKA thePoorlyNamedFunction() :)
function variablesObjectForProperty(prop) {
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
      return [];
    }
  } else if (prop.kind() == "link") {
    var propLink = prop;

    if (prop.repeated && repeated) {
      return [];
    } else {
      // TODO: There might be a bug here, where the name of the prop itself
      // and the name of the linked prop don't match, and so we get the
      // wrong field name if the prop is an object.
      return variablesObjectForProperty(propLink.lookupMyself(), repeated);
    }
  } else if (prop.kind() == "object" || prop.kind() == "method") {
    var propObject = prop;
    var result = {};

    var _iterator = _createForOfIteratorHelper(propObject.properties.attrs),
        _step;

    try {
      for (_iterator.s(); !(_step = _iterator.n()).done;) {
        var field = _step.value;
        var fieldVariables = variablesObjectForProperty(field, repeated);
        result["".concat(field.name)] = fieldVariables;
      }
    } catch (err) {
      _iterator.e(err);
    } finally {
      _iterator.f();
    }

    if (prop.repeated && repeated) {
      return [];
    } else {
      return result;
    }
  }
}

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

      var _iterator2 = _createForOfIteratorHelper(reply.properties.attrs),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var field = _step2.value;

          if (field.required && result[field.name] == undefined) {
            throw "response incomplete; missing required field ".concat(field);
          }
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return result;
    }
  }, {
    key: "variablesObject",
    value: function variablesObject(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var request = method.request;
      return variablesObjectForProperty(request, true);
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

          result.push("".concat((0, _changeCase.camelCase)(prop.name))); // added camelCase

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
          requestVariables.push("$".concat((0, _changeCase.camelCase)(prop.name), ": ").concat(this.graphqlTypeName(prop, true)) // added camelCase
          );
          inputVariables.push("".concat((0, _changeCase.camelCase)(prop.name), ": $").concat((0, _changeCase.camelCase)(prop.name))); // added camelCase
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
          requestVariables.push("$".concat((0, _changeCase.camelCase)(prop.name), ": ").concat(this.graphqlTypeName(prop, true)) // added camelCase
          );
          inputVariables.push("".concat((0, _changeCase.camelCase)(prop.name), ": $").concat((0, _changeCase.camelCase)(prop.name))); // added camelCase
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJ2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eSIsInByb3AiLCJyZXBlYXRlZCIsImtpbmQiLCJwcm9wTGluayIsImxvb2t1cE15c2VsZiIsInByb3BPYmplY3QiLCJyZXN1bHQiLCJwcm9wZXJ0aWVzIiwiYXR0cnMiLCJmaWVsZCIsImZpZWxkVmFyaWFibGVzIiwibmFtZSIsIlNpR3JhcGhxbCIsInN5c3RlbU9iamVjdCIsImFyZ3MiLCJtZXRob2QiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJtZXRob2ROYW1lIiwicmVwbHkiLCJsb29rdXBOYW1lIiwib3ZlcnJpZGVOYW1lIiwidHlwZU5hbWUiLCJkYXRhIiwicmVxdWlyZWQiLCJ1bmRlZmluZWQiLCJyZXF1ZXN0IiwiaW5wdXRUeXBlIiwicGFyZW50TmFtZSIsImxpbmtQcm9wIiwicmVhbFByb3AiLCJncmFwaHFsVHlwZU5hbWUiLCJhc3NvY2lhdGlvbnMiLCJhc3NvY2lhdGlvbkxpc3QiLCJwdXNoIiwiZmllbGROYW1lIiwiYXNzb2NPYmoiLCJnZXRCeUZpZWxkTmFtZSIsImFzc29jU3lzdGVtIiwicmVnaXN0cnkiLCJnZXQiLCJhc3NvY01ldGhvZCIsImZpZWxkTGlzdCIsImpvaW4iLCJzeXN0ZW1PYmplY3RNZW1vIiwiaGlkZGVuIiwic2tpcCIsImFzc29jaWF0aW9uRmllbGRMaXN0IiwicmVhbE9iaiIsInJlcXVlc3RWYXJpYWJsZXMiLCJpbnB1dFZhcmlhYmxlcyIsIm92ZXJyaWRlRmllbGRzIiwicmVzdWx0U3RyaW5nIiwiZ3FsIiwiY29uc29sZSIsImxvZyJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBR0E7O0FBRUE7O0FBQ0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUF1QkE7QUFDQTtBQUNPLFNBQVNBLDBCQUFULENBQW9DQyxJQUFwQyxFQUF3RTtBQUFBLE1BQXZCQyxRQUF1Qix1RUFB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QUM3QyxJQUQ2QyxDQUE3QixpQkFFVnpCLFVBRlUsdUJBRWFnQyxjQUFjLENBQUNQLElBQWYsQ0FDN0IsSUFENkIsQ0FGYixtQkFJUkQsU0FKUSxTQUFsQjtBQUtBWSxNQUFBQSxPQUFPLENBQUNDLEdBQVIsQ0FBWUgsWUFBWjtBQUNBLGlCQUFPQyxzQkFBUCxzQkFDSUQsWUFESjtBQUdEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcE1ldGhvZCwgUHJvcE9iamVjdCwgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcbmltcG9ydCB7IFByb3BMaW5rIH0gZnJvbSBcIi4uL3Byb3AvbGlua1wiO1xuaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuXG5pbXBvcnQgeyBwYXNjYWxDYXNlLCBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBncWwgZnJvbSBcImdyYXBocWwtdGFnXCI7XG5pbXBvcnQgeyBEb2N1bWVudE5vZGUgfSBmcm9tIFwiZ3JhcGhxbFwiO1xuaW1wb3J0IHsgQXNzb2NpYXRpb24gfSBmcm9tIFwiLi9hc3NvY2lhdGlvbnNcIjtcblxuaW50ZXJmYWNlIFF1ZXJ5QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xuICBvdmVycmlkZUZpZWxkcz86IHN0cmluZztcbiAgYXNzb2NpYXRpb25zPzoge1xuICAgIFtrZXk6IHN0cmluZ106IHN0cmluZ1tdO1xuICB9O1xufVxuXG5pbnRlcmZhY2UgVmFyaWFibGVzT2JqZWN0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbn1cblxuaW50ZXJmYWNlIFZhbGlkYXRlUmVzdWx0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgZGF0YTogUmVjb3JkPHN0cmluZywgYW55PjtcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xufVxuXG4vLyBTZWNvbmQgYXJndW1lbnQgaXMgaWYgeW91IHdhbnQgYSByZXBlYXRlZCBmaWVsZFxuLy8gQUtBIHRoZVBvb3JseU5hbWVkRnVuY3Rpb24oKSA6KVxuZXhwb3J0IGZ1bmN0aW9uIHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KHByb3A6IFByb3BzLCByZXBlYXRlZCA9IGZhbHNlKTogYW55IHtcbiAgaWYgKFxuICAgIHByb3Aua2luZCgpID09IFwidGV4dFwiIHx8XG4gICAgcHJvcC5raW5kKCkgPT0gXCJudW1iZXJcIiB8fFxuICAgIHByb3Aua2luZCgpID09IFwiY29kZVwiIHx8XG4gICAgcHJvcC5raW5kKCkgPT0gXCJlbnVtXCJcbiAgKSB7XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibWFwXCIpIHtcbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfVxuICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgY29uc3QgcHJvcExpbmsgPSBwcm9wIGFzIFByb3BMaW5rO1xuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIC8vIFRPRE86IFRoZXJlIG1pZ2h0IGJlIGEgYnVnIGhlcmUsIHdoZXJlIHRoZSBuYW1lIG9mIHRoZSBwcm9wIGl0c2VsZlxuICAgICAgLy8gYW5kIHRoZSBuYW1lIG9mIHRoZSBsaW5rZWQgcHJvcCBkb24ndCBtYXRjaCwgYW5kIHNvIHdlIGdldCB0aGVcbiAgICAgIC8vIHdyb25nIGZpZWxkIG5hbWUgaWYgdGhlIHByb3AgaXMgYW4gb2JqZWN0LlxuICAgICAgcmV0dXJuIHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KFxuICAgICAgICBwcm9wTGluay5sb29rdXBNeXNlbGYoKSxcbiAgICAgICAgcmVwZWF0ZWQsXG4gICAgICApO1xuICAgIH1cbiAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiIHx8IHByb3Aua2luZCgpID09IFwibWV0aG9kXCIpIHtcbiAgICBjb25zdCBwcm9wT2JqZWN0ID0gcHJvcCBhcyBQcm9wT2JqZWN0O1xuICAgIGNvbnN0IHJlc3VsdDogUmVjb3JkPHN0cmluZywgdW5rbm93bj4gPSB7fTtcbiAgICBmb3IgKGNvbnN0IGZpZWxkIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgY29uc3QgZmllbGRWYXJpYWJsZXMgPSB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShmaWVsZCwgcmVwZWF0ZWQpO1xuICAgICAgcmVzdWx0W2Ake2ZpZWxkLm5hbWV9YF0gPSBmaWVsZFZhcmlhYmxlcztcbiAgICB9XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHJlc3VsdDtcbiAgICB9XG4gIH1cbn1cblxuZXhwb3J0IGNsYXNzIFNpR3JhcGhxbCB7XG4gIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXM7XG5cbiAgY29uc3RydWN0b3Ioc3lzdGVtT2JqZWN0OiBTaUdyYXBocWxbXCJzeXN0ZW1PYmplY3RcIl0pIHtcbiAgICB0aGlzLnN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdDtcbiAgfVxuXG4gIHZhbGlkYXRlUmVzdWx0KGFyZ3M6IFZhbGlkYXRlUmVzdWx0QXJncyk6IFJlY29yZDxzdHJpbmcsIGFueT4ge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGNvbnN0IGxvb2t1cE5hbWUgPVxuICAgICAgYXJncy5vdmVycmlkZU5hbWUgfHxcbiAgICAgIGAke2NhbWVsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9JHtwYXNjYWxDYXNlKGFyZ3MubWV0aG9kTmFtZSl9YDtcbiAgICBjb25zdCByZXN1bHQgPSBhcmdzLmRhdGEuZGF0YVtsb29rdXBOYW1lXTtcbiAgICBmb3IgKGNvbnN0IGZpZWxkIG9mIHJlcGx5LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChmaWVsZC5yZXF1aXJlZCAmJiByZXN1bHRbZmllbGQubmFtZV0gPT0gdW5kZWZpbmVkKSB7XG4gICAgICAgIHRocm93IGByZXNwb25zZSBpbmNvbXBsZXRlOyBtaXNzaW5nIHJlcXVpcmVkIGZpZWxkICR7ZmllbGR9YDtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIHZhcmlhYmxlc09iamVjdChhcmdzOiBWYXJpYWJsZXNPYmplY3RBcmdzKTogUmVjb3JkPHN0cmluZywgYW55PiB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgcmVxdWVzdCA9IG1ldGhvZC5yZXF1ZXN0O1xuICAgIHJldHVybiB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShyZXF1ZXN0LCB0cnVlKTtcbiAgfVxuXG4gIGdyYXBocWxUeXBlTmFtZShwcm9wOiBQcm9wcywgaW5wdXRUeXBlPzogYm9vbGVhbik6IHN0cmluZyB7XG4gICAgbGV0IHJlc3VsdCA9IFwiXCI7XG4gICAgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIgfHwgcHJvcC5raW5kKCkgPT0gXCJlbnVtXCIpIHtcbiAgICAgIGxldCByZXF1ZXN0ID0gXCJcIjtcbiAgICAgIGlmIChpbnB1dFR5cGUgJiYgcHJvcC5raW5kKCkgIT0gXCJlbnVtXCIpIHtcbiAgICAgICAgcmVxdWVzdCA9IFwiUmVxdWVzdFwiO1xuICAgICAgfVxuICAgICAgcmVzdWx0ID0gYCR7cGFzY2FsQ2FzZShwcm9wLnBhcmVudE5hbWUpfSR7cGFzY2FsQ2FzZShcbiAgICAgICAgcHJvcC5uYW1lLFxuICAgICAgKX0ke3JlcXVlc3R9YDtcbiAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwidGV4dFwiIHx8IHByb3Aua2luZCgpID09IFwicGFzc3dvcmRcIikge1xuICAgICAgaWYgKHByb3AubmFtZSA9PSBcImlkXCIpIHtcbiAgICAgICAgcmVzdWx0ID0gXCJJRFwiO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgcmVzdWx0ID0gXCJTdHJpbmdcIjtcbiAgICAgIH1cbiAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibnVtYmVyXCIpIHtcbiAgICAgIHJlc3VsdCA9IFwiU3RyaW5nXCI7XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgICAgY29uc3QgbGlua1Byb3AgPSBwcm9wIGFzIFByb3BMaW5rO1xuICAgICAgY29uc3QgcmVhbFByb3AgPSBsaW5rUHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIHJldHVybiB0aGlzLmdyYXBocWxUeXBlTmFtZShyZWFsUHJvcCwgaW5wdXRUeXBlKTtcbiAgICB9XG4gICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbiAgICAgIHJldHVybiBgJHtyZXN1bHR9IWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiByZXN1bHQ7XG4gICAgfVxuICB9XG5cbiAgYXNzb2NpYXRpb25GaWVsZExpc3QoXG4gICAgYXNzb2NpYXRpb25zOiBRdWVyeUFyZ3NbXCJhc3NvY2lhdGlvbnNcIl0sXG4gICAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcyxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCBhc3NvY2lhdGlvbkxpc3QgPSBhc3NvY2lhdGlvbnMgJiYgYXNzb2NpYXRpb25zW3N5c3RlbU9iamVjdC50eXBlTmFtZV07XG4gICAgaWYgKGFzc29jaWF0aW9uTGlzdCkge1xuICAgICAgY29uc3QgcmVzdWx0OiBzdHJpbmdbXSA9IFtdO1xuICAgICAgcmVzdWx0LnB1c2goXCJhc3NvY2lhdGlvbnMge1wiKTtcbiAgICAgIGZvciAoY29uc3QgZmllbGROYW1lIG9mIGFzc29jaWF0aW9uTGlzdCkge1xuICAgICAgICBjb25zdCBhc3NvY09iaiA9IHN5c3RlbU9iamVjdC5hc3NvY2lhdGlvbnMuZ2V0QnlGaWVsZE5hbWUoZmllbGROYW1lKTtcbiAgICAgICAgY29uc3QgYXNzb2NTeXN0ZW0gPSByZWdpc3RyeS5nZXQoYXNzb2NPYmoudHlwZU5hbWUpO1xuICAgICAgICBjb25zdCBhc3NvY01ldGhvZCA9IGFzc29jU3lzdGVtLm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICAgICAgYXNzb2NPYmoubWV0aG9kTmFtZSxcbiAgICAgICAgKSBhcyBQcm9wTWV0aG9kO1xuXG4gICAgICAgIHJlc3VsdC5wdXNoKGAke2ZpZWxkTmFtZX0ge2ApO1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICB0aGlzLmZpZWxkTGlzdChhc3NvY01ldGhvZC5yZXBseSwgYXNzb2NpYXRpb25zLCBhc3NvY1N5c3RlbSksXG4gICAgICAgICk7XG4gICAgICAgIHJlc3VsdC5wdXNoKGB9YCk7XG4gICAgICB9XG4gICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICByZXR1cm4gcmVzdWx0LmpvaW4oXCIgXCIpO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJcIjtcbiAgICB9XG4gIH1cblxuICBmaWVsZExpc3QoXG4gICAgcHJvcE9iamVjdDogUHJvcE9iamVjdCxcbiAgICBhc3NvY2lhdGlvbnM6IFF1ZXJ5QXJnc1tcImFzc29jaWF0aW9uc1wiXSxcbiAgICBzeXN0ZW1PYmplY3RNZW1vOiBPYmplY3RUeXBlcyxcbiAgKTogc3RyaW5nIHtcbiAgICBsZXQgc3lzdGVtT2JqZWN0O1xuICAgIGlmIChzeXN0ZW1PYmplY3RNZW1vKSB7XG4gICAgICBzeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3RNZW1vO1xuICAgIH0gZWxzZSB7XG4gICAgICBzeXN0ZW1PYmplY3QgPSB0aGlzLnN5c3RlbU9iamVjdDtcbiAgICB9XG4gICAgY29uc3QgcmVzdWx0OiBzdHJpbmdbXSA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiBwcm9wT2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChwcm9wLmhpZGRlbiB8fCBwcm9wLnNraXApIHtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9XG4gICAgICByZXN1bHQucHVzaChgJHtjYW1lbENhc2UocHJvcC5uYW1lKX1gKTsgLy8gYWRkZWQgY2FtZWxDYXNlXG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICByZXN1bHQucHVzaChcIntcIik7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHByb3AgYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaCh0aGlzLmFzc29jaWF0aW9uRmllbGRMaXN0KGFzc29jaWF0aW9ucywgc3lzdGVtT2JqZWN0KSk7XG4gICAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgIH1cbiAgICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm1hcFwiKSB7XG4gICAgICAgIHJlc3VsdC5wdXNoKFwieyBrZXkgdmFsdWUgfVwiKTtcbiAgICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgICBjb25zdCByZWFsT2JqID0gcHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgICAgaWYgKHJlYWxPYmoua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgICByZXN1bHQucHVzaChcIntcIik7XG4gICAgICAgIH1cbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QocmVhbE9iaiBhcyBQcm9wT2JqZWN0LCB1bmRlZmluZWQsIHN5c3RlbU9iamVjdCksXG4gICAgICAgICk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gYCR7cmVzdWx0LmpvaW4oXCIgXCIpfWA7XG4gIH1cblxuICBxdWVyeShhcmdzOiBRdWVyeUFyZ3MpOiBEb2N1bWVudE5vZGUge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IG1ldGhvZE5hbWUgPVxuICAgICAgYXJncy5vdmVycmlkZU5hbWUgfHxcbiAgICAgIGAke2NhbWVsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9JHtwYXNjYWxDYXNlKGFyZ3MubWV0aG9kTmFtZSl9YDtcblxuICAgIGNvbnN0IHJlcXVlc3QgPSBtZXRob2QucmVxdWVzdDtcbiAgICBjb25zdCByZXF1ZXN0VmFyaWFibGVzID0gW107XG4gICAgY29uc3QgaW5wdXRWYXJpYWJsZXMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcmVxdWVzdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICByZXF1ZXN0VmFyaWFibGVzLnB1c2goXG4gICAgICAgIGAkJHtjYW1lbENhc2UocHJvcC5uYW1lKX06ICR7dGhpcy5ncmFwaHFsVHlwZU5hbWUocHJvcCwgdHJ1ZSl9YCwgLy8gYWRkZWQgY2FtZWxDYXNlXG4gICAgICApO1xuICAgICAgaW5wdXRWYXJpYWJsZXMucHVzaChgJHtjYW1lbENhc2UocHJvcC5uYW1lKX06ICQke2NhbWVsQ2FzZShwcm9wLm5hbWUpfWApOyAvLyBhZGRlZCBjYW1lbENhc2VcbiAgICB9XG5cbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBsZXQgZmllbGRMaXN0OiBzdHJpbmc7XG4gICAgaWYgKGFyZ3Mub3ZlcnJpZGVGaWVsZHMpIHtcbiAgICAgIGZpZWxkTGlzdCA9IGAke2FyZ3Mub3ZlcnJpZGVGaWVsZHN9YDtcbiAgICB9IGVsc2Uge1xuICAgICAgZmllbGRMaXN0ID0gdGhpcy5maWVsZExpc3QocmVwbHksIGFyZ3MuYXNzb2NpYXRpb25zLCB0aGlzLnN5c3RlbU9iamVjdCk7XG4gICAgfVxuXG4gICAgY29uc3QgcmVzdWx0U3RyaW5nID0gYHF1ZXJ5ICR7bWV0aG9kTmFtZX0oJHtyZXF1ZXN0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0pIHsgJHttZXRob2ROYW1lfShpbnB1dDogeyAke2lucHV0VmFyaWFibGVzLmpvaW4oXG4gICAgICBcIiwgXCIsXG4gICAgKX0gfSkgeyAke2ZpZWxkTGlzdH0gfSB9YDtcbiAgICByZXR1cm4gZ3FsYFxuICAgICAgJHtyZXN1bHRTdHJpbmd9XG4gICAgYDtcbiAgfVxuXG4gIG11dGF0aW9uKGFyZ3M6IFF1ZXJ5QXJncyk6IERvY3VtZW50Tm9kZSB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgbWV0aG9kTmFtZSA9XG4gICAgICBhcmdzLm92ZXJyaWRlTmFtZSB8fFxuICAgICAgYCR7Y2FtZWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ke3Bhc2NhbENhc2UoYXJncy5tZXRob2ROYW1lKX1gO1xuXG4gICAgY29uc3QgcmVxdWVzdCA9IG1ldGhvZC5yZXF1ZXN0O1xuICAgIGNvbnN0IHJlcXVlc3RWYXJpYWJsZXMgPSBbXTtcbiAgICBjb25zdCBpbnB1dFZhcmlhYmxlcyA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiByZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIHJlcXVlc3RWYXJpYWJsZXMucHVzaChcbiAgICAgICAgYCQke2NhbWVsQ2FzZShwcm9wLm5hbWUpfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLCAvLyBhZGRlZCBjYW1lbENhc2VcbiAgICAgICk7XG4gICAgICBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke2NhbWVsQ2FzZShwcm9wLm5hbWUpfTogJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9YCk7IC8vIGFkZGVkIGNhbWVsQ2FzZVxuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgbXV0YXRpb24gJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIGNvbnNvbGUubG9nKHJlc3VsdFN0cmluZyk7XG4gICAgcmV0dXJuIGdxbGBcbiAgICAgICR7cmVzdWx0U3RyaW5nfVxuICAgIGA7XG4gIH1cbn1cbiJdfQ==