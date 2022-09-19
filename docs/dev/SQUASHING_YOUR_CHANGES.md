# Squashing Your Changes

This document contains an opinionated guide for squashing the commits on your local branch and pushing them to
your corresponding remote branch.

## Disclaimer

The guide does not necessarily reflect present-day best practices and is opinionated.
It is designed for those wanting to get the rebase over with in a clean manner.

## Guide

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