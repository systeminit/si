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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zeXN0ZW1PYmplY3QvZ3JhcGhxbC50cyJdLCJuYW1lcyI6WyJ2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eSIsInByb3AiLCJyZXBlYXRlZCIsImtpbmQiLCJwcm9wTGluayIsImxvb2t1cE15c2VsZiIsInByb3BPYmplY3QiLCJyZXN1bHQiLCJwcm9wZXJ0aWVzIiwiYXR0cnMiLCJmaWVsZCIsImZpZWxkVmFyaWFibGVzIiwibmFtZSIsIlNpR3JhcGhxbCIsInN5c3RlbU9iamVjdCIsImFyZ3MiLCJtZXRob2QiLCJtZXRob2RzIiwiZ2V0RW50cnkiLCJtZXRob2ROYW1lIiwicmVwbHkiLCJsb29rdXBOYW1lIiwib3ZlcnJpZGVOYW1lIiwidHlwZU5hbWUiLCJkYXRhIiwicmVxdWlyZWQiLCJ1bmRlZmluZWQiLCJyZXF1ZXN0IiwiaW5wdXRUeXBlIiwicGFyZW50TmFtZSIsImxpbmtQcm9wIiwicmVhbFByb3AiLCJncmFwaHFsVHlwZU5hbWUiLCJhc3NvY2lhdGlvbnMiLCJhc3NvY2lhdGlvbkxpc3QiLCJwdXNoIiwiZmllbGROYW1lIiwiYXNzb2NPYmoiLCJnZXRCeUZpZWxkTmFtZSIsImFzc29jU3lzdGVtIiwicmVnaXN0cnkiLCJnZXQiLCJhc3NvY01ldGhvZCIsImZpZWxkTGlzdCIsImpvaW4iLCJzeXN0ZW1PYmplY3RNZW1vIiwiaGlkZGVuIiwic2tpcCIsImFzc29jaWF0aW9uRmllbGRMaXN0IiwicmVhbE9iaiIsInJlcXVlc3RWYXJpYWJsZXMiLCJpbnB1dFZhcmlhYmxlcyIsIm92ZXJyaWRlRmllbGRzIiwicmVzdWx0U3RyaW5nIiwiZ3FsIiwiY29uc29sZSIsImxvZyJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7Ozs7O0FBR0E7O0FBRUE7O0FBQ0E7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7QUF1QkE7QUFDTyxTQUFTQSwwQkFBVCxDQUFvQ0MsSUFBcEMsRUFBd0U7QUFBQSxNQUF2QkMsUUFBdUIsdUVBQVosS0FB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QVAsc0JBQ0lELFlBREo7QUFHRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByb3BNZXRob2QsIFByb3BPYmplY3QsIFByb3BzIH0gZnJvbSBcIi4uL2F0dHJMaXN0XCI7XG5pbXBvcnQgeyBQcm9wTGluayB9IGZyb20gXCIuLi9wcm9wL2xpbmtcIjtcbmltcG9ydCB7IE9iamVjdFR5cGVzIH0gZnJvbSBcIi4uL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcblxuaW1wb3J0IHsgcGFzY2FsQ2FzZSwgY2FtZWxDYXNlIH0gZnJvbSBcImNoYW5nZS1jYXNlXCI7XG5pbXBvcnQgZ3FsIGZyb20gXCJncmFwaHFsLXRhZ1wiO1xuaW1wb3J0IHsgRG9jdW1lbnROb2RlIH0gZnJvbSBcImdyYXBocWxcIjtcbmltcG9ydCB7IEFzc29jaWF0aW9uIH0gZnJvbSBcIi4vYXNzb2NpYXRpb25zXCI7XG5cbmludGVyZmFjZSBRdWVyeUFyZ3Mge1xuICBtZXRob2ROYW1lOiBzdHJpbmc7XG4gIG92ZXJyaWRlTmFtZT86IHN0cmluZztcbiAgb3ZlcnJpZGVGaWVsZHM/OiBzdHJpbmc7XG4gIGFzc29jaWF0aW9ucz86IHtcbiAgICBba2V5OiBzdHJpbmddOiBzdHJpbmdbXTtcbiAgfTtcbn1cblxuaW50ZXJmYWNlIFZhcmlhYmxlc09iamVjdEFyZ3Mge1xuICBtZXRob2ROYW1lOiBzdHJpbmc7XG59XG5cbmludGVyZmFjZSBWYWxpZGF0ZVJlc3VsdEFyZ3Mge1xuICBtZXRob2ROYW1lOiBzdHJpbmc7XG4gIGRhdGE6IFJlY29yZDxzdHJpbmcsIGFueT47XG4gIG92ZXJyaWRlTmFtZT86IHN0cmluZztcbn1cblxuLy8gU2Vjb25kIGFyZ3VtZW50IGlzIGlmIHlvdSB3YW50IGEgcmVwZWF0ZWQgZmllbGRcbmV4cG9ydCBmdW5jdGlvbiB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShwcm9wOiBQcm9wcywgcmVwZWF0ZWQgPSBmYWxzZSk6IGFueSB7XG4gIGlmIChcbiAgICBwcm9wLmtpbmQoKSA9PSBcInRleHRcIiB8fFxuICAgIHByb3Aua2luZCgpID09IFwibnVtYmVyXCIgfHxcbiAgICBwcm9wLmtpbmQoKSA9PSBcImNvZGVcIiB8fFxuICAgIHByb3Aua2luZCgpID09IFwiZW51bVwiXG4gICkge1xuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBcIlwiO1xuICAgIH1cbiAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm1hcFwiKSB7XG4gICAgaWYgKHByb3AucmVwZWF0ZWQgJiYgcmVwZWF0ZWQpIHtcbiAgICAgIHJldHVybiBbXTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIHt9O1xuICAgIH1cbiAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcImxpbmtcIikge1xuICAgIGNvbnN0IHByb3BMaW5rID0gcHJvcCBhcyBQcm9wTGluaztcbiAgICBpZiAocHJvcC5yZXBlYXRlZCAmJiByZXBlYXRlZCkge1xuICAgICAgcmV0dXJuIFtdO1xuICAgIH0gZWxzZSB7XG4gICAgICAvLyBUT0RPOiBUaGVyZSBtaWdodCBiZSBhIGJ1ZyBoZXJlLCB3aGVyZSB0aGUgbmFtZSBvZiB0aGUgcHJvcCBpdHNlbGZcbiAgICAgIC8vIGFuZCB0aGUgbmFtZSBvZiB0aGUgbGlua2VkIHByb3AgZG9uJ3QgbWF0Y2gsIGFuZCBzbyB3ZSBnZXQgdGhlXG4gICAgICAvLyB3cm9uZyBmaWVsZCBuYW1lIGlmIHRoZSBwcm9wIGlzIGFuIG9iamVjdC5cbiAgICAgIHJldHVybiB2YXJpYWJsZXNPYmplY3RGb3JQcm9wZXJ0eShcbiAgICAgICAgcHJvcExpbmsubG9va3VwTXlzZWxmKCksXG4gICAgICAgIHJlcGVhdGVkLFxuICAgICAgKTtcbiAgICB9XG4gIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJvYmplY3RcIiB8fCBwcm9wLmtpbmQoKSA9PSBcIm1ldGhvZFwiKSB7XG4gICAgY29uc3QgcHJvcE9iamVjdCA9IHByb3AgYXMgUHJvcE9iamVjdDtcbiAgICBjb25zdCByZXN1bHQ6IFJlY29yZDxzdHJpbmcsIHVua25vd24+ID0ge307XG4gICAgZm9yIChjb25zdCBmaWVsZCBvZiBwcm9wT2JqZWN0LnByb3BlcnRpZXMuYXR0cnMpIHtcbiAgICAgIGNvbnN0IGZpZWxkVmFyaWFibGVzID0gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkoZmllbGQsIHJlcGVhdGVkKTtcbiAgICAgIHJlc3VsdFtgJHtmaWVsZC5uYW1lfWBdID0gZmllbGRWYXJpYWJsZXM7XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcGVhdGVkICYmIHJlcGVhdGVkKSB7XG4gICAgICByZXR1cm4gW107XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiByZXN1bHQ7XG4gICAgfVxuICB9XG59XG5cbmV4cG9ydCBjbGFzcyBTaUdyYXBocWwge1xuICBzeXN0ZW1PYmplY3Q6IE9iamVjdFR5cGVzO1xuXG4gIGNvbnN0cnVjdG9yKHN5c3RlbU9iamVjdDogU2lHcmFwaHFsW1wic3lzdGVtT2JqZWN0XCJdKSB7XG4gICAgdGhpcy5zeXN0ZW1PYmplY3QgPSBzeXN0ZW1PYmplY3Q7XG4gIH1cblxuICB2YWxpZGF0ZVJlc3VsdChhcmdzOiBWYWxpZGF0ZVJlc3VsdEFyZ3MpOiBSZWNvcmQ8c3RyaW5nLCBhbnk+IHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCByZXBseSA9IG1ldGhvZC5yZXBseTtcbiAgICBjb25zdCBsb29rdXBOYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG4gICAgY29uc3QgcmVzdWx0ID0gYXJncy5kYXRhLmRhdGFbbG9va3VwTmFtZV07XG4gICAgZm9yIChjb25zdCBmaWVsZCBvZiByZXBseS5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAoZmllbGQucmVxdWlyZWQgJiYgcmVzdWx0W2ZpZWxkLm5hbWVdID09IHVuZGVmaW5lZCkge1xuICAgICAgICB0aHJvdyBgcmVzcG9uc2UgaW5jb21wbGV0ZTsgbWlzc2luZyByZXF1aXJlZCBmaWVsZCAke2ZpZWxkfWA7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHQ7XG4gIH1cblxuICB2YXJpYWJsZXNPYmplY3QoYXJnczogVmFyaWFibGVzT2JqZWN0QXJncyk6IFJlY29yZDxzdHJpbmcsIGFueT4ge1xuICAgIGNvbnN0IG1ldGhvZCA9IHRoaXMuc3lzdGVtT2JqZWN0Lm1ldGhvZHMuZ2V0RW50cnkoXG4gICAgICBhcmdzLm1ldGhvZE5hbWUsXG4gICAgKSBhcyBQcm9wTWV0aG9kO1xuICAgIGNvbnN0IHJlcXVlc3QgPSBtZXRob2QucmVxdWVzdDtcbiAgICByZXR1cm4gdmFyaWFibGVzT2JqZWN0Rm9yUHJvcGVydHkocmVxdWVzdCwgdHJ1ZSk7XG4gIH1cblxuICBncmFwaHFsVHlwZU5hbWUocHJvcDogUHJvcHMsIGlucHV0VHlwZT86IGJvb2xlYW4pOiBzdHJpbmcge1xuICAgIGxldCByZXN1bHQgPSBcIlwiO1xuICAgIGlmIChwcm9wLmtpbmQoKSA9PSBcIm9iamVjdFwiIHx8IHByb3Aua2luZCgpID09IFwiZW51bVwiKSB7XG4gICAgICBsZXQgcmVxdWVzdCA9IFwiXCI7XG4gICAgICBpZiAoaW5wdXRUeXBlICYmIHByb3Aua2luZCgpICE9IFwiZW51bVwiKSB7XG4gICAgICAgIHJlcXVlc3QgPSBcIlJlcXVlc3RcIjtcbiAgICAgIH1cbiAgICAgIHJlc3VsdCA9IGAke3Bhc2NhbENhc2UocHJvcC5wYXJlbnROYW1lKX0ke3Bhc2NhbENhc2UoXG4gICAgICAgIHByb3AubmFtZSxcbiAgICAgICl9JHtyZXF1ZXN0fWA7XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcInRleHRcIiB8fCBwcm9wLmtpbmQoKSA9PSBcInBhc3N3b3JkXCIpIHtcbiAgICAgIGlmIChwcm9wLm5hbWUgPT0gXCJpZFwiKSB7XG4gICAgICAgIHJlc3VsdCA9IFwiSURcIjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJlc3VsdCA9IFwiU3RyaW5nXCI7XG4gICAgICB9XG4gICAgfSBlbHNlIGlmIChwcm9wLmtpbmQoKSA9PSBcIm51bWJlclwiKSB7XG4gICAgICByZXN1bHQgPSBcIlN0cmluZ1wiO1xuICAgIH0gZWxzZSBpZiAocHJvcC5raW5kKCkgPT0gXCJsaW5rXCIpIHtcbiAgICAgIGNvbnN0IGxpbmtQcm9wID0gcHJvcCBhcyBQcm9wTGluaztcbiAgICAgIGNvbnN0IHJlYWxQcm9wID0gbGlua1Byb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICByZXR1cm4gdGhpcy5ncmFwaHFsVHlwZU5hbWUocmVhbFByb3AsIGlucHV0VHlwZSk7XG4gICAgfVxuICAgIGlmIChwcm9wLnJlcXVpcmVkKSB7XG4gICAgICByZXR1cm4gYCR7cmVzdWx0fSFgO1xuICAgIH0gZWxzZSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH1cbiAgfVxuXG4gIGFzc29jaWF0aW9uRmllbGRMaXN0KFxuICAgIGFzc29jaWF0aW9uczogUXVlcnlBcmdzW1wiYXNzb2NpYXRpb25zXCJdLFxuICAgIHN5c3RlbU9iamVjdDogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgY29uc3QgYXNzb2NpYXRpb25MaXN0ID0gYXNzb2NpYXRpb25zICYmIGFzc29jaWF0aW9uc1tzeXN0ZW1PYmplY3QudHlwZU5hbWVdO1xuICAgIGlmIChhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICAgIHJlc3VsdC5wdXNoKFwiYXNzb2NpYXRpb25zIHtcIik7XG4gICAgICBmb3IgKGNvbnN0IGZpZWxkTmFtZSBvZiBhc3NvY2lhdGlvbkxpc3QpIHtcbiAgICAgICAgY29uc3QgYXNzb2NPYmogPSBzeXN0ZW1PYmplY3QuYXNzb2NpYXRpb25zLmdldEJ5RmllbGROYW1lKGZpZWxkTmFtZSk7XG4gICAgICAgIGNvbnN0IGFzc29jU3lzdGVtID0gcmVnaXN0cnkuZ2V0KGFzc29jT2JqLnR5cGVOYW1lKTtcbiAgICAgICAgY29uc3QgYXNzb2NNZXRob2QgPSBhc3NvY1N5c3RlbS5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgICAgIGFzc29jT2JqLm1ldGhvZE5hbWUsXG4gICAgICAgICkgYXMgUHJvcE1ldGhvZDtcblxuICAgICAgICByZXN1bHQucHVzaChgJHtmaWVsZE5hbWV9IHtgKTtcbiAgICAgICAgcmVzdWx0LnB1c2goXG4gICAgICAgICAgdGhpcy5maWVsZExpc3QoYXNzb2NNZXRob2QucmVwbHksIGFzc29jaWF0aW9ucywgYXNzb2NTeXN0ZW0pLFxuICAgICAgICApO1xuICAgICAgICByZXN1bHQucHVzaChgfWApO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goXCJ9XCIpO1xuICAgICAgcmV0dXJuIHJlc3VsdC5qb2luKFwiIFwiKTtcbiAgICB9IGVsc2Uge1xuICAgICAgcmV0dXJuIFwiXCI7XG4gICAgfVxuICB9XG5cbiAgZmllbGRMaXN0KFxuICAgIHByb3BPYmplY3Q6IFByb3BPYmplY3QsXG4gICAgYXNzb2NpYXRpb25zOiBRdWVyeUFyZ3NbXCJhc3NvY2lhdGlvbnNcIl0sXG4gICAgc3lzdGVtT2JqZWN0TWVtbzogT2JqZWN0VHlwZXMsXG4gICk6IHN0cmluZyB7XG4gICAgbGV0IHN5c3RlbU9iamVjdDtcbiAgICBpZiAoc3lzdGVtT2JqZWN0TWVtbykge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gc3lzdGVtT2JqZWN0TWVtbztcbiAgICB9IGVsc2Uge1xuICAgICAgc3lzdGVtT2JqZWN0ID0gdGhpcy5zeXN0ZW1PYmplY3Q7XG4gICAgfVxuICAgIGNvbnN0IHJlc3VsdDogc3RyaW5nW10gPSBbXTtcbiAgICBmb3IgKGNvbnN0IHByb3Agb2YgcHJvcE9iamVjdC5wcm9wZXJ0aWVzLmF0dHJzKSB7XG4gICAgICBpZiAocHJvcC5oaWRkZW4gfHwgcHJvcC5za2lwKSB7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgcmVzdWx0LnB1c2goYCR7cHJvcC5uYW1lfWApO1xuICAgICAgaWYgKHByb3Aua2luZCgpID09IFwib2JqZWN0XCIpIHtcbiAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICByZXN1bHQucHVzaChcbiAgICAgICAgICB0aGlzLmZpZWxkTGlzdChwcm9wIGFzIFByb3BPYmplY3QsIHVuZGVmaW5lZCwgc3lzdGVtT2JqZWN0KSxcbiAgICAgICAgKTtcbiAgICAgICAgcmVzdWx0LnB1c2godGhpcy5hc3NvY2lhdGlvbkZpZWxkTGlzdChhc3NvY2lhdGlvbnMsIHN5c3RlbU9iamVjdCkpO1xuICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICB9XG4gICAgICBpZiAocHJvcC5raW5kKCkgPT0gXCJtYXBcIikge1xuICAgICAgICByZXN1bHQucHVzaChcInsga2V5IHZhbHVlIH1cIik7XG4gICAgICB9IGVsc2UgaWYgKHByb3Aua2luZCgpID09IFwibGlua1wiKSB7XG4gICAgICAgIC8vIEB0cy1pZ25vcmVcbiAgICAgICAgY29uc3QgcmVhbE9iaiA9IHByb3AubG9va3VwTXlzZWxmKCk7XG4gICAgICAgIGlmIChyZWFsT2JqLmtpbmQoKSA9PSBcIm9iamVjdFwiKSB7XG4gICAgICAgICAgcmVzdWx0LnB1c2goXCJ7XCIpO1xuICAgICAgICB9XG4gICAgICAgIHJlc3VsdC5wdXNoKFxuICAgICAgICAgIHRoaXMuZmllbGRMaXN0KHJlYWxPYmogYXMgUHJvcE9iamVjdCwgdW5kZWZpbmVkLCBzeXN0ZW1PYmplY3QpLFxuICAgICAgICApO1xuICAgICAgICBpZiAocmVhbE9iai5raW5kKCkgPT0gXCJvYmplY3RcIikge1xuICAgICAgICAgIHJlc3VsdC5wdXNoKHRoaXMuYXNzb2NpYXRpb25GaWVsZExpc3QoYXNzb2NpYXRpb25zLCBzeXN0ZW1PYmplY3QpKTtcbiAgICAgICAgICByZXN1bHQucHVzaChcIn1cIik7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIGAke3Jlc3VsdC5qb2luKFwiIFwiKX1gO1xuICB9XG5cbiAgcXVlcnkoYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLFxuICAgICAgKTtcbiAgICAgIGlucHV0VmFyaWFibGVzLnB1c2goYCR7cHJvcC5uYW1lfTogJCR7cHJvcC5uYW1lfWApO1xuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgcXVlcnkgJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIHJldHVybiBncWxgXG4gICAgICAke3Jlc3VsdFN0cmluZ31cbiAgICBgO1xuICB9XG5cbiAgbXV0YXRpb24oYXJnczogUXVlcnlBcmdzKTogRG9jdW1lbnROb2RlIHtcbiAgICBjb25zdCBtZXRob2QgPSB0aGlzLnN5c3RlbU9iamVjdC5tZXRob2RzLmdldEVudHJ5KFxuICAgICAgYXJncy5tZXRob2ROYW1lLFxuICAgICkgYXMgUHJvcE1ldGhvZDtcbiAgICBjb25zdCBtZXRob2ROYW1lID1cbiAgICAgIGFyZ3Mub3ZlcnJpZGVOYW1lIHx8XG4gICAgICBgJHtjYW1lbENhc2UodGhpcy5zeXN0ZW1PYmplY3QudHlwZU5hbWUpfSR7cGFzY2FsQ2FzZShhcmdzLm1ldGhvZE5hbWUpfWA7XG5cbiAgICBjb25zdCByZXF1ZXN0ID0gbWV0aG9kLnJlcXVlc3Q7XG4gICAgY29uc3QgcmVxdWVzdFZhcmlhYmxlcyA9IFtdO1xuICAgIGNvbnN0IGlucHV0VmFyaWFibGVzID0gW107XG4gICAgZm9yIChjb25zdCBwcm9wIG9mIHJlcXVlc3QucHJvcGVydGllcy5hdHRycykge1xuICAgICAgcmVxdWVzdFZhcmlhYmxlcy5wdXNoKFxuICAgICAgICBgJCR7cHJvcC5uYW1lfTogJHt0aGlzLmdyYXBocWxUeXBlTmFtZShwcm9wLCB0cnVlKX1gLFxuICAgICAgKTtcbiAgICAgIGlucHV0VmFyaWFibGVzLnB1c2goYCR7cHJvcC5uYW1lfTogJCR7cHJvcC5uYW1lfWApO1xuICAgIH1cblxuICAgIGNvbnN0IHJlcGx5ID0gbWV0aG9kLnJlcGx5O1xuICAgIGxldCBmaWVsZExpc3Q6IHN0cmluZztcbiAgICBpZiAoYXJncy5vdmVycmlkZUZpZWxkcykge1xuICAgICAgZmllbGRMaXN0ID0gYCR7YXJncy5vdmVycmlkZUZpZWxkc31gO1xuICAgIH0gZWxzZSB7XG4gICAgICBmaWVsZExpc3QgPSB0aGlzLmZpZWxkTGlzdChyZXBseSwgYXJncy5hc3NvY2lhdGlvbnMsIHRoaXMuc3lzdGVtT2JqZWN0KTtcbiAgICB9XG5cbiAgICBjb25zdCByZXN1bHRTdHJpbmcgPSBgbXV0YXRpb24gJHttZXRob2ROYW1lfSgke3JlcXVlc3RWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSkgeyAke21ldGhvZE5hbWV9KGlucHV0OiB7ICR7aW5wdXRWYXJpYWJsZXMuam9pbihcbiAgICAgIFwiLCBcIixcbiAgICApfSB9KSB7ICR7ZmllbGRMaXN0fSB9IH1gO1xuICAgIGNvbnNvbGUubG9nKHJlc3VsdFN0cmluZyk7XG4gICAgcmV0dXJuIGdxbGBcbiAgICAgICR7cmVzdWx0U3RyaW5nfVxuICAgIGA7XG4gIH1cbn1cbiJdfQ==