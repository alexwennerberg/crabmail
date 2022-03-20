// WIP
//
use crate::models::*;
use nanotemplate::template;

impl Lists {
    pub fn to_gmi(&self) -> String {
        template(
            r#"
            # Mail Archive 
         "#,
            &[("title", "tbd")],
        )
        .unwrap()
    }
}

impl List {
    pub fn to_gmi(&self) -> Vec<String> {
        vec![]
    }
}

impl Thread {
    pub fn to_gmi(&self) -> String {
        let mut out = format!(
            r#"# {}
        "#,
            self.messages[0].subject.replace("\n", " ")
        );
        for msg in &self.messages {
            let msg = template(
                r#"
## {subject}
From: {from}
Date: {date}
In-Reply-To: adsf
Message-Id: {msg_id}
To: ...
Cc: ...
---------------------
{body}
"#,
                &[
                    ("subject", &h(&msg.subject)),
                    ("date", &h(&msg.date)),
                    ("msg_id", &h(&msg.id)),
                    ("from", &h(&msg.from.address)),
                    ("body", &escape_body(&msg.body)),
                ],
            )
            .unwrap();
            out.push_str(&msg);
        }
        out
    }
}

// TODO ...
fn escape_body(s: &str) -> String {
    let mut out = "  ".to_string();
    out.push_str(&s.replace("\n", "\n  "));
    out
}

// escape header
fn h(s: &str) -> String {
    s.replace("\n", " ")
}
