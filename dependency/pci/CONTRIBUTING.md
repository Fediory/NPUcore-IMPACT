Thank you for your interest in contributing to the Robigalia project! There
are many ways to contribute, and we appreciate all of them.

If you have questions, please send an email to
[robigalia-dev@lists.robigalia.org][list] or hop on [#robigalia][irc].

As a reminder, all participants in our community are expected to honor our
[code of conduct][conduct].

[list]: mailto:robigalia-dev@lists.robigalia.org
[irc]: https://webchat.freenode.net/?channels=%23robigalia
[conduct]: https://robigalia.org/conduct.html

## Feature Proposal

To propose a change to the way that Robigo or any other Robigalia codebase
works, please open an issue directly on the relevant repository. Include the
text, `[feature-proposal]` in the title and someone will come by and re-tag
the issue appropriately. Here's a template you can use to file a feature
proposal, though it's not necessary to use it exactly:

```
One-paragraph summary

# Details

As many details as you think are necessary to communicate how the feature
should work.

# Benefits

Benefits of the feature.

# Drawbacks

Drawbacks of the feature.
```

There is [an example feature proposal][prop] if you want to see what it looks
like.

[prop]: https://gitlab.com/robigalia/meta/issues/1

## Bug Reports

While bugs are unfortunate, they're a reality in software. We can't fix what we
don't know about, so please report liberally. If you're not sure if something
is a bug or not, feel free to file a bug anyway.

**If you believe reporting your bug publicly represents a security risk to
users of software developed by the Robigalia project,
please follow our [instructions for reporting security
vulnerabilities](https://robigalia.org/security.html)**.

If you have the chance, before reporting a bug, please [search existing
issues](https://gitlab.com/groups/robigalia/issues) as it's possible that
someone else has already reported your error. This doesn't always work, and
sometimes it's hard to know what to search for, so consider this extra credit.
We won't mind if you accidentally file a duplicate report, we'll typically
close it and point out which issue is a duplicate of.

Opening an issue is as easy as following [this
link](https://gitlab.com/groups/robigalia/issues) and filling out the fields
after clicking "New Issue".  Here's a template that you can use to file a bug,
though it's not necessary to use it exactly:

    <short summary of the bug>

    I tried this:

    <testcase demonstrating the bug>

    I expected to see this happen: <explanation>

    Instead, this happened: <explanation>

All three components are important: what you did, what you expected, what
happened instead.


## Submitting code

To coordinate changes to our software, we use a feature of GitLab called
"merge requests". Alternatively, you can email patches (ideally with git
send-email) to [the development mailing list][list].

### Commit messages

Commit messages should be self-contained and describe the motivation for a
change. The subject line should be short, with any elaborations in the body.
If the commit closes an issue, write "Closes #issuenumber" at the end of the
body.

For some guidelines, read [this][chrisbeams] for the basics, and [this][pro]
to become a pro.

[chrisbeams]: http://chris.beams.io/posts/git-commit/
[pro]: https://robots.thoughtbot.com/5-useful-tips-for-a-better-commit-message

### Commit signing

For a [variety of reasons][horror], we use commit signing to verify the
authenticity and authorship of changes to our projects. If you don't already
have one, create a PGP key. There are many ways to do this. [This blog
post][keygen] by Alex Cabal has a decent explanation of how to do it and some
of the tradeoffs involved.

Once you have a keypair and it is uploaded to the keyservers, [learn how to
use gpg with git][horror2]. Please sign all commits you intend to include in a
merge request.

[horror]: https://mikegerwitz.com/papers/git-horror-story
[horror2]: https://mikegerwitz.com/papers/git-horror-story#trust-ensure
[keygen]: https://alexcabal.com/creating-the-perfect-gpg-keypair/

### Forking the repositories.

We use a custom system for managing our large amount of repositories. We have
one "master" repository, [devbox][devbox], which contains scripts for fetching
the other repositories. It also includes a [Vagrant][vagrant] configuration,
which is the easiest way to get started with Rust and seL4. See the readme in
that repository to see how to use the scripts and, optionally, set up Vagrant.
Part of the setup is running the `add_remotes.sh` script, which will rename
the official repo to `upstream` and add an `origin` remote pointing to a repo
under your own username.

To fork a repository, navigate to it on GitLab and click the "Fork project"
button", which is to the left of the repository URL:

![fork project button location image][forkimg]

Then, you can interact with that repository using the `origin` remote in the
relevant sub-repository of `devbox`.

[devbox]: https://gitlab.com/robigalia/devbox
[vagrant]: https://www.vagrantup.com/
[forkimg]: https://gitlab.com/robigalia/meta/raw/master/fork_button.png

### Signing the Developer Certificate of Origin

Before we can accept code you have written into our repositories, you must
agree to the Developers Certificate of Origin. You can view it [in this
repository][DCO]. It is quite minimalist, and is also used by the Linux
kernel. To do this, fork the meta repository and run a command like:

    gpg --sign --armor < DCO.txt > agreements/${your_gitlab_username}

Create a new commit with that file and send a merge request.

[DCO]: https://gitlab.com/robigalia/meta/blob/master/DCO.txt

### Submitting a merge request

To submit a merge request, push the commits you want to include to a branch on
your fork of the relevant repository. Refer to the [official GitLab
documentation][mergereq] to see how to proceed from there.

Please open merge requests against the `master` branch. If needed, we will
merge into a separate branch manually.

[mergereq]: http://doc.gitlab.com/ce/gitlab-basics/add-merge-request.html

All merge requests are reviewed by another person.

If you want to request that a specific person reviews your pull request, you
can specify the "assignee" when you create the pull request. For example,
Corey usually reviews changes to sel4-sys, so you could specify him as the
assignee if you send a merge request to sel4-sys.

## Writing Documentation

Documentation improvements are very welcome. The source of robigalia.org is
in its [own repository][web], and individual crate documentation is hosted on
[doc.robigalia.org](https://doc.robigalia.org).

Documentation merge requests function in the same way as other merge requests.

[web]: https://gitlab.com/robigalia/robigalia.org

## Helpful Links and Information

- The [seL4 website](https://sel4.systems/) has things like the manual and how
  to build seL4.
- The [Rust website](https://rust-lang.org/) for all your Rust-related
  questions.

*This document adapted from <https://github.com/rust-lang/rust/blob/master/CONTRIBUTING.md>*
