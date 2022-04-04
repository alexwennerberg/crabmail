# 🦀Crabmail🦀

[self-hosted](https://git.alexwennerberg.com/crabmail/) / [github mirror](https://github.com/alexwennerberg/crabmail)

A static mail HTML and [Gemini](https://gemini.circumlunar.space/) archive for
the 21st century, written in Rust. Includes helpful "modern" features that
existing solutions lack, like:

* Responsive UI
* Single-page threads
* Working mailto: links
* Thread-based Atom feeds

Not implemented yet / designed:
* Attachment handling?

EMAIL FOREVER!

* [lists.flounder.online demo](https://lists.flounder.online)
* gemini://lists.flounder.online On gemini!

## Installation and usage

To use crabmail to host your own public inbox archive-first mailing list, check out [Self-hosted Public Inbox in 2022](https://alex.flounder.online/tech/howtolist.gmi)

To install:
```
git clone git://git.alexwennerberg.com/crabmail/
cd crabmail && cargo install --path .
```

Copy `crabmail.conf` and set the variables as needed.

Get a maildir folder, for example, via `mbsync`. Crabmail will create sub-lists
for each folder inside this maildir.

Run crabmail [maildir root] -c [config-file.conf].

For more thorough documentation, run `man doc/crabmail.1`. You can also move
these wherever your docs manpages may live

If you want to use an mbox file (for example, to mirror another archive), use
[mblaze](https://github.com/leahneukirchen/mblaze) to import it into a maildir.
Mblaze also has some tools that you may find supplementary to crabmail.

For example:
```
mkdir -p lists/mylist/cur lists/mylist/tmp lists/mylist/new
mdeliver -M lists/mylist < mylist.mbox
crabmail lists
```

Open `site/index.html` in a web browser 

## NOTES

This is only tested on Linux. Notably: Any character other than "/" is allowed
in filenames used in message-id. Make sure this doesn't break anything or cause
a security vuln on your filesystem.

## Contributing 

For patches, use `git-send-email` or `git-format-patch`
to send a patch to the [crabmail public inbox](https://lists.flounder.online/crabmail/)

`git-format-patch` is preferred for non-trivial or multi-commit changes

## Etc

Crabmail is AGPLv3 licenses, but some files are licensed under 0BSD or other
more permissive licenses. I call this out when I can.

For a similar project, check out [bubger](https://git.causal.agency/bubger/about/)

Consider supporting me and my projects on [patreon](https://www.patreon.com/alexwennerberg)
