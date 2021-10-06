import { From, Inference } from "si-inference";
import { SiEntity } from "si-entity";
import { VMScript } from "vm2";
export declare enum SecretKind {
    DockerHub = "dockerHub",
    AwsAccessKey = "awsAccessKey",
    HelmRepo = "helmRepo",
    AzureServicePrincipal = "azureServicePrincipal"
}
export interface DecryptedSecret {
    id: string;
    name: string;
    objectType: "credential";
    kind: SecretKind;
    message: Record<string, any>;
}
export interface InferContextEntry {
    entity: SiEntity;
    secret?: Record<string, DecryptedSecret | null>;
}
export declare type InferContext = InferContextEntry[];
export interface EvaluateFromResult {
    inputs: SiEntity[];
    dataResult: DataResult;
    targetEntity?: SiEntity;
}
export declare function evaluateFrom(inference: Inference, targetEntity: SiEntity, context: InferContext): EvaluateFromResult;
export declare type DataObject = {
    entityId: string;
    name?: string;
    properties: Record<string, any>;
};
export declare type DataResult = {
    [system: string]: DataObject[];
};
export declare function populateData(inference: Inference, dataFrom: From, inputs: SiEntity[], dataResult: DataResult): DataResult;
export interface CodeResult {
    code: VMScript;
    if?: VMScript;
}
export declare function getPathFromInference(inference: Inference): string[];
export declare function evaluateInferenceLambda(inference: Inference, targetEntity: SiEntity, context: InferContext): SiEntity;
