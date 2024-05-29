/*
 * Copyright (c) 2024, Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Web-based Admin.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

use super::*;

impl Builder<Schemas, ()> {
    pub fn build_spam_lists(self) -> Self {
        // Anti-SPAM settings
        self.new_schema("spam-settings")
            .reload_prefix("lookup")
            .new_field("lookup.spam-config.add-spam")
            .label("Add X-Spam-Status header to messages")
            .help("Whether to add the X-Spam-Status header to messages that are detected as SPAM")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("lookup.spam-config.add-spam-result")
            .label("Add X-Spam-Result header to messages")
            .help("Whether to add the X-Spam-Result header to messages that are detected as SPAM")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("spam.header.is-spam")
            .label("SPAM header")
            .help("Move messages to the Junk folder if this header is present")
            .default("X-Spam-Status: Yes")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("lookup.spam-config.learn-enable")
            .label("Automatically train the Bayes classifier")
            .help("Whether the bayes classifier should be trained automatically")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("lookup.spam-config.learn-balance")
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
            .new_field("lookup.spam-config.learn-ham.replies")
            .label("Train on messages sent from authenticated users")
            .help("Whether messages sent from authenticated users should be learned as ham")
            .default("true")
            .typ(Type::Boolean)
            .build()
            .new_field("lookup.spam-config.learn-ham-threshold")
            .label("Ham threshold")
            .help("When to learn ham (score >= threshold)")
            .default("-0.5")
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
            .new_field("lookup.spam-config.learn-spam-threshold")
            .label("Spam threshold")
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
            .new_field("lookup.spam-config.threshold-spam")
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
            .new_field("lookup.spam-config.threshold-discard")
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
            .new_field("lookup.spam-config.threshold-reject")
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
            .new_field("lookup.spam-config.directory")
            .label("Directory")
            .help("Directory to use for local domain lookups (leave empty for default)")
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "directory",
                    field: "type",
                    filter: Default::default(),
                },
                multi: false,
            })
            .build()
            .new_field("lookup.spam-config.lookup")
            .label("Lookup")
            .help("Lookup store to use for Bayes tokens and ids (leave empty for default)")
            .typ(Type::Select {
                source: Source::Dynamic {
                    schema: "store",
                    field: "type",
                    filter: Default::default(),
                },
                multi: false,
            })
            .source_filter(&[
                "foundationdb",
                "mysql",
                "postgresql",
                "sqlite",
                "rocksdb",
                "redis",
            ])
            .build()
            .new_field("cache.bayes.capacity")
            .label("Capacity")
            .help("Starting capacity for the Bayes cache")
            .default("8192")
            .typ(Type::Input)
            .input_check([Transformer::Trim], [Validator::Required])
            .build()
            .new_field("cache.bayes.ttl.positive")
            .label("Positive TTL")
            .help("Time to live for Bayes tokens that were found in the database")
            .default("1h")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_field("cache.bayes.ttl.negative")
            .label("Negative TTL")
            .help("Time to live for Bayes tokens that do not exist in the database")
            .default("1h")
            .typ(Type::Duration)
            .input_check([], [Validator::Required])
            .build()
            .new_form_section()
            .title("Header")
            .fields([
                "spam.header.is-spam",
                "lookup.spam-config.add-spam",
                "lookup.spam-config.add-spam-result",
            ])
            .build()
            .new_form_section()
            .title("Thresholds")
            .fields([
                "lookup.spam-config.threshold-spam",
                "lookup.spam-config.threshold-discard",
                "lookup.spam-config.threshold-reject",
            ])
            .build()
            .new_form_section()
            .title("Data")
            .fields(["lookup.spam-config.directory", "lookup.spam-config.lookup"])
            .build()
            .new_form_section()
            .title("Bayes Autolearn")
            .fields([
                "lookup.spam-config.learn-balance",
                "lookup.spam-config.learn-spam-threshold",
                "lookup.spam-config.learn-ham-threshold",
                "lookup.spam-config.learn-enable",
                "lookup.spam-config.learn-ham.replies",
            ])
            .build()
            .new_form_section()
            .title("Bayes Token Cache")
            .fields([
                "cache.bayes.capacity",
                "cache.bayes.ttl.positive",
                "cache.bayes.ttl.negative",
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
            .label("Score")
            .help("The score for the tag")
            .input_check(
                [Transformer::Trim],
                [
                    Validator::Required,
                    Validator::MinValue((-100.0).into()),
                    Validator::MaxValue(100.0.into()),
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
