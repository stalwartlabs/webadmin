/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs Ltd <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::*;

impl Builder<Schemas, ()> {
    pub fn build_spam_lists(self) -> Self {
        // Anti-SPAM settings
        self.new_schema("spam-settings")
            .new_field("spam-filter.enable")
            .label("Enable spam filtering")
            .help("Whether to enable the spam filter")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.auto-update")
            .label("Automatically update spam filter rules")
            .help("Whether to automatically update the spam filter rules")
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.resource")
            .label("Rules URL")
            .help(concat!(
                "Override the URL to download spam filter rules from. ",
                "By default spam filter rules are downloaded from ",
                "https://github.com/stalwartlabs/spam-filter.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("spam-filter.header.status.enable")
            .label("Add spam status header to messages")
            .help("Whether to add a SPAM/HAM status header to messages")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.header.status.name")
            .label("Status header")
            .help("Name of the spam status header")
            .default("X-Spam-Status")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("spam-filter.header.result.enable")
            .label("Add tag scores header to messages")
            .help("Whether to include the assigned tags and scores as a header")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.header.result.name")
            .label("Results header")
            .help("Name of the spam results header")
            .default("X-Spam-Result")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_field("spam-filter.score.spam")
            .label("Spam threshold")
            .help("Mark as SPAM messages with a score above this threshold")
            .default("5.0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((-100.0).into()),
                    Validator::MaxValue(100.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.score.discard")
            .label("Discard threshold")
            .help("Discard messages with a score above this threshold")
            .default("0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((-100.0).into()),
                    Validator::MaxValue(100.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.score.reject")
            .label("Reject threshold")
            .help("Reject messages with a score above this threshold")
            .default("0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((-100.0).into()),
                    Validator::MaxValue(100.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.grey-list.duration")
            .label("Duration")
            .help(concat!(
                "Time to keep an IP address in the grey list. ",
                "The grey list is used to delay messages from unknown senders."
            ))
            .typ(Type::Duration)
            .input_check([], [])
            .build()
            .new_field("spam-filter.trusted-reply.duration")
            .label("Duration")
            .help(concat!(
                "Time to keep track of Message-Ids sent from authenticated users. ",
                "Replies to messages sent from authenticated users are considered ham."
            ))
            .typ(Type::Duration)
            .default("30d")
            .input_check([], [])
            .build()
            .new_field("spam-filter.bayes.auto-learn.trusted-reply")
            .label("Train the Bayes classifier on trusted replies")
            .help(concat!(
                "Whether replies to messages sent from authenticated ",
                "users should be learned as ham.",
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.dnsbl.max-check.ip")
            .label("IP Checks")
            .help("Maximum number of DNSBL checks for IP addresses")
            .default("20")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue((1i64).into())],
            )
            .build()
            .new_field("spam-filter.dnsbl.max-check.domain")
            .label("Domain Checks")
            .help("Maximum number of DNSBL checks for domain names")
            .default("20")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue((1i64).into())],
            )
            .build()
            .new_field("spam-filter.dnsbl.max-check.email")
            .label("E-mail Checks")
            .help("Maximum number of DNSBL checks for E-mail addresses")
            .default("20")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue((1i64).into())],
            )
            .build()
            .new_field("spam-filter.dnsbl.max-check.url")
            .label("URL Checks")
            .help("Maximum number of DNSBL checks for URLs")
            .default("20")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue((1i64).into())],
            )
            .build()
            .new_form_section()
            .title("Spam Filter Settings")
            .fields([
                "spam-filter.score.spam",
                "spam-filter.score.discard",
                "spam-filter.score.reject",
                "spam-filter.enable",
            ])
            .build()
            .new_form_section()
            .title("Trusted Replies")
            .fields([
                "spam-filter.trusted-reply.duration",
                "spam-filter.bayes.auto-learn.trusted-reply",
            ])
            .build()
            .new_form_section()
            .title("Greylisting")
            .fields(["spam-filter.grey-list.duration"])
            .build()
            .new_form_section()
            .title("DNSBL Limits")
            .fields([
                "spam-filter.dnsbl.max-check.ip",
                "spam-filter.dnsbl.max-check.domain",
                "spam-filter.dnsbl.max-check.email",
                "spam-filter.dnsbl.max-check.url",
            ])
            .build()
            .new_form_section()
            .title("Headers")
            .fields([
                "spam-filter.header.status.name",
                "spam-filter.header.status.enable",
                "spam-filter.header.result.name",
                "spam-filter.header.result.enable",
            ])
            .build()
            .new_form_section()
            .title("External Rules")
            .fields(["spam-filter.resource", "spam-filter.auto-update"])
            .build()
            .build()
            // Bayes settings
            .new_schema("spam-bayes")
            .new_field("spam-filter.bayes.enable")
            .label("Enable Bayes classification")
            .help("Whether the bayes classifier should be enabled")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.bayes.account.enable")
            .label("Enable user-specific Bayes classification")
            .help("Whether accounts can train their own bayes classifier")
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.bayes.auto-learn.enable")
            .label("Automatically train the Bayes classifier")
            .help("Whether the bayes classifier should be trained automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.bayes.classify.balance")
            .label("Balance")
            .help("Keep difference for spam/ham learns for at least this value")
            .default("0.9")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((-100.0).into()),
                    Validator::MaxValue(100.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.auto-learn.threshold.ham")
            .label("Learn Ham threshold")
            .help("When to learn ham (score >= threshold)")
            .default("-1.0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((-100.0).into()),
                    Validator::MaxValue(100.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.auto-learn.threshold.spam")
            .label("Learn Spam threshold")
            .help("When to learn spam (score >= threshold)")
            .default("6.0")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((-100.0).into()),
                    Validator::MaxValue(100.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.score.spam")
            .label("Spam threshold")
            .help("Classify as SPAM messages with a score above this threshold")
            .default("0.7")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((0.01).into()),
                    Validator::MaxValue(1.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.score.ham")
            .label("Ham threshold")
            .help("Classify as HAM messages with a score below this threshold")
            .default("0.5")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((0.01).into()),
                    Validator::MaxValue(1.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.account.score.spam")
            .label("Spam threshold")
            .help("Classify as SPAM messages with a score above this threshold")
            .default(".7")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((0.01).into()),
                    Validator::MaxValue(1.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.account.score.ham")
            .label("Ham threshold")
            .help("Classify as HAM messages with a score below this threshold")
            .default("0.5")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((0.01).into()),
                    Validator::MaxValue(1.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.classify.strength")
            .label("Strength")
            .help("The strength of the bayes classifier")
            .default("0.05")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((0.0).into()),
                    Validator::MaxValue(1.0.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.classify.tokens.hits")
            .label("Token hits")
            .help("The minimum number of token hits required for classification")
            .default("2")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((1i64).into()),
                    Validator::MaxValue(100i64.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.classify.tokens.min")
            .label("Minimum tokens")
            .help("The minimum number of token required for classification")
            .default("11")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((1i64).into()),
                    Validator::MaxValue(100i64.into()),
                ],
            )
            .build()
            .new_field("spam-filter.bayes.classify.learns")
            .label("Minimum learns")
            .help("The minimum number of learns required for classification")
            .default("200")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((1i64).into()),
                    Validator::MaxValue(1000i64.into()),
                ],
            )
            .build()
            .new_field("spam-filter.header.bayes.enable")
            .label("Add Bayes score header to messages")
            .help(concat!(
                "Whether to add a header with the Bayes score to messages. ",
                "This setting applies only to the user-specific Bayes classifier."
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.header.bayes.name")
            .label("Bayes header")
            .help("Name of the bayes score header")
            .default("X-Spam-Bayes")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .build()
            .new_form_section()
            .title("Bayes Classifier")
            .fields([
                "spam-filter.bayes.score.spam",
                "spam-filter.bayes.score.ham",
                "spam-filter.bayes.classify.balance",
                "spam-filter.bayes.classify.learns",
                "spam-filter.bayes.classify.strength",
                "spam-filter.bayes.classify.tokens.min",
                "spam-filter.bayes.classify.tokens.hits",
                "spam-filter.bayes.enable",
            ])
            .build()
            .new_form_section()
            .title("Auto-learn")
            .fields([
                "spam-filter.bayes.auto-learn.threshold.spam",
                "spam-filter.bayes.auto-learn.threshold.ham",
                "spam-filter.bayes.auto-learn.enable",
            ])
            .build()
            .new_form_section()
            .title("Account Classifier")
            .fields([
                "spam-filter.bayes.account.score.spam",
                "spam-filter.bayes.account.score.ham",
                "spam-filter.bayes.account.enable",
            ])
            .build()
            .new_form_section()
            .title("Headers")
            .fields([
                "spam-filter.header.bayes.name",
                "spam-filter.header.bayes.enable",
            ])
            .build()
            .build()
            // LLM settings
            .new_schema("spam-llm")
            .reload_prefix("lookup")
            .new_field("lookup.spam-config.llm-model")
            .label("Model")
            .help("The AI model to use for the LLM classifier")
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "ai-models",
                    field: "model",
                    filter: Default::default(),
                },
                typ: SelectType::Single,
            })
            .enterprise_feature()
            .build()
            .new_field("lookup.spam-config.llm-prompt")
            .label("Prompt")
            .help("The prompt to use for the LLM classifier")
            .typ(Type::Text)
            .enterprise_feature()
            .build()
            .new_field("lookup.spam-config.add-llm-result")
            .label("Add LLM result header to messages")
            .help("Whether to add the X-Spam-Llm-Result header to messages")
            .default("true")
            .typ(Type::Boolean)
            .default("true")
            .enterprise_feature()
            .build()
            .new_form_section()
            .title("LLM Classifier")
            .fields([
                "lookup.spam-config.llm-model",
                "lookup.spam-config.llm-prompt",
                "lookup.spam-config.add-llm-result",
            ])
            .build()
            .build()
            // SPAM free domains
            .new_schema("spam-free")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.spam-free")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the free domains list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Free domain names")
            .list_subtitle("Manage domain names from free e-mail providers")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // Disposable domains
            .new_schema("spam-disposable")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.spam-disposable")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the disposable domains list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Disposable domain names")
            .list_subtitle("Manage domain names from disposable e-mail providers")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // URL Redirectors
            .new_schema("spam-redirect")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.spam-redirect")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the URL redirectors list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("URL redirector domains")
            .list_subtitle("Manage domain names from URL redirection services")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // Domain allow list
            .new_schema("spam-allow")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.spam-allow")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the allow domains list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Trusted domain names")
            .list_subtitle("Manage trusted domain names")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // Domain block list
            .new_schema("spam-block")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.spam-block")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the blocked domains list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Blocked domain names")
            .list_subtitle("Manage blocked domain names")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // DMARC allow list
            .new_schema("spam-dmarc")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.spam-dmarc")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the DMARC domains allow list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("DMARC domain names")
            .list_subtitle("Manage domain names that are known to have valid DMARC records")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // SPF/DKIM allow list
            .new_schema("spam-spdk")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.spam-spdk")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the SPF and DKIM domains allow list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("SPF and DKIM domain names")
            .list_subtitle("Manage domain names that are known to have valid SPF or DKIM records")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // SPAM trap addresses
            .new_schema("spam-trap")
            .reload_prefix("lookup")
            .names("address", "addresses")
            .prefix("lookup.spam-trap")
            .new_id_field()
            .label("E-mail Address")
            .help("The e-mail address to be added to the SPAM trap list")
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::IsRegex],
            )
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("SPAM trap addresses")
            .list_subtitle("Manage e-mail addresses designated as SPAM traps")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // Scores
            .new_schema("spam-scores")
            .reload_prefix("lookup")
            .names("score", "scores")
            .prefix("lookup.spam-scores")
            .new_id_field()
            .label("Tag name")
            .help("The spam tag name")
            .input_check(
                [Transformer::RemoveSpaces, Transformer::Uppercase],
                [Validator::Required, Validator::IsId],
            )
            .build()
            .new_value_field()
            .label("Score or action")
            .help("The score for the tag or action to perform (reject or discard)")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_form_section()
            .fields(["_id", "_value"])
            .build()
            .list_title("SPAM Scores")
            .list_subtitle("Manage scores assigned to spam tags")
            .list_fields(["_id", "_value"])
            .build()
            // MIME-types
            .new_schema("spam-mime")
            .reload_prefix("lookup")
            .names("type", "types")
            .prefix("lookup.spam-mime")
            .new_id_field()
            .label("Extension")
            .help("The file name extension")
            .input_check(
                [Transformer::RemoveSpaces],
                [Validator::Required, Validator::IsId],
            )
            .build()
            .new_value_field()
            .label("Rule")
            .help("The mime-type rule for this file name extension")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_form_section()
            .fields(["_id", "_value"])
            .build()
            .list_title("MIME Types")
            .list_subtitle("Manage rules for file name extensions")
            .list_fields(["_id", "_value"])
            .build()
    }
}
