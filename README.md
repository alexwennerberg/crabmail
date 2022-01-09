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

## Install and usage

(DRAFT) see https://alex.flounder.online/tech/howtolist.gmi for more detail

git clone https://git.alexwennerberg.com/crabmail/
cd crabmail && cargo install --path .

Copy crabmail.conf and set the variables as needed.  

Run crabmail [maildir root] -c [config-file.conf].

If you want to use an mbox, use https://github.com/leahneukirchen/mblaze to
import it into a maildir. Mblaze also has some tools that you may find
supplementary to crabmail.

Open site/index.html in a web browser 

For project discussion and patches, use the Mailing list:
https://lists.flounder.online/crabmail/

Crabmail is AGPLv3 licenses, but I'm happy to release code snippets from this
repo (which reimplements some common Rust functions) in a more permissive
license. Email me!

See also 
https://git.causal.agency/bubger/about/
