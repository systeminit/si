"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

var _regenerator = _interopRequireDefault(require("@babel/runtime/regenerator"));

var _asyncToGenerator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncToGenerator"));

var _chalk = _interopRequireDefault(require("chalk"));

var _figlet = _interopRequireDefault(require("figlet"));

var _commander = _interopRequireDefault(require("commander"));

var _registry = require("../registry");

var _protobuf = require("../codegen/protobuf");

var _rust = require("../codegen/rust");

var _listr = _interopRequireDefault(require("listr"));

require("../loader");

var _child_process = _interopRequireDefault(require("child_process"));

var _util = _interopRequireDefault(require("util"));

function _createForOfIteratorHelper(o) { if (typeof Symbol === "undefined" || o[Symbol.iterator] == null) { if (Array.isArray(o) || (o = _unsupportedIterableToArray(o))) { var i = 0; var F = function F() {}; return { s: F, n: function n() { if (i >= o.length) return { done: true }; return { done: false, value: o[i++] }; }, e: function e(_e) { throw _e; }, f: F }; } throw new TypeError("Invalid attempt to iterate non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); } var it, normalCompletion = true, didErr = false, err; return { s: function s() { it = o[Symbol.iterator](); }, n: function n() { var step = it.next(); normalCompletion = step.done; return step; }, e: function e(_e2) { didErr = true; err = _e2; }, f: function f() { try { if (!normalCompletion && it["return"] != null) it["return"](); } finally { if (didErr) throw err; } } }; }

function _unsupportedIterableToArray(o, minLen) { if (!o) return; if (typeof o === "string") return _arrayLikeToArray(o, minLen); var n = Object.prototype.toString.call(o).slice(8, -1); if (n === "Object" && o.constructor) n = o.constructor.name; if (n === "Map" || n === "Set") return Array.from(o); if (n === "Arguments" || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(n)) return _arrayLikeToArray(o, minLen); }

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
            var cp;
            return _regenerator["default"].wrap(function _callee2$(_context2) {
              while (1) {
                switch (_context2.prev = _context2.next) {
                  case 0:
                    cp = new _protobuf.ProtobufFormatter(_registry.registry.getObjectsForServiceName(serviceName));
                    _context2.next = 3;
                    return cp.generateProto();

                  case 3:
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

      if (systemObjects.some(function (o) {
        return o.kind() != "baseObject";
      })) {
        tasks.push({
          title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/mod.rs"), " for ").concat(serviceName),
          task: function () {
            var _task3 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee3() {
              return _regenerator["default"].wrap(function _callee3$(_context3) {
                while (1) {
                  switch (_context3.prev = _context3.next) {
                    case 0:
                      _context3.next = 2;
                      return codegenRust.generateGenMod();

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

        if (codegenRust.hasServiceMethods()) {
          tasks.push({
            title: "Rust service ".concat(_chalk["default"].keyword("orange")("gen/service.rs"), " for ").concat(serviceName),
            task: function () {
              var _task4 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee4() {
                return _regenerator["default"].wrap(function _callee4$(_context4) {
                  while (1) {
                    switch (_context4.prev = _context4.next) {
                      case 0:
                        _context4.next = 2;
                        return codegenRust.generateGenService();

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
        }

        if (codegenRust.hasModels()) {
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
                    var _task6 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee6() {
                      return _regenerator["default"].wrap(function _callee6$(_context6) {
                        while (1) {
                          switch (_context6.prev = _context6.next) {
                            case 0:
                              _context6.next = 2;
                              return codegenRust.generateGenModel(systemObject);

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

            for (_iterator3.s(); !(_step3 = _iterator3.n()).done;) {
              _loop3();
            }
          } catch (err) {
            _iterator3.e(err);
          } finally {
            _iterator3.f();
          }
        }

        if (codegenRust.hasEntityIntegrationServcices()) {
          tasks.push({
            title: "Rust ".concat(_chalk["default"].keyword("orange")("gen/agent/mod.rs"), " for ").concat(serviceName),
            task: function () {
              var _task7 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee7() {
                return _regenerator["default"].wrap(function _callee7$(_context7) {
                  while (1) {
                    switch (_context7.prev = _context7.next) {
                      case 0:
                        _context7.next = 2;
                        return codegenRust.generateGenAgentMod();

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

          var _iterator4 = _createForOfIteratorHelper(codegenRust.entityIntegrationServices()),
              _step4;

          try {
            var _loop4 = function _loop4() {
              var agent = _step4.value;
              tasks.push({
                title: "Rust agent ".concat(_chalk["default"].keyword("orange")(serviceName), " ").concat(agent.agentName),
                task: function () {
                  var _task8 = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee8() {
                    return _regenerator["default"].wrap(function _callee8$(_context8) {
                      while (1) {
                        switch (_context8.prev = _context8.next) {
                          case 0:
                            _context8.next = 2;
                            return codegenRust.generateGenAgent(agent);

                          case 2:
                          case "end":
                            return _context8.stop();
                        }
                      }
                    }, _callee8);
                  }));

                  function task() {
                    return _task8.apply(this, arguments);
                  }

                  return task;
                }()
              });
            };

            for (_iterator4.s(); !(_step4 = _iterator4.n()).done;) {
              _loop4();
            }
          } catch (err) {
            _iterator4.e(err);
          } finally {
            _iterator4.f();
          }
        }
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
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9iaW4vc2ktZ2VuZXJhdGUudHMiXSwibmFtZXMiOlsiZXhlY0NtZCIsInV0aWwiLCJwcm9taXNpZnkiLCJjaGlsZFByb2Nlc3MiLCJleGVjIiwiY29uc29sZSIsImxvZyIsImNoYWxrIiwiZ3JlZW5CcmlnaHQiLCJmaWdsZXQiLCJ0ZXh0U3luYyIsImhvcml6b250YWxMYXlvdXQiLCJwcm9ncmFtIiwidmVyc2lvbiIsImRlc2NyaXB0aW9uIiwib3B0aW9uIiwicGFyc2UiLCJwcm9jZXNzIiwiYXJndiIsIm1haW4iLCJyZW5kZXJlciIsInZlcmJvc2UiLCJ0YXNrcyIsIkxpc3RyIiwidGl0bGUiLCJrZXl3b3JkIiwidGFzayIsImdlbmVyYXRlUHJvdG9idWYiLCJnZW5lcmF0ZVJ1c3QiLCJnZW5lcmF0ZUphdmFzY3JpcHRMaWJyYXJ5IiwiY29uY3VycmVudCIsInJ1biIsImVyciIsInB1c2giLCJyZWdpc3RyeSIsInNlcnZpY2VOYW1lcyIsInNlcnZpY2VOYW1lIiwiY3AiLCJQcm90b2J1ZkZvcm1hdHRlciIsImdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZSIsImdlbmVyYXRlUHJvdG8iLCJjb2RlZ2VuUnVzdCIsIkNvZGVnZW5SdXN0Iiwic3lzdGVtT2JqZWN0cyIsInNvbWUiLCJvIiwia2luZCIsImdlbmVyYXRlR2VuTW9kIiwiaGFzU2VydmljZU1ldGhvZHMiLCJnZW5lcmF0ZUdlblNlcnZpY2UiLCJoYXNNb2RlbHMiLCJnZW5lcmF0ZUdlbk1vZGVsTW9kIiwic3lzdGVtT2JqZWN0IiwidHlwZU5hbWUiLCJnZW5lcmF0ZUdlbk1vZGVsIiwiaGFzRW50aXR5SW50ZWdyYXRpb25TZXJ2Y2ljZXMiLCJnZW5lcmF0ZUdlbkFnZW50TW9kIiwiZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcyIsImFnZW50IiwiYWdlbnROYW1lIiwiZ2VuZXJhdGVHZW5BZ2VudCJdLCJtYXBwaW5ncyI6Ijs7Ozs7Ozs7QUFBQTs7QUFDQTs7QUFFQTs7QUFDQTs7QUFD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QUE7QUFBQTtBQUFBO0FBQUEsbUNBQ0VlLFdBQVcsQ0FBQ21CLGdCQUFaLENBQTZCRixLQUE3QixDQURGOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBLG1CQUFGOztBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBSkssZUFBWDtBQVg2Qzs7QUFVL0MsbUVBQTZEO0FBQUE7QUFTNUQ7QUFuQjhDO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFvQmhEO0FBQ0Y7QUEzRTBCOztBQUc3QiwyREFBbUQ7QUFBQTtBQXlFbEQ7QUE1RTRCO0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBOEU3QixTQUFPLElBQUluQyxpQkFBSixDQUFVRCxLQUFWLEVBQWlCO0FBQUVRLElBQUFBLFVBQVUsRUFBRTtBQUFkLEdBQWpCLENBQVA7QUFDRCIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCBjaGFsayBmcm9tIFwiY2hhbGtcIjtcbmltcG9ydCBmaWdsZXQgZnJvbSBcImZpZ2xldFwiO1xuaW1wb3J0IHBhdGggZnJvbSBcInBhdGhcIjtcbmltcG9ydCBwcm9ncmFtIGZyb20gXCJjb21tYW5kZXJcIjtcbmltcG9ydCB7IHJlZ2lzdHJ5IH0gZnJvbSBcIi4uL3JlZ2lzdHJ5XCI7XG5pbXBvcnQgeyBQcm90b2J1ZkZvcm1hdHRlciB9IGZyb20gXCIuLi9jb2RlZ2VuL3Byb3RvYnVmXCI7XG5pbXBvcnQgeyBDb2RlZ2VuUnVzdCB9IGZyb20gXCIuLi9jb2RlZ2VuL3J1c3RcIjtcbmltcG9ydCBMaXN0ciwgeyBMaXN0clJlbmRlcmVyVmFsdWUgfSBmcm9tIFwibGlzdHJcIjtcbmltcG9ydCBcIi4uL2xvYWRlclwiO1xuaW1wb3J0IGZzIGZyb20gXCJmc1wiO1xuaW1wb3J0IHsgcHJvbWlzaWZ5IH0gZnJvbSBcInV0aWxcIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCB1dGlsIGZyb20gXCJ1dGlsXCI7XG5jb25zdCBleGVjQ21kID0gdXRpbC5wcm9taXNpZnkoY2hpbGRQcm9jZXNzLmV4ZWMpO1xuXG5jb25zb2xlLmxvZyhcbiAgY2hhbGsuZ3JlZW5CcmlnaHQoZmlnbGV0LnRleHRTeW5jKFwiTGV0cyBnbyFcIiwgeyBob3Jpem9udGFsTGF5b3V0OiBcImZ1bGxcIiB9KSksXG4pO1xuXG5wcm9ncmFtXG4gIC52ZXJzaW9uKFwiMC4wLjFcIilcbiAgLmRlc2NyaXB0aW9uKFwiQ29kZSBHZW5lcmF0aW9uIHRvIHJ1bGUgdGhlbSBhbGxcIilcbiAgLm9wdGlvbihcIi12LCAtLXZlcmJvc2VcIiwgXCJzaG93IHZlcmJvc2Ugb3V0cHV0XCIpXG4gIC5wYXJzZShwcm9jZXNzLmFyZ3YpO1xuXG5tYWluKHByb2dyYW0pO1xuXG5mdW5jdGlvbiBtYWluKHByb2dyYW06IHByb2dyYW0uQ29tbWFuZCk6IHZvaWQge1xuICAvLyBAdHMtaWdub3JlXG4gIGxldCByZW5kZXJlcjogTGlzdHJSZW5kZXJlclZhbHVlPGFueT47XG4gIGlmIChwcm9ncmFtLnZlcmJvc2UpIHtcbiAgICByZW5kZXJlciA9IFwidmVyYm9zZVwiO1xuICB9IGVsc2Uge1xuICAgIHJlbmRlcmVyID0gXCJkZWZhdWx0XCI7XG4gIH1cbiAgY29uc3QgdGFza3MgPSBuZXcgTGlzdHIoXG4gICAgW1xuICAgICAge1xuICAgICAgICB0aXRsZTogYEdlbmVyYXRpbmcgJHtjaGFsay5rZXl3b3JkKFwiZGFya3NlYWdyZWVuXCIpKFwiUHJvdG9idWZcIil9YCxcbiAgICAgICAgdGFzazogKCk6IExpc3RyID0+IHtcbiAgICAgICAgICByZXR1cm4gZ2VuZXJhdGVQcm90b2J1ZigpO1xuICAgICAgICB9LFxuICAgICAgfSxcbiAgICAgIHtcbiAgICAgICAgdGl0bGU6IGBHZW5lcmF0aW5nICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcIlJ1c3RcIil9YCxcbiAgICAgICAgdGFzazogKCk6IExpc3RyID0+IHtcbiAgICAgICAgICByZXR1cm4gZ2VuZXJhdGVSdXN0KCk7XG4gICAgICAgIH0sXG4gICAgICB9LFxuICAgICAge1xuICAgICAgICB0aXRsZTogYEdlbmVyYXRpbmcgJHtjaGFsay5rZXl3b3JkKFwieWVsbG93XCIpKFwiSmF2YXNjcmlwdCBMaWJyYXJ5XCIpfWAsXG4gICAgICAgIHRhc2s6ICgpOiBMaXN0ciA9PiB7XG4gICAgICAgICAgcmV0dXJuIGdlbmVyYXRlSmF2YXNjcmlwdExpYnJhcnkoKTtcbiAgICAgICAgfSxcbiAgICAgIH0sXG4gICAgXSxcbiAgICB7XG4gICAgICByZW5kZXJlcixcbiAgICAgIGNvbmN1cnJlbnQ6IHRydWUsXG4gICAgfSxcbiAgKTtcbiAgdGFza3MucnVuKCkuY2F0Y2goKGVycjogRXJyb3IpOiB2b2lkID0+IHtcbiAgICBjb25zb2xlLmxvZyhlcnIpO1xuICB9KTtcbn1cblxuZnVuY3Rpb24gZ2VuZXJhdGVKYXZhc2NyaXB0TGlicmFyeSgpOiBMaXN0ciB7XG4gIGNvbnN0IHRhc2tzID0gW107XG4gIHRhc2tzLnB1c2goe1xuICAgIHRpdGxlOiBgSmF2YXNjcmlwdCBsaWJyYXJ5IGZvciBzaS1yZWdpc3RyeWAsXG4gICAgdGFzazogYXN5bmMgKCkgPT4ge1xuICAgICAgYXdhaXQgZXhlY0NtZChcIm5wbSBydW4gYnVpbGRcIik7XG4gICAgfSxcbiAgfSk7XG4gIHJldHVybiBuZXcgTGlzdHIodGFza3MsIHsgY29uY3VycmVudDogdHJ1ZSB9KTtcbn1cblxuZnVuY3Rpb24gZ2VuZXJhdGVQcm90b2J1ZigpOiBMaXN0ciB7XG4gIGNvbnN0IHRhc2tzID0gW107XG4gIGZvciAoY29uc3Qgc2VydmljZU5hbWUgb2YgcmVnaXN0cnkuc2VydmljZU5hbWVzKCkpIHtcbiAgICB0YXNrcy5wdXNoKHtcbiAgICAgIHRpdGxlOiBgUHJvdG9idWYgU2VydmljZSAke2NoYWxrLmtleXdvcmQoXCJkYXJrc2VhZ3JlZW5cIikoc2VydmljZU5hbWUpfWAsXG4gICAgICB0YXNrOiBhc3luYyAoKSA9PiB7XG4gICAgICAgIGNvbnN0IGNwID0gbmV3IFByb3RvYnVmRm9ybWF0dGVyKFxuICAgICAgICAgIHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSksXG4gICAgICAgICk7XG4gICAgICAgIGF3YWl0IGNwLmdlbmVyYXRlUHJvdG8oKTtcbiAgICAgIH0sXG4gICAgfSk7XG4gIH1cbiAgcmV0dXJuIG5ldyBMaXN0cih0YXNrcywgeyBjb25jdXJyZW50OiB0cnVlIH0pO1xufVxuXG5mdW5jdGlvbiBnZW5lcmF0ZVJ1c3QoKTogTGlzdHIge1xuICBjb25zdCB0YXNrcyA9IFtdO1xuXG4gIGZvciAoY29uc3Qgc2VydmljZU5hbWUgb2YgcmVnaXN0cnkuc2VydmljZU5hbWVzKCkpIHtcbiAgICBjb25zdCBjb2RlZ2VuUnVzdCA9IG5ldyBDb2RlZ2VuUnVzdChzZXJ2aWNlTmFtZSk7XG4gICAgY29uc3Qgc3lzdGVtT2JqZWN0cyA9IHJlZ2lzdHJ5LmdldE9iamVjdHNGb3JTZXJ2aWNlTmFtZShzZXJ2aWNlTmFtZSk7XG5cbiAgICBpZiAoc3lzdGVtT2JqZWN0cy5zb21lKG8gPT4gby5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIpKSB7XG4gICAgICB0YXNrcy5wdXNoKHtcbiAgICAgICAgdGl0bGU6IGBSdXN0ICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcbiAgICAgICAgICBcImdlbi9tb2QucnNcIixcbiAgICAgICAgKX0gZm9yICR7c2VydmljZU5hbWV9YCxcbiAgICAgICAgdGFzazogYXN5bmMgKCk6IFByb21pc2U8dm9pZD4gPT4ge1xuICAgICAgICAgIGF3YWl0IGNvZGVnZW5SdXN0LmdlbmVyYXRlR2VuTW9kKCk7XG4gICAgICAgIH0sXG4gICAgICB9KTtcblxuICAgICAgaWYgKGNvZGVnZW5SdXN0Lmhhc1NlcnZpY2VNZXRob2RzKCkpIHtcbiAgICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgICAgdGl0bGU6IGBSdXN0IHNlcnZpY2UgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKFxuICAgICAgICAgICAgXCJnZW4vc2VydmljZS5yc1wiLFxuICAgICAgICAgICl9IGZvciAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgICAgdGFzazogYXN5bmMgKCk6IFByb21pc2U8dm9pZD4gPT4ge1xuICAgICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5TZXJ2aWNlKCk7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG4gICAgICB9XG5cbiAgICAgIGlmIChjb2RlZ2VuUnVzdC5oYXNNb2RlbHMoKSkge1xuICAgICAgICB0YXNrcy5wdXNoKHtcbiAgICAgICAgICB0aXRsZTogYFJ1c3QgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKFxuICAgICAgICAgICAgXCJnZW4vbW9kZWwvbW9kLnJzXCIsXG4gICAgICAgICAgKX0gZm9yICR7c2VydmljZU5hbWV9YCxcbiAgICAgICAgICB0YXNrOiBhc3luYyAoKTogUHJvbWlzZTx2b2lkPiA9PiB7XG4gICAgICAgICAgICBhd2FpdCBjb2RlZ2VuUnVzdC5nZW5lcmF0ZUdlbk1vZGVsTW9kKCk7XG4gICAgICAgICAgfSxcbiAgICAgICAgfSk7XG5cbiAgICAgICAgZm9yIChjb25zdCBzeXN0ZW1PYmplY3Qgb2YgcmVnaXN0cnkuZ2V0T2JqZWN0c0ZvclNlcnZpY2VOYW1lKFxuICAgICAgICAgIHNlcnZpY2VOYW1lLFxuICAgICAgICApKSB7XG4gICAgICAgICAgaWYgKHN5c3RlbU9iamVjdC5raW5kKCkgIT0gXCJiYXNlT2JqZWN0XCIpIHtcbiAgICAgICAgICAgIHRhc2tzLnB1c2goe1xuICAgICAgICAgICAgICB0aXRsZTogYFJ1c3QgbW9kZWwgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKHNlcnZpY2VOYW1lKX0gJHtcbiAgICAgICAgICAgICAgICBzeXN0ZW1PYmplY3QudHlwZU5hbWVcbiAgICAgICAgICAgICAgfWAsXG4gICAgICAgICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICAgICAgICBhd2FpdCBjb2RlZ2VuUnVzdC5nZW5lcmF0ZUdlbk1vZGVsKHN5c3RlbU9iamVjdCk7XG4gICAgICAgICAgICAgIH0sXG4gICAgICAgICAgICB9KTtcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH1cblxuICAgICAgaWYgKGNvZGVnZW5SdXN0Lmhhc0VudGl0eUludGVncmF0aW9uU2VydmNpY2VzKCkpIHtcbiAgICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgICAgdGl0bGU6IGBSdXN0ICR7Y2hhbGsua2V5d29yZChcIm9yYW5nZVwiKShcbiAgICAgICAgICAgIFwiZ2VuL2FnZW50L21vZC5yc1wiLFxuICAgICAgICAgICl9IGZvciAke3NlcnZpY2VOYW1lfWAsXG4gICAgICAgICAgdGFzazogYXN5bmMgKCk6IFByb21pc2U8dm9pZD4gPT4ge1xuICAgICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5BZ2VudE1vZCgpO1xuICAgICAgICAgIH0sXG4gICAgICAgIH0pO1xuXG4gICAgICAgIGZvciAoY29uc3QgYWdlbnQgb2YgY29kZWdlblJ1c3QuZW50aXR5SW50ZWdyYXRpb25TZXJ2aWNlcygpKSB7XG4gICAgICAgICAgdGFza3MucHVzaCh7XG4gICAgICAgICAgICB0aXRsZTogYFJ1c3QgYWdlbnQgJHtjaGFsay5rZXl3b3JkKFwib3JhbmdlXCIpKHNlcnZpY2VOYW1lKX0gJHtcbiAgICAgICAgICAgICAgYWdlbnQuYWdlbnROYW1lXG4gICAgICAgICAgICB9YCxcbiAgICAgICAgICAgIHRhc2s6IGFzeW5jICgpOiBQcm9taXNlPHZvaWQ+ID0+IHtcbiAgICAgICAgICAgICAgYXdhaXQgY29kZWdlblJ1c3QuZ2VuZXJhdGVHZW5BZ2VudChhZ2VudCk7XG4gICAgICAgICAgICB9LFxuICAgICAgICAgIH0pO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICB9XG5cbiAgcmV0dXJuIG5ldyBMaXN0cih0YXNrcywgeyBjb25jdXJyZW50OiBmYWxzZSB9KTtcbn1cbiJdfQ==