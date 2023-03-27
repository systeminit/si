import { ulid } from "ulidx";

import { PrismaClient, User } from '@prisma/client';
import { UserId, UserWithTosStatus } from './users.service';
import { tracker } from "../lib/tracker";

const prisma = new PrismaClient();

export type TosAgreement = {
  id: string; // primary key, probably not really used for anything
  userId: UserId;
  tosVersionId: string; // see note below about these being sortable
  timestamp: ISODateTimeString;
  ipAddress: IpAddressString;
};

export type TosVersionId = string;

export type TosVersions = {
  // not ULIDs - these should be sortable, usually just an ISO date string
  // can add version suffix if we have multiple versions on same day
  // will need to think through how custom tos works when we get there...
  id: TosVersionId;
  pdfUrl: string;
  markdown: string;
  html?: string;
};

export const LATEST_TOS_VERSION_ID = '2023-03-30';

export async function findLatestTosForUser(user: UserWithTosStatus) {
  // eventually this logic may be more complex...
  if (user.agreedTosVersion === LATEST_TOS_VERSION_ID) return null;
  return LATEST_TOS_VERSION_ID;
}

export async function saveTosAgreement(
  user: User,
  tosVersionId: TosVersionId,
  ipAddress: IpAddressString,
) {
  // do we care about recording multiple agreements to the same tos version?
  const newAgreement = await prisma.tosAgreement.create({
    data: {
      id: ulid(),
      userId: user.id,
      tosVersionId,
      ipAddress,
      timestamp: new Date().toISOString(),
    },
  });
  tracker.trackEvent(user, 'legal_agreed', { version: tosVersionId });

  return newAgreement;
}
