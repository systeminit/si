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
  // If the length is longer than 1, we assume you have already parsed the
  // connection annotations. Otherwise, we need to parse them.
  if (targetCa.length === 1) {
    // @ts-ignore ignoring
    targetCa = parseConnectionAnnotation(targetCa[0]);
  }
  if (referenceCa.length === 1) {
    // @ts-ignore ignoring
    referenceCa = parseConnectionAnnotation(referenceCa[0]);
  }
  // a fitting target annotation is either the same as the reference one or a supertype thereof
  const lowerTargetCa = _.map(targetCa, (a: string) => a.toLowerCase());
  const lowerReferenceCa = _.map(referenceCa, (a: string) => a.toLowerCase());

  return lowerTargetCa.length >= lowerReferenceCa.length
    && _.isEqual(lowerTargetCa.slice(-lowerReferenceCa.length), lowerReferenceCa);
}
