# Preparing Your Changes

This document contains information related to preparing changes for a pull request.

## Committing

We highly recommend following the [Convential Commits](https://www.conventionalcommits.org/en/v1.0.0/#specification) format when committing changes.
Our prefixes are derived from the official specification as well as the those found in [commitlint](https://github.com/conventional-changelog/commitlint/tree/master/%40commitlint/config-conventional), based on [Angular's commit conventions](https://github.com/angular/angular/blob/master/CONTRIBUTING.md).
When in doubt, use `feat`, `fix`, or `chore`!

Moreover, please sign your commits using `git commit -s`.
You can amend an existing commit with `git commit -s --amend`, if needed.

## Guide: Rebasing Your Changes

This is an opinionated guide for rebasing your local branch with the latest changes from `main`.
It does not necessarily reflect present-day best practices and is designed for those who would like to perform the
aforementioned action(s) without spending too much time thinking about them.

1. Ensure you have [VS Code](https://code.visualstudio.com/) installed.
2. Ensure your local tree is clean and everything is pushed up to the corresponding remote branch.
    1. This will make it easier if we want to see the diff on GitHub later.
3. Open VS Code and create a new terminal within the application.
    1. We will execute this guide's commands in this terminal in order to get `CMD + LEFT CLICK` functionality for files with conflicts.
4. Run `git pull --rebase origin main` to start the process.
    1. If there is at least “conflict area” for that one commit that git cannot figure out, it’ll drop you into interactive rebase mode.
    2. It will keep you in interactive rebase until you have finishing “continuing” through all the commits.
5. Run `git status` to see what is wrong and where there are conflicts.
6. Open all files with conflicts by clicking `CMD + LEFT CLICK` on each one.
7. In each “conflict area” in a given file, you’ll have options (font size is small) at the top to help resolve the conflict(s).
    1. Affected files are marked with a red exclamation point in the VS Code file picker.
    2. In those options, “Current” refers to `HEAD`, which is `main` in our case.
    3. In those same options, “Incoming” refers to changes on our branch.
    4. You can the options or manually intervene to make changes. Sometimes, you may want to accept everything on HEAD or your local branch and just triage manually. Sometimes, you’ll want to not accept anything and manually triage the whole thing. Sometimes you’ll want to do both. It depends!
    5. Finally, it can be useful to have your branch diff open on GitHub to see what you changed before the rebase: `https://github.com/systeminit/si/compare/main...<your-branch>`.
8. Once all conflict areas for “unmerged paths” (files with conflicts) have been resolved, run `git add` with either the entire current working directory and below (`.`) or specific files/directories (e.g. `lib/dal/src lib/sdf-server/src/`) as the next argument(s).
9. Now run `git status` again. The output should indicate that conflicts have been resolved and that we can continue rebasing.
10. If everything looks good in the output, run `git rebase --continue`. You will have an opportunity to amend your commit message here, if desired.
    1. You will not have to necessarily the “human fix this conflict area” process for every commit.
    2. It will only happen for commits with conflict areas.
11. Once the interactive rebase ends (or never even started if there were no conflicts), you should be good to push! Now, run `git push`.
    1. You will likely have to add the `-f/--force` flag since we are overwriting history (technically?) on the remote.
    2. Be careful when using the force flag! Try to push without using the force flag first if you are unsure.
12. You are done! Congratulations!

## Guide: Squashing Your Changes

This is an opinionated guide for squashing the commits on your local branch and pushing them to
your corresponding remote branch.
It does not necessarily reflect present-day best practices and is designed for those who would like to perform the
aforementioned action(s) without spending too much time thinking about them.

1. Ensure your local tree is clean and everything is pushed up to the corresponding remote branch.
    1. This will make it easier if we want to see the diff on GitHub later.
2. Count the numer of commits that you'd like to squash.
    1. Navigating to your branch diff on GitHub can be helpful here: `https://github.com/systeminit/si/compare/main...<your-branch-name>`
3. Run `git reset --soft HEAD~N` where `N` is the name of commits (example: `git reset --soft HEAD~2` where you'd like to squash two commits into one).
4. Run `git status` to see all staged changes from the commits that were soft reset.
5. Now, commit your changes (e.g. `git commit -s`).
6. Finally, run `git push`.
    1. You will likely have to add the `-f/--force` flag since we are overwriting history (technically?) on the remote.
    2. Be careful when using the force flag! Try to push without using the force flag first if you are unsure.
7. You are done! Congratulations!
