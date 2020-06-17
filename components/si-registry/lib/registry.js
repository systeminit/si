"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.registry = exports.Registry = void 0;

var _classCallCheck2 = _interopRequireDefault(require("@babel/runtime/helpers/classCallCheck"));

var _createClass2 = _interopRequireDefault(require("@babel/runtime/helpers/createClass"));

var _defineProperty2 = _interopRequireDefault(require("@babel/runtime/helpers/defineProperty"));

var _systemComponent = require("./systemComponent");

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

var Registry = /*#__PURE__*/function () {
  function Registry() {
    (0, _classCallCheck2["default"])(this, Registry);
    (0, _defineProperty2["default"])(this, "objects", void 0);
    this.objects = [];
  }

  (0, _createClass2["default"])(Registry, [{
    key: "get",
    value: function get(typeName) {
      var result = this.objects.find(function (v) {
        return v.typeName == typeName;
      });

      if (result) {
        return result;
      } else {
        throw "Cannot get object named ".concat(typeName, " in the registry");
      }
    }
  }, {
    key: "serviceNames",
    value: function serviceNames() {
      var names = new Set();

      var _iterator = _createForOfIteratorHelper(this.objects),
          _step;

      try {
        for (_iterator.s(); !(_step = _iterator.n()).done;) {
          var object = _step.value;

          if (object.serviceName) {
            names.add(object.serviceName);
          }
        }
      } catch (err) {
        _iterator.e(err);
      } finally {
        _iterator.f();
      }

      var arrayNames = [];

      var _iterator2 = _createForOfIteratorHelper(names.values()),
          _step2;

      try {
        for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
          var name = _step2.value;
          arrayNames.push("".concat(name));
        }
      } catch (err) {
        _iterator2.e(err);
      } finally {
        _iterator2.f();
      }

      return arrayNames;
    }
  }, {
    key: "getObjectsForServiceName",
    value: function getObjectsForServiceName(serviceName) {
      var results = [];

      var _iterator3 = _createForOfIteratorHelper(this.objects),
          _step3;

      try {
        for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
          var object = _step3.value;

          if (object.serviceName == serviceName) {
            results.push(object);
          }
        }
      } catch (err) {
        _iterator3.e(err);
      } finally {
        _iterator3.f();
      }

      return results;
    } // Find a property!

  }, {
    key: "lookupProp",
    value: function lookupProp(lookup) {
      var foundObject = this.objects.find(function (c) {
        return c.typeName == lookup.typeName;
      });

      if (!foundObject) {
        throw "Cannot find object: ".concat(foundObject);
      }

      if (!lookup.names) {
        return foundObject.rootProp;
      }

      var firstName = lookup.names[0];
      var returnProp = foundObject.fields.getEntry(firstName);

      if (!returnProp) {
        throw "Cannot find prop on object ".concat(foundObject.typeName, ": ").concat(firstName);
      }

      if (returnProp.kind() != "object" && lookup.names.length > 1) {
        throw "You asked for sub-properties of a non-object type on ".concat(foundObject.typeName, " property ").concat(firstName);
      }

      for (var i = 1; i < lookup.names.length; i++) {
        var lookupName = lookup.names[i]; // @ts-ignore

        var lookupResult = returnProp["properties"].getEntry(lookupName);

        if (!lookupResult) {
          throw "Cannot find prop \"".concat(lookupName, "\" on ").concat(returnProp.name);
        }

        if (i != lookup.names.length - 1 && lookupResult.kind() != "object") {
          console.log({
            i: i,
            length: lookup.names.length,
            lookupName: lookupName,
            lookupResult: lookupResult
          });
          throw "Cannot look up a sub-property of a non object Prop: ".concat(foundObject.typeName, " property ").concat(lookupName, " is ").concat(lookupResult.kind());
        }

        returnProp = lookupResult;
      }

      return returnProp;
    } // These are "basic" objects - they don't have any extra behavior or
    // automatic fields. They just store the fields you give them.

  }, {
    key: "base",
    value: function base(constructorArgs) {
      var compy = new _systemComponent.BaseObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    } // These are "system" objects - they have what is needed to be an object
    // inside our system. They come with things like types, IDs, tenancy,
    // etc.

  }, {
    key: "system",
    value: function system(constructorArgs) {
      var compy = new _systemComponent.SystemObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }, {
    key: "component",
    value: function component(constructorArgs) {
      var compy = new _systemComponent.ComponentObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }, {
    key: "entity",
    value: function entity(constructorArgs) {
      var compy = new _systemComponent.EntityObject(constructorArgs);
      this.objects.push(compy);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }, {
    key: "componentAndEntity",
    value: function componentAndEntity(constructorArgs) {
      var compy = new _systemComponent.ComponentAndEntityObject(constructorArgs);
      this.objects.push(compy.component);
      this.objects.push(compy.entity);
      this.objects.push(compy.entityEvent);

      if (constructorArgs.options) {
        constructorArgs.options(compy);
      }

      return compy;
    }
  }]);
  return Registry;
}();

exports.Registry = Registry;
var registry = new Registry();
exports.registry = registry;
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9yZWdpc3RyeS50cyJdLCJuYW1lcyI6WyJSZWdpc3RyeSIsIm9iamVjdHMiLCJ0eXBlTmFtZSIsInJlc3VsdCIsImZpbmQiLCJ2IiwibmFtZXMiLCJTZXQiLCJvYmplY3QiLCJzZXJ2aWNlTmFtZSIsImFkZCIsImFycmF5TmFtZXMiLCJ2YWx1ZXMiLCJuYW1lIiwicHVzaCIsInJlc3VsdHMiLCJsb29rdXAiLCJmb3VuZE9iamVjdCIsImMiLCJyb290UHJvcCIsImZpcnN0TmFtZSIsInJldHVyblByb3AiLCJmaWVsZHMiLCJnZXRFbnRyeSIsImtpbmQiLCJsZW5ndGgiLCJpIiwibG9va3VwTmFtZSIsImxvb2t1cFJlc3VsdCIsImNvbnNvbGUiLCJsb2ciLCJjb25zdHJ1Y3RvckFyZ3MiLCJjb21weSIsIkJhc2VPYmplY3QiLCJvcHRpb25zIiwiU3lzdGVtT2JqZWN0IiwiQ29tcG9uZW50T2JqZWN0IiwiRW50aXR5T2JqZWN0IiwiQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IiwiY29tcG9uZW50IiwiZW50aXR5IiwiZW50aXR5RXZlbnQiLCJyZWdpc3RyeSJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7Ozs7Ozs7O0lBaUJhQSxRO0FBR1gsc0JBQWM7QUFBQTtBQUFBO0FBQ1osU0FBS0MsT0FBTCxHQUFlLEVBQWY7QUFDRDs7Ozt3QkFFR0MsUSxFQUErQjtBQUNqQyxVQUFNQyxNQUFNLEdBQUcsS0FBS0YsT0FBTCxDQUFhRyxJQUFiLENBQWtCLFVBQUFDLENBQUM7QUFBQSxlQUFJQSxDQUFDLENBQUNILFFBQUYsSUFBY0EsUUFBbEI7QUFBQSxPQUFuQixDQUFmOztBQUNBLFVBQUlDLE1BQUosRUFBWTtBQUNWLGVBQU9BLE1BQVA7QUFDRCxPQUZELE1BRU87QUFDTCxnREFBaUNELFFBQWpDO0FBQ0Q7QUFDRjs7O21DQUV3QjtBQUN2QixVQUFNSSxLQUFLLEdBQUcsSUFBSUMsR0FBSixFQUFkOztBQUR1QixpREFFRixLQUFLTixPQUZI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SyxHQUFHLElBQUlLLDZCQUFKLENBQWlCTixlQUFqQixDQUFkO0FBQ0EsV0FBSzlCLE9BQUwsQ0FBYWEsSUFBYixDQUFrQmtCLEtBQWxCOztBQUNBLFVBQUlELGVBQWUsQ0FBQ0csT0FBcEIsRUFBNkI7QUFDM0JILFFBQUFBLGVBQWUsQ0FBQ0csT0FBaEIsQ0FBd0JGLEtBQXhCO0FBQ0Q7O0FBQ0QsYUFBT0EsS0FBUDtBQUNEOzs7dUNBR0NELGUsRUFDMEI7QUFDMUIsVUFBTUMsS0FBSyxHQUFHLElBQUlNLHlDQUFKLENBQTZCUCxlQUE3QixDQUFkO0FBQ0EsV0FBSzlCLE9BQUwsQ0FBYWEsSUFBYixDQUFrQmtCLEtBQUssQ0FBQ08sU0FBeEI7QUFDQSxXQUFLdEMsT0FBTCxDQUFhYSxJQUFiLENBQWtCa0IsS0FBSyxDQUFDUSxNQUF4QjtBQUNBLFdBQUt2QyxPQUFMLENBQWFhLElBQWIsQ0FBa0JrQixLQUFLLENBQUNTLFdBQXhCOztBQUNBLFVBQUlWLGVBQWUsQ0FBQ0csT0FBcEIsRUFBNkI7QUFDM0JILFFBQUFBLGVBQWUsQ0FBQ0csT0FBaEIsQ0FBd0JGLEtBQXhCO0FBQ0Q7O0FBQ0QsYUFBT0EsS0FBUDtBQUNEOzs7Ozs7QUFHSSxJQUFNVSxRQUFRLEdBQUcsSUFBSTFDLFFBQUosRUFBakIiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQge1xuICBPYmplY3RUeXBlcyxcbiAgQmFzZU9iamVjdENvbnN0cnVjdG9yLFxuICBTeXN0ZW1PYmplY3QsXG4gIEJhc2VPYmplY3QsXG4gIENvbXBvbmVudE9iamVjdCxcbiAgRW50aXR5T2JqZWN0LFxuICBDb21wb25lbnRBbmRFbnRpdHlPYmplY3QsXG4gIENvbXBvbmVudEFuZEVudGl0eU9iamVjdENvbnN0cnVjdG9yLFxufSBmcm9tIFwiLi9zeXN0ZW1Db21wb25lbnRcIjtcbmltcG9ydCB7IFByb3BzIH0gZnJvbSBcIi4vYXR0ckxpc3RcIjtcblxuZXhwb3J0IGludGVyZmFjZSBQcm9wTG9va3VwIHtcbiAgdHlwZU5hbWU6IHN0cmluZztcbiAgbmFtZXM/OiBzdHJpbmdbXTtcbn1cblxuZXhwb3J0IGNsYXNzIFJlZ2lzdHJ5IHtcbiAgb2JqZWN0czogT2JqZWN0VHlwZXNbXTtcblxuICBjb25zdHJ1Y3RvcigpIHtcbiAgICB0aGlzLm9iamVjdHMgPSBbXTtcbiAgfVxuXG4gIGdldCh0eXBlTmFtZTogc3RyaW5nKTogT2JqZWN0VHlwZXMge1xuICAgIGNvbnN0IHJlc3VsdCA9IHRoaXMub2JqZWN0cy5maW5kKHYgPT4gdi50eXBlTmFtZSA9PSB0eXBlTmFtZSk7XG4gICAgaWYgKHJlc3VsdCkge1xuICAgICAgcmV0dXJuIHJlc3VsdDtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgYENhbm5vdCBnZXQgb2JqZWN0IG5hbWVkICR7dHlwZU5hbWV9IGluIHRoZSByZWdpc3RyeWA7XG4gICAgfVxuICB9XG5cbiAgc2VydmljZU5hbWVzKCk6IHN0cmluZ1tdIHtcbiAgICBjb25zdCBuYW1lcyA9IG5ldyBTZXQoKTtcbiAgICBmb3IgKGNvbnN0IG9iamVjdCBvZiB0aGlzLm9iamVjdHMpIHtcbiAgICAgIGlmIChvYmplY3Quc2VydmljZU5hbWUpIHtcbiAgICAgICAgbmFtZXMuYWRkKG9iamVjdC5zZXJ2aWNlTmFtZSk7XG4gICAgICB9XG4gICAgfVxuICAgIGNvbnN0IGFycmF5TmFtZXMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IG5hbWUgb2YgbmFtZXMudmFsdWVzKCkpIHtcbiAgICAgIGFycmF5TmFtZXMucHVzaChgJHtuYW1lfWApO1xuICAgIH1cbiAgICByZXR1cm4gYXJyYXlOYW1lcztcbiAgfVxuXG4gIGdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZTogc3RyaW5nKTogT2JqZWN0VHlwZXNbXSB7XG4gICAgY29uc3QgcmVzdWx0cyA9IFtdO1xuICAgIGZvciAoY29uc3Qgb2JqZWN0IG9mIHRoaXMub2JqZWN0cykge1xuICAgICAgaWYgKG9iamVjdC5zZXJ2aWNlTmFtZSA9PSBzZXJ2aWNlTmFtZSkge1xuICAgICAgICByZXN1bHRzLnB1c2gob2JqZWN0KTtcbiAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIHJlc3VsdHM7XG4gIH1cblxuICAvLyBGaW5kIGEgcHJvcGVydHkhXG4gIGxvb2t1cFByb3AobG9va3VwOiBQcm9wTG9va3VwKTogUHJvcHMge1xuICAgIGNvbnN0IGZvdW5kT2JqZWN0ID0gdGhpcy5vYmplY3RzLmZpbmQoYyA9PiBjLnR5cGVOYW1lID09IGxvb2t1cC50eXBlTmFtZSk7XG4gICAgaWYgKCFmb3VuZE9iamVjdCkge1xuICAgICAgdGhyb3cgYENhbm5vdCBmaW5kIG9iamVjdDogJHtmb3VuZE9iamVjdH1gO1xuICAgIH1cbiAgICBpZiAoIWxvb2t1cC5uYW1lcykge1xuICAgICAgcmV0dXJuIGZvdW5kT2JqZWN0LnJvb3RQcm9wO1xuICAgIH1cbiAgICBjb25zdCBmaXJzdE5hbWUgPSBsb29rdXAubmFtZXNbMF07XG4gICAgbGV0IHJldHVyblByb3AgPSBmb3VuZE9iamVjdC5maWVsZHMuZ2V0RW50cnkoZmlyc3ROYW1lKTtcbiAgICBpZiAoIXJldHVyblByb3ApIHtcbiAgICAgIHRocm93IGBDYW5ub3QgZmluZCBwcm9wIG9uIG9iamVjdCAke2ZvdW5kT2JqZWN0LnR5cGVOYW1lfTogJHtmaXJzdE5hbWV9YDtcbiAgICB9XG4gICAgaWYgKHJldHVyblByb3Aua2luZCgpICE9IFwib2JqZWN0XCIgJiYgbG9va3VwLm5hbWVzLmxlbmd0aCA+IDEpIHtcbiAgICAgIHRocm93IGBZb3UgYXNrZWQgZm9yIHN1Yi1wcm9wZXJ0aWVzIG9mIGEgbm9uLW9iamVjdCB0eXBlIG9uICR7Zm91bmRPYmplY3QudHlwZU5hbWV9IHByb3BlcnR5ICR7Zmlyc3ROYW1lfWA7XG4gICAgfVxuICAgIGZvciAobGV0IGkgPSAxOyBpIDwgbG9va3VwLm5hbWVzLmxlbmd0aDsgaSsrKSB7XG4gICAgICBjb25zdCBsb29rdXBOYW1lID0gbG9va3VwLm5hbWVzW2ldO1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgY29uc3QgbG9va3VwUmVzdWx0ID0gcmV0dXJuUHJvcFtcInByb3BlcnRpZXNcIl0uZ2V0RW50cnkobG9va3VwTmFtZSk7XG4gICAgICBpZiAoIWxvb2t1cFJlc3VsdCkge1xuICAgICAgICB0aHJvdyBgQ2Fubm90IGZpbmQgcHJvcCBcIiR7bG9va3VwTmFtZX1cIiBvbiAke3JldHVyblByb3AubmFtZX1gO1xuICAgICAgfVxuXG4gICAgICBpZiAoaSAhPSBsb29rdXAubmFtZXMubGVuZ3RoIC0gMSAmJiBsb29rdXBSZXN1bHQua2luZCgpICE9IFwib2JqZWN0XCIpIHtcbiAgICAgICAgY29uc29sZS5sb2coe1xuICAgICAgICAgIGksXG4gICAgICAgICAgbGVuZ3RoOiBsb29rdXAubmFtZXMubGVuZ3RoLFxuICAgICAgICAgIGxvb2t1cE5hbWUsXG4gICAgICAgICAgbG9va3VwUmVzdWx0LFxuICAgICAgICB9KTtcbiAgICAgICAgdGhyb3cgYENhbm5vdCBsb29rIHVwIGEgc3ViLXByb3BlcnR5IG9mIGEgbm9uIG9iamVjdCBQcm9wOiAke1xuICAgICAgICAgIGZvdW5kT2JqZWN0LnR5cGVOYW1lXG4gICAgICAgIH0gcHJvcGVydHkgJHtsb29rdXBOYW1lfSBpcyAke2xvb2t1cFJlc3VsdC5raW5kKCl9YDtcbiAgICAgIH1cblxuICAgICAgcmV0dXJuUHJvcCA9IGxvb2t1cFJlc3VsdDtcbiAgICB9XG4gICAgcmV0dXJuIHJldHVyblByb3A7XG4gIH1cblxuICAvLyBUaGVzZSBhcmUgXCJiYXNpY1wiIG9iamVjdHMgLSB0aGV5IGRvbid0IGhhdmUgYW55IGV4dHJhIGJlaGF2aW9yIG9yXG4gIC8vIGF1dG9tYXRpYyBmaWVsZHMuIFRoZXkganVzdCBzdG9yZSB0aGUgZmllbGRzIHlvdSBnaXZlIHRoZW0uXG4gIGJhc2UoY29uc3RydWN0b3JBcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpOiBCYXNlT2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBCYXNlT2JqZWN0KGNvbnN0cnVjdG9yQXJncyk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cblxuICAvLyBUaGVzZSBhcmUgXCJzeXN0ZW1cIiBvYmplY3RzIC0gdGhleSBoYXZlIHdoYXQgaXMgbmVlZGVkIHRvIGJlIGFuIG9iamVjdFxuICAvLyBpbnNpZGUgb3VyIHN5c3RlbS4gVGhleSBjb21lIHdpdGggdGhpbmdzIGxpa2UgdHlwZXMsIElEcywgdGVuYW5jeSxcbiAgLy8gZXRjLlxuICBzeXN0ZW0oY29uc3RydWN0b3JBcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpOiBTeXN0ZW1PYmplY3Qge1xuICAgIGNvbnN0IGNvbXB5ID0gbmV3IFN5c3RlbU9iamVjdChjb25zdHJ1Y3RvckFyZ3MpO1xuICAgIHRoaXMub2JqZWN0cy5wdXNoKGNvbXB5KTtcbiAgICBpZiAoY29uc3RydWN0b3JBcmdzLm9wdGlvbnMpIHtcbiAgICAgIGNvbnN0cnVjdG9yQXJncy5vcHRpb25zKGNvbXB5KTtcbiAgICB9XG4gICAgcmV0dXJuIGNvbXB5O1xuICB9XG5cbiAgY29tcG9uZW50KGNvbnN0cnVjdG9yQXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKTogQ29tcG9uZW50T2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBDb21wb25lbnRPYmplY3QoY29uc3RydWN0b3JBcmdzKTtcbiAgICB0aGlzLm9iamVjdHMucHVzaChjb21weSk7XG4gICAgaWYgKGNvbnN0cnVjdG9yQXJncy5vcHRpb25zKSB7XG4gICAgICBjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucyhjb21weSk7XG4gICAgfVxuICAgIHJldHVybiBjb21weTtcbiAgfVxuXG4gIGVudGl0eShjb25zdHJ1Y3RvckFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcik6IEVudGl0eU9iamVjdCB7XG4gICAgY29uc3QgY29tcHkgPSBuZXcgRW50aXR5T2JqZWN0KGNvbnN0cnVjdG9yQXJncyk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cblxuICBjb21wb25lbnRBbmRFbnRpdHkoXG4gICAgY29uc3RydWN0b3JBcmdzOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3RDb25zdHJ1Y3RvcixcbiAgKTogQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBDb21wb25lbnRBbmRFbnRpdHlPYmplY3QoY29uc3RydWN0b3JBcmdzKTtcbiAgICB0aGlzLm9iamVjdHMucHVzaChjb21weS5jb21wb25lbnQpO1xuICAgIHRoaXMub2JqZWN0cy5wdXNoKGNvbXB5LmVudGl0eSk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkuZW50aXR5RXZlbnQpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cbn1cblxuZXhwb3J0IGNvbnN0IHJlZ2lzdHJ5ID0gbmV3IFJlZ2lzdHJ5KCk7XG4iXX0=