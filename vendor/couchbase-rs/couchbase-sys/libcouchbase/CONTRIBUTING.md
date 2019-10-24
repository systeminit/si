# Contributing

In addition to filing bugs, you may contribute by submitting patches to fix bugs in the library. Contributions may be
submitting to <http://review.couchbase.com>.  We use Gerrit as our code review system - and thus submitting a change
would require an account there. Note that pull requests will not be ignored but will be responded to much quicker once
they are converted into Gerrit.

For something to be accepted into the codebase, it must be formatted properly and have undergone proper testing. While
there are no formatting guidelines per se, the code should look similar to the existing code within the library.

## Branches and Tags

Released versions of the library are marked as annotated tags inside the repository.

* The `master` branch represents the mainline branch. The master branch typically consists of content going into the
  next release.

## Contributing Patches

If you wish to contribute a new feature or a bug fix to the library, try to follow the following guidelines to help
ensure your change gets merged upstream.

### Before you begin

For any code change, ensure the new code you write looks similar to the code surrounding it. We have no strict code
style policies, but do request that your code stand out as little as possible from its surrounding neighborhood (unless
of course your change is stylistic in nature).

If your change is going to involve a substantial amount of time or effort, please attempt to discuss it with the project
developers first who will provide assistance and direction where possible.

Additionally, note that the library uses C89 (AKA "ANSI C") with some extensions that are known to work on both GCC and
Visual Studio for `.c` files, and C++11 for `.cc` files. Please ensure your code conforms to this, unless the new code
is specific to a given platform.

#### For new features

Ensure the feature you are adding does not already exist, and think about how this feature may be useful for other
users. In general less intrusive changes are more likely to be accepted.

#### For fixing bugs

Ensure the bug you are fixing is actually a bug (and not a usage) error, and that it has not been fixed in a more recent
version. Please read the release notes as well as the issue tracker to see a list of open and resolved issues.

### Code Review

#### Signing up on Gerrit

Everything that is merged into the library goes through a code review process.  The code review process is done via
[Gerrit](http://review.couchbase.org) (Unfortunately we cannot merge pull requests, though it is fairly simple to
convert a pull request to gerrit, as seen later. If you know a way to integrate pull requests with Gerrit, please let us
know).

To sign up for a gerrit account, go to http://review.couchbase.org and click on the _Register_ link at the top
right. Once you've signed in you will need to sign the CLA (Contributor License Agreement) by going you your gerrit
account page and selecting the _Agreements_ link on the left. When you've done that, be sure to notify us in IRC (at
_#libcouchbase_) and/or send an email to **matt**AT**couchbase**DOTCOM as you will require manual approval before being
able to submit a request for change.

Once approved, you should add your public SSH key to gerrit.

#### Setting up your fork with Gerrit

Assuming you have a repository created like so:

```
$ git clone git://github.com/couchbase/libcouchbase.git
```

you can simply perform two simple steps to get started with gerrit:

```
$ git remote add gerrit ssh://${USERNAME}@review.couchbase.org:29418/libcouchbase
$ scp -P 29418 ${USERNAME}@review.couchbase.org:hooks/commit-msg .git/hooks
$ chmod a+x .git/hooks/commit-msg
```

The last change is required for annotating each commit message with a special header known as `Change-Id`. This allows
Gerrit to group together different revisions of the same patch.

#### Pushing a changeset

Now that you have your change and a gerrit account to push to, you need to upload the change for review. To do so,
invoke the following incantation:

```
$ git push gerrit HEAD:refs/for/master
```

Where `gerrit` is the name of the _remote_ added earlier. You may encounter some errors when pushing. The most common
are:

* "You are not authorized to push to this repository". You will get this if your account has not yet been approved.
* "Missing Change-Id". You need to install the `commit-msg` hook as described above.  Note that even once you do this,
  you will need to ensure that any prior commits already have this header - this may be done by doing an interactive
  rebase (e.g.  `git rebase -i origin/master` and selecting `reword` for all the commits; which will automatically fill
  in the Change-Id).


Once you've pushed your changeset you can add people to review. Currently these are:

* Sergey Avseyev
* Brett Lawson
* Ellis Breen
