crabmail-config(5)

# NAME

crabmail-config - configuration file formats for *crabmail*(1)

# CONFIGURATION

Each file uses an _ini_ format defined as follows: key=value\n. Keys cannot
contain =. Values cannot contain \n. There are no other restrictions.

Section headers are defined by [brackets]. These are optional, and can define
archive-specific configuration

This variables are often not escaped, so treat this file as trusted input (ie,
don't fill in values from user input somehow).

# CRABMAIL.CONF

## GLOBAL

*base_url*
	The base URL where your archive is hosted, used to generate atom feeds

*email_fmt*
	The structure of how emails are mapped to mail folders, for example
	lists+%s@flounder.online will replace %s with the archive name. You can
	override this in archive-specific configuration

*title_fmt*
	The structure of how names of each mail archive are structured, for example
	%s public inbox_ or %s mailing list. Will be overwritten if title is set for
	individual archives.

*description*
	A description for your homepage, will be displayed as a single line. HTML is
	allowed if you want.

*reply_add_link*
	Add a link to the thread in the "reply to" url. This allows users to see the
	thread context when receiving responses via email. TBD: This will allow poorly designed
	clients (eg Gmail) to still be threaded properly. Default: false

## Archives

Each mail archive is specified with a [subsection] header.

*title*
	Title of the archive. Default tile for archives is set by *title_fmt*

*email*
	Email of this archive. Overrides *email_fmt*.

*description*
	A description for your archive. HTML is allowed, but be judicious.

# SEE ALSO

*crabmail*(1) 

# AUTHORS
Alex Wennerberg <alex@alexwennerberg.com 
