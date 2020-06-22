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
        throw new Error("Cannot get object named ".concat(typeName, " in the registry"));
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
        throw new Error("Cannot find object: ".concat(foundObject));
      }

      if (!lookup.names) {
        return foundObject.rootProp;
      }

      var firstName = lookup.names[0];
      var returnProp = foundObject.fields.getEntry(firstName);

      if (!returnProp) {
        throw new Error("Cannot find prop on object ".concat(foundObject.typeName, ": ").concat(firstName));
      }

      if (returnProp.kind() != "object" && lookup.names.length > 1) {
        throw new Error("You asked for sub-properties of a non-object type on ".concat(foundObject.typeName, " property ").concat(firstName));
      }

      for (var i = 1; i < lookup.names.length; i++) {
        var lookupName = lookup.names[i]; // @ts-ignore

        var lookupResult = returnProp["properties"].getEntry(lookupName);

        if (!lookupResult) {
          throw new Error("Cannot find prop \"".concat(lookupName, "\" on ").concat(returnProp.name));
        }

        if (i != lookup.names.length - 1 && lookupResult.kind() != "object") {
          console.log({
            i: i,
            length: lookup.names.length,
            lookupName: lookupName,
            lookupResult: lookupResult
          });
          throw new Error("Cannot look up a sub-property of a non object Prop: ".concat(foundObject.typeName, " property ").concat(lookupName, " is ").concat(lookupResult.kind()));
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uL3NyYy9yZWdpc3RyeS50cyJdLCJuYW1lcyI6WyJSZWdpc3RyeSIsIm9iamVjdHMiLCJ0eXBlTmFtZSIsInJlc3VsdCIsImZpbmQiLCJ2IiwiRXJyb3IiLCJuYW1lcyIsIlNldCIsIm9iamVjdCIsInNlcnZpY2VOYW1lIiwiYWRkIiwiYXJyYXlOYW1lcyIsInZhbHVlcyIsIm5hbWUiLCJwdXNoIiwicmVzdWx0cyIsImxvb2t1cCIsImZvdW5kT2JqZWN0IiwiYyIsInJvb3RQcm9wIiwiZmlyc3ROYW1lIiwicmV0dXJuUHJvcCIsImZpZWxkcyIsImdldEVudHJ5Iiwia2luZCIsImxlbmd0aCIsImkiLCJsb29rdXBOYW1lIiwibG9va3VwUmVzdWx0IiwiY29uc29sZSIsImxvZyIsImNvbnN0cnVjdG9yQXJncyIsImNvbXB5IiwiQmFzZU9iamVjdCIsIm9wdGlvbnMiLCJTeXN0ZW1PYmplY3QiLCJDb21wb25lbnRPYmplY3QiLCJFbnRpdHlPYmplY3QiLCJDb21wb25lbnRBbmRFbnRpdHlPYmplY3QiLCJjb21wb25lbnQiLCJlbnRpdHkiLCJlbnRpdHlFdmVudCIsInJlZ2lzdHJ5Il0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7QUFBQTs7Ozs7Ozs7SUFpQmFBLFE7QUFHWCxzQkFBYztBQUFBO0FBQUE7QUFDWixTQUFLQyxPQUFMLEdBQWUsRUFBZjtBQUNEOzs7O3dCQUVHQyxRLEVBQStCO0FBQ2pDLFVBQU1DLE1BQU0sR0FBRyxLQUFLRixPQUFMLENBQWFHLElBQWIsQ0FBa0IsVUFBQUMsQ0FBQztBQUFBLGVBQUlBLENBQUMsQ0FBQ0gsUUFBRixJQUFjQSxRQUFsQjtBQUFBLE9BQW5CLENBQWY7O0FBQ0EsVUFBSUMsTUFBSixFQUFZO0FBQ1YsZUFBT0EsTUFBUDtBQUNELE9BRkQsTUFFTztBQUNMLGNBQU0sSUFBSUcsS0FBSixtQ0FBcUNKLFFBQXJDLHNCQUFOO0FBQ0Q7QUFDRjs7O21DQUV3QjtBQUN2QixVQUFNSyxLQUFLLEdBQUcsSUFBSUMsR0FB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a0IsS0FBbEI7O0FBQ0EsVUFBSUQsZUFBZSxDQUFDRyxPQUFwQixFQUE2QjtBQUMzQkgsUUFBQUEsZUFBZSxDQUFDRyxPQUFoQixDQUF3QkYsS0FBeEI7QUFDRDs7QUFDRCxhQUFPQSxLQUFQO0FBQ0Q7OzsyQkFFTUQsZSxFQUFzRDtBQUMzRCxVQUFNQyxLQUFLLEdBQUcsSUFBSUssNkJBQUosQ0FBaUJOLGVBQWpCLENBQWQ7QUFDQSxXQUFLL0IsT0FBTCxDQUFhYyxJQUFiLENBQWtCa0IsS0FBbEI7O0FBQ0EsVUFBSUQsZUFBZSxDQUFDRyxPQUFwQixFQUE2QjtBQUMzQkgsUUFBQUEsZUFBZSxDQUFDRyxPQUFoQixDQUF3QkYsS0FBeEI7QUFDRDs7QUFDRCxhQUFPQSxLQUFQO0FBQ0Q7Ozt1Q0FHQ0QsZSxFQUMwQjtBQUMxQixVQUFNQyxLQUFLLEdBQUcsSUFBSU0seUNBQUosQ0FBNkJQLGVBQTdCLENBQWQ7QUFDQSxXQUFLL0IsT0FBTCxDQUFhYyxJQUFiLENBQWtCa0IsS0FBSyxDQUFDTyxTQUF4QjtBQUNBLFdBQUt2QyxPQUFMLENBQWFjLElBQWIsQ0FBa0JrQixLQUFLLENBQUNRLE1BQXhCO0FBQ0EsV0FBS3hDLE9BQUwsQ0FBYWMsSUFBYixDQUFrQmtCLEtBQUssQ0FBQ1MsV0FBeEI7O0FBQ0EsVUFBSVYsZUFBZSxDQUFDRyxPQUFwQixFQUE2QjtBQUMzQkgsUUFBQUEsZUFBZSxDQUFDRyxPQUFoQixDQUF3QkYsS0FBeEI7QUFDRDs7QUFDRCxhQUFPQSxLQUFQO0FBQ0Q7Ozs7OztBQUdJLElBQU1VLFFBQVEsR0FBRyxJQUFJM0MsUUFBSixFQUFqQiIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7XG4gIE9iamVjdFR5cGVzLFxuICBCYXNlT2JqZWN0Q29uc3RydWN0b3IsXG4gIFN5c3RlbU9iamVjdCxcbiAgQmFzZU9iamVjdCxcbiAgQ29tcG9uZW50T2JqZWN0LFxuICBFbnRpdHlPYmplY3QsXG4gIENvbXBvbmVudEFuZEVudGl0eU9iamVjdCxcbiAgQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0Q29uc3RydWN0b3IsXG59IGZyb20gXCIuL3N5c3RlbUNvbXBvbmVudFwiO1xuaW1wb3J0IHsgUHJvcHMgfSBmcm9tIFwiLi9hdHRyTGlzdFwiO1xuXG5leHBvcnQgaW50ZXJmYWNlIFByb3BMb29rdXAge1xuICB0eXBlTmFtZTogc3RyaW5nO1xuICBuYW1lcz86IHN0cmluZ1tdO1xufVxuXG5leHBvcnQgY2xhc3MgUmVnaXN0cnkge1xuICBvYmplY3RzOiBPYmplY3RUeXBlc1tdO1xuXG4gIGNvbnN0cnVjdG9yKCkge1xuICAgIHRoaXMub2JqZWN0cyA9IFtdO1xuICB9XG5cbiAgZ2V0KHR5cGVOYW1lOiBzdHJpbmcpOiBPYmplY3RUeXBlcyB7XG4gICAgY29uc3QgcmVzdWx0ID0gdGhpcy5vYmplY3RzLmZpbmQodiA9PiB2LnR5cGVOYW1lID09IHR5cGVOYW1lKTtcbiAgICBpZiAocmVzdWx0KSB7XG4gICAgICByZXR1cm4gcmVzdWx0O1xuICAgIH0gZWxzZSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoYENhbm5vdCBnZXQgb2JqZWN0IG5hbWVkICR7dHlwZU5hbWV9IGluIHRoZSByZWdpc3RyeWApO1xuICAgIH1cbiAgfVxuXG4gIHNlcnZpY2VOYW1lcygpOiBzdHJpbmdbXSB7XG4gICAgY29uc3QgbmFtZXMgPSBuZXcgU2V0KCk7XG4gICAgZm9yIChjb25zdCBvYmplY3Qgb2YgdGhpcy5vYmplY3RzKSB7XG4gICAgICBpZiAob2JqZWN0LnNlcnZpY2VOYW1lKSB7XG4gICAgICAgIG5hbWVzLmFkZChvYmplY3Quc2VydmljZU5hbWUpO1xuICAgICAgfVxuICAgIH1cbiAgICBjb25zdCBhcnJheU5hbWVzID0gW107XG4gICAgZm9yIChjb25zdCBuYW1lIG9mIG5hbWVzLnZhbHVlcygpKSB7XG4gICAgICBhcnJheU5hbWVzLnB1c2goYCR7bmFtZX1gKTtcbiAgICB9XG4gICAgcmV0dXJuIGFycmF5TmFtZXM7XG4gIH1cblxuICBnZXRPYmplY3RzRm9yU2VydmljZU5hbWUoc2VydmljZU5hbWU6IHN0cmluZyk6IE9iamVjdFR5cGVzW10ge1xuICAgIGNvbnN0IHJlc3VsdHMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IG9iamVjdCBvZiB0aGlzLm9iamVjdHMpIHtcbiAgICAgIGlmIChvYmplY3Quc2VydmljZU5hbWUgPT0gc2VydmljZU5hbWUpIHtcbiAgICAgICAgcmVzdWx0cy5wdXNoKG9iamVjdCk7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiByZXN1bHRzO1xuICB9XG5cbiAgLy8gRmluZCBhIHByb3BlcnR5IVxuICBsb29rdXBQcm9wKGxvb2t1cDogUHJvcExvb2t1cCk6IFByb3BzIHtcbiAgICBjb25zdCBmb3VuZE9iamVjdCA9IHRoaXMub2JqZWN0cy5maW5kKGMgPT4gYy50eXBlTmFtZSA9PSBsb29rdXAudHlwZU5hbWUpO1xuICAgIGlmICghZm91bmRPYmplY3QpIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihgQ2Fubm90IGZpbmQgb2JqZWN0OiAke2ZvdW5kT2JqZWN0fWApO1xuICAgIH1cbiAgICBpZiAoIWxvb2t1cC5uYW1lcykge1xuICAgICAgcmV0dXJuIGZvdW5kT2JqZWN0LnJvb3RQcm9wO1xuICAgIH1cbiAgICBjb25zdCBmaXJzdE5hbWUgPSBsb29rdXAubmFtZXNbMF07XG4gICAgbGV0IHJldHVyblByb3AgPSBmb3VuZE9iamVjdC5maWVsZHMuZ2V0RW50cnkoZmlyc3ROYW1lKTtcbiAgICBpZiAoIXJldHVyblByb3ApIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgYENhbm5vdCBmaW5kIHByb3Agb24gb2JqZWN0ICR7Zm91bmRPYmplY3QudHlwZU5hbWV9OiAke2ZpcnN0TmFtZX1gLFxuICAgICAgKTtcbiAgICB9XG4gICAgaWYgKHJldHVyblByb3Aua2luZCgpICE9IFwib2JqZWN0XCIgJiYgbG9va3VwLm5hbWVzLmxlbmd0aCA+IDEpIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgYFlvdSBhc2tlZCBmb3Igc3ViLXByb3BlcnRpZXMgb2YgYSBub24tb2JqZWN0IHR5cGUgb24gJHtmb3VuZE9iamVjdC50eXBlTmFtZX0gcHJvcGVydHkgJHtmaXJzdE5hbWV9YCxcbiAgICAgICk7XG4gICAgfVxuICAgIGZvciAobGV0IGkgPSAxOyBpIDwgbG9va3VwLm5hbWVzLmxlbmd0aDsgaSsrKSB7XG4gICAgICBjb25zdCBsb29rdXBOYW1lID0gbG9va3VwLm5hbWVzW2ldO1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgY29uc3QgbG9va3VwUmVzdWx0ID0gcmV0dXJuUHJvcFtcInByb3BlcnRpZXNcIl0uZ2V0RW50cnkobG9va3VwTmFtZSk7XG4gICAgICBpZiAoIWxvb2t1cFJlc3VsdCkge1xuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXG4gICAgICAgICAgYENhbm5vdCBmaW5kIHByb3AgXCIke2xvb2t1cE5hbWV9XCIgb24gJHtyZXR1cm5Qcm9wLm5hbWV9YCxcbiAgICAgICAgKTtcbiAgICAgIH1cblxuICAgICAgaWYgKGkgIT0gbG9va3VwLm5hbWVzLmxlbmd0aCAtIDEgJiYgbG9va3VwUmVzdWx0LmtpbmQoKSAhPSBcIm9iamVjdFwiKSB7XG4gICAgICAgIGNvbnNvbGUubG9nKHtcbiAgICAgICAgICBpLFxuICAgICAgICAgIGxlbmd0aDogbG9va3VwLm5hbWVzLmxlbmd0aCxcbiAgICAgICAgICBsb29rdXBOYW1lLFxuICAgICAgICAgIGxvb2t1cFJlc3VsdCxcbiAgICAgICAgfSk7XG4gICAgICAgIHRocm93IG5ldyBFcnJvcihcbiAgICAgICAgICBgQ2Fubm90IGxvb2sgdXAgYSBzdWItcHJvcGVydHkgb2YgYSBub24gb2JqZWN0IFByb3A6ICR7XG4gICAgICAgICAgICBmb3VuZE9iamVjdC50eXBlTmFtZVxuICAgICAgICAgIH0gcHJvcGVydHkgJHtsb29rdXBOYW1lfSBpcyAke2xvb2t1cFJlc3VsdC5raW5kKCl9YCxcbiAgICAgICAgKTtcbiAgICAgIH1cblxuICAgICAgcmV0dXJuUHJvcCA9IGxvb2t1cFJlc3VsdDtcbiAgICB9XG4gICAgcmV0dXJuIHJldHVyblByb3A7XG4gIH1cblxuICAvLyBUaGVzZSBhcmUgXCJiYXNpY1wiIG9iamVjdHMgLSB0aGV5IGRvbid0IGhhdmUgYW55IGV4dHJhIGJlaGF2aW9yIG9yXG4gIC8vIGF1dG9tYXRpYyBmaWVsZHMuIFRoZXkganVzdCBzdG9yZSB0aGUgZmllbGRzIHlvdSBnaXZlIHRoZW0uXG4gIGJhc2UoY29uc3RydWN0b3JBcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpOiBCYXNlT2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBCYXNlT2JqZWN0KGNvbnN0cnVjdG9yQXJncyk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cblxuICAvLyBUaGVzZSBhcmUgXCJzeXN0ZW1cIiBvYmplY3RzIC0gdGhleSBoYXZlIHdoYXQgaXMgbmVlZGVkIHRvIGJlIGFuIG9iamVjdFxuICAvLyBpbnNpZGUgb3VyIHN5c3RlbS4gVGhleSBjb21lIHdpdGggdGhpbmdzIGxpa2UgdHlwZXMsIElEcywgdGVuYW5jeSxcbiAgLy8gZXRjLlxuICBzeXN0ZW0oY29uc3RydWN0b3JBcmdzOiBCYXNlT2JqZWN0Q29uc3RydWN0b3IpOiBTeXN0ZW1PYmplY3Qge1xuICAgIGNvbnN0IGNvbXB5ID0gbmV3IFN5c3RlbU9iamVjdChjb25zdHJ1Y3RvckFyZ3MpO1xuICAgIHRoaXMub2JqZWN0cy5wdXNoKGNvbXB5KTtcbiAgICBpZiAoY29uc3RydWN0b3JBcmdzLm9wdGlvbnMpIHtcbiAgICAgIGNvbnN0cnVjdG9yQXJncy5vcHRpb25zKGNvbXB5KTtcbiAgICB9XG4gICAgcmV0dXJuIGNvbXB5O1xuICB9XG5cbiAgY29tcG9uZW50KGNvbnN0cnVjdG9yQXJnczogQmFzZU9iamVjdENvbnN0cnVjdG9yKTogQ29tcG9uZW50T2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBDb21wb25lbnRPYmplY3QoY29uc3RydWN0b3JBcmdzKTtcbiAgICB0aGlzLm9iamVjdHMucHVzaChjb21weSk7XG4gICAgaWYgKGNvbnN0cnVjdG9yQXJncy5vcHRpb25zKSB7XG4gICAgICBjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucyhjb21weSk7XG4gICAgfVxuICAgIHJldHVybiBjb21weTtcbiAgfVxuXG4gIGVudGl0eShjb25zdHJ1Y3RvckFyZ3M6IEJhc2VPYmplY3RDb25zdHJ1Y3Rvcik6IEVudGl0eU9iamVjdCB7XG4gICAgY29uc3QgY29tcHkgPSBuZXcgRW50aXR5T2JqZWN0KGNvbnN0cnVjdG9yQXJncyk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cblxuICBjb21wb25lbnRBbmRFbnRpdHkoXG4gICAgY29uc3RydWN0b3JBcmdzOiBDb21wb25lbnRBbmRFbnRpdHlPYmplY3RDb25zdHJ1Y3RvcixcbiAgKTogQ29tcG9uZW50QW5kRW50aXR5T2JqZWN0IHtcbiAgICBjb25zdCBjb21weSA9IG5ldyBDb21wb25lbnRBbmRFbnRpdHlPYmplY3QoY29uc3RydWN0b3JBcmdzKTtcbiAgICB0aGlzLm9iamVjdHMucHVzaChjb21weS5jb21wb25lbnQpO1xuICAgIHRoaXMub2JqZWN0cy5wdXNoKGNvbXB5LmVudGl0eSk7XG4gICAgdGhpcy5vYmplY3RzLnB1c2goY29tcHkuZW50aXR5RXZlbnQpO1xuICAgIGlmIChjb25zdHJ1Y3RvckFyZ3Mub3B0aW9ucykge1xuICAgICAgY29uc3RydWN0b3JBcmdzLm9wdGlvbnMoY29tcHkpO1xuICAgIH1cbiAgICByZXR1cm4gY29tcHk7XG4gIH1cbn1cblxuZXhwb3J0IGNvbnN0IHJlZ2lzdHJ5ID0gbmV3IFJlZ2lzdHJ5KCk7XG4iXX0=