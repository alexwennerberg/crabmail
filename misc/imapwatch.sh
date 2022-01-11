# uses IMAP NOTIFY extension
# Derived from
# https://github.com/johan-adriaans/shell-imap-notify/blob/master/imap-notify

if [ -z "$3" ]; then
  echo "Imap idle listener"
  echo "Usage: $0 user@domain.com server:993 /usr/bin/notify_command"
  exit 1
fi

read_secret()
{
    stty -echo
    trap 'stty echo' EXIT
    read "$@"
    stty echo
    trap - EXIT
    echo
}

user=$1
server=$2
command=$3
printf "Password:"
read_secret password

start_idle () {
  echo ". login \"$user\" \"$password\""
  echo ". select lists"
  # Change lists to a different folder 
  echo ". notify set (subtree lists (MessageNew MessageExpunge))"
  while true; do
    sleep 600;
    echo ". noop"
  done
}

# Start ssl connection
echo "Starting imap idle client, logging in as $user at $server"
while read -r line ; do
  # Debug info, turn this off for silent operation
  echo "$line"
  if echo "$line" | grep -Eq ". STATUS .*"; then
    echo "Message added or deleted, executing $command"
    $command
  fi
done < <(openssl s_client -crlf -quiet -connect "$server" 2>/dev/null < <(start_idle))
