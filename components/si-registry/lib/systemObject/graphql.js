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

          if (prop.hidden || prop.skip) {
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

      var resultString = "query ".concat(methodName, "(").concat(requestVariables.join(", "), ") { ").concat(methodName, "(input: { ").concat(inputVariables.join(", "), " }) { ").concat(fieldList, " } }");
      console.log("query ".concat(resultString));
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJ2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eSIsInByb3AiLCJyZXBlYXRlZCIsImtpbmQiLCJwcm9wTGluayIsImxvb2t1cE15c2VsZiIsInByb3BPYmplY3QiLCJyZXN1bHQiLCJwcm9wZXJ0aWVzIiwiYXR0cnMiLCJmaWVsZCIsImZpZWxkVmFyaWFibGVzIiwibmFtZSIsIlNpR3JhcGhxbCIsInN5c3RlbU9iamVjdCIsImFyZ3MiLCJtZXRob2QiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJtZXRob2ROYW1lIiwicmVwbHkiLCJsb29rdXBOYW1lIiwib3ZlcnJpZGVOYW1lIiwidHlwZU5hbWUiLCJkYXRhIiwicmVxdWlyZWQiLCJ1bmRlZmluZWQiLCJyZXF1ZXN0IiwiaW5wdXRUeXBlIiwicGFyZW50TmFtZSIsImxpbmtQcm9wIiwicmVhbFByb3AiLCJncmFwaHFsVHlwZU5hbWUiLCJhc3NvY2lhdGlvbnMiLCJhc3NvY2lhdGlvbkxpc3QiLCJwdXNoIiwiZmllbGROYW1lIiwiYXNzb2NPYmoiLCJnZXRCeUZpZWxkTmFtZSIsImFzc29jU3lzdGVtIiwicmVnaXN0cnkiLCJnZXQiLCJhc3NvY01ldGhvZCIsImZpZWxkTGlzdCIsImpvaW4iLCJzeXN0ZW1PYmplY3RNZW1vIiwiaGlkZGVuIiwic2tpcCIsImFzc29jaWF0aW9uRmllbGRMaXN0IiwicmVhbE9iaiIsInJlcXVlc3RWYXJpYWJsZXMiLCJpbnB1dFZhcmlhYmxlcyIsIm92ZXJyaWRlRmllbGRzIiwicmVzdWx0U3RyaW5nIiwiY29uc29sZSIsImxvZyIsImdxbCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBR0E7O0FBRUE7O0FBQ0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUF1QkE7QUFDQTtBQUNPLFNBQVNBLDBCQUFULENBQW9DQyxJQUFwQyxFQUF3RTtBQUFBLE1BQXZCQyxRQUF1Qix1RUFB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QTdCLGlCQUVWekIsVUFGVSx1QkFFYWdDLGNBQWMsQ0FBQ1AsSUFBZixDQUM3QixJQUQ2QixDQUZiLG1CQUlSRCxTQUpRLFNBQWxCO0FBS0FXLE1BQUFBLE9BQU8sQ0FBQ0MsR0FBUixDQUFZRixZQUFaO0FBQ0EsaUJBQU9HLHNCQUFQLHNCQUNJSCxZQURKO0FBR0QiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcm9wTWV0aG9kLCBQcm9wT2JqZWN0LCBQcm9wcyB9IGZyb20gXCIuLi9hdHRyTGlzdFwiO1xuaW1wb3J0IHsgUHJvcExpbmsgfSBmcm9tIFwiLi4vcHJvcC9saW5rXCI7XG5pbXBvcnQgeyBPYmplY3RUeXBlcyB9IGZyb20gXCIuLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5cbmltcG9ydCB7IHBhc2NhbENhc2UsIGNhbWVsQ2FzZSB9IGZyb20gXCJjaGFuZ2UtY2FzZVwiO1xuaW1wb3J0IGdxbCBmcm9tIFwiZ3JhcGhxbC10YWdcIjtcbmltcG9ydCB7IERvY3VtZW50Tm9kZSB9IGZyb20gXCJncmFwaHFsXCI7XG5pbXBvcnQgeyBBc3NvY2lhdGlvbiB9IGZyb20gXCIuL2Fzc29jaWF0aW9uc1wiO1xuXG5pbnRlcmZhY2UgUXVlcnlBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xuICBvdmVycmlkZU5hbWU/OiBzdHJpbmc7XG4gIG92ZXJyaWRlRmllbGRzPzogc3RyaW5nO1xuICBhc3NvY2lhdGlvbnM/OiB7XG4gICAgW2tleTogc3RyaW5nXTogc3RyaW5nW107XG4gIH07XG59XG5cbmludGVyZmFjZSBWYXJpYWJsZXNPYmplY3RBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xufVxuXG5pbnRlcmZhY2UgVmFsaWRhdGVSZXN1bHRBcmdzIHtcbiAgbWV0aG9kTmFtZTogc3RyaW5nO1xuICBkYXRhOiBSZWNvcmQ8c3RyaW5nLCBhbnk+O1xuICBvdmVycmlkZU5hbWU/OiBzdHJpbmc7XG59XG5cbi8vIFNlY29uZCBhcmd1bWVudCBpcyBpZiB5b3Ugd2FudCBhIHJlcGVhdGVkIGZpZWxkXG4vLyBBS0EgdGhlUG9vcmx5TmFtZWRGdW5jdGlvbigpIDopXG5leHBvcnQgZnVuY3Rpb24gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkocHJvcDogUHJvcHMsIHJlcGVhdGVkID0gZmFsc2UpOiBhbnkge1xuICBpZiAoXG4gICAgcHJvcC5raW5kKCkgPT0gXCJ0ZXh0XCIgfHxcbiAgICBwcm9wLmtpbmQoKSA9PSBcIm51bWJlclwiIHx8XG4gICAgcHJvcC5raW5kKCkgPT0gXCJjb2RlXCIgfHxcbiAgICBwcm9wLmtpbmQoKSA9PSBcImVudW1cIlxuICApIHtcbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gXCJcIjtcbiAgICB9XG4gIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJtYXBcIikge1xuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9XG4gIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICBjb25zdCBwcm9wTGluayA9IHByb3AgYXMgUHJvcExpbms7XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgLy8gVE9ETzogVGhlcmUgbWlnaHQgYmUgYSBidWcgaGVyZSwgd2hlcmUgdGhlIG5hbWUgb2YgdGhlIHByb3AgaXRzZWxmXG4gICAgICAvLyBhbmQgdGhlIG5hbWUgb2YgdGhlIGxpbmtlZCBwcm9wIGRvbid0IG1hdGNoLCBhbmQgc28gd2UgZ2V0IHRoZVxuICAgICAgLy8gd3JvbmcgZmllbGQgbmFtZSBpZiB0aGUgcHJvcCBpcyBhbiBvYmplY3QuXG4gICAgICByZXR1cm4gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkoXG4gICAgICAgIHByb3BMaW5rLmxvb2t1cE15c2VsZigpLFxuICAgICAgICByZXBlYXRlZCxcbiAgICAgICk7XG4gICAgfVxuICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIgfHwgcHJvcC5raW5kKCkgPT0gXCJtZXRob2RcIikge1xuICAgIGNvbnN0IHByb3BPYmplY3QgPSBwcm9wIGFzIFByb3BPYmplY3Q7XG4gICAgY29uc3QgcmVzdWx0OiBSZWNvcmQ8c3RyaW5nLCB1bmtub3duPiA9IHt9O1xuICAgIGZvciAoY29uc3QgZmllbGQgb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBjb25zdCBmaWVsZFZhcmlhYmxlcyA9IHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KGZpZWxkLCByZXBlYXRlZCk7XG4gICAgICByZXN1bHRbYCR7ZmllbGQubmFtZX1gXSA9IGZpZWxkVmFyaWFibGVzO1xuICAgIH1cbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH1cbiAgfVxufVxuXG5leHBvcnQgY2xhc3MgU2lHcmFwaHFsIHtcbiAgc3lzdGVtT2JqZWN0OiBPYmplY3RUeXBlcztcblxuICBjb25zdHJ1Y3RvcihzeXN0ZW1PYmplY3Q6IFNpR3JhcGhxbFtcInN5c3RlbU9iamVjdFwiXSkge1xuICAgIHRoaXMuc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0O1xuICB9XG5cbiAgdmFsaWRhdGVSZXN1bHQoYXJnczogVmFsaWRhdGVSZXN1bHRBcmdzKTogUmVjb3JkPHN0cmluZywgYW55PiB7XG4gICAgY29uc3QgbWV0aG9kID0gdGhpcy5zeXN0ZW1PYmplY3QubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgIGFyZ3MubWV0aG9kTmFtZSxcbiAgICApIGFzIFByb3BNZXRob2Q7XG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgY29uc3QgbG9va3VwTmFtZSA9XG4gICAgICBhcmdzLm92ZXJyaWRlTmFtZSB8fFxuICAgICAgYCR7Y2FtZWxDYXNlKHRoaXMuc3lzdGVtT2JqZWN0LnR5cGVOYW1lKX0ke3Bhc2NhbENhc2UoYXJncy5tZXRob2ROYW1lKX1gO1xuICAgIGNvbnN0IHJlc3VsdCA9IGFyZ3MuZGF0YS5kYXRhW2xvb2t1cE5hbWVdO1xuICAgIGZvciAoY29uc3QgZmllbGQgb2YgcmVwbHkucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKGZpZWxkLnJlcXVpcmVkICYmIHJlc3VsdFtmaWVsZC5uYW1lXSA9PSB1bmRlZmluZWQpIHtcbiAgICAgICAgdGhyb3cgYHJlc3BvbnNlIGluY29tcGxldGU7IG1pc3NpbmcgcmVxdWlyZWQgZmllbGQgJHtmaWVsZH1gO1xuICAgICAgfVxuICAgIH1cbiAgICByZXR1cm4gcmVzdWx0O1xuICB9XG5cbiAgdmFyaWFibGVzT2JqZWN0KGFyZ3M6IFZhcmlhYmxlc09iamVjdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgcmV0dXJuIHZhcmlhYmxlc09iamVjdEZvclByb3BlcnR5KHJlcXVlc3QsIHRydWUpO1xuICB9XG5cbiAgZ3JhcGhxbFR5cGVOYW1lKHByb3A6IFByb3BzLCBpbnB1dFR5cGU/OiBib29sZWFuKTogc3RyaW5nIHtcbiAgICBsZXQgcmVzdWx0ID0gXCJcIjtcbiAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIiB8fCBwcm9wLmtpbmQoKSA9PSBcImVudW1cIikge1xuICAgICAgbGV0IHJlcXVlc3QgPSBcIlwiO1xuICAgICAgaWYgKGlucHV0VHlwZSAmJiBwcm9wLmtpbmQoKSAhPSBcImVudW1cIikge1xuICAgICAgICByZXF1ZXN0ID0gXCJSZXF1ZXN0XCI7XG4gICAgICB9XG4gICAgICByZXN1bHQgPSBgJHtwYXNjYWxDYXNlKHByb3AucGFyZW50TmFtZSl9JHtwYXNjYWxDYXNlKFxuICAgICAgICBwcm9wLm5hbWUsXG4gICAgICApfSR7cmVxdWVzdH1gO1xuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJ0ZXh0XCIgfHwgcHJvcC5raW5kKCkgPT0gXCJwYXNzd29yZFwiKSB7XG4gICAgICBpZiAocHJvcC5uYW1lID09IFwiaWRcIikge1xuICAgICAgICByZXN1bHQgPSBcIklEXCI7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICByZXN1bHQgPSBcIlN0cmluZ1wiO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJudW1iZXJcIikge1xuICAgICAgcmVzdWx0ID0gXCJTdHJpbmdcIjtcbiAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICBjb25zdCBsaW5rUHJvcCA9IHByb3AgYXMgUHJvcExpbms7XG4gICAgICBjb25zdCByZWFsUHJvcCA9IGxpbmtQcm9wLmxvb2t1cE15c2VsZigpO1xuICAgICAgcmV0dXJuIHRoaXMuZ3JhcGhxbFR5cGVOYW1lKHJlYWxQcm9wLCBpbnB1dFR5cGUpO1xuICAgIH1cbiAgICBpZiAocHJvcC5yZXF1aXJlZCkge1xuICAgICAgcmV0dXJuIGAke3Jlc3VsdH0hYDtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHJlc3VsdDtcbiAgICB9XG4gIH1cblxuICBhc3NvY2lhdGlvbkZpZWxkTGlzdChcbiAgICBhc3NvY2lhdGlvbnM6IFF1ZXJ5QXJnc1tcImFzc29jaWF0aW9uc1wiXSxcbiAgICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzLFxuICApOiBzdHJpbmcge1xuICAgIGNvbnN0IGFzc29jaWF0aW9uTGlzdCA9IGFzc29jaWF0aW9ucyAmJiBhc3NvY2lhdGlvbnNbc3lzdGVtT2JqZWN0LnR5cGVOYW1lXTtcbiAgICBpZiAoYXNzb2NpYXRpb25MaXN0KSB7XG4gICAgICBjb25zdCByZXN1bHQ6IHN0cmluZ1tdID0gW107XG4gICAgICByZXN1bHQucHVzaChcImFzc29jaWF0aW9ucyB7XCIpO1xuICAgICAgZm9yIChjb25zdCBmaWVsZE5hbWUgb2YgYXNzb2NpYXRpb25MaXN0KSB7XG4gICAgICAgIGNvbnN0IGFzc29jT2JqID0gc3lzdGVtT2JqZWN0LmFzc29jaWF0aW9ucy5nZXRCeUZpZWxkTmFtZShmaWVsZE5hbWUpO1xuICAgICAgICBjb25zdCBhc3NvY1N5c3RlbSA9IHJlZ2lzdHJ5LmdldChhc3NvY09iai50eXBlTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jTWV0aG9kID0gYXNzb2NTeXN0ZW0ubWV0aG9kcy5nZXRFbnRyeShcbiAgICAgICAgICBhc3NvY09iai5tZXRob2ROYW1lLFxuICAgICAgICApIGFzIFByb3BNZXRob2Q7XG5cbiAgICAgICAgcmVzdWx0LnB1c2goYCR7ZmllbGROYW1lfSB7YCk7XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KGFzc29jTWV0aG9kLnJlcGx5LCBhc3NvY2lhdGlvbnMsIGFzc29jU3lzdGVtKSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goYH1gKTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdC5wdXNoKFwifVwiKTtcbiAgICAgIHJldHVybiByZXN1bHQuam9pbihcIiBcIik7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlwiO1xuICAgIH1cbiAgfVxuXG4gIGZpZWxkTGlzdChcbiAgICBwcm9wT2JqZWN0OiBQcm9wT2JqZWN0LFxuICAgIGFzc29jaWF0aW9uczogUXVlcnlBcmdzW1wiYXNzb2NpYXRpb25zXCJdLFxuICAgIHN5c3RlbU9iamVjdE1lbW86IE9iamVjdFR5cGVzLFxuICApOiBzdHJpbmcge1xuICAgIGxldCBzeXN0ZW1PYmplY3Q7XG4gICAgaWYgKHN5c3RlbU9iamVjdE1lbW8pIHtcbiAgICAgIHN5c3RlbU9iamVjdCA9IHN5c3RlbU9iamVjdE1lbW87XG4gICAgfSBlbHNlIHtcbiAgICAgIHN5c3RlbU9iamVjdCA9IHRoaXMuc3lzdGVtT2JqZWN0O1xuICAgIH1cbiAgICBjb25zdCByZXN1bHQ6IHN0cmluZ1tdID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHByb3BPYmplY3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgaWYgKHByb3AuaGlkZGVuIHx8IHByb3Auc2tpcCkge1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIHJlc3VsdC5wdXNoKGAke3Byb3AubmFtZX1gKTsgLy8gd2l0aG91dCBjYW1lbENhc2VcbiAgICAgIC8vIHJlc3VsdC5wdXNoKGAke2NhbWVsQ2FzZShwcm9wLm5hbWUpfWApOyAvLyB3aXRoIGNhbWVsQ2FzZVxuICAgICAgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICB0aGlzLmZpZWxkTGlzdChwcm9wIGFzIFByb3BPYmplY3QsIHVuZGVmaW5lZCwgc3lzdGVtT2JqZWN0KSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICB9XG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJtYXBcIikge1xuICAgICAgICByZXN1bHQucHVzaChcInsga2V5IHZhbHVlIH1cIik7XG4gICAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgY29uc3QgcmVhbE9iaiA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHJlYWxPYmogYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICBpZiAocmVhbE9iai5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKHRoaXMuYXNzb2NpYXRpb25GaWVsZExpc3QoYXNzb2NpYXRpb25zLCBzeXN0ZW1PYmplY3QpKTtcbiAgICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIGAke3Jlc3VsdC5qb2luKFwiIFwiKX1gO1xuICB9XG5cbiAgcXVlcnkoYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLCAvLyB3aXRob3V0IGNhbWVsQ2FzZVxuICAgICAgICAvLyBgJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3AsIHRydWUpfWAsIC8vIHdpdGggY2FtZWxDYXNlXG4gICAgICApO1xuICAgICAgaW5wdXRWYXJpYWJsZXMucHVzaChgJHtwcm9wLm5hbWV9OiAkJHtwcm9wLm5hbWV9YCk7IC8vIHdpdGhvdXQgY2FtZWxDYXNlXG4gICAgICAvLyBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke2NhbWVsQ2FzZShwcm9wLm5hbWUpfTogJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9YCk7IC8vIHdpdGggY2FtZWxDYXNlXG4gICAgfVxuXG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgbGV0IGZpZWxkTGlzdDogc3RyaW5nO1xuICAgIGlmIChhcmdzLm92ZXJyaWRlRmllbGRzKSB7XG4gICAgICBmaWVsZExpc3QgPSBgJHthcmdzLm92ZXJyaWRlRmllbGRzfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGZpZWxkTGlzdCA9IHRoaXMuZmllbGRMaXN0KHJlcGx5LCBhcmdzLmFzc29jaWF0aW9ucywgdGhpcy5zeXN0ZW1PYmplY3QpO1xuICAgIH1cblxuICAgIGNvbnN0IHJlc3VsdFN0cmluZyA9IGBxdWVyeSAke21ldGhvZE5hbWV9KCR7cmVxdWVzdFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9KSB7ICR7bWV0aG9kTmFtZX0oaW5wdXQ6IHsgJHtpbnB1dFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9IH0pIHsgJHtmaWVsZExpc3R9IH0gfWA7XG4gICAgY29uc29sZS5sb2cocmVzdWx0U3RyaW5nKVxuICAgIHJldHVybiBncWxgXG4gICAgICAke3Jlc3VsdFN0cmluZ31cbiAgICBgO1xuICB9XG5cbiAgbXV0YXRpb24oYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLCAvLyB3aXRob3V0IGNhbWVsQ2FzZVxuICAgICAgICAvLyBgJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9OiAke3RoaXMuZ3JhcGhxbFR5cGVOYW1lKHByb3AsIHRydWUpfWAsIC8vIHdpdGggY2FtZWxDYXNlXG4gICAgICApO1xuICAgICAgaW5wdXRWYXJpYWJsZXMucHVzaChgJHtwcm9wLm5hbWV9OiAkJHtwcm9wLm5hbWV9YCk7IC8vIHdpdGhvdXQgY2FtZWxDYXNlXG4gICAgICAvLyBpbnB1dFZhcmlhYmxlcy5wdXNoKGAke2NhbWVsQ2FzZShwcm9wLm5hbWUpfTogJCR7Y2FtZWxDYXNlKHByb3AubmFtZSl9YCk7IC8vIHdpdGggY2FtZWxDYXNlXG4gICAgfVxuXG4gICAgY29uc3QgcmVwbHkgPSBtZXRob2QucmVwbHk7XG4gICAgbGV0IGZpZWxkTGlzdDogc3RyaW5nO1xuICAgIGlmIChhcmdzLm92ZXJyaWRlRmllbGRzKSB7XG4gICAgICBmaWVsZExpc3QgPSBgJHthcmdzLm92ZXJyaWRlRmllbGRzfWA7XG4gICAgfSBlbHNlIHtcbiAgICAgIGZpZWxkTGlzdCA9IHRoaXMuZmllbGRMaXN0KHJlcGx5LCBhcmdzLmFzc29jaWF0aW9ucywgdGhpcy5zeXN0ZW1PYmplY3QpO1xuICAgIH1cblxuICAgIGNvbnN0IHJlc3VsdFN0cmluZyA9IGBtdXRhdGlvbiAke21ldGhvZE5hbWV9KCR7cmVxdWVzdFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9KSB7ICR7bWV0aG9kTmFtZX0oaW5wdXQ6IHsgJHtpbnB1dFZhcmlhYmxlcy5qb2luKFxuICAgICAgXCIsIFwiLFxuICAgICl9IH0pIHsgJHtmaWVsZExpc3R9IH0gfWA7XG4gICAgY29uc29sZS5sb2cocmVzdWx0U3RyaW5nKTtcbiAgICByZXR1cm4gZ3FsYFxuICAgICAgJHtyZXN1bHRTdHJpbmd9XG4gICAgYDtcbiAgfVxufVxuIl19
