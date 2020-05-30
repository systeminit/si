"use strict";

var _interopRequireDefault = require("@babel/runtime/helpers/interopRequireDefault");

Object.defineProperty(exports, "__esModule", {
  value: true
});
exports.makePath = makePath;
exports.writeCode = writeCode;

var _regenerator = _interopRequireDefault(require("@babel/runtime/regenerator"));

var _asyncToGenerator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncToGenerator"));

var _asyncIterator2 = _interopRequireDefault(require("@babel/runtime/helpers/asyncIterator"));

var _stringio = require("@rauschma/stringio");

var _child_process = _interopRequireDefault(require("child_process"));

var _fs = _interopRequireDefault(require("fs"));

var _path = _interopRequireDefault(require("path"));

var _xxhash = _interopRequireDefault(require("xxhash"));

function makePath(_x) {
  return _makePath.apply(this, arguments);
}

function _makePath() {
  _makePath = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee(pathPart) {
    var absolutePathName;
    return _regenerator["default"].wrap(function _callee$(_context) {
      while (1) {
        switch (_context.prev = _context.next) {
          case 0:
            absolutePathName = _path["default"].resolve(pathPart);

            if (_fs["default"].existsSync(absolutePathName)) {
              _context.next = 4;
              break;
            }

            _context.next = 4;
            return _fs["default"].promises.mkdir(absolutePathName, {
              recursive: true
            });

          case 4:
            return _context.abrupt("return", absolutePathName);

          case 5:
          case "end":
            return _context.stop();
        }
      }
    }, _callee);
  }));
  return _makePath.apply(this, arguments);
}

function writeCode(_x2, _x3) {
  return _writeCode.apply(this, arguments);
}

function _writeCode() {
  _writeCode = (0, _asyncToGenerator2["default"])( /*#__PURE__*/_regenerator["default"].mark(function _callee2(filename, code) {
    var pathname, basename, createdPath, codeFilename, codeOutput, rustfmtChild, _iteratorNormalCompletion, _didIteratorError, _iteratorError, _iterator, _step, _value, line, codeHash, existingCode, existingCodeHash;

    return _regenerator["default"].wrap(function _callee2$(_context2) {
      while (1) {
        switch (_context2.prev = _context2.next) {
          case 0:
            pathname = _path["default"].dirname(filename);
            basename = _path["default"].basename(filename);
            _context2.next = 4;
            return makePath(pathname);

          case 4:
            createdPath = _context2.sent;
            codeFilename = _path["default"].join(createdPath, basename);
            codeOutput = code;

            if (!_fs["default"].existsSync(codeFilename)) {
              _context2.next = 55;
              break;
            }

            if (!codeFilename.endsWith(".rs")) {
              _context2.next = 48;
              break;
            }

            // @ts-ignore - we know what this is, right? ;0
            rustfmtChild = _child_process["default"].spawn("rustfmt", ["--emit", "stdout"], {
              stdio: ["pipe", "pipe", "pipe"]
            }); //const exitPromise = onExit(rustfmtChild);

            _context2.next = 12;
            return (0, _stringio.streamWrite)(rustfmtChild.stdin, code);

          case 12:
            _context2.next = 14;
            return (0, _stringio.streamEnd)(rustfmtChild.stdin);

          case 14:
            codeOutput = "";
            _iteratorNormalCompletion = true;
            _didIteratorError = false;
            _context2.prev = 17;
            _iterator = (0, _asyncIterator2["default"])((0, _stringio.chunksToLinesAsync)(rustfmtChild.stdout));

          case 19:
            _context2.next = 21;
            return _iterator.next();

          case 21:
            _step = _context2.sent;
            _iteratorNormalCompletion = _step.done;
            _context2.next = 25;
            return _step.value;

          case 25:
            _value = _context2.sent;

            if (_iteratorNormalCompletion) {
              _context2.next = 32;
              break;
            }

            line = _value;
            codeOutput += line;

          case 29:
            _iteratorNormalCompletion = true;
            _context2.next = 19;
            break;

          case 32:
            _context2.next = 38;
            break;

          case 34:
            _context2.prev = 34;
            _context2.t0 = _context2["catch"](17);
            _didIteratorError = true;
            _iteratorError = _context2.t0;

          case 38:
            _context2.prev = 38;
            _context2.prev = 39;

            if (!(!_iteratorNormalCompletion && _iterator["return"] != null)) {
              _context2.next = 43;
              break;
            }

            _context2.next = 43;
            return _iterator["return"]();

          case 43:
            _context2.prev = 43;

            if (!_didIteratorError) {
              _context2.next = 46;
              break;
            }

            throw _iteratorError;

          case 46:
            return _context2.finish(43);

          case 47:
            return _context2.finish(38);

          case 48:
            codeHash = _xxhash["default"].hash64(Buffer.from(codeOutput), 1234, "base64");
            _context2.next = 51;
            return _fs["default"].promises.readFile(codeFilename);

          case 51:
            existingCode = _context2.sent;
            existingCodeHash = _xxhash["default"].hash64(existingCode, 1234, "base64");

            if (!(codeHash == existingCodeHash)) {
              _context2.next = 55;
              break;
            }

            return _context2.abrupt("return");

          case 55:
            _context2.next = 57;
            return _fs["default"].promises.writeFile(codeFilename, codeOutput);

          case 57:
          case "end":
            return _context2.stop();
        }
      }
    }, _callee2, null, [[17, 34, 38, 48], [39,, 43, 47]]);
  }));
  return _writeCode.apply(this, arguments);
}
//# sourceMappingURL=data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9jb2RlZ2VuL2ZzLnRzIl0sIm5hbWVzIjpbIm1ha2VQYXRoIiwicGF0aFBhcnQiLCJhYnNvbHV0ZVBhdGhOYW1lIiwicGF0aCIsInJlc29sdmUiLCJmcyIsImV4aXN0c1N5bmMiLCJwcm9taXNlcyIsIm1rZGlyIiwicmVjdXJzaXZlIiwid3JpdGVDb2RlIiwiZmlsZW5hbWUiLCJjb2RlIiwicGF0aG5hbWUiLCJkaXJuYW1lIiwiYmFzZW5hbWUiLCJjcmVhdGVkUGF0aCIsImNvZGVGaWxlbmFtZSIsImpvaW4iLCJjb2RlT3V0cHV0IiwiZW5kc1dpdGgiLCJydXN0Zm10Q2hpbGQiLCJjaGlsZFByb2Nlc3MiLCJzcGF3biIsInN0ZGlvIiwic3RkaW4iLCJzdGRvdXQiLCJsaW5lIiwiY29kZUhhc2giLCJYWEhhc2giLCJoYXNoNjQiLCJCdWZmZXIiLCJmcm9tIiwicmVhZEZpbGUiLCJleGlzdGluZ0NvZGUiLCJleGlzdGluZ0NvZGVIYXNoIiwid3JpdGVGaWxlIl0sIm1hcHBpbmdzIjoiOzs7Ozs7Ozs7Ozs7Ozs7O0FBQUE7O0FBTUE7O0FBQ0E7O0FBQ0E7O0FBQ0E7O1NBRXNCQSxROzs7Ozs0RkFBZixpQkFBd0JDLFFBQXhCO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUNDQyxZQUFBQSxnQkFERCxHQUNvQkMsaUJBQUtDLE9BQUwsQ0FBYUgsUUFBYixDQURwQjs7QUFBQSxnQkFFQUksZUFBR0MsVUFBSCxDQUFjSixnQkFBZCxDQUZBO0FBQUE7QUFBQTtBQUFBOztBQUFBO0FBQUEsbUJBR0dHLGVBQUdFLFFBQUgsQ0FBWUMsS0FBWixDQUFrQk4sZ0JBQWxCLEVBQW9DO0FBQUVPLGNBQUFBLFNBQVMsRUFBRTtBQUFiLGFBQXBDLENBSEg7O0FBQUE7QUFBQSw2Q0FLRVAsZ0JBTEY7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUEsRzs7OztTQVFlUSxTOzs7Ozs2RkFBZixrQkFBeUJDLFFBQXpCLEVBQTJDQyxJQUEzQztBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQ0NDLFlBQUFBLFFBREQsR0FDWVYsaUJBQUtXLE9BQUwsQ0FBYUgsUUFBYixDQURaO0FBRUNJLFlBQUFBLFFBRkQsR0FFWVosaUJBQUtZLFFBQUwsQ0FBY0osUUFBZCxDQUZaO0FBQUE7QUFBQSxtQkFHcUJYLFFBQVEsQ0FBQ2EsUUFBRCxDQUg3Qjs7QUFBQTtBQUdDRyxZQUFBQSxXQUhEO0FBSUNDLFlBQUFBLFlBSkQsR0FJZ0JkLGlCQUFLZSxJQUFMLENBQVVGLFdBQVYsRUFBdUJELFFBQXZCLENBSmhCO0FBS0RJLFlBQUFBLFVBTEMsR0FLWVAsSUFMWjs7QUFBQSxpQkFNRFAsZUFBR0MsVUFBSCxDQUFjVyxZQUFkLENBTkM7QUFBQTtBQUFBO0FBQUE7O0FBQUEsaUJBT0NBLFlBQVksQ0FBQ0csUUFBYixDQUFzQixLQUF0QixDQVBEO0FBQUE7QUFBQTtBQUFBOztBQVFEO0FBQ01DLFlBQUFBLFlBVEwsR0FTb0JDLDBCQUFhQyxLQUFiLENBQW1CLFNBQW5CLEVBQThCLENBQUMsUUFBRCxFQUFXLFFBQVgsQ0FBOUIsRUFBb0Q7QUFDdkVDLGNBQUFBLEtBQUssRUFBRSxDQUFDLE1BQUQsRUFBUyxNQUFULEVBQWlCLE1BQWpCO0FBRGdFLGFBQXBELENBVHBCLEVBWUQ7O0FBWkM7QUFBQSxtQkFhSywyQkFBWUgsWUFBWSxDQUFDSSxLQUF6QixFQUFnQ2IsSUFBaEMsQ0FiTDs7QUFBQTtBQUFBO0FBQUEsbUJBY0sseUJBQVVTLFlBQVksQ0FBQ0ksS0FBdkIsQ0FkTDs7QUFBQTtBQWVETixZQUFBQSxVQUFVLEdBQUcsRUFBYjtBQWZDO0FBQUE7QUFBQTtBQUFBLHdEQWdCd0Isa0NBQW1CRSxZQUFZLENBQUNLLE1BQWhDLENBaEJ4Qjs7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBOztBQWdCZ0JDLFlBQUFBLElBaEJoQjtBQWlCQ1IsWUFBQUEsVUFBVSxJQUFJUSxJQUFkOztBQWpCRDtBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQTs7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTtBQUFBO0FBQUE7O0FBQUE7QUFBQTs7QUFBQTtBQUFBOztBQUFBO0FBQUE7QUFBQTtBQUFBOztBQUFBOztBQUFBO0FBQUE7O0FBQUE7QUFBQTs7QUFBQTtBQXFCR0MsWUFBQUEsUUFyQkgsR0FxQmNDLG1CQUFPQyxNQUFQLENBQWNDLE1BQU0sQ0FBQ0MsSUFBUCxDQUFZYixVQUFaLENBQWQsRUFBdUMsSUFBdkMsRUFBNkMsUUFBN0MsQ0FyQmQ7QUFBQTtBQUFBLG1CQXNCd0JkLGVBQUdFLFFBQUgsQ0FBWTBCLFFBQVosQ0FBcUJoQixZQUFyQixDQXRCeEI7O0FBQUE7QUFzQkdpQixZQUFBQSxZQXRCSDtBQXVCR0MsWUFBQUEsZ0JBdkJILEdBdUJzQk4sbUJBQU9DLE1BQVAsQ0FBY0ksWUFBZCxFQUE0QixJQUE1QixFQUFrQyxRQUFsQyxDQXZCdEI7O0FBQUEsa0JBd0JDTixRQUFRLElBQUlPLGdCQXhCYjtBQUFBO0FBQUE7QUFBQTs7QUFBQTs7QUFBQTtBQUFBO0FBQUEsbUJBNEJDOUIsZUFBR0UsUUFBSCxDQUFZNkIsU0FBWixDQUFzQm5CLFlBQXRCLEVBQW9DRSxVQUFwQyxDQTVCRDs7QUFBQTtBQUFBO0FBQUE7QUFBQTtBQUFBO0FBQUE7QUFBQSxHIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHtcbiAgb25FeGl0LFxuICBjaHVua3NUb0xpbmVzQXN5bmMsXG4gIHN0cmVhbVdyaXRlLFxuICBzdHJlYW1FbmQsXG59IGZyb20gXCJAcmF1c2NobWEvc3RyaW5naW9cIjtcbmltcG9ydCBjaGlsZFByb2Nlc3MgZnJvbSBcImNoaWxkX3Byb2Nlc3NcIjtcbmltcG9ydCBmcyBmcm9tIFwiZnNcIjtcbmltcG9ydCBwYXRoIGZyb20gXCJwYXRoXCI7XG5pbXBvcnQgWFhIYXNoIGZyb20gXCJ4eGhhc2hcIjtcblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIG1ha2VQYXRoKHBhdGhQYXJ0OiBzdHJpbmcpOiBQcm9taXNlPHN0cmluZz4ge1xuICBjb25zdCBhYnNvbHV0ZVBhdGhOYW1lID0gcGF0aC5yZXNvbHZlKHBhdGhQYXJ0KTtcbiAgaWYgKCFmcy5leGlzdHNTeW5jKGFic29sdXRlUGF0aE5hbWUpKSB7XG4gICAgYXdhaXQgZnMucHJvbWlzZXMubWtkaXIoYWJzb2x1dGVQYXRoTmFtZSwgeyByZWN1cnNpdmU6IHRydWUgfSk7XG4gIH1cbiAgcmV0dXJuIGFic29sdXRlUGF0aE5hbWU7XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiB3cml0ZUNvZGUoZmlsZW5hbWU6IHN0cmluZywgY29kZTogc3RyaW5nKTogUHJvbWlzZTx2b2lkPiB7XG4gIGNvbnN0IHBhdGhuYW1lID0gcGF0aC5kaXJuYW1lKGZpbGVuYW1lKTtcbiAgY29uc3QgYmFzZW5hbWUgPSBwYXRoLmJhc2VuYW1lKGZpbGVuYW1lKTtcbiAgY29uc3QgY3JlYXRlZFBhdGggPSBhd2FpdCBtYWtlUGF0aChwYXRobmFtZSk7XG4gIGNvbnN0IGNvZGVGaWxlbmFtZSA9IHBhdGguam9pbihjcmVhdGVkUGF0aCwgYmFzZW5hbWUpO1xuICBsZXQgY29kZU91dHB1dCA9IGNvZGU7XG4gIGlmIChmcy5leGlzdHNTeW5jKGNvZGVGaWxlbmFtZSkpIHtcbiAgICBpZiAoY29kZUZpbGVuYW1lLmVuZHNXaXRoKFwiLnJzXCIpKSB7XG4gICAgICAvLyBAdHMtaWdub3JlIC0gd2Uga25vdyB3aGF0IHRoaXMgaXMsIHJpZ2h0PyA7MFxuICAgICAgY29uc3QgcnVzdGZtdENoaWxkID0gY2hpbGRQcm9jZXNzLnNwYXduKFwicnVzdGZtdFwiLCBbXCItLWVtaXRcIiwgXCJzdGRvdXRcIl0sIHtcbiAgICAgICAgc3RkaW86IFtcInBpcGVcIiwgXCJwaXBlXCIsIFwicGlwZVwiXSxcbiAgICAgIH0pO1xuICAgICAgLy9jb25zdCBleGl0UHJvbWlzZSA9IG9uRXhpdChydXN0Zm10Q2hpbGQpO1xuICAgICAgYXdhaXQgc3RyZWFtV3JpdGUocnVzdGZtdENoaWxkLnN0ZGluLCBjb2RlKTtcbiAgICAgIGF3YWl0IHN0cmVhbUVuZChydXN0Zm10Q2hpbGQuc3RkaW4pO1xuICAgICAgY29kZU91dHB1dCA9IFwiXCI7XG4gICAgICBmb3IgYXdhaXQgKGNvbnN0IGxpbmUgb2YgY2h1bmtzVG9MaW5lc0FzeW5jKHJ1c3RmbXRDaGlsZC5zdGRvdXQpKSB7XG4gICAgICAgIGNvZGVPdXRwdXQgKz0gbGluZTtcbiAgICAgIH1cbiAgICAgIC8vYXdhaXQgZXhpdFByb21pc2U7XG4gICAgfVxuICAgIGNvbnN0IGNvZGVIYXNoID0gWFhIYXNoLmhhc2g2NChCdWZmZXIuZnJvbShjb2RlT3V0cHV0KSwgMTIzNCwgXCJiYXNlNjRcIik7XG4gICAgY29uc3QgZXhpc3RpbmdDb2RlID0gYXdhaXQgZnMucHJvbWlzZXMucmVhZEZpbGUoY29kZUZpbGVuYW1lKTtcbiAgICBjb25zdCBleGlzdGluZ0NvZGVIYXNoID0gWFhIYXNoLmhhc2g2NChleGlzdGluZ0NvZGUsIDEyMzQsIFwiYmFzZTY0XCIpO1xuICAgIGlmIChjb2RlSGFzaCA9PSBleGlzdGluZ0NvZGVIYXNoKSB7XG4gICAgICByZXR1cm47XG4gICAgfVxuICB9XG4gIGF3YWl0IGZzLnByb21pc2VzLndyaXRlRmlsZShjb2RlRmlsZW5hbWUsIGNvZGVPdXRwdXQpO1xufVxuIl19