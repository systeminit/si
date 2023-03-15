import _ from 'lodash';
import { ulid } from "ulidx";
import { marked } from 'marked';

import { PrismaClient } from '@prisma/client';
import { UserId, UserWithTosStatus } from './users.service';

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

// could store in db, or just in file-system of this repo...
// will need flexible solution eventually for custom TOS versions, but should be simple for now
const tosVersions: TosVersions[] = [
  {
    id: '2023-03-02',
    pdfUrl: 'https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf',
    markdown: `
# Terms of service (2023-01-01)

old terms

## Subheading

blah blah blah`,
  },
  {
    id: '2023-03-09',
    pdfUrl: 'https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf',
    markdown: `
# Terms of service (2023-03-09)

latest TOS version

## Subheading

blah blah blah`,
  },
];

// hacky... just generating html once for each. Will do this somewhere else once we have real data
tosVersions.forEach((tos) => {
  tos.html = marked.parse(tos.markdown);
});

export const LATEST_TOS_VERSION_ID = _.max(_.map(tosVersions, (t) => t.id))!;

export async function getTosVersionById(id: TosVersionId) {
  return _.find(tosVersions, (t) => t.id === id);
}

export async function findLatestTosForUser(user: UserWithTosStatus) {
  // eventually this logic may be more complex...
  if (user.agreedTosVersion === LATEST_TOS_VERSION_ID) return null;
  return getTosVersionById(LATEST_TOS_VERSION_ID);
}

export async function saveTosAgreement(
  userId: UserId,
  tosVersionId: TosVersionId,
  ipAddress: IpAddressString,
) {
  // do we care about recording multiple agreements to the same tos version?
  const newAgreement = await prisma.tosAgreement.create({
    data: {
      id: ulid(),
      userId,
      tosVersionId,
      ipAddress,
      timestamp: new Date().toISOString(),
    },
  });

  return newAgreement;
}
