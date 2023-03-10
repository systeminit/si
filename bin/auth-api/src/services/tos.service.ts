import _ from 'lodash';
import { ulid } from "ulidx";
import * as Auth0 from 'auth0';
import { marked } from 'marked';

import { createWorkspace } from "./workspaces.service";
import { User, UserId } from './users.service';

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

const tosAgreements: TosAgreement[] = [];

// could store in db, or just in file-system of this repo...
// will need flexible solution eventually for custom TOS versions, but should be simpl for now
const tosVersions: TosVersions[] = [
  {
    id: '2023-01-01',
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

const LATEST_TOS_VERSION_ID = _.max(_.map(tosVersions, (t) => t.id))!;

export async function getTosVersionById(id: TosVersionId) {
  return _.find(tosVersions, (t) => t.id === id);
}

export async function findLatestAgreedVersionForUser(userId: UserId) {
  const entries = _.filter(tosAgreements, (t) => t.userId === userId);
  const sorted = _.sortBy(entries, (t) => t.tosVersionId);
  return sorted?.[0]?.tosVersionId;
}

export async function findLatestTosForUser(user: User) {
  if (!user.needsTosUpdate) return;
  return getTosVersionById(LATEST_TOS_VERSION_ID);
}

/**
 * populates user's TOS agreement status (do they need to agree to new TOS)
 * */
export async function loadTosStatusForUser(user: User) {
  const latestAgreedVersionId = await findLatestAgreedVersionForUser(user.id);
  user.needsTosUpdate = !latestAgreedVersionId || latestAgreedVersionId !== LATEST_TOS_VERSION_ID;
}

export async function saveTosAgreement(userId: UserId, tosVersionId: TosVersionId, ipAddress: IpAddressString) {
  // do we care if we record multiple agreements for the same version?

  const agreement: TosAgreement = {
    id: ulid(),
    userId,
    tosVersionId,
    ipAddress,
    timestamp: new Date().toISOString(),
  };
  tosAgreements.push(agreement);

  return agreement;
}
