import chai from 'chai';
import chaiSubset from 'chai-subset';
// import chalk from 'chalk';
import nock from 'nock';
import { cleanupInMemoryCache } from '../../src/lib/cache';
import { prisma } from '../../src/main';
import { routesLoaded } from '../../src/routes';

export async function testSuiteBefore() {
  // tools for partial object checking
  chai.use(chaiSubset);

  // disable all http request to anything but localhost
  // note - this stops tracking requests to posthog
  nock.disableNetConnect();
  nock.enableNetConnect('127.0.0.1');

  await prisma.$connect();
  await routesLoaded;

  // TODO: might want to reset the test db between each suite?
  // (meaning truncate all tables)

  // console.log(chalk.blue('testSuiteBefore complete'));
}

export async function testSuiteAfter() {
  // TODO: might want this to be part of a more generic cleanup/shutdown system
  await prisma.$disconnect();
  cleanupInMemoryCache();

  // nockAfterAll();
  // if (globalThis.gc) globalThis.gc();
  // console.log(chalk.blue('testSuiteAfter complete'));
}
