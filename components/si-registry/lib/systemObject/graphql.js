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

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

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
    key: "extractResult",
    value: function extractResult(args) {
      var method = this.systemObject.methods.getEntry(args.methodName);
      var reply = method.reply;
      var lookupName = args.overrideName || "".concat((0, _changeCase.camelCase)(this.systemObject.typeName)).concat((0, _changeCase.pascalCase)(args.methodName));
      var result = args.data.data[lookupName];
      return result;
    }
  }, {
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
            throw new Error("response incomplete; missing required field ".concat(field));
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
        // @ts-ignore - we don't know about numberKind below
        if (prop.numberKind == "int32") {
          result = "Int";
        } else {
          result = "String";
        }
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

          if (prop.skip) {
            continue;
          }

          result.push("".concat(prop.name)); // without camelCase
          // result.push(`${camelCase(prop.name)}`); // with camelCase

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
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop, true)) // without camelCase
          // `$${camelCase(prop.name)}: ${this.graphqlTypeName(prop, true)}`, // with camelCase
          );
          inputVariables.push("".concat(prop.name, ": $").concat(prop.name)); // without camelCase
          // inputVariables.push(`${camelCase(prop.name)}: $${camelCase(prop.name)}`); // with camelCase
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

      var resultString = "query ".concat(methodName, "(").concat(requestVariables.join(", "), ") { ").concat(methodName, "(input: { ").concat(inputVariables.join(", "), " }) { ").concat(fieldList, " } }"); // Log query
      // console.log(`query ${resultString}`);

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
          requestVariables.push("$".concat(prop.name, ": ").concat(this.graphqlTypeName(prop, true)) // without camelCase
          // `$${camelCase(prop.name)}: ${this.graphqlTypeName(prop, true)}`, // with camelCase
          );
          inputVariables.push("".concat(prop.name, ": $").concat(prop.name)); // without camelCase
          // inputVariables.push(`${camelCase(prop.name)}: $${camelCase(prop.name)}`); // with camelCase
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJ2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eSIsInByb3AiLCJyZXBlYXRlZCIsImtpbmQiLCJwcm9wTGluayIsImxvb2t1cE15c2VsZiIsInByb3BPYmplY3QiLCJyZXN1bHQiLCJwcm9wZXJ0aWVzIiwiYXR0cnMiLCJmaWVsZCIsImZpZWxkVmFyaWFibGVzIiwibmFtZSIsIlNpR3JhcGhxbCIsInN5c3RlbU9iamVjdCIsImFyZ3MiLCJtZXRob2QiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJtZXRob2ROYW1lIiwicmVwbHkiLCJsb29rdXBOYW1lIiwib3ZlcnJpZGVOYW1lIiwidHlwZU5hbWUiLCJkYXRhIiwicmVxdWlyZWQiLCJ1bmRlZmluZWQiLCJFcnJvciIsInJlcXVlc3QiLCJpbnB1dFR5cGUiLCJwYXJlbnROYW1lIiwibnVtYmVyS2luZCIsImxpbmtQcm9wIiwicmVhbFByb3AiLCJncmFwaHFsVHlwZU5hbWUiLCJhc3NvY2lhdGlvbnMiLCJhc3NvY2lhdGlvbkxpc3QiLCJwdXNoIiwiZmllbGROYW1lIiwiYXNzb2NPYmoiLCJnZXRCeUZpZWxkTmFtZSIsImFzc29jU3lzdGVtIiwicmVnaXN0cnkiLCJnZXQiLCJhc3NvY01ldGhvZCIsImZpZWxkTGlzdCIsImpvaW4iLCJzeXN0ZW1PYmplY3RNZW1vIiwic2tpcCIsImFzc29jaWF0aW9uRmllbGRMaXN0IiwicmVhbE9iaiIsInJlcXVlc3RWYXJpYWJsZXMiLCJpbnB1dFZhcmlhYmxlcyIsIm92ZXJyaWRlRmllbGRzIiwicmVzdWx0U3RyaW5nIiwiZ3FsIiwiY29uc29sZSIsImxvZyJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBR0E7O0FBRUE7O0FBQ0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUF1QkE7QUFDQTtBQUNPLFNBQVNBLDBCQUFULENBQW9DQyxJQUFwQyxFQUF3RTtBQUFBLE1BQXZC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LElBQWYsV0FBdUJwQyxJQUFJLENBQUNXLElBQTVCLGdCQUFzQ1gsSUFBSSxDQUFDVyxJQUEzQyxHQUwyQyxDQUtTO0FBQ3BEO0FBQ0Q7QUFsQnFDO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBb0J0QyxVQUFNUSxLQUFLLEdBQUdKLE1BQU0sQ0FBQ0ksS0FBckI7QUFDQSxVQUFJeUIsU0FBSjs7QUFDQSxVQUFJOUIsSUFBSSxDQUFDc0MsY0FBVCxFQUF5QjtBQUN2QlIsUUFBQUEsU0FBUyxhQUFNOUIsSUFBSSxDQUFDc0MsY0FBWCxDQUFUO0FBQ0QsT0FGRCxNQUVPO0FBQ0xSLFFBQUFBLFNBQVMsR0FBRyxLQUFLQSxTQUFMLENBQWV6QixLQUFmLEVBQXNCTCxJQUFJLENBQUNvQixZQUEzQixFQUF5QyxLQUFLckIsWUFBOUMsQ0FBWjtBQUNEOztBQUVELFVBQU13QyxZQUFZLHNCQUFlbkMsVUFBZixjQUE2QmdDLGdCQUFnQixDQUFDTCxJQUFqQixDQUM3QyxJQUQ2QyxDQUE3QixpQkFFVjNCLFVBRlUsdUJBRWFpQyxjQUFjLENBQUNOLElBQWYsQ0FDN0IsSUFENkIsQ0FGYixtQkFJUkQsU0FKUSxTQUFsQjtBQUtBVyxNQUFBQSxPQUFPLENBQUNDLEdBQVIsQ0FBWUgsWUFBWjtBQUNBLGlCQUFPQyxzQkFBUCxzQkFDSUQsWUFESjtBQUdEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgUHJvcE1ldGhvZCwgUHJvcE9iamVjdCwgUHJvcHMgfSBmcm9tIFwiLi4vYXR0ckxpc3RcIjtcbmltcG9ydCB7IFByb3BMaW5rIH0gZnJvbSBcIi4uL3Byb3AvbGlua1wiO1xuaW1wb3J0IHsgT2JqZWN0VHlwZXMgfSBmcm9tIFwiLi4vc3lzdGVtQ29tcG9uZW50XCI7XG5pbXBvcnQgeyByZWdpc3RyeSB9IGZyb20gXCIuLi9yZWdpc3RyeVwiO1xuXG5pbXBvcnQgeyBwYXNjYWxDYXNlLCBjYW1lbENhc2UgfSBmcm9tIFwiY2hhbmdlLWNhc2VcIjtcbmltcG9ydCBncWwgZnJvbSBcImdyYXBocWwtdGFnXCI7XG5pbXBvcnQgeyBEb2N1bWVudE5vZGUgfSBmcm9tIFwiZ3JhcGhxbFwiO1xuaW1wb3J0IHsgQXNzb2NpYXRpb24gfSBmcm9tIFwiLi9hc3NvY2lhdGlvbnNcIjtcblxuZXhwb3J0IGludGVyZmFjZSBRdWVyeUFyZ3Mge1xuICBtZXRob2ROYW1lOiBzdHJpbmc7XG4gIG92ZXJyaWRlTmFtZT86IHN0cmluZztcbiAgb3ZlcnJpZGVGaWVsZHM/OiBzdHJpbmc7XG4gIGFzc29jaWF0aW9ucz86IHtcbiAgICBba2V5OiBzdHJpbmddOiBzdHJpbmdbXTtcbiAgfTtcbn1cblxuZXhwb3J0IGludGVyZmFjZSBWYXJpYWJsZXNPYmplY3RBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xufVxuXG5leHBvcnQgaW50ZXJmYWNlIFZhbGlkYXRlUmVzdWx0QXJncyB7XG4gIG1ldGhvZE5hbWU6IHN0cmluZztcbiAgZGF0YTogUmVjb3JkPHN0cmluZywgYW55PjtcbiAgb3ZlcnJpZGVOYW1lPzogc3RyaW5nO1xufVxuXG4vLyBTZWNvbmQgYXJndW1lbnQgaXMgaWYgeW91IHdhbnQgYSByZXBlYXRlZCBmaWVsZFxuLy8gQUtBIHRoZVBvb3JseU5hbWVkRnVuY3Rpb24oKSA6KVxuZXhwb3J0IGZ1bmN0aW9uIHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KHByb3A6IFByb3BzLCByZXBlYXRlZCA9IGZhbHNlKTogYW55IHtcbiAgaWYgKFxuICAgIHByb3Aua2luZCgpID09IFwidGV4dFwiIHx8XG4gICAgcHJvcC5raW5kKCkgPT0gXCJudW1iZXJcIiB8fFxuICAgIHByb3Aua2luZCgpID09IFwiY29kZVwiIHx8XG4gICAgcHJvcC5raW5kKCkgPT0gXCJlbnVtXCJcbiAgKSB7XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibWFwXCIpIHtcbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfVxuICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgY29uc3QgcHJvcExpbmsgPSBwcm9wIGFzIFByb3BMaW5rO1xuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIC8vIFRPRE86IFRoZXJlIG1pZ2h0IGJlIGEgYnVnIGhlcmUsIHdoZXJlIHRoZSBuYW1lIG9mIHRoZSBwcm9wIGl0c2VsZlxuICAgICAgLy8gYW5kIHRoZSBuYW1lIG9mIHRoZSBsaW5rZWQgcHJvcCBkb24ndCBtYXRjaCwgYW5kIHNvIHdlIGdldCB0aGVcbiAgICAgIC8vIHdyb25nIGZpZWxkIG5hbWUgaWYgdGhlIHByb3AgaXMgYW4gb2JqZWN0LlxuICAgICAgcmV0dXJuIHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KHByb3BMaW5rLmxvb2t1cE15c2VsZigpLCByZXBlYXRlZCk7XG4gICAgfVxuICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIgfHwgcHJvcC5raW5kKCkgPT0gXCJtZXRob2RcIikge1xuICAgIGNvbnN0IHByb3BPYmplY3QgPSBwcm9wIGFzIFByb3BPYmplY3Q7XG4gICAgY29uc3QgcmVzdWx0OiBSZWNvcmQ8c3RyaW5nLCB1bmtub3duPiA9IHt9O1xuICAgIGZvciAoY29uc3QgZmllbGQgb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBjb25zdCBmaWVsZFZhcmlhYmxlcyA9IHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KGZpZWxkLCByZXBlYXRlZCk7XG4gICAgICByZXN1bHRbYCR7ZmllbGQubmFtZX1gXSA9IGZpZWxkVmFyaWFibGVzO1xuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgU2lHcmFwaHFsIHtcbiAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcblxuICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFNpR3JhcGhxbFtcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgZXh0cmFjdFJlc3VsdChhcmdzOiBWYWxpZGF0ZVJlc3VsdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBjb25zdCBsb29rdXBOYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG4gICAgY29uc3QgcmVzdWx0ID0gYXJncy5kYXRhLmRhdGFbbG9va3VwTmFtZV07XG4gICAgcmV0dXJuIHJlc3VsdDtcbiAgfVxuXG4gIHZhbGlkYXRlUmVzdWx0KGFyZ3M6IFZhbGlkYXRlUmVzdWx0QXJncyk6IFJlY29yZDxzdHJpbmcsIGFueT4ge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGNvbnN0IGxvb2t1cE5hbWUgPVxuICAgICAgYXJncy5vdmVycmlkZU5hbWUgfHxcbiAgICAgIGAke2NhbWVsQ2FzZSh0aGlzLnN5c3RlbU9iamVjdC50eXBlTmFtZSl9JHtwYXNjYWxDYXNlKGFyZ3MubWV0aG9kTmFtZSl9YDtcbiAgICBjb25zdCByZXN1bHQgPSBhcmdzLmRhdGEuZGF0YVtsb29rdXBOYW1lXTtcbiAgICBmb3IgKGNvbnN0IGZpZWxkIG9mIHJlcGx5LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGlmIChmaWVsZC5yZXF1aXJlZCAmJiByZXN1bHRbZmllbGQubmFtZV0gPT0gdW5kZWZpbmVkKSB7XG4gICAgICAgIHRocm93IG5ldyBFcnJvcihgcmVzcG9uc2UgaW5jb21wbGV0ZTsgbWlzc2luZyByZXF1aXJlZCBmaWVsZCAke2ZpZWxkfWApO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgdmFyaWFibGVzT2JqZWN0KGFyZ3M6IFZhcmlhYmxlc09iamVjdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgcmV0dXJuIHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KHJlcXVlc3QsIHRydWUpO1xuICB9XG5cbiAgZ3JhcGhxbFR5cGVOYW1lKHByb3A6IFByb3BzLCBpbnB1dFR5cGU/OiBib29sZWFuKTogc3RyaW5nIHtcbiAgICBsZXQgcmVzdWx0ID0gXCJcIjtcbiAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIiB8fCBwcm9wLmtpbmQoKSA9PSBcImVudW1cIikge1xuICAgICAgbGV0IHJlcXVlc3QgPSBcIlwiO1xuICAgICAgaWYgKGlucHV0VHlwZSAmJiBwcm9wLmtpbmQoKSAhPSBcImVudW1cIikge1xuICAgICAgICByZXF1ZXN0ID0gXCJSZXF1ZXN0XCI7XG4gICAgICB9XG4gICAgICByZXN1bHQgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLm5hbWUsXG4gICAgICApfSR7cmVxdWVzdH1gO1xuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJ0ZXh0XCIgfHwgcHJvcC5raW5kKCkgPT0gXCJwYXNzd29yZFwiKSB7XG4gICAgICBpZiAocHJvcC5uYW1lID09IFwiaWRcIikge1xuICAgICAgICByZXN1bHQgPSBcIklEXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHQgPSBcIlN0cmluZ1wiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJudW1iZXJcIikge1xuICAgICAgLy8gQHRzLWlnbm9yZSAtIHdlIGRvbid0IGtub3cgYWJvdXQgbnVtYmVyS2luZCBiZWxvd1xuICAgICAgaWYgKHByb3AubnVtYmVyS2luZCA9PSBcImludDMyXCIpIHtcbiAgICAgICAgcmVzdWx0ID0gXCJJbnRcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdCA9IFwiU3RyaW5nXCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgICAgY29uc3QgbGlua1Byb3AgPSBwcm9wIGFzIFByb3BMaW5rO1xuICAgICAgY29uc3QgcmVhbFByb3AgPSBsaW5rUHJvcC5sb29rdXBNeXNlbGYoKTtcbiAgICAgIHJldHVybiB0aGlzLmdyYXBocWxUeXBlTmFtZShyZWFsUHJvcCwgaW5wdXRUeXBlKTtcbiAgICB9XG4gICAgaWYgKHByb3AucmVxdWlyZWQpIHtcbiAgICAgIHJldHVybiBgJHtyZXN1bHR9IWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiByZXN1bHQ7XG4gICAgfVxuICB9XG5cbiAgYXNzb2NpYXRpb25GaWVsZExpc3QoXG4gICAgYXNzb2NpYXRpb25zOiBRdWVyeUFyZ3NbXCJhc3NvY2lhdGlvbnNcIl0sXG4gICAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcyxcbiAgKTogc3RyaW5nIHtcbiAgICBjb25zdCBhc3NvY2lhdGlvbkxpc3QgPSBhc3NvY2lhdGlvbnMgJiYgYXNzb2NpYXRpb25zW3N5c3RlbU9iamVjdC50eXBlTmFtZV07XG5cbiAgICBpZiAoYXNzb2NpYXRpb25MaXN0KSB7XG4gICAgICBjb25zdCByZXN1bHQ6IHN0cmluZ1tdID0gW107XG4gICAgICByZXN1bHQucHVzaChcImFzc29jaWF0aW9ucyB7XCIpO1xuICAgICAgZm9yIChjb25zdCBmaWVsZE5hbWUgb2YgYXNzb2NpYXRpb25MaXN0KSB7XG4gICAgICAgIGNvbnN0IGFzc29jT2JqID0gc3lzdGVtT2JqZWN0LmFzc29jaWF0aW9ucy5nZXRCeUZpZWxkTmFtZShmaWVsZE5hbWUpO1xuICAgICAgICBjb25zdCBhc3NvY1N5c3RlbSA9IHJlZ2lzdHJ5LmdldChhc3NvY09iai50eXBlTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jTWV0aG9kID0gYXNzb2NTeXN0ZW0ubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgICAgICBhc3NvY09iai5tZXRob2ROYW1lLFxuICAgICAgICApIGFzIFByb3BNZXRob2Q7XG5cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfSB7YCk7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KGFzc29jTWV0aG9kLnJlcGx5LCBhc3NvY2lhdGlvbnMsIGFzc29jU3lzdGVtKSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goYH1gKTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgIHJldHVybiByZXN1bHQuam9pbihcIiBcIik7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlwiO1xuICAgIH1cbiAgfVxuXG4gIGZpZWxkTGlzdChcbiAgICBwcm9wT2JqZWN0OiBQcm9wT2JqZWN0LFxuICAgIGFzc29jaWF0aW9uczogUXVlcnlBcmdzW1wiYXNzb2NpYXRpb25zXCJdLFxuICAgIHN5c3RlbU9iamVjdE1lbW86IE9iamVjdFR5cGVzLFxuICApOiBzdHJpbmcge1xuICAgIGxldCBzeXN0ZW1PYmplY3Q7XG4gICAgaWYgKHN5c3RlbU9iamVjdE1lbW8pIHtcbiAgICAgIHN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdE1lbW87XG4gICAgfSBlbHNlIHtcbiAgICAgIHN5c3RlbU9iamVjdCA9IHRoaXMuc3lzdGVtT2JqZWN0O1xuICAgIH1cbiAgICBjb25zdCByZXN1bHQ6IHN0cmluZ1tdID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKHByb3Auc2tpcCkge1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdC5wdXNoKGAke3Byb3AubmFtZX1gKTsgLy8gd2l0aG91dCBjYW1lbENhc2VcbiAgICAgIC8vIHJlc3VsdC5wdXNoKGAke2NhbWVsQ2FzZShwcm9wLm5hbWUpfWApOyAvLyB3aXRoIGNhbWVsQ2FzZVxuICAgICAgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICB0aGlzLmZpZWxkTGlzdChwcm9wIGFzIFByb3BPYmplY3QsIHVuZGVmaW5lZCwgc3lzdGVtT2JqZWN0KSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICB9XG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJtYXBcIikge1xuICAgICAgICByZXN1bHQucHVzaChcInsga2V5IHZhbHVlIH1cIik7XG4gICAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgY29uc3QgcmVhbE9iaiA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHJlYWxPYmogYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICBpZiAocmVhbE9iai5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKHRoaXMuYXNzb2NpYXRpb25GaWVsZExpc3QoYXNzb2NpYXRpb25zLCBzeXN0ZW1PYmplY3QpKTtcbiAgICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIGAke3Jlc3VsdC5qb2luKFwiIFwiKX1gO1xuICB9XG5cbiAgcXVlcnkoYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLCAvLyB3aXRob3V0IGNhbWVsQ2FzZVxuICAgICAgICAvLyBgJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3AsIHRydWUpfWAsIC8vIHdpdGggY2FtZWxDYXNlXG4gICAgICApO1xuICAgICAgaW5wdXRWYXJpYWJsZXMucHVzaChgJHtwcm9wLm5hbWV9OiAkJHtwcm9wLm5hbWV9YCk7IC8vIHdpdGhvdXQgY2FtZWxDYXNlXG4gICAgICAvLyBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke2NhbWVsQ2FzZShwcm9wLm5hbWUpfTogJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9YCk7IC8vIHdpdGggY2FtZWxDYXNlXG4gICAgfVxuXG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgbGV0IGZpZWxkTGlzdDogc3RyaW5nO1xuICAgIGlmIChhcmdzLm92ZXJyaWRlRmllbGRzKSB7XG4gICAgICBmaWVsZExpc3QgPSBgJHthcmdzLm92ZXJyaWRlRmllbGRzfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGZpZWxkTGlzdCA9IHRoaXMuZmllbGRMaXN0KHJlcGx5LCBhcmdzLmFzc29jaWF0aW9ucywgdGhpcy5zeXN0ZW1PYmplY3QpO1xuICAgIH1cblxuICAgIGNvbnN0IHJlc3VsdFN0cmluZyA9IGBxdWVyeSAke21ldGhvZE5hbWV9KCR7cmVxdWVzdFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9KSB7ICR7bWV0aG9kTmFtZX0oaW5wdXQ6IHsgJHtpbnB1dFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9IH0pIHsgJHtmaWVsZExpc3R9IH0gfWA7XG5cbiAgICAvLyBMb2cgcXVlcnlcbiAgICAvLyBjb25zb2xlLmxvZyhgcXVlcnkgJHtyZXN1bHRTdHJpbmd9YCk7XG5cbiAgICByZXR1cm4gZ3FsYFxuICAgICAgJHtyZXN1bHRTdHJpbmd9XG4gICAgYDtcbiAgfVxuXG4gIG11dGF0aW9uKGFyZ3M6IFF1ZXJ5QXJncyk6IERvY3VtZW50Tm9kZSB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgbWV0aG9kTmFtZSA9XG4gICAgICBhcmdzLm92ZXJyaWRlTmFtZSB8fFxuICAgICAgYCR7Y2FtZWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ke3Bhc2NhbENhc2UoYXJncy5tZXRob2ROYW1lKX1gO1xuXG4gICAgY29uc3QgcmVxdWVzdCA9IG1ldGhvZC5yZXF1ZXN0O1xuICAgIGNvbnN0IHJlcXVlc3RWYXJpYWJsZXMgPSBbXTtcbiAgICBjb25zdCBpbnB1dFZhcmlhYmxlcyA9IFtdO1xuICAgIGZvciAoY29uc3QgcHJvcCBvZiByZXF1ZXN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIHJlcXVlc3RWYXJpYWJsZXMucHVzaChcbiAgICAgICAgYCQke3Byb3AubmFtZX06ICR7dGhpcy5ncmFwaHFsVHlwZU5hbWUocHJvcCwgdHJ1ZSl9YCwgLy8gd2l0aG91dCBjYW1lbENhc2VcbiAgICAgICAgLy8gYCQke2NhbWVsQ2FzZShwcm9wLm5hbWUpfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLCAvLyB3aXRoIGNhbWVsQ2FzZVxuICAgICAgKTtcbiAgICAgIGlucHV0VmFyaWFibGVzLnB1c2goYCR7cHJvcC5uYW1lfTogJCR7cHJvcC5uYW1lfWApOyAvLyB3aXRob3V0IGNhbWVsQ2FzZVxuICAgICAgLy8gaW5wdXRWYXJpYWJsZXMucHVzaChgJHtjYW1lbENhc2UocHJvcC5uYW1lKX06ICQke2NhbWVsQ2FzZShwcm9wLm5hbWUpfWApOyAvLyB3aXRoIGNhbWVsQ2FzZVxuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgbXV0YXRpb24gJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIGNvbnNvbGUubG9nKHJlc3VsdFN0cmluZyk7XG4gICAgcmV0dXJuIGdxbGBcbiAgICAgICR7cmVzdWx0U3RyaW5nfVxuICAgIGA7XG4gIH1cbn1cbiJdfQ==