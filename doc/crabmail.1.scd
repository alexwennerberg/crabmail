crabmail(1)

# NAME

crabmail - an HTML email archive generator

# SYNOPSIS

crabmail [-rR] [-c CONFIG] [-d OUT_DIR] maildir

# DESCRIPTION

Crabmail writes html pages for the maildir folders contained within *maildir*

*-g*
	include gemini output. default true if neither this nor *-h* is present.

*-h*
	include html output. default true if neither this nor *-h* is present.

*-c CONFIG*

	The config file to read from. Default is ./crabmail.conf

*-d OUT_DIR*
	
	The out dir to write HTML to. Defaults it ./site

*maildir*

	Path to a maildir folder. All subfolders (with the exception of the special
	maildir folders cur, new, tmp) will be rendered as sub-pages in your HTML
	archive. 

Threads will be written to *OUT_DIR*/threads/[thread-id].html -- Thread IDs can
have basically any character in them, except for forward slashes, which are
replaced by semicolons: keep this in mind if you are using a non-unix
filesystem.

Threading is determined by a combination of in-reply-to and subject headers.

Caching is minimal at the moment and subject to change.

# SEE ALSO

*crabmail-config*(5) 

# AUTHORS
Alex Wennerberg <alex@alexwennerberg.com>
