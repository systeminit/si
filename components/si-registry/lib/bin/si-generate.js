"use strict";

var _interopRequireWildcard = require("@babel/runtime/helpers/interopRequireWildcard");

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

var _regenerator = _interopRequireDefault(require("@babel/runtime/regenerator"));

var _asyncToGenerator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncToGenerator"));

var _chalk = _interopRequireDefault(require("chalk"));

var _figlet = _interopRequireDefault(require("figlet"));

var _path = _interopRequireDefault(require("path"));

var _commander = _interopRequireDefault(require("commander"));

var _registry = require("../registry");

var _protobuf = require("../codegen/protobuf");

var _rust = require("../codegen/rust");

var _listr = _interopRequireDefault(require("listr"));

require("../loader");

var _fs = _interopRequireDefault(require("fs"));

var _util = _interopRequireWildcard(require("util"));

var _child_process = _interopRequireDefault(require("child_process"));

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(n); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

function _arrayLikeToArray(arr, len) { if (len == null || len > arr.length) len = arr.length; for (var i = 0, arr2 = new Array(len); i < len; i++) { arr2[i] = arr[i]; } return arr2; }

var execCmd = _util["default"].promisify(_child_process["default"].exec);

console.log(_chalk["default"].greenBright(_figlet["default"].textSync("Lets go!", {
  horizontalLayout: "full"
})));

_commander["default"].version("0.0.1").description("Code Generation to rule them all").option("-v, --verbose", "show verbose output").parse(process.argv);

main(_commander["default"]);

function main(program) {
  // @ts-ignore
  var renderer;

  if (program.verbose) {
    renderer = "verbose";
  } else {
    renderer = "default";
  }

  var tasks = new _listr["default"]([{
    title: "Generating ".concat(_chalk["default"].keyword("darkseagreen")("Protobuf")),
    task: function task() {
      return generateProtobuf();
    }
  }, {
    title: "Generating ".concat(_chalk["default"].keyword("orange")("Rust")),
    task: function task() {
      return generateRust();
    }
  }, {
    title: "Generating ".concat(_chalk["default"].keyword("yellow")("Javascript Library")),
    task: function task() {
      return generateJavascriptLibrary();
    }
  }], {
    renderer: renderer,
    concurrent: true
  });
  tasks.run()["catch"](function (err) {
    console.log(err);
  });
}

function generateJavascriptLibrary() {
  var tasks = [];
  tasks.push({
    title: "Javascript library for si-registry",
    task: function () {
      var _task = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee() {
        return _regenerator["default"].wrap(function _callee$(_context) {
          while (1) {
            switch (_context.prev = _context.next) {
              case 0:
                _context.next = 2;
                return execCmd("npm run build");

              case 2:
              case "end":
                return _context.stop();
            }
          }
        }, _callee);
      }));

      function task() {
        return _task.apply(this, arguments);
      }

      return task;
    }()
  });
  return new _listr["default"](tasks, {
    concurrent: true
  });
}

function generateProtobuf() {
  var tasks = [];

  var _iterator = _createForOfIteratorHelper(_registry.registry.serviceNames()),
      _step;

  try {
    var _loop = function _loop() {
      var serviceName = _step.value;
      tasks.push({
        title: "Protobuf Service ".concat(_chalk["default"].keyword("darkseagreen")(serviceName)),
        task: function () {
          var _task2 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee2() {
            var cp, protoFile, writeFileAsync;
            return _regenerator["default"].wrap(function _callee2$(_context2) {
              while (1) {
                switch (_context2.prev = _context2.next) {
                  case 0:
                    cp = new _protobuf.ProtobufFormatter(_registry.registry.getObjectsForServiceName(serviceName));
                    protoFile = _path["default"].join("./proto", "si.".concat(serviceName, ".proto"));
                    writeFileAsync = (0, _util.promisify)(_fs["default"].writeFile);
                    _context2.next = 5;
                    return writeFileAsync(protoFile, cp.generateString());

                  case 5:
                  case "end":
                    return _context2.stop();
                }
              }
            }, _callee2);
          }));

          function task() {
            return _task2.apply(this, arguments);
          }

          return task;
        }()
      });
    };

    for (_iterator.s(); !(_step = _iterator.n()).done;) {
      _loop();
    }
  } catch (err) {
    _iterator.e(err);
  } finally {
    _iterator.f();
  }

  return new _listr["default"](tasks, {
    concurrent: true
  });
}

function generateRust() {
  var tasks = [];

  var _iterator2 = _createForOfIteratorHelper(_registry.registry.serviceNames()),
      _step2;

  try {
    var _loop2 = function _loop2() {
      var serviceName = _step2.value;
      var codegenRust = new _rust.CodegenRust(serviceName);

      var systemObjects = _registry.registry.getObjectsForServiceName(serviceName);

      if (codegenRust.hasServiceMethods()) {
        tasks.push({
          title: "Rust service ".concat(_chalk["default"].keyword("orange")("gen/service.rs"), " for ").concat(serviceName),
          task: function () {
            var _task3 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee3() {
              return _regenerator["default"].wrap(function _callee3$(_context3) {
                while (1) {
                  switch (_context3.prev = _context3.next) {
                    case 0:
                      _context3.next = 2;
                      return codegenRust.generateGenService();

                    case 2:
                    case "end":
                      return _context3.stop();
                  }
                }
              }, _callee3);
            }));

            function task() {
              return _task3.apply(this, arguments);
            }

            return task;
          }()
        });
      }

      if (systemObjects.some(function (o) {
        return o.kind() != "baseObject";
      })) {
        tasks.push({
          title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/mod.rs"), " for ").concat(serviceName),
          task: function () {
            var _task4 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee4() {
              return _regenerator["default"].wrap(function _callee4$(_context4) {
                while (1) {
                  switch (_context4.prev = _context4.next) {
                    case 0:
                      _context4.next = 2;
                      return codegenRust.generateGenMod();

                    case 2:
                    case "end":
                      return _context4.stop();
                  }
                }
              }, _callee4);
            }));

            function task() {
              return _task4.apply(this, arguments);
            }

            return task;
          }()
        });
        tasks.push({
          title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/model/mod.rs"), " for ").concat(serviceName),
          task: function () {
            var _task5 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee5() {
              return _regenerator["default"].wrap(function _callee5$(_context5) {
                while (1) {
                  switch (_context5.prev = _context5.next) {
                    case 0:
                      _context5.next = 2;
                      return codegenRust.generateGenModelMod();

                    case 2:
                    case "end":
                      return _context5.stop();
                  }
                }
              }, _callee5);
            }));

            function task() {
              return _task5.apply(this, arguments);
            }

            return task;
          }()
        });

        var _iterator3 = _createForOfIteratorHelper(_registry.registry.getObjectsForServiceName(serviceName)),
            _step3;

        try {
          var _loop3 = function _loop3() {
            var systemObject = _step3.value;

            if (systemObject.kind() != "baseObject") {
              tasks.push({
                title: "Rust model ".concat(_chalk["default"].keyword("orange")(serviceName), " ").concat(systemObject.typeName),
                task: function () {
                  var _task7 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee7() {
                    return _regenerator["default"].wrap(function _callee7$(_context7) {
                      while (1) {
                        switch (_context7.prev = _context7.next) {
                          case 0:
                            _context7.next = 2;
                            return codegenRust.generateGenModel(systemObject);

                          case 2:
                          case "end":
                            return _context7.stop();
                        }
                      }
                    }, _callee7);
                  }));

                  function task() {
                    return _task7.apply(this, arguments);
                  }

                  return task;
                }()
              });
            }
          };

          for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
            _loop3();
          }
        } catch (err) {
          _iterator3.e(err);
        } finally {
          _iterator3.f();
        }

        tasks.push({
          title: "Rust format ".concat(serviceName),
          task: function () {
            var _task6 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee6() {
              return _regenerator["default"].wrap(function _callee6$(_context6) {
                while (1) {
                  switch (_context6.prev = _context6.next) {
                    case 0:
                      _context6.next = 2;
                      return codegenRust.formatCode();

                    case 2:
                    case "end":
                      return _context6.stop();
                  }
                }
              }, _callee6);
            }));

            function task() {
              return _task6.apply(this, arguments);
            }

            return task;
          }()
        });
      }
    };

    for (_iterator2.s(); !(_step2 = _iterator2.n()).done;) {
      _loop2();
    }
  } catch (err) {
    _iterator2.e(err);
  } finally {
    _iterator2.f();
  }

  return new _listr["default"](tasks, {
    concurrent: false
  });
}
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9iaW4vc2ktZ2VuZXJhdGUudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiY29uc29sZSIsImxvZyIsImNoYWxrIiwiZ3JlZW5CcmlnaHQiLCJmaWdsZXQiLCJ0ZXh0U3luYyIsImhvcml6b250YWxMYXlvdXQiLCJwcm9ncmFtIiwidmVyc2lvbiIsImRlc2NyaXB0aW9uIiwib3B0aW9uIiwicGFyc2UiLCJwcm9jZXNzIiwiYXJndiIsIm1haW4iLCJyZW5kZXJlciIsInZlcmJvc2UiLCJ0YXNrcyIsIkxpc3RyIiwidGl0bGUiLCJrZXl3b3JkIiwidGFzayIsImdlbmVyYXRlUHJvdG9idWYiLCJnZW5lcmF0ZVJ1c3QiLCJnZW5lcmF0ZUphdmFzY3JpcHRMaWJyYXJ5IiwiY29uY3VycmVudCIsInJ1biIsImVyciIsInB1c2giLCJyZWdpc3RyeSIsInNlcnZpY2VOYW1lcyIsInNlcnZpY2VOYW1lIiwiY3AiLCJQcm90b2J1ZkZvcm1hdHRlciIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsInByb3RvRmlsZSIsInBhdGgiLCJqb2luIiwid3JpdGVGaWxlQXN5bmMiLCJmcyIsIndyaXRlRmlsZSIsImdlbmVyYXRlU3RyaW5nIiwiY29kZWdlblJ1c3QiLCJDb2RlZ2VuUnVzdCIsInN5c3RlbU9iamVjdHMiLCJoYXNTZXJ2aWNlTWV0aG9kcyIsImdlbmVyYXRlR2VuU2VydmljZSIsInNvbWUiLCJvIiwia2luZCIsImdlbmVyYXRlR2VuTW9kIiwiZ2VuZXJhdGVHZW5Nb2RlbE1vZCIsInN5c3RlbU9iamVjdCIsInR5cGVOYW1lIiwiZ2VuZXJhdGVHZW5Nb2RlbCIsImZvcm1hdENvZGUiXSwibWFwcGluZ3MiOiI7Ozs7Ozs7Ozs7QUFBQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7QUFDQTs7Ozs7Ozs7QUFFQSxJ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WUFBM0IsRUFBeUM7QUFDdkNoQyxjQUFBQSxLQUFLLENBQUNXLElBQU4sQ0FBVztBQUNUVCxnQkFBQUEsS0FBSyx1QkFBZ0JqQixrQkFBTWtCLE9BQU4sQ0FBYyxRQUFkLEVBQXdCVyxXQUF4QixDQUFoQixjQUNIcUIsWUFBWSxDQUFDQyxRQURWLENBREk7QUFJVGhDLGdCQUFBQSxJQUFJO0FBQUEsNkdBQUU7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsbUNBQ0VxQixXQUFXLENBQUNZLGdCQUFaLENBQTZCRixZQUE3QixDQURGOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLG1CQUFGOztBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBSkssZUFBWDtBQVFEO0FBL0JrRDs7QUFtQnJELGlFQUVHO0FBQUE7QUFXRjtBQWhDb0Q7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFrQ3JEbkMsUUFBQUEsS0FBSyxDQUFDVyxJQUFOLENBQVc7QUFDVFQsVUFBQUEsS0FBSyx3QkFBaUJZLFdBQWpCLENBREk7QUFFVFYsVUFBQUEsSUFBSTtBQUFBLHVHQUFFO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLDZCQUNFcUIsV0FBVyxDQUFDYSxVQUFaLEVBREY7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsYUFBRjs7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUZLLFNBQVg7QUFNRDtBQTFEMEI7O0FBRzdCLDJEQUFtRDtBQUFBO0FBd0RsRDtBQTNENEI7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUE2RDdCLFNBQU8sSUFBSXJDLGlCQUFKLENBQVVELEtBQVYsRUFBaUI7QUFBRVEsSUFBQUEsVUFBVSxFQUFFO0FBQWQsR0FBakIsQ0FBUDtBQUNEIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IGNoYWxrIGZyb20gXCJjaGFsa1wiO1xuaW1wb3J0IGZpZ2xldCBmcm9tIFwiZmlnbGV0XCI7XG5pbXBvcnQgcGF0aCBmcm9tIFwicGF0aFwiO1xuaW1wb3J0IHByb2dyYW0gZnJvbSBcImNvbW1hbmRlclwiO1xuaW1wb3J0IHsgcmVnaXN0cnkgfSBmcm9tIFwiLi4vcmVnaXN0cnlcIjtcbmltcG9ydCB7IFByb3RvYnVmRm9ybWF0dGVyIH0gZnJvbSBcIi4uL2NvZGVnZW4vcHJvdG9idWZcIjtcbmltcG9ydCB7IENvZGVnZW5SdXN0IH0gZnJvbSBcIi4uL2NvZGVnZW4vcnVzdFwiO1xuaW1wb3J0IExpc3RyLCB7IExpc3RyUmVuZGVyZXJWYWx1ZSB9IGZyb20gXCJsaXN0clwiO1xuaW1wb3J0IFwiLi4vbG9hZGVyXCI7XG5pbXBvcnQgZnMgZnJvbSBcImZzXCI7XG5pbXBvcnQgeyBwcm9taXNpZnkgfSBmcm9tIFwidXRpbFwiO1xuaW1wb3J0IGNoaWxkUHJvY2VzcyBmcm9tIFwiY2hpbGRfcHJvY2Vzc1wiO1xuaW1wb3J0IHV0aWwgZnJvbSBcInV0aWxcIjtcbmNvbnN0IGV4ZWNDbWQgPSB1dGlsLnByb21pc2lmeShjaGlsZFByb2Nlc3MuZXhlYyk7XG5cbmNvbnNvbGUubG9nKFxuICBjaGFsay5ncmVlbkJyaWdodChmaWdsZXQudGV4dFN5bmMoXCJMZXRzIGdvIVwiLCB7IGhvcml6b250YWxMYXlvdXQ6IFwiZnVsbFwiIH0pKSxcbik7XG5cbnByb2dyYW1cbiAgLnZlcnNpb24oXCIwLjAuMVwiKVxuICAuZGVzY3JpcHRpb24oXCJDb2RlIEdlbmVyYXRpb24gdG8gcnVsZSB0aGVtIGFsbFwiKVxuICAub3B0aW9uKFwiLXYsIC0tdmVyYm9zZVwiLCBcInNob3cgdmVyYm9zZSBvdXRwdXRcIilcbiAgLnBhcnNlKHByb2Nlc3MuYXJndik7XG5cbm1haW4ocHJvZ3JhbSk7XG5cbmZ1bmN0aW9uIG1haW4ocHJvZ3JhbTogcHJvZ3JhbS5Db21tYW5kKTogdm9pZCB7XG4gIC8vIEB0cy1pZ25vcmVcbiAgbGV0IHJlbmRlcmVyOiBMaXN0clJlbmRlcmVyVmFsdWU8YW55PjtcbiAgaWYgKHByb2dyYW0udmVyYm9zZSkge1xuICAgIHJlbmRlcmVyID0gXCJ2ZXJib3NlXCI7XG4gIH0gZWxzZSB7XG4gICAgcmVuZGVyZXIgPSBcImRlZmF1bHRcIjtcbiAgfVxuICBjb25zdCB0YXNrcyA9IG5ldyBMaXN0cihcbiAgICBbXG4gICAgICB7XG4gICAgICAgIHRpdGxlOiBgR2VuZXJhdGluZyAke2NoYWxrLmtleXdvcmQoXCJkYXJrc2VhZ3JlZW5cIikoXCJQcm90b2J1ZlwiKX1gLFxuICAgICAgICB0YXNrOiAoKTogTGlzdHIgPT4ge1xuICAgICAgICAgIHJldHVybiBnZW5lcmF0ZVByb3RvYnVmKCk7XG4gICAgICAgIH0sXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICB0aXRsZTogYEdlbmVyYXRpbmcgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKFwiUnVzdFwiKX1gLFxuICAgICAgICB0YXNrOiAoKTogTGlzdHIgPT4ge1xuICAgICAgICAgIHJldHVybiBnZW5lcmF0ZVJ1c3QoKTtcbiAgICAgICAgfSxcbiAgICAgIH0sXG4gICAgICB7XG4gICAgICAgIHRpdGxlOiBgR2VuZXJhdGluZyAke2NoYWxrLmtleXdvcmQoXCJ5ZWxsb3dcIikoXCJKYXZhc2NyaXB0IExpYnJhcnlcIil9YCxcbiAgICAgICAgdGFzazogKCk6IExpc3RyID0+IHtcbiAgICAgICAgICByZXR1cm4gZ2VuZXJhdGVKYXZhc2NyaXB0TGlicmFyeSgpO1xuICAgICAgICB9LFxuICAgICAgfSxcbiAgICBdLFxuICAgIHtcbiAgICAgIHJlbmRlcmVyLFxuICAgICAgY29uY3VycmVudDogdHJ1ZSxcbiAgICB9LFxuICApO1xuICB0YXNrcy5ydW4oKS5jYXRjaCgoZXJyOiBFcnJvcik6IHZvaWQgPT4ge1xuICAgIGNvbnNvbGUubG9nKGVycik7XG4gIH0pO1xufVxuXG5mdW5jdGlvbiBnZW5lcmF0ZUphdmFzY3JpcHRMaWJyYXJ5KCk6IExpc3RyIHtcbiAgY29uc3QgdGFza3MgPSBbXTtcbiAgdGFza3MucHVzaCh7XG4gICAgdGl0bGU6IGBKYXZhc2NyaXB0IGxpYnJhcnkgZm9yIHNpLXJlZ2lzdHJ5YCxcbiAgICB0YXNrOiBhc3luYyAoKSA9PiB7XG4gICAgICBhd2FpdCBleGVjQ21kKFwibnBtIHJ1biBidWlsZFwiKTtcbiAgICB9LFxuICB9KTtcbiAgcmV0dXJuIG5ldyBMaXN0cih0YXNrcywgeyBjb25jdXJyZW50OiB0cnVlIH0pO1xufVxuXG5mdW5jdGlvbiBnZW5lcmF0ZVByb3RvYnVmKCk6IExpc3RyIHtcbiAgY29uc3QgdGFza3MgPSBbXTtcbiAgZm9yIChjb25zdCBzZXJ2aWNlTmFtZSBvZiByZWdpc3RyeS5zZXJ2aWNlTmFtZXMoKSkge1xuICAgIHRhc2tzLnB1c2goe1xuICAgICAgdGl0bGU6IGBQcm90b2J1ZiBTZXJ2aWNlICR7Y2hhbGsua2V5d29yZChcImRhcmtzZWFncmVlblwiKShzZXJ2aWNlTmFtZSl9YCxcbiAgICAgIHRhc2s6IGFzeW5jICgpID0+IHtcbiAgICAgICAgY29uc3QgY3AgPSBuZXcgUHJvdG9idWZGb3JtYXR0ZXIoXG4gICAgICAgICAgcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKHNlcnZpY2VOYW1lKSxcbiAgICAgICAgKTtcbiAgICAgICAgY29uc3QgcHJvdG9GaWxlID0gcGF0aC5qb2luKFwiLi9wcm90b1wiLCBgc2kuJHtzZXJ2aWNlTmFtZX0ucHJvdG9gKTtcbiAgICAgICAgY29uc3Qgd3JpdGVGaWxlQXN5bmMgPSBwcm9taXNpZnkoZnMud3JpdGVGaWxlKTtcbiAgICAgICAgYXdhaXQgd3JpdGVGaWxlQXN5bmMocHJvdG9GaWxlLCBjcC5nZW5lcmF0ZVN0cmluZygpKTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cbiAgcmV0dXJuIG5ldyBMaXN0cih0YXNrcywgeyBjb25jdXJyZW50OiB0cnVlIH0pO1xufVxuXG5mdW5jdGlvbiBnZW5lcmF0ZVJ1c3QoKTogTGlzdHIge1xuICBjb25zdCB0YXNrcyA9IFtdO1xuXG4gIGZvciAoY29uc3Qgc2VydmljZU5hbWUgb2YgcmVnaXN0cnkuc2VydmljZU5hbWVzKCkpIHtcbiAgICBjb25zdCBjb2RlZ2VuUnVzdCA9IG5ldyBDb2RlZ2VuUnVzdChzZXJ2aWNlTmFtZSk7XG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0cyA9IHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSk7XG5cbiAgICBpZiAoY29kZWdlblJ1c3QuaGFzU2VydmljZU1ldGhvZHMoKSkge1xuICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgIHRpdGxlOiBgUnVzdCBzZXJ2aWNlICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcbiAgICAgICAgICBcImdlbi9zZXJ2aWNlLnJzXCIsXG4gICAgICAgICl9IGZvciAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICBhd2FpdCBjb2RlZ2VuUnVzdC5nZW5lcmF0ZUdlblNlcnZpY2UoKTtcbiAgICAgICAgfSxcbiAgICAgIH0pO1xuICAgIH1cblxuICAgIGlmIChzeXN0ZW1PYmplY3RzLnNvbWUobyA9PiBvLmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIikpIHtcbiAgICAgIHRhc2tzLnB1c2goe1xuICAgICAgICB0aXRsZTogYFJ1c3QgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKFxuICAgICAgICAgIFwiZ2VuL21vZC5yc1wiLFxuICAgICAgICApfSBmb3IgJHtzZXJ2aWNlTmFtZX1gLFxuICAgICAgICB0YXNrOiBhc3luYyAoKTogUHJvbWlzZTx2b2lkPiA9PiB7XG4gICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5Nb2QoKTtcbiAgICAgICAgfSxcbiAgICAgIH0pO1xuXG4gICAgICB0YXNrcy5wdXNoKHtcbiAgICAgICAgdGl0bGU6IGBSdXN0ICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcbiAgICAgICAgICBcImdlbi9tb2RlbC9tb2QucnNcIixcbiAgICAgICAgKX0gZm9yICR7c2VydmljZU5hbWV9YCxcbiAgICAgICAgdGFzazogYXN5bmMgKCk6IFByb21pc2U8dm9pZD4gPT4ge1xuICAgICAgICAgIGF3YWl0IGNvZGVnZW5SdXN0LmdlbmVyYXRlR2VuTW9kZWxNb2QoKTtcbiAgICAgICAgfSxcbiAgICAgIH0pO1xuXG4gICAgICBmb3IgKGNvbnN0IHN5c3RlbU9iamVjdCBvZiByZWdpc3RyeS5nZXRPYmplY3RzRm9yU2VydmljZU5hbWUoXG4gICAgICAgIHNlcnZpY2VOYW1lLFxuICAgICAgKSkge1xuICAgICAgICBpZiAoc3lzdGVtT2JqZWN0LmtpbmQoKSAhPSBcImJhc2VPYmplY3RcIikge1xuICAgICAgICAgIHRhc2tzLnB1c2goe1xuICAgICAgICAgICAgdGl0bGU6IGBSdXN0IG1vZGVsICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShzZXJ2aWNlTmFtZSl9ICR7XG4gICAgICAgICAgICAgIHN5c3RlbU9iamVjdC50eXBlTmFtZVxuICAgICAgICAgICAgfWAsXG4gICAgICAgICAgICB0YXNrOiBhc3luYyAoKTogUHJvbWlzZTx2b2lkPiA9PiB7XG4gICAgICAgICAgICAgIGF3YWl0IGNvZGVnZW5SdXN0LmdlbmVyYXRlR2VuTW9kZWwoc3lzdGVtT2JqZWN0KTtcbiAgICAgICAgICAgIH0sXG4gICAgICAgICAgfSk7XG4gICAgICAgIH1cbiAgICAgIH1cblxuICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgIHRpdGxlOiBgUnVzdCBmb3JtYXQgJHtzZXJ2aWNlTmFtZX1gLFxuICAgICAgICB0YXNrOiBhc3luYyAoKTogUHJvbWlzZTx2b2lkPiA9PiB7XG4gICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZm9ybWF0Q29kZSgpO1xuICAgICAgICB9LFxuICAgICAgfSk7XG4gICAgfVxuICB9XG5cbiAgcmV0dXJuIG5ldyBMaXN0cih0YXNrcywgeyBjb25jdXJyZW50OiBmYWxzZSB9KTtcbn1cbiJdfQ==