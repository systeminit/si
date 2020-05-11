export declare type Associations = BelongsTo;
interface AssociationConstructor {
    typeName: Association["typeName"];
    methodName: Association["methodName"];
    methodArgumentName: BelongsTo["methodArgumentName"];
    fieldName?: Association["fieldName"];
}
export declare class Association {
    typeName: string;
    methodName: string;
    methodArgumentName: string;
    fieldName: string;
    constructor(args: AssociationConstructor);
}
interface BelongsToConstructor extends Omit<AssociationConstructor, "methodName" | "methodArgumentName"> {
    fromFieldPath: BelongsTo["fromFieldPath"];
    methodName?: Association["methodName"];
    methodArgumentName?: Association["methodArgumentName"];
}
export declare class BelongsTo extends Association {
    fromFieldPath: string[];
    constructor(args: BelongsToConstructor);
    kind(): string;
}
interface HasManyConstructor extends Omit<AssociationConstructor, "methodName" | "methodArgumentName" | "fromFieldPath"> {
    fromFieldPath?: HasMany["fromFieldPath"];
    methodName?: Association["methodName"];
    methodArgumentName?: Association["methodArgumentName"];
}
export declare class HasMany extends Association {
    fromFieldPath: string[];
    constructor(args: HasManyConstructor);
    kind(): string;
}
interface HasListConstructor extends Omit<AssociationConstructor, "methodName" | "methodArgumentName"> {
    fromFieldPath: HasList["fromFieldPath"];
    methodName?: Association["methodName"];
    methodArgumentName?: Association["methodArgumentName"];
}
export declare class HasList extends Association {
    fromFieldPath: string[];
    constructor(args: HasListConstructor);
    kind(): string;
}
interface InListConstructor extends Omit<AssociationConstructor, "methodName" | "methodArgumentName" | "fromFieldPath"> {
    toFieldPath: InList["toFieldPath"];
    fromFieldPath?: InList["fromFieldPath"];
    methodName?: Association["methodName"];
    methodArgumentName?: Association["methodArgumentName"];
}
export declare class InList extends Association {
    fromFieldPath: string[];
    toFieldPath: string[];
    constructor(args: InListConstructor);
    kind(): string;
}
export declare class AssociationList {
    associations: Associations[];
    all(): AssociationList["associations"];
    getByFieldName(fieldName: string): Associations;
    getByTypeName(typeName: string): Associations;
    belongsTo(args: BelongsToConstructor): BelongsTo;
    hasMany(args: HasManyConstructor): HasMany;
    hasList(args: HasListConstructor): HasMany;
    inList(args: InListConstructor): HasMany;
}
export {};
