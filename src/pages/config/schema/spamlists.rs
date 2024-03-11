use super::*;

/*const LIST_TYPE: &[(&str, &str)] = &[
    ("0", "Text"),
    ("1", "Glob pattern"),
    ("2", "Regular expression"),
    ("3", "IP Address"),
    ("4", "IP Mask"),
];*/

impl Builder<Schemas, ()> {
    pub fn build_spam_lists(self) -> Self {
        // SPAM free domains
        self.new_schema("spam-free")
            .names("domain", "domains")
            .prefix("lookup.spam-free")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the free domains list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsGlobPattern],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Free domain names")
            .list_subtitle("Manage domain names from free e-mail providers")
            .list_fields(["_id"])
            .list_actions([Action::Create, Action::Delete, Action::Search])
            .build()
            // Scores
            .new_schema("spam-scores")
            .names("score", "scores")
            .prefix("lookup.spam-scores")
            .new_id_field()
            .label("Tag name")
            .help("The spam tag name")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsGlobPattern],
            )
            .build()
            .new_value_field()
            .label("Score")
            .help("The score for the tag")
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue(-100),
                    Validator::MaxValue(100),
                ],
            )
            .build()
            .new_form_section()
            .fields(["_id", "_value"])
            .build()
            .list_title("SPAM Scores")
            .list_subtitle("Manage scores assigned to spam tags")
            .list_fields(["_id", "_value"])
            .build()
    }
}
