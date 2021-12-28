crabmail(1)

# NAME

crabmail - an HTML email archive generator

# SYNOPSIS

crabmail [-r] mbox-file

# OPTIONS

*-r*
	Use relative timestamps ("2 hours ago") instead of absolute ones. This makes
	your archive more readable, but only if the index is being updated frequently

*mailto:...*
	Opens the composer with the address in the "to" field.
	If _aerc_ is already running, the composer is started in this instance,
	otherwise _aerc_ will be started.

# SEE ALSO

*crabmail-config*(5) 

# AUTHORS

Alex Wennerberg <alex@alexwennerberg.com>
