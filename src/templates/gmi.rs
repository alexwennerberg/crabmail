use crate::models::*;
use nanotemplate::template;

impl Lists<'_> {
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

impl Thread<'_> {
    pub fn to_gmi(&self) -> String {
        String::new()
    }
}
