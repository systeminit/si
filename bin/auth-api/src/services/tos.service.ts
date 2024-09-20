import { ulid } from "ulidx";

import { PrismaClient, User } from "@prisma/client";
import { TosVersion } from "@si/ts-lib/src/terms-of-service";
import { UserId } from "./users.service";
import { tracker } from "../lib/tracker";
import { posthog } from "../lib/posthog";

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

export async function findLatestTosForUser(user: User) {
  const saasReleaseEnabled = await posthog.isFeatureEnabled("auth_portal_saas_release", user.id);

  const latestTosVersion = saasReleaseEnabled ? TosVersion.v20240919 : TosVersion.v20230330;

  return latestTosVersion;
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
  tracker.trackEvent(user, "legal_agreed", { version: tosVersionId });

  return newAgreement;
}
