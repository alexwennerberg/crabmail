use anyhow::{anyhow, Context, Result};
use mailparse::ParsedMail;
use mailparse::*;
use std::collections::HashMap;

// https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/
use maildir::MailEntry;

#[derive(Default)]
pub struct Arena {
    nodes: Vec<Container>,
    container_index: HashMap<String, usize>, // Message ID, node index
}

// Rust Implementation of https://www.jwz.org/doc/threading.html
// See "The Algorithm"
// WIP. this is hard
// TODO dont store the entire email here. we can read from disk
impl Arena {
    pub fn add_message(&mut self, mut message: MailEntry) -> Result<()> {
        let parsed = message.parsed()?; // escape now if invalid msg
        let m_id = parsed
            .headers
            .get_first_value("message-id")
            .and_then(|m| {
                msgidparse(&m).ok().and_then(|i| match i.len() {
                    0 => None,
                    _ => Some(i[0].clone()),
                })
            })
            .context("No valid message ID")?;
        let mut references = msgidparse(
            &parsed
                .headers
                .get_first_value("references")
                .unwrap_or("".to_owned()),
        )?;
        if let Some(irt) = msgidparse(
            &parsed
                .headers
                .get_first_value("in_reply_to")
                .unwrap_or("".to_string()),
        )?
        .iter()
        .next()
        {
            references.push(irt.to_string())
        }
        let this_id: usize;
        match self.container_index.get(&m_id) {
            Some(c) => {
                this_id = *c;
                match self.nodes[*c].data {
                    None => {
                        self.nodes[*c].data = Some(message);
                    }
                    Some(_) => panic!("Duplicate ID"), // TODO
                }
            }
            None => {
                let node = self.new_node(Some(message));
                self.container_index.insert(m_id.clone(), node);
                this_id = node;
            }
        };
        for (n, reference) in references.iter().enumerate() {
            match self.container_index.get(reference) {
                Some(_) => {}
                None => {
                    let new_node = self.new_node(None);
                    self.container_index.insert(reference.to_string(), new_node);
                }
            };
            // Link the References field's Containers together in the order implied by the
            // References header.
            if n > 1 && n != references.len() - 1 {
                // TODO check loop
                let i = self.container_index.get(&references[n - 1]).unwrap();
                self.set_child(*i, *self.container_index.get(reference).unwrap());
            }
        }
        // Set the parent of this message to be the last element in References
        // remove existing reference
        if let Some(par) = self.nodes[this_id].parent {
            self.nodes[par].child = None;
        }
        self.nodes[this_id].parent = None;

        if references.len() > 1 {
            //handle last element
            let last_el = &references[references.len() - 1];
            let last_con = self.container_index.get(last_el).unwrap();
            self.set_child(*last_con, this_id);
        }

        Ok(())
    }
    pub fn finalize(mut self) {
        let mut stack: &mut Vec<usize> = &mut self
            .container_index
            .iter()
            .filter_map(|(k, v)| {
                if self.nodes[*v].parent.is_none() {
                    return Some(*v);
                }
                None
            })
            .collect();
        let root_set = stack.clone();
        while stack.len() > 0 {
            let top = stack.pop().unwrap();
            let container = &self.nodes[top];
            if container.data.is_none() && container.child.is_some() {
                self.set_child(container.parent.unwrap(), container.child.unwrap());
            }
        }

        // group by subject
        // let subject_table = HashMap::new();
        // for item in root_set {
        //
    }

    fn set_child(&mut self, parent: usize, child: usize) {
        // TODO check non-recursive. do nothing.
        let a = &mut self.nodes[parent];
        a.child = Some(child);
        let b = &mut self.nodes[child];
        b.parent = Some(parent)
    }
    fn children_recursive() {}
    fn new_node(&mut self, data: Option<MailEntry>) -> usize {
        // Get the next free index
        let next_index = self.nodes.len();

        // Push the node into the arena
        self.nodes.push(Container {
            parent: None,
            child: None,
            next: None,
            data: data,
        });

        // Return the node identifier
        next_index
    }
}

pub struct Container {
    parent: Option<usize>,
    child: Option<usize>,
    next: Option<usize>,
    data: Option<MailEntry>,
}

/// The actual data which will be stored within the tree
///     pub data: T,
///     }
///
pub struct Message {
    subject: String,
    id: String,
    references: MessageIdList,
    path: std::path::PathBuf,
}

impl Message {
    pub fn from(mut entry: MailEntry) -> Result<Self> {
        let parsed = entry.parsed()?; // escape now if invalid msg
        let subject = parsed
            .headers
            .get_first_value("Subject")
            .unwrap_or("[No Subject]".to_string());
        let id = parsed
            .headers
            .get_first_value("message-id")
            .and_then(|m| {
                msgidparse(&m).ok().and_then(|i| match i.len() {
                    0 => None,
                    _ => Some(i[0].clone()),
                })
            })
            .context("No valid message ID")?;
        let mut references = msgidparse(
            &parsed
                .headers
                .get_first_value("references")
                .unwrap_or("".to_owned()),
        )?;
        if let Some(irt) = msgidparse(
            &parsed
                .headers
                .get_first_value("in_reply_to")
                .unwrap_or("".to_string()),
        )?
        .iter()
        .next()
        {
            references.push(irt.to_string())
        }
        Ok(Self {
            subject,
            id,
            references,
            path: entry.path().to_owned(),
        })
    }
}
