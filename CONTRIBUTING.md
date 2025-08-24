# Contributing to the System Initiative software

The System Initiative software is managed by System Initiative, and we eagerly welcome contributions from the community. There are many ways to get involved!

## Provide Feedback

You may find things that can be improved as you use System Initiative (SI).
You can help by [reporting Github issues](https://github.com/systeminit/si/issues) when:

* SI crashes, or you encounter a bug that can only be resolved by re-installing or restarting SI
* An error occurs that is unrecoverable, causes data integrity or data loss issues, or generally prevents you from being successful with SI
* A new feature or an enhancement to an existing feature will improve the utility or usability of SI

Before creating a new issue, please confirm that an existing issue doesn't already exist.

## Participate in the Community

You can engage with the System Initiative community by:

* Helping other users on [Discord](https://discord.com/invite/system-init)
* Improving documentation within the repository or on the [System Initiative docs](https://docs.systeminit.com/) site
* Participating in general discussions about technology and more

## Contributing Code

You can contribute to the System Initiative software by:

* Enhancing current functionality
* Fixing bugs
* Adding new features and capabilities

If you are looking to make a substantial, complex, or wide-sweeping contribution, please engage with us [on Discord](https://discord.com/invite/system-init) before you begin so we can collaborate with you.

### Steps to Contribute Code

Follow the following steps to ensure your contribution goes smoothly.

1. Read and follow the steps outlined in the [System Initiative Contributing Policy](README.md#contributing).
1. Configure your development environment by either following the guide in the [README](README.md) or the [dev docs](DEV_DOCS.md).
1. [Fork](https://help.github.com/articles/working-with-forks/) the GitHub Repository allowing you to make the changes in your own copy of the repository.
1. Create a [GitHub issue](https://github.com/systeminit/si/issues) if one doesn't exist already.
1. Make the changes you would like to include, adding new tests where possible, and make sure all relevant existing [tests are passing by using the guide in our dev docs](DEV_DOCS.md).
1. [Prepare your changes by checking out the relevant section in our dev docs](DEV_DOCS.md) and ensure your commits are descriptive. The document contains an optional commit template, if desired.
1. Ensure that you are in the [CONTRIBUTORS](CONTRIBUTORS.md) file (see the [Adding Yourself to the Contributors List](#adding-yourself-to-the-contributors-list) section for instructions)
1. Create a pull request on GitHub. If you're new to GitHub, read about [pull requests](https://help.github.com/articles/about-pull-requests/). You are welcome to submit your pull request for commentary or review before it is complete by creating a [draft pull request](https://help.github.com/en/articles/about-pull-requests#draft-pull-requests). Please include specific questions or items you'd like feedback on.
1. A member of the System Initiative team will review your PR within three business days (excluding holidays in the USA, Canada, UK, and Brazil) and either merge, comment, and/or assign someone for review.
1. Work with the reviewer to complete a code review. For each change, create a new commit and push it to make changes to your pull request. When necessary, the reviewer can trigger CI to run tests prior to merging.
1. Once you believe your pull request is ready to be reviewed, ensure the pull request is no longer a draft by [marking it ready for review](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/changing-the-stage-of-a-pull-request).
1. The reviewer will look over your contribution and either approve it or provide comments letting you know if there is anything left to do. We try to give you the opportunity to make the required changes yourself, but in some cases, we may perform the changes ourselves if it makes sense to (minor changes or for urgent issues). We do our best to review PRs promptly, but complex changes could require more time.
1. After completing your review, a System Initiative team member will trigger merge to run all tests. Upon passing, your change will be merged into `main`, and your pull requests will be closed. All merges to `main` create a new release, and all final changes are attributed to you.
1. Have a dance party!

Note: In some cases, we might decide that a PR should be closed without merging. We'll make sure to provide clear reasoning when this happens.

### Adding Yourself to the Contributors List

When making a pull request to the System Initiative software, you must add yourself to the [CONTRIBUTORS](CONTRIBUTORS.md) list.
You will only have to do this the first time that you contribute to the software.
For this, we recommend adding yourself with a separate commit (does not have to be a separate PR) to the file.

#### What Does this Mean for You?

Here is what being a contributor means for you:

* License all our contributions to the project under the Apache License, Version 2.0
* Have the legal rights to license our contributions ourselves, or get permission to license them from our employers, clients, or others who may have them

For more information, see the [README](README.md) and feel free to reach out to us [on Discord](https://discord.com/invite/system-init).

Now, let's walk through how to add yourself to the list.

#### Editing the File (1/3)

In [the file](CONTRIBUTING.md), there is a delimiter (e.g. `-----------`) followed by a list of names and associated GitHub usernames.
The format of the contributor lines are as follows:

```
* <NAME-YOU-WOULD-LIKE-TO-BE-REFERRED-TO-AS> (@<GITHUB-USERNAME>)
```

Here is an example:

```
* Nick Gerace (@nickgerace)
```

You do not have to use your legal name.
You can provide a name you would like to referred to as, a nickname, etc.
In fact, you can use your GitHub username in the "name" slot.
Here is an example:

```
* nickgerace (@nickgerace)
```

Your name can be of multiple words and use multiple whitespaces too.
Here's a totally real example:

```
* Todd Howard Skyrim McFallout-y Starfield Pants (@totallyrealusername)
```

Above all, ensure that the format described at the beginning is preserved.

#### Polish Your Addition and Ensure the List is Ready (2/3)

When making changes, ensure the following:

- Your individual line was appended to the bottom of the list
- No additional newlines were added
- Your individual line has no trailing or leading spaces
- Your individual line matches the aforementioned format
- Your GitHub username appears exactly once
- Nothing else in the contributors file changed

#### Commit Your Change (3/3)

We recommend adding yourself with a separate commit (does not have to be a separate PR) to the file with the following commit title format:

```
chore: add <GITHUB-USERNAME> to contributors
```

Here is an example:

```
chore: add nickgerace to contributors
```

After the commit is pushed, you should be good to go!
