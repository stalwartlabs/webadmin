/*
 * SPDX-FileCopyrightText: 2020 Stalwart Labs LLC <hello@stalw.art>
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-SEL
 */

use super::*;

const SCOPES: &[(&str, &str)] = &[
    ("url", "URL"),
    ("domain", "Domain"),
    ("email", "E-mail"),
    ("ip", "IP"),
    ("header", "Header"),
    ("body", "Body"),
    ("any", "Any"),
];

impl Builder<Schemas, ()> {
    #![allow(clippy::useless_concat)]
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
            .new_field("spam-filter.card-is-ham")
            .label("Do not classify messages from contacts as spam")
            .help(concat!(
                "Never classify messages as spam if they are sent ",
                "from addresses present in the user's address book.",
            ))
            .default("true")
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
            .help("Mark as Spam messages with a score above this threshold")
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
            .default("50")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue((1i64).into())],
            )
            .build()
            .new_field("spam-filter.dnsbl.max-check.domain")
            .label("Domain Checks")
            .help("Maximum number of DNSBL checks for domain names")
            .default("50")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue((1i64).into())],
            )
            .build()
            .new_field("spam-filter.dnsbl.max-check.email")
            .label("E-mail Checks")
            .help("Maximum number of DNSBL checks for E-mail addresses")
            .default("50")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [Validator::Required, Validator::MinValue((1i64).into())],
            )
            .build()
            .new_field("spam-filter.dnsbl.max-check.url")
            .label("URL Checks")
            .help("Maximum number of DNSBL checks for URLs")
            .default("50")
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
                "spam-filter.card-is-ham",
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
            .new_field("spam-filter.bayes.auto-learn.card-is-ham")
            .label("Train as ham messages from senders in the address book")
            .help(concat!(
                "If a message is classified as spam, ",
                "but the sender is in the address book, learn it as ham."
            ))
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
            .help("Classify as Spam messages with a score above this threshold")
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
            .help("Classify as Ham messages with a score below this threshold")
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
            .help("Classify as Spam messages with a score above this threshold")
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
                "spam-filter.bayes.auto-learn.card-is-ham",
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
            // Pyzor settings
            .new_schema("spam-pyzor")
            .new_field("spam-filter.pyzor.enable")
            .label("Enable Pyzor classifier")
            .help(concat!(
                "Whether to enable the Pyzor classifier. ",
                "Pyzor is a collaborative, networked system to detect and report spam."
            ))
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.pyzor.port")
            .label("Enable Pyzor")
            .help("Whether to enable the Pyzor filter")
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.pyzor.host")
            .label("Hostname")
            .help("The hostname of the Pyzor server")
            .default("public.pyzor.org")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("spam-filter.pyzor.port")
            .label("Port")
            .help("The port to connect to the Pyzor server")
            .default("24441")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((100i64).into()),
                    Validator::MaxValue(65535i64.into()),
                ],
            )
            .build()
            .new_field("spam-filter.pyzor.timeout")
            .label("Timeout")
            .help(concat!(
                "The timeout for the Pyzor server. ",
                "If the server does not respond within this time, the check is considered failed."
            ))
            .typ(Type::Duration)
            .default("5s")
            .input_check([], [Validator::Required])
            .build()
            .new_field("spam-filter.pyzor.count")
            .label("Count")
            .help("The number of times the hash appears in the Pyzor blocklist")
            .default("5")
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
            .new_field("spam-filter.pyzor.wl-count")
            .label("WL Count")
            .help("The number of times the hash appears in the Pyzor allowlist")
            .default("10")
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
            .new_field("spam-filter.pyzor.ratio")
            .label("Ratio")
            .help(concat!(
                "The ratio of the number of times the hash appears ",
                "in the Pyzor allowlist to the blocklist"
            ))
            .default("0.2")
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
            .new_form_section()
            .title("Pyzor Settings")
            .fields([
                "spam-filter.pyzor.host",
                "spam-filter.pyzor.port",
                "spam-filter.pyzor.timeout",
                "spam-filter.pyzor.enable",
            ])
            .build()
            .new_form_section()
            .title("Classification")
            .fields([
                "spam-filter.pyzor.count",
                "spam-filter.pyzor.wl-count",
                "spam-filter.pyzor.ratio",
            ])
            .build()
            .build()
            // Reputation settings
            .new_schema("spam-reputation")
            .new_field("spam-filter.reputation.enable")
            .label("Enable Reputation tracking")
            .help(concat!(
                "Whether to enable the tracking the reputation of ",
                "IP addresses, domains, senders and ASNs."
            ))
            .default("false")
            .typ(Type::Boolean)
            .build()
            .new_field("spam-filter.reputation.expiry")
            .label("Expiration")
            .help(concat!(
                "The time to keep reputation tokens in the cache. ",
                "After this time the reputation token is considered stale."
            ))
            .typ(Type::Duration)
            .default("30d")
            .input_check([], [Validator::Required])
            .build()
            .new_field("spam-filter.reputation.score")
            .label("Adjust score")
            .help("Ratio to use when adjusting the score based on reputation")
            .default("0.98")
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
            .new_field("spam-filter.reputation.factor")
            .label("Factor")
            .help("The factor to use when adjusting the score based on reputation")
            .default("0.5")
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
            .new_field("spam-filter.reputation.weight.ip")
            .label("IP Weight")
            .help("The weight to assign to the IP reputation")
            .default("0.2")
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
            .new_field("spam-filter.reputation.weight.domain")
            .label("Domain Weight")
            .help("The weight to assign to the domain reputation")
            .default("0.2")
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
            .new_field("spam-filter.reputation.weight.asn")
            .label("ASN Weight")
            .help("The weight to assign to the ASN reputation")
            .default("0.1")
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
            .new_field("spam-filter.reputation.weight.sender")
            .label("Sender Weight")
            .help("The weight to assign to the sender reputation")
            .default("0.5")
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
            .new_form_section()
            .title("Reputation tracking")
            .fields([
                "spam-filter.reputation.expiry",
                "spam-filter.reputation.score",
                "spam-filter.reputation.factor",
                "spam-filter.reputation.enable",
            ])
            .build()
            .new_form_section()
            .title("Weights")
            .fields([
                "spam-filter.reputation.weight.ip",
                "spam-filter.reputation.weight.sender",
                "spam-filter.reputation.weight.domain",
                "spam-filter.reputation.weight.asn",
            ])
            .build()
            .build()
            // LLM settings
            .new_schema("spam-llm")
            .new_field("spam-filter.llm.enable")
            .label("Enable LLM classifier")
            .help("Whether to add a header containing the LLM response to messages")
            .default("false")
            .typ(Type::Boolean)
            .enterprise_feature()
            .build()
            .new_field("spam-filter.llm.model")
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
            .new_field("spam-filter.llm.temperature")
            .label("Temperature")
            .help("The temperature to use for the LLM classifier")
            .default("0.5")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((0.0).into()),
                    Validator::MaxValue(1.0.into()),
                ],
            )
            .enterprise_feature()
            .build()
            .new_field("spam-filter.llm.prompt")
            .label("Prompt")
            .help("The prompt to use for the LLM classifier")
            .typ(Type::Text)
            .enterprise_feature()
            .build()
            .new_field("spam-filter.llm.separator")
            .label("Separator")
            .help(concat!(
                "The separator character used to parse the LLM response.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .default(",")
            .enterprise_feature()
            .build()
            .new_field("spam-filter.llm.index.category")
            .label("Category Index")
            .help(concat!(
                "The position of the category field in the LLM response.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .default("0")
            .enterprise_feature()
            .build()
            .new_field("spam-filter.llm.index.confidence")
            .label("Confidence Index")
            .help(concat!(
                "The position of the confidence field in the LLM response.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .enterprise_feature()
            .build()
            .new_field("spam-filter.llm.index.explanation")
            .label("Explanation Index")
            .help(concat!(
                "The position of the explanation field in the LLM response.",
            ))
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .enterprise_feature()
            .build()
            .new_field("spam-filter.llm.categories")
            .typ(Type::Array(ArrayType::Text))
            .input_check([], [Validator::Required])
            .enterprise_feature()
            .label("Categories")
            .help("The expected categories in the LLM response")
            .build()
            .new_field("spam-filter.llm.confidence")
            .typ(Type::Array(ArrayType::Text))
            .enterprise_feature()
            .label("Confidence")
            .help("The expected confidence levels in the LLM response")
            .build()
            .new_field("spam-filter.header.llm.enable")
            .label("Add LLM result header to messages")
            .help("Whether to add a header containing the LLM response to messages")
            .default("true")
            .typ(Type::Boolean)
            .enterprise_feature()
            .build()
            .new_field("spam-filter.header.llm.name")
            .label("LLM header")
            .help("Name of the LLM results header")
            .default("X-Spam-LLM")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [])
            .enterprise_feature()
            .build()
            .new_form_section()
            .title("LLM Classifier")
            .fields([
                "spam-filter.llm.model",
                "spam-filter.llm.temperature",
                "spam-filter.llm.prompt",
                "spam-filter.llm.enable",
            ])
            .build()
            .new_form_section()
            .title("Response Format")
            .fields([
                "spam-filter.llm.separator",
                "spam-filter.llm.index.category",
                "spam-filter.llm.index.confidence",
                "spam-filter.llm.index.explanation",
                "spam-filter.llm.categories",
                "spam-filter.llm.confidence",
            ])
            .build()
            .new_form_section()
            .title("Headers")
            .fields([
                "spam-filter.header.llm.name",
                "spam-filter.header.llm.enable",
            ])
            .build()
            .build()
            // SPAM rules
            .new_schema("spam-rule")
            .names("rule", "rules")
            .prefix("spam-filter.rule")
            .suffix("scope")
            .new_id_field()
            .label("Rule ID")
            .help("Unique identifier for the rule")
            .build()
            .new_field("enable")
            .label("Enable rule")
            .help("Whether to enable this rule")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("condition")
            .label("Rule")
            .help(concat!(
                "Expression that returns the tag to assign to the message.",
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(ExpressionValidator::new(SPAM_FILTER_VARS, &[])),
                ],
            )
            .build()
            .new_field("priority")
            .label("Priority")
            .help("The priority of the rule")
            .default("500")
            .typ(Type::Input)
            .input_check(
                [Transformer::Trim],
                [
                    Validator::MinValue((-99999).into()),
                    Validator::MaxValue(99999.into()),
                ],
            )
            .build()
            .new_field("scope")
            .label("Scope")
            .help("Where to apply the rule")
            .default("any")
            .typ(Type::Select {
                source: Source::Static(SCOPES),
                typ: SelectType::Single,
            })
            .build()
            .new_form_section()
            .title("Rule Configuration")
            .fields(["_id", "condition", "priority", "scope", "enable"])
            .build()
            .list_title("Rules")
            .list_subtitle("Manage spam filter rules")
            .list_fields(["_id", "scope", "priority", "enable"])
            .build()
            // SPAM DNSBls
            .new_schema("spam-dnsbl")
            .names("list", "lists")
            .prefix("spam-filter.dnsbl.server")
            .suffix("scope")
            .new_id_field()
            .label("Rule ID")
            .help("Unique identifier for the DNSBL server")
            .build()
            .new_field("enable")
            .label("Enable the DNSBL server")
            .help("Whether to enable this DNSBL server")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("zone")
            .label("Zone")
            .help(concat!("Expression that returns the DNS zone to query.",))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(ExpressionValidator::new(SPAM_FILTER_VARS, &[])),
                ],
            )
            .build()
            .new_field("tag")
            .label("Tag")
            .help(concat!(
                "Expression that returns the tag to assign to the message.",
            ))
            .typ(Type::Expression)
            .input_check(
                [],
                [
                    Validator::Required,
                    Validator::IsValidExpression(ExpressionValidator::new(SPAM_FILTER_VARS, &[])),
                ],
            )
            .build()
            .new_field("scope")
            .label("Scope")
            .help("Where to use the DNSBL server")
            .default("any")
            .typ(Type::Select {
                source: Source::Static(SCOPES),
                typ: SelectType::Single,
            })
            .build()
            .new_form_section()
            .title("DNSBl Configuration")
            .fields(["_id", "zone", "tag", "scope", "enable"])
            .build()
            .list_title("DNSBl Servers")
            .list_subtitle("Manage DNS block and allowlists")
            .list_fields(["_id", "scope", "enable"])
            .build()
            // URL Redirectors
            .new_schema("spam-redirect")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.url-redirectors")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the URL redirectors list")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("URL redirector domains")
            .list_subtitle("Manage domain names from URL redirection services")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // Domain trusted list
            .new_schema("spam-trusted")
            .reload_prefix("lookup")
            .names("domain", "domains")
            .prefix("lookup.trusted-domains")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the trusted domains list")
            .input_check([Transformer::Trim], [Validator::Required])
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
            .prefix("lookup.blocked-domains")
            .new_id_field()
            .label("Domain Name")
            .help("The domain name to be added to the blocked domains list")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Blocked domain names")
            .list_subtitle("Manage blocked domain names")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // SPAM trap addresses
            .new_schema("spam-trap")
            .reload_prefix("lookup")
            .names("address", "addresses")
            .prefix("lookup.spam-traps")
            .new_id_field()
            .label("E-mail Address")
            .help("The e-mail address to be added to the SPAM trap list")
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_form_section()
            .field("_id")
            .build()
            .list_title("Spam trap addresses")
            .list_subtitle("Manage e-mail addresses designated as SPAM traps")
            .list_fields(["_id"])
            .no_list_action(Action::Modify)
            .build()
            // Scores
            .new_schema("spam-score")
            .names("score", "scores")
            .prefix("spam-filter.list.scores")
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
            .list_title("Spam Scores")
            .list_subtitle("Manage scores assigned to spam tags")
            .list_fields(["_id", "_value"])
            .build()
            // MIME-types
            .new_schema("spam-mime")
            .reload_prefix("lookup")
            .names("type", "types")
            .prefix("spam-filter.list.file-extensions")
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
