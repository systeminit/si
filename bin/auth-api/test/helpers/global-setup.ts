import { execSync } from 'node:child_process';
import chalk from 'chalk';

import '../../src/init-env';

console.log(chalk.magentaBright('>>> global test env - setup <<<'));

// locally we can assume the test db exists already because it's part of our docker compose setup
// on CI, we may need to do something else to create it

console.log('> Migrating test db');
execSync('pnpm run db:reset');
console.log('> Test db ready');

console.log(chalk.magentaBright('>>> global test env - ready <<<'));
