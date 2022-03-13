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
        String::new()
    }
}
