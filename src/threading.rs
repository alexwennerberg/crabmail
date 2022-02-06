// Take an iterator of emails and build a thread
// jmap threading algorithm
// For new implementations, it is
// suggested that two messages belong in the same Thread if both of the
// following conditions apply:

// 1.  An identical message id [RFC5322] appears in both messages in any
//     of the Message-Id, In-Reply-To, and References header fields.
// 2.  After stripping automatically added prefixes such as "Fwd:",
//     "Re:", "[List-Tag]", etc., and ignoring white space, the subjects
//     are the same.  This avoids the situation where a person replies
//     to an old message as a convenient way of finding the right
//     recipient to send to but changes the subject and starts a new
//     conversation.

// If messages are delivered out of order for some reason, a user may
// have two Emails in the same Thread but without headers that associate
// them with each other.  The arrival of a third Email may provide the
// missing references to join them all together into a single Thread.
// Since the "threadId" of an Email is immutable, if the server wishes
// to merge the Threads, it MUST handle this by deleting and reinserting
// (with a new Email id) the Emails that change "threadId".

// A *Thread* object has the following properties:

// o  id: "Id" (immutable; server-set)


//    The id of the Thread.

// o  emailIds: "Id[]" (server-set)

//    The ids of the Emails in the Thread, sorted by the "receivedAt"
//    date of the Email, oldest first.  If two Emails have an identical
//    date, the sort is server dependent but MUST be stable (sorting by
//    id is recommended).

use mail_parser::Message;

fn Index {
}

impl Index {
    fn build(emails: impl Iterator<Item = Message>) -> Self {
    }
}
