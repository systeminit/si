export type Associations = BelongsTo;

interface AssociationConstructor {
  typeName: Association["typeName"];
  methodName: Association["methodName"];
  methodArgumentName: BelongsTo["methodArgumentName"];
  fieldName?: Association["fieldName"];
}

export class Association {
  typeName: string;
  methodName: string;
  methodArgumentName: string;
  fieldName: string;

  constructor(args: AssociationConstructor) {
    this.typeName = args.typeName;
    this.methodName = args.methodName;
    this.methodArgumentName = args.methodArgumentName;
    if (args.fieldName == undefined) {
      this.fieldName = args.typeName;
    } else {
      this.fieldName = args.fieldName;
    }
  }
}

interface BelongsToConstructor
  extends Omit<AssociationConstructor, "methodName" | "methodArgumentName"> {
  fromFieldPath: BelongsTo["fromFieldPath"];
  methodName?: Association["methodName"];
  methodArgumentName?: Association["methodArgumentName"];
}

export class BelongsTo extends Association {
  fromFieldPath: string[];

  constructor(args: BelongsToConstructor) {
    if (args.methodName == undefined) {
      args.methodName = "get";
    }
    if (args.methodArgumentName == undefined) {
      args.methodArgumentName = `${args.typeName}Id`;
    }
    super(args as AssociationConstructor);
    this.fromFieldPath = this.fromFieldPath;
  }
}

export class AssociationList {
  associations: Associations[] = [];

  all(): AssociationList["associations"] {
    return this.associations;
  }

  belongsTo(args: BelongsToConstructor): BelongsTo {
    const assoc = new BelongsTo(args);
    this.associations.push(assoc);
    return assoc;
  }
}
