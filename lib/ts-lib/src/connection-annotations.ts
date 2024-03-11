import * as _ from "lodash-es";

export type ConnectionAnnotation = {
  tokens: string[];
};

export function parseConnectionAnnotation(annotation: string): string[] {
  let token = annotation;
  const typeArray = [];

  do {
    const [_, newAnnotation, tail] = token.match(/^([\w ]+)(?:<(.+)>)?$/) ?? [];

    if (!newAnnotation) {
      throw new Error(`Couldn't parse connection annotation "${annotation}"`);
    }

    typeArray.push(newAnnotation.toLowerCase().trim());

    if (!tail) break;

    token = tail;
  } while (token != null);

  return typeArray;
}

export function connectionAnnotationFitsReference(
  { tokens: targetCa }: ConnectionAnnotation,
  { tokens: referenceCa }: ConnectionAnnotation,
) {
  // a fitting target annotation is either the same as the reference one or a supertype thereof
  const lowerTargetCa = _.map(targetCa, (a) => a.toLowerCase());
  const lowerReferenceCa = _.map(referenceCa, (a) => a.toLowerCase());

  return lowerTargetCa.length >= lowerReferenceCa.length
    && _.isEqual(lowerTargetCa.slice(-lowerReferenceCa.length), lowerReferenceCa);
}
