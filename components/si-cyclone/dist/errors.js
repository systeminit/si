"use strict";
var __extends = (this && this.__extends) || (function () {
    var extendStatics = function (d, b) {
        extendStatics = Object.setPrototypeOf ||
            ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
            function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
        return extendStatics(d, b);
    };
    return function (d, b) {
        if (typeof b !== "function" && b !== null)
            throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.InvalidFromPathForSchemaError = exports.InvalidToPathForSchemaError = exports.InvalidObjectKeysError = exports.UnexpectedInferenceToNameError = exports.ValueTypeError = exports.InvalidTargetPropError = exports.InferenceError = void 0;
var _1 = require(".");
var InferenceError = /** @class */ (function (_super) {
    __extends(InferenceError, _super);
    function InferenceError(message) {
        var _this = _super.call(this, message) || this;
        _this.name = "InferenceError";
        return _this;
    }
    return InferenceError;
}(Error));
exports.InferenceError = InferenceError;
var InvalidTargetPropError = /** @class */ (function (_super) {
    __extends(InvalidTargetPropError, _super);
    function InvalidTargetPropError(args) {
        var _this = this;
        var message = "Invalid target prop type in value function; expected '" + args.expected + "' found '" + args.found + "'";
        _this = _super.call(this, message) || this;
        _this.name = "InvalidTargetPropError";
        return _this;
    }
    return InvalidTargetPropError;
}(InferenceError));
exports.InvalidTargetPropError = InvalidTargetPropError;
var ValueTypeError = /** @class */ (function (_super) {
    __extends(ValueTypeError, _super);
    function ValueTypeError(args) {
        var _this = this;
        var message;
        var path = _1.getPathFromInference(args.inference);
        if (args.inference.to.path) {
            message = "Inference '" + args.inference.name + "' for " + args.targetEntity.entityType + "[" + path.join(", ") + "] expects a " + args.targetType + "; received " + JSON.stringify(args.value);
        }
        else {
            message = "Inference '" + args.inference.name + "' for " + args.targetEntity.entityType + ".name expects a " + args.targetType + "; received " + JSON.stringify(args.value);
        }
        _this = _super.call(this, message) || this;
        _this.name = "ValueTypeError";
        return _this;
    }
    return ValueTypeError;
}(InferenceError));
exports.ValueTypeError = ValueTypeError;
var UnexpectedInferenceToNameError = /** @class */ (function (_super) {
    __extends(UnexpectedInferenceToNameError, _super);
    function UnexpectedInferenceToNameError(args) {
        var _this = this;
        var message = "Inference '" + args.inference.name + "' for " + args.targetEntity.entityType + "[" + args.inference.to.path.join(", ") + "] expected a 'to' path, but instead found a name";
        _this = _super.call(this, message) || this;
        _this.name = "UnexpectedInferenceToNameError";
        return _this;
    }
    return UnexpectedInferenceToNameError;
}(InferenceError));
exports.UnexpectedInferenceToNameError = UnexpectedInferenceToNameError;
var InvalidObjectKeysError = /** @class */ (function (_super) {
    __extends(InvalidObjectKeysError, _super);
    function InvalidObjectKeysError(args) {
        var _a;
        var _this = this;
        var message = "Inference '" + args.inference.name + "' for object " + args.targetEntity.entityType + "[" + ((_a = args.inference.to.path) === null || _a === void 0 ? void 0 : _a.join(", ")) + "] has invalid keys: " + args.invalidKeys.join(", ") + " (valid keys: " + args.validKeys.join(", ") + ")";
        _this = _super.call(this, message) || this;
        _this.name = "ValueTypeError";
        return _this;
    }
    return InvalidObjectKeysError;
}(InferenceError));
exports.InvalidObjectKeysError = InvalidObjectKeysError;
var InvalidToPathForSchemaError = /** @class */ (function (_super) {
    __extends(InvalidToPathForSchemaError, _super);
    function InvalidToPathForSchemaError(args) {
        var _this = this;
        var path = _1.getPathFromInference(args.inference);
        var message = "Inference '" + args.inference.name + "' for object " + args.targetEntity.entityType + " has an invalid 'to' path: [" + path.join(", ") + "]; inference and schema must match!";
        _this = _super.call(this, message) || this;
        _this.name = "InvalidToPathForSchemaError";
        return _this;
    }
    return InvalidToPathForSchemaError;
}(InferenceError));
exports.InvalidToPathForSchemaError = InvalidToPathForSchemaError;
var InvalidFromPathForSchemaError = /** @class */ (function (_super) {
    __extends(InvalidFromPathForSchemaError, _super);
    function InvalidFromPathForSchemaError(args) {
        var _this = this;
        var message = "Inference '" + args.inference.name + "' for object " + args.targetEntity.entityType + " has an invalid 'from' path: [" + args.path.join(", ") + "]; inference and schema must match!";
        _this = _super.call(this, message) || this;
        _this.name = "InvalidFromPathForSchemaError";
        return _this;
    }
    return InvalidFromPathForSchemaError;
}(InferenceError));
exports.InvalidFromPathForSchemaError = InvalidFromPathForSchemaError;
