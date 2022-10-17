/*
  vite plugin to make current git branch/hash/etc available
  NOTE - started from from https://github.com/qduld/vite-plugin-git-revision
  but module is not popular/maintained so we can customize a bit here for our own needs
*/

/* eslint-disable @typescript-eslint/no-explicit-any */

import { execSync } from "child_process";
import path from "path";
import { Plugin } from "vite";

interface ViteGitRevisionPlugin {
  // git work tree
  gitWorkTree?: any;
  shortSha?: boolean;
}

const defaultOptions: ViteGitRevisionPlugin = {
  shortSha: true,
};

export default (options: ViteGitRevisionPlugin): Plugin => {
  options = Object.assign(defaultOptions, options || {});

  function runGitCommand(command: string, fallback?: string) {
    const cmd = [
      "git",
      ...(options.gitWorkTree
        ? [
            `--git-dir=${path.join(options.gitWorkTree, ".git")}`,
            `--work-tree=${options.gitWorkTree}`,
          ]
        : []),
      command,
    ].join(" ");
    try {
      const result = execSync(cmd)
        .toString()
        .replace(/[\s\r\n]+$/, "");
      return result;
    } catch (err) {
      if (fallback !== undefined) return fallback;
      throw err;
    }
  }

  // TODO: currently failing on docker builds because the copied files are no longer in a git repo
  // we'll want to do something to make it available, like dump the info to a file...
  const gitBranch = runGitCommand("rev-parse --abbrev-ref HEAD", "unknown");
  const gitSha = runGitCommand(
    `rev-parse ${options.shortSha ? "--short" : ""} HEAD`,
    "unknown",
  );

  return {
    name: "vite:git-revision",
    config(config: any) {
      // these variables will be replaced in the build process
      config.define.__VITE_GIT_BRANCH__ = JSON.stringify(gitBranch);
      config.define.__VITE_GIT_SHA__ = JSON.stringify(gitSha);
    },
  };
};
