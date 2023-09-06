# Preparing Your Changes

This document contains information related to preparing changes for a pull request.

## Commit Message Format

We do not require a particular commit message format of any kind, but we do require that individual commits be descriptive, relative to size and impact.
For example, if a descriptive title covers what the commit does in practice, then an additional description below the title is not required.
However, if the commit has an out-sized impact relative to other commits, its description will need to reflect that.

Reviewers may ask you to amend your commits if they are not descriptive enough.
Since the descriptiveness of a commit is subjective, please feel free to talk to us [on Discord](https://discord.com/invite/system-init) if you have any questions.

### Optional Commit Template

If you would like an optional commit template, see the following:

```text
<present-tense-verb-with-capitalized-first-letter> <everything-else-without-puncutation-at-the-end>

<sentences-in-paragraph-format-or-bullet-points>
```

Here is an example with a paragraph in the description:

```text
Reduce idle memory utilization in starfield-server

With this change, starfield-server no longer waits for acknowledgement
from the BGS API. As soon as the request is successful, the green
thread is dropped, which frees up memory since the task is usually idle
for ~10 seconds or more.
```

Here is another example, but with bullet points in the description:

```text
Replace fallout queue with TES queue

- Replace fallout queue with TES queue for its durability benefits
- Refactor the core test harness to use TES queue
- Build and publish new TES queue Docker images on commits to "main"
```

Finally, here is an example with a more complex description:

```text
Use multi-threaded work queue operations in starfield-server

Iterating through work queue items has historically been sequential in
starfield-server. With this change, rayon is leveraged to boost overall
performance within green threads.

starfield-server changes:
- Replace sequential work queue with rayon parallel iterator

Test harness changes:
- Refactor the core test harness to create an in-line work queue
```

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
