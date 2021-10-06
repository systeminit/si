"use strict";
var __values = (this && this.__values) || function(o) {
    var s = typeof Symbol === "function" && Symbol.iterator, m = s && o[s], i = 0;
    if (m) return m.call(o);
    if (o && typeof o.length === "number") return {
        next: function () {
            if (o && i >= o.length) o = void 0;
            return { value: o && o[i++], done: !o };
        }
    };
    throw new TypeError(s ? "Object is not iterable." : "Symbol.iterator is not defined.");
};
var __read = (this && this.__read) || function (o, n) {
    var m = typeof Symbol === "function" && o[Symbol.iterator];
    if (!m) return o;
    var i = m.call(o), r, ar = [], e;
    try {
        while ((n === void 0 || n-- > 0) && !(r = i.next()).done) ar.push(r.value);
    }
    catch (error) { e = { error: error }; }
    finally {
        try {
            if (r && !r.done && (m = i["return"])) m.call(i);
        }
        finally { if (e) throw e.error; }
    }
    return ar;
};
var __spreadArray = (this && this.__spreadArray) || function (to, from) {
    for (var i = 0, il = from.length, j = to.length; i < il; i++, j++)
        to[j] = from[i];
    return to;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.evaluateInferenceLambda = exports.getPathFromInference = exports.populateData = exports.evaluateFrom = exports.SecretKind = void 0;
var si_entity_1 = require("si-entity");
var si_registry_1 = require("si-registry");
var debug_1 = __importDefault(require("debug"));
var lodash_1 = __importDefault(require("lodash"));
var vm2_1 = require("vm2");
var errors_1 = require("./errors");
var SecretKind;
(function (SecretKind) {
    SecretKind["DockerHub"] = "dockerHub";
    SecretKind["AwsAccessKey"] = "awsAccessKey";
    SecretKind["HelmRepo"] = "helmRepo";
    SecretKind["AzureServicePrincipal"] = "azureServicePrincipal";
})(SecretKind = exports.SecretKind || (exports.SecretKind = {}));
function evaluateFrom(inference, targetEntity, context) {
    var e_1, _a;
    var result = {
        inputs: [],
        dataResult: { baseline: [] },
    };
    var _loop_1 = function (selector) {
        if (selector.entityType) {
            var selected = lodash_1.default.map(lodash_1.default.filter(context, function (c) { return c.entity.entityType == selector.entityType; }), function (c) { return c.entity; });
            populateData(inference, selector.data, selected, result.dataResult);
            result.inputs = lodash_1.default.union(result.inputs, selected);
        }
        if (selector.targetEntity) {
            var selected = [lodash_1.default.cloneDeep(targetEntity)];
            selected[0].isTarget = true;
            populateData(inference, selector.data, selected, result.dataResult);
            result.inputs = lodash_1.default.union(result.inputs, selected);
        }
        if (selector.entityId) {
            var selected = lodash_1.default.find(context, function (c) { return c.entity.id == selector.entityId; });
            if (selected) {
                populateData(inference, selector.data, [selected.entity], result.dataResult);
                result.inputs = lodash_1.default.union(result.inputs, [selected.entity]);
            }
        }
    };
    try {
        for (var _b = __values(inference.from), _c = _b.next(); !_c.done; _c = _b.next()) {
            var selector = _c.value;
            _loop_1(selector);
        }
    }
    catch (e_1_1) { e_1 = { error: e_1_1 }; }
    finally {
        try {
            if (_c && !_c.done && (_a = _b.return)) _a.call(_b);
        }
        finally { if (e_1) throw e_1.error; }
    }
    return result;
}
exports.evaluateFrom = evaluateFrom;
function populateData(inference, dataFrom, inputs, dataResult) {
    var e_2, _a, e_3, _b, e_4, _c;
    //const dataResult: DataResult = { baseline: [] };
    var name = undefined;
    var fromList = [];
    if (lodash_1.default.isArray(dataFrom)) {
        fromList = dataFrom;
    }
    else {
        fromList = [dataFrom];
    }
    for (var x = 0; x < inputs.length; x++) {
        var entity = inputs[x];
        try {
            for (var fromList_1 = (e_2 = void 0, __values(fromList)), fromList_1_1 = fromList_1.next(); !fromList_1_1.done; fromList_1_1 = fromList_1.next()) {
                var fromEntry = fromList_1_1.value;
                if (fromEntry.name) {
                    name = lodash_1.default.cloneDeep(entity.name);
                }
                else {
                    var fromProp = si_registry_1.findProp(__spreadArray([entity.entityType], __read(fromEntry.path)));
                    if (!fromProp) {
                        throw new errors_1.InvalidFromPathForSchemaError({
                            inference: inference,
                            targetEntity: entity,
                            path: fromEntry.path,
                        });
                    }
                    var systemProperties = entity.getPropertyForAllSystems({
                        path: fromEntry.path,
                    });
                    if (systemProperties) {
                        try {
                            for (var _d = (e_3 = void 0, __values(Object.keys(systemProperties))), _e = _d.next(); !_e.done; _e = _d.next()) {
                                var system = _e.value;
                                lodash_1.default.set(dataResult, __spreadArray([system, x, "properties"], __read(fromEntry.path)), systemProperties[system]);
                            }
                        }
                        catch (e_3_1) { e_3 = { error: e_3_1 }; }
                        finally {
                            try {
                                if (_e && !_e.done && (_b = _d.return)) _b.call(_d);
                            }
                            finally { if (e_3) throw e_3.error; }
                        }
                    }
                }
            }
        }
        catch (e_2_1) { e_2 = { error: e_2_1 }; }
        finally {
            try {
                if (fromList_1_1 && !fromList_1_1.done && (_a = fromList_1.return)) _a.call(fromList_1);
            }
            finally { if (e_2) throw e_2.error; }
        }
        try {
            for (var _f = (e_4 = void 0, __values(Object.keys(dataResult))), _g = _f.next(); !_g.done; _g = _f.next()) {
                var system = _g.value;
                if (!dataResult[system][x]) {
                    dataResult[system][x] = {
                        properties: {},
                        entityId: entity.id,
                    };
                }
                else {
                    dataResult[system][x].entityId = entity.id;
                }
                if (name) {
                    dataResult[system][x].name = name;
                }
            }
        }
        catch (e_4_1) { e_4 = { error: e_4_1 }; }
        finally {
            try {
                if (_g && !_g.done && (_c = _f.return)) _c.call(_f);
            }
            finally { if (e_4) throw e_4.error; }
        }
    }
    return dataResult;
}
exports.populateData = populateData;
function createVm(system, data) {
    var debug = debug_1.default("cyclone:inference:lambda");
    var forEachEntity = function forEachEntity(callback) {
        var e_5, _a;
        try {
            for (var data_1 = __values(data), data_1_1 = data_1.next(); !data_1_1.done; data_1_1 = data_1.next()) {
                var e = data_1_1.value;
                callback(e);
            }
        }
        catch (e_5_1) { e_5 = { error: e_5_1 }; }
        finally {
            try {
                if (data_1_1 && !data_1_1.done && (_a = data_1.return)) _a.call(data_1);
            }
            finally { if (e_5) throw e_5.error; }
        }
    };
    var firstEntity = data[0] || {};
    var vm = new vm2_1.VM({
        timeout: 2000,
        sandbox: {
            debug: debug,
            _: lodash_1.default,
            system: system,
            data: data,
            forEachEntity: forEachEntity,
            firstEntity: firstEntity,
        },
        eval: false,
        wasm: false,
        fixAsync: true,
    });
    return vm;
}
function getTargetProp(inference, targetEntity) {
    var targetProp = undefined;
    if (inference.to.path) {
        targetProp = si_registry_1.findProp(__spreadArray([targetEntity.entityType], __read(inference.to.path)));
    }
    else if (inference.to.name) {
        targetProp = "name";
    }
    return targetProp;
}
function createProvenance(_a) {
    var inputs = _a.inputs, inference = _a.inference;
    var provenanceContext = lodash_1.default.map(inputs, function (e) {
        return { id: e.id, entityType: e.entityType };
    });
    var provenance = {
        context: provenanceContext,
        inference: inference,
    };
    return provenance;
}
function compileCode(inference) {
    var code = new vm2_1.VMScript(inference.code);
    code.compile();
    var result = { code: code };
    if (inference.if) {
        var ifCode = new vm2_1.VMScript(inference.if);
        ifCode.compile();
        result.if = ifCode;
    }
    return result;
}
function getPathFromInference(inference) {
    if (inference.to.path) {
        if (inference.to.extraPath) {
            var newPath = lodash_1.default.cloneDeep(inference.to.path);
            newPath.push.apply(newPath, __spreadArray([], __read(inference.to.extraPath)));
            return newPath;
        }
        return inference.to.path;
    }
}
exports.getPathFromInference = getPathFromInference;
function setValueOnTargetEntity(args) {
    if (lodash_1.default.isUndefined(args.targetProp)) {
        throw new errors_1.InvalidToPathForSchemaError({
            inference: args.inference,
            targetEntity: args.targetEntity,
        });
    }
    else if (args.targetProp == "name") {
        setNameOnTargetEntity(args);
    }
    else if (args.targetProp.type == "string") {
        setStringOnTargetEntity(args);
    }
    else if (args.targetProp.type == "number") {
        setNumberOnTargetEntity(args);
    }
    else if (args.targetProp.type == "boolean") {
        setBooleanOnTargetEntity(args);
    }
    else if (args.targetProp.type == "object") {
        setObjectOnTargetEntity(args);
    }
    else if (args.targetProp.type == "map") {
        setMapOnTargetEntity(args);
    }
    else if (args.targetProp.type == "array") {
        setArrayOnTargetEntity(args);
    }
    args.targetEntity.updateFromOps({
        inference: args.inference,
        setOps: args.setOps,
    });
    return args.targetEntity;
}
function setArrayOnTargetEntity(_a) {
    var value = _a.value, inputs = _a.inputs, inference = _a.inference, targetEntity = _a.targetEntity, targetProp = _a.targetProp, system = _a.system, setOps = _a.setOps;
    if (targetProp != "name") {
        if (targetProp.type == "array") {
            if (lodash_1.default.isArray(value)) {
                var nextIndex = targetEntity.nextIndex(inference.to.path);
                for (var x = 0; x < value.length; x++) {
                    var index = x + nextIndex;
                    var newInference = lodash_1.default.cloneDeep(inference);
                    if (newInference.to.path) {
                        if (newInference.to.extraPath) {
                            newInference.to.extraPath.push("" + index);
                        }
                        else {
                            newInference.to.extraPath = ["" + index];
                        }
                        var newTargetProp = targetProp.itemProperty;
                        // @ts-ignore
                        var newValue = value[index];
                        if (newTargetProp && newValue) {
                            setValueOnTargetEntity({
                                value: newValue,
                                inputs: inputs,
                                inference: newInference,
                                targetEntity: targetEntity,
                                targetProp: newTargetProp,
                                system: system,
                                setOps: setOps,
                            });
                        }
                    }
                    else {
                        throw new errors_1.UnexpectedInferenceToNameError({
                            targetEntity: targetEntity,
                            targetType: "array",
                            inference: inference,
                            value: value,
                        });
                    }
                }
            }
            else {
                throw new errors_1.ValueTypeError({
                    targetEntity: targetEntity,
                    targetType: "array",
                    inference: inference,
                    value: value,
                });
            }
        }
        else {
            throw new errors_1.InvalidTargetPropError({
                expected: "array",
                found: targetProp.type,
            });
        }
    }
    else {
        throw new errors_1.InvalidTargetPropError({ expected: "array", found: "name" });
    }
    return targetEntity;
}
function setMapOnTargetEntity(_a) {
    var e_6, _b;
    var value = _a.value, inputs = _a.inputs, inference = _a.inference, targetEntity = _a.targetEntity, targetProp = _a.targetProp, system = _a.system, setOps = _a.setOps;
    if (targetProp != "name") {
        if (targetProp.type == "map") {
            if (lodash_1.default.isObject(value)) {
                var newKeys = Object.keys(value);
                try {
                    for (var newKeys_1 = __values(newKeys), newKeys_1_1 = newKeys_1.next(); !newKeys_1_1.done; newKeys_1_1 = newKeys_1.next()) {
                        var key = newKeys_1_1.value;
                        var newInference = lodash_1.default.cloneDeep(inference);
                        if (newInference.to.path) {
                            if (newInference.to.extraPath) {
                                newInference.to.extraPath.push(key);
                            }
                            else {
                                newInference.to.extraPath = [key];
                            }
                            var newTargetProp = targetProp.valueProperty;
                            // @ts-ignore
                            var newValue = value[key];
                            if (newTargetProp && newValue) {
                                setValueOnTargetEntity({
                                    value: newValue,
                                    inputs: inputs,
                                    inference: newInference,
                                    targetEntity: targetEntity,
                                    targetProp: newTargetProp,
                                    system: system,
                                    setOps: setOps,
                                });
                            }
                        }
                        else {
                            throw new errors_1.UnexpectedInferenceToNameError({
                                targetEntity: targetEntity,
                                targetType: "map",
                                inference: inference,
                                value: value,
                            });
                        }
                    }
                }
                catch (e_6_1) { e_6 = { error: e_6_1 }; }
                finally {
                    try {
                        if (newKeys_1_1 && !newKeys_1_1.done && (_b = newKeys_1.return)) _b.call(newKeys_1);
                    }
                    finally { if (e_6) throw e_6.error; }
                }
            }
            else {
                throw new errors_1.ValueTypeError({
                    targetEntity: targetEntity,
                    targetType: "map",
                    inference: inference,
                    value: value,
                });
            }
        }
        else {
            throw new errors_1.InvalidTargetPropError({
                expected: "map",
                found: targetProp.type,
            });
        }
    }
    else {
        throw new errors_1.InvalidTargetPropError({ expected: "map", found: "name" });
    }
    return targetEntity;
}
function setObjectOnTargetEntity(_a) {
    var e_7, _b;
    var value = _a.value, inputs = _a.inputs, inference = _a.inference, targetEntity = _a.targetEntity, targetProp = _a.targetProp, system = _a.system, setOps = _a.setOps;
    if (targetProp != "name") {
        if (targetProp.type == "object") {
            if (lodash_1.default.isObject(value)) {
                var newKeys = Object.keys(value);
                var validKeys = lodash_1.default.map(targetProp.properties, function (p) { return p.name; });
                var invalidKeys = lodash_1.default.difference(newKeys, validKeys);
                if (invalidKeys.length != 0) {
                    throw new errors_1.InvalidObjectKeysError({
                        targetEntity: targetEntity,
                        targetType: "object",
                        inference: inference,
                        value: value,
                        invalidKeys: invalidKeys,
                        validKeys: validKeys,
                    });
                }
                var _loop_2 = function (key) {
                    var newInference = lodash_1.default.cloneDeep(inference);
                    if (newInference.to.path) {
                        if (newInference.to.extraPath) {
                            newInference.to.extraPath.push(key);
                        }
                        else {
                            newInference.to.extraPath = [key];
                        }
                        // @ts-ignore
                        var newValue = value[key];
                        var newTargetProp = lodash_1.default.find(targetProp.properties, function (p) { return p.name == key; });
                        if (newTargetProp && newValue) {
                            setValueOnTargetEntity({
                                value: newValue,
                                inputs: inputs,
                                inference: newInference,
                                targetEntity: targetEntity,
                                targetProp: newTargetProp,
                                system: system,
                                setOps: setOps,
                            });
                        }
                    }
                    else {
                        throw new errors_1.UnexpectedInferenceToNameError({
                            targetEntity: targetEntity,
                            targetType: "object",
                            inference: inference,
                            value: value,
                        });
                    }
                };
                try {
                    for (var newKeys_2 = __values(newKeys), newKeys_2_1 = newKeys_2.next(); !newKeys_2_1.done; newKeys_2_1 = newKeys_2.next()) {
                        var key = newKeys_2_1.value;
                        _loop_2(key);
                    }
                }
                catch (e_7_1) { e_7 = { error: e_7_1 }; }
                finally {
                    try {
                        if (newKeys_2_1 && !newKeys_2_1.done && (_b = newKeys_2.return)) _b.call(newKeys_2);
                    }
                    finally { if (e_7) throw e_7.error; }
                }
            }
            else {
                throw new errors_1.ValueTypeError({
                    targetEntity: targetEntity,
                    targetType: "object",
                    inference: inference,
                    value: value,
                });
            }
        }
        else {
            throw new errors_1.InvalidTargetPropError({
                expected: "object",
                found: targetProp.type,
            });
        }
    }
    else {
        throw new errors_1.InvalidTargetPropError({ expected: "object", found: "name" });
    }
    return targetEntity;
}
function setStringOnTargetEntity(_a) {
    var value = _a.value, inputs = _a.inputs, inference = _a.inference, targetEntity = _a.targetEntity, system = _a.system, setOps = _a.setOps;
    if (lodash_1.default.isString(value)) {
        var provenance = createProvenance({ inputs: inputs, inference: inference });
        setOps.push({
            op: si_entity_1.OpType.Set,
            source: si_entity_1.OpSource.Inferred,
            path: getPathFromInference(inference),
            value: value,
            system: system,
            provenance: provenance,
        });
    }
    else {
        throw new errors_1.ValueTypeError({
            targetEntity: targetEntity,
            targetType: "string",
            inference: inference,
            value: value,
        });
    }
    return targetEntity;
}
function setNumberOnTargetEntity(_a) {
    var value = _a.value, inputs = _a.inputs, inference = _a.inference, targetEntity = _a.targetEntity, system = _a.system, setOps = _a.setOps;
    var provenance = createProvenance({ inputs: inputs, inference: inference });
    if (lodash_1.default.isString(value)) {
        if (lodash_1.default.isNaN(lodash_1.default.toNumber(value))) {
            throw new errors_1.ValueTypeError({
                targetEntity: targetEntity,
                targetType: "number",
                inference: inference,
                value: value,
            });
        }
        setOps.push({
            op: si_entity_1.OpType.Set,
            source: si_entity_1.OpSource.Inferred,
            path: getPathFromInference(inference),
            value: value,
            system: system,
            provenance: provenance,
        });
    }
    else if (lodash_1.default.isNumber(value)) {
        setOps.push({
            op: si_entity_1.OpType.Set,
            source: si_entity_1.OpSource.Inferred,
            path: getPathFromInference(inference),
            value: "" + value,
            system: system,
            provenance: provenance,
        });
    }
    else {
        throw new errors_1.ValueTypeError({
            targetEntity: targetEntity,
            targetType: "string",
            inference: inference,
            value: value,
        });
    }
    return targetEntity;
}
function setBooleanOnTargetEntity(_a) {
    var value = _a.value, inputs = _a.inputs, inference = _a.inference, targetEntity = _a.targetEntity, system = _a.system, setOps = _a.setOps;
    if (lodash_1.default.isBoolean(value)) {
        var provenance = createProvenance({ inputs: inputs, inference: inference });
        setOps.push({
            op: si_entity_1.OpType.Set,
            source: si_entity_1.OpSource.Inferred,
            path: getPathFromInference(inference),
            value: value,
            system: system,
            provenance: provenance,
        });
    }
    else {
        throw new errors_1.ValueTypeError({
            targetEntity: targetEntity,
            targetType: "boolean",
            inference: inference,
            value: value,
        });
    }
    return targetEntity;
}
function setNameOnTargetEntity(_a) {
    var targetEntity = _a.targetEntity, inference = _a.inference, value = _a.value;
    if (lodash_1.default.isString(value)) {
        if (targetEntity.name.startsWith("si-")) {
            targetEntity.name = value;
        }
    }
    else {
        throw new errors_1.ValueTypeError({
            targetEntity: targetEntity,
            targetType: "string",
            inference: inference,
            value: value,
        });
    }
    return targetEntity;
}
function evaluateInferenceLambda(inference, targetEntity, context) {
    var e_8, _a;
    var debug = debug_1.default("cyclone:inference:lambda");
    var _b = evaluateFrom(inference, targetEntity, context), inputs = _b.inputs, dataResult = _b.dataResult;
    var targetProp = getTargetProp(inference, targetEntity);
    var compiled = compileCode(inference);
    try {
        for (var _c = __values(Object.keys(dataResult)), _d = _c.next(); !_d.done; _d = _c.next()) {
            var system = _d.value;
            var data = dataResult[system];
            var vm = createVm(system, data);
            if (compiled.if) {
                var ifResult = vm.run(compiled.if);
                if (!ifResult) {
                    debug("lambda has an if condition that returned false for system " + system + ". Existing inference will be removed, and no new values will be set.");
                    targetEntity.updateFromOps({ inference: inference, setOps: [] });
                    return;
                }
            }
            var value = vm.run(compiled.code);
            if (lodash_1.default.isUndefined(value)) {
                debug("lambda returned undefined for system " + system + ". Existing inference will be removed", {
                    inputs: inputs,
                    data: data,
                });
                targetEntity.updateFromOps({ inference: inference, setOps: [] });
            }
            else {
                setValueOnTargetEntity({
                    targetEntity: targetEntity,
                    targetProp: targetProp,
                    inference: inference,
                    inputs: inputs,
                    value: value,
                    system: system,
                    setOps: [],
                });
            }
        }
    }
    catch (e_8_1) { e_8 = { error: e_8_1 }; }
    finally {
        try {
            if (_d && !_d.done && (_a = _c.return)) _a.call(_c);
        }
        finally { if (e_8) throw e_8.error; }
    }
    return targetEntity;
}
exports.evaluateInferenceLambda = evaluateInferenceLambda;
