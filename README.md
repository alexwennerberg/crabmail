# 🦀Crabmail🦀

A static mail HTML archive for the 21st century, written in Rust. Includes
helpful "modern" features that existing solutions lack, like:

* Responsive UI
* Single-page threads
* Working mailto: links
* Thread-based Atom feeds

Not implemented yet / designed:
* Attachment handling?
* Gemini support

EMAIL FOREVER!

## Install and usage ``` git clone https://git.alexwennerberg.com/crabmail/ cd
crabmail && cargo install --path .  ```

Copy `crabmail.conf` and set the variables as needed.  Run `crabmail
[some-mbox-file.mbox] -c [config-file.conf]` Open `site/index.html` in a web
browser 

Try downloading the [gemini mailing list
archive](https://lists.orbitalfox.eu/archives/gemini/) for test data.

For project discussion and patches, use the [Mailing
list](https://lists.flounder.online/crabmail/)

See the companion project,
[imap2mbox](https://git.alexwennerberg.com/imap2mbox/)

Crabmail is AGPLv3 licenses, but I'm happy to release code snippets from this
repo (which reimplements some common Rust funxtions) in a less permissive
license. Email me!
