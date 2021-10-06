import { SiEntity } from "si-entity";
import { Inference } from "si-inference";
export declare class InferenceError extends Error {
    constructor(message: string);
}
export declare class InvalidTargetPropError extends InferenceError {
    constructor(args: {
        expected: string;
        found: string;
    });
}
export interface ValueTypeErrorConstructor {
    targetEntity: SiEntity;
    targetType: string;
    inference: Inference;
    value: any;
}
export declare class ValueTypeError extends InferenceError {
    constructor(args: ValueTypeErrorConstructor);
}
export declare class UnexpectedInferenceToNameError extends InferenceError {
    constructor(args: ValueTypeErrorConstructor);
}
export interface InvalidObjectKeysErrorConstructor extends ValueTypeErrorConstructor {
    invalidKeys: string[];
    validKeys: string[];
}
export declare class InvalidObjectKeysError extends InferenceError {
    constructor(args: InvalidObjectKeysErrorConstructor);
}
export declare class InvalidToPathForSchemaError extends InferenceError {
    constructor(args: {
        inference: Inference;
        targetEntity: SiEntity;
    });
}
export declare class InvalidFromPathForSchemaError extends InferenceError {
    constructor(args: {
        inference: Inference;
        targetEntity: SiEntity;
        path: string[];
    });
}
