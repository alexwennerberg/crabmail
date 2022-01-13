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

## Installation and usage

To use crabmail to host your own archive-first mailing list, check out [Self-hosted Archives-first Mailing Lists in 2022](https://alex.flounder.online/tech/howtolist.gmi)

To install:
```
git clone git://git.alexwennerberg.com/crabmail/
cd crabmail && cargo install --path .
```

Copy `crabmail.conf` and set the variables as needed.

Get a maildir folder, for example, via `mbsync`. Crabmail will create sub-lists for each folder inside this maildir.

Run crabmail [maildir root] -c [config-file.conf].

For more thorough documentation, run `man doc/crabmail.1`. You can also move
these wherever your docs manmages may live

If you want to use an mbox file (for example, to mirror another archive), use
[mblaze](https://github.com/leahneukirchen/mblaze) to import it into a maildir.
Mblaze also has some tools that you may find supplementary to crabmail.

Open `site/index.html` in a web browser 

For project discussion and patches, and to see a live example, check out the [crabmail mailing list](https://lists.flounder.online/crabmail/)

Crabmail is AGPLv3 licenses, but some files are licensed under 0BSD.

See also 
https://git.causal.agency/bubger/about/
