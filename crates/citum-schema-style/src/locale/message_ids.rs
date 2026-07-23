/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Canonical message-ID lookups and MF2 message resolution.
//!
//! The mappings here connect structured term identifiers (`GeneralTerm`,
//! `ContributorRole`, `LocatorType`) to the dotted message IDs used by the MF2
//! evaluator, and provide the `resolve_message_text` entry point that the
//! term-lookup API on `Locale` (in `locale/mod.rs`) uses to fall through to
//! the MF2 layer before legacy lookups.

use super::Locale;
use super::message::MessageArgs;
use super::types::{GeneralTerm, GrammaticalGender, MessageSyntax, TermForm};
use crate::citation::LocatorType;
use crate::template::ContributorRole;

impl Locale {
    /// Map a GeneralTerm to its canonical message ID suffix (e.g., GeneralTerm::EtAl → "et-al").
    pub(super) fn general_term_to_message_id(term: &GeneralTerm) -> &str {
        match term {
            GeneralTerm::And => "and",
            GeneralTerm::RoleConjunction => "role-conjunction",
            GeneralTerm::EtAl => "et-al",
            GeneralTerm::AndOthers => "and-others",
            GeneralTerm::Accessed => "accessed",
            GeneralTerm::Cited => "cited",
            GeneralTerm::Retrieved => "retrieved",
            GeneralTerm::NoDate => "no-date",
            GeneralTerm::Ibid => "ibid",
            GeneralTerm::In => "in",
            GeneralTerm::At => "at",
            GeneralTerm::By => "by",
            GeneralTerm::From => "from",
            GeneralTerm::Of => "of",
            GeneralTerm::To => "to",
            GeneralTerm::Anonymous => "anonymous",
            GeneralTerm::Circa => "circa",
            GeneralTerm::Forthcoming => "forthcoming",
            GeneralTerm::Online => "online",
            GeneralTerm::AvailableAt => "available-at",
            GeneralTerm::ReviewOf => "review-of",
            GeneralTerm::Here => "here",
            GeneralTerm::Deposited => "deposited",
            GeneralTerm::Patent => "patent",
            GeneralTerm::Issued => "issued",
            GeneralTerm::Volume => "volume",
            GeneralTerm::Issue => "issue",
            GeneralTerm::Page => "page",
            GeneralTerm::Chapter => "chapter",
            GeneralTerm::Edition => "edition",
            GeneralTerm::Section => "section",
            GeneralTerm::Version => "version",
            GeneralTerm::OriginalWorkPublished => "original-work-published",
            GeneralTerm::PersonalCommunication => "personal-communication",
            GeneralTerm::Unknown(s) => s.as_str(),
        }
    }

    /// Map a GeneralTerm to its legacy CSL key string for alias lookup.
    pub(super) fn general_term_to_legacy_key(term: &GeneralTerm) -> &str {
        match term {
            GeneralTerm::EtAl => "et_al",
            GeneralTerm::NoDate => "no_date",
            _ => Self::general_term_to_message_id(term),
        }
    }

    pub(super) fn role_message_id(role: &ContributorRole, form: &TermForm) -> Option<&'static str> {
        let prefix = match role {
            ContributorRole::Editor => "role.editor",
            ContributorRole::Translator => "role.translator",
            ContributorRole::Guest => "role.guest",
            _ => return None,
        };

        match *form {
            TermForm::Long => Some(match prefix {
                "role.editor" => "role.editor.label-long",
                "role.translator" => "role.translator.label-long",
                "role.guest" => "role.guest.label-long",
                _ => return None,
            }),
            TermForm::Short => Some(match prefix {
                "role.editor" => "role.editor.label",
                "role.translator" => "role.translator.label",
                "role.guest" => "role.guest.label",
                _ => return None,
            }),
            TermForm::Verb => Some(match prefix {
                "role.editor" => "role.editor.verb",
                "role.translator" => "role.translator.verb",
                "role.guest" => "role.guest.verb",
                _ => return None,
            }),
            // CSL reference (scripts/locales-en-US.xml) distinguishes
            // "verb-short" from "verb" for editor ("ed. by" vs "edited by")
            // and translator ("trans. by" vs "translated by"). Guest has no
            // CSL-defined short verb form, so it falls back to the long verb.
            TermForm::VerbShort => Some(match prefix {
                "role.editor" => "role.editor.verb-short",
                "role.translator" => "role.translator.verb-short",
                "role.guest" => "role.guest.verb",
                _ => return None,
            }),
            _ => None,
        }
    }

    pub(super) fn locator_message_id(
        locator: &LocatorType,
        form: &TermForm,
    ) -> Option<&'static str> {
        let prefix = match locator {
            LocatorType::Page => "term.page-label",
            LocatorType::Chapter => "term.chapter-label",
            LocatorType::Volume => "term.volume-label",
            LocatorType::Section => "term.section-label",
            LocatorType::Figure => "term.figure-label",
            LocatorType::Note => "term.note-label",
            _ => return None,
        };

        match *form {
            TermForm::Long => Some(match prefix {
                "term.page-label" => "term.page-label-long",
                "term.chapter-label" => "term.chapter-label-long",
                "term.volume-label" => "term.volume-label-long",
                "term.section-label" => "term.section-label-long",
                "term.figure-label" => "term.figure-label-long",
                "term.note-label" => "term.note-label-long",
                _ => return None,
            }),
            TermForm::Short => Some(prefix),
            _ => None,
        }
    }

    pub(super) fn general_message_id(term: &GeneralTerm, form: &TermForm) -> Option<&'static str> {
        match (term, form) {
            (GeneralTerm::And, _) => Some("term.and"),
            (GeneralTerm::RoleConjunction, _) => Some("term.role-conjunction"),
            (GeneralTerm::EtAl, _) => Some("term.et-al"),
            (GeneralTerm::AndOthers, _) => Some("term.and-others"),
            (GeneralTerm::Accessed, _) => Some("term.accessed"),
            (GeneralTerm::Cited, _) => Some("term.cited"),
            (GeneralTerm::Retrieved, _) => Some("term.retrieved"),
            (GeneralTerm::NoDate, TermForm::Long) => Some("term.no-date-long"),
            (GeneralTerm::NoDate, _) => Some("term.no-date"),
            (GeneralTerm::Anonymous, TermForm::Long) => Some("term.anonymous-long"),
            (GeneralTerm::Anonymous, _) => Some("term.anonymous"),
            (GeneralTerm::Forthcoming, _) => Some("term.forthcoming"),
            (GeneralTerm::Circa, TermForm::Long) => Some("term.circa-long"),
            (GeneralTerm::Circa, _) => Some("term.circa"),
            _ => None,
        }
    }

    pub(super) fn gender_selector_key(gender: &GrammaticalGender) -> &str {
        match gender {
            GrammaticalGender::Masculine => "masculine",
            GrammaticalGender::Feminine => "feminine",
            GrammaticalGender::Neuter => "neuter",
            GrammaticalGender::Common => "common",
            GrammaticalGender::Unknown(s) => s.as_str(),
        }
    }

    /// Resolve a locale message by ID with caller-supplied MF2 arguments.
    ///
    /// This is the public boundary for style-template `message` components:
    /// templates select the message ID and pass named arguments, while the
    /// active locale owns the message body and evaluator.
    pub fn resolve_message(&self, message_id: &str, args: &MessageArgs<'_>) -> Option<String> {
        let message = self.messages.get(message_id)?;

        if !message.contains('{') {
            return Some(message.clone());
        }

        if self.evaluation.message_syntax == MessageSyntax::Static {
            return None;
        }

        self.evaluator
            .evaluate_for_locale(message, args, &self.locale)
    }

    /// Resolve a style-template message call, including term-backed message IDs.
    ///
    /// `term.*` IDs are allowed to fall through to the structured locale term
    /// resolver so checked-in styles can use the message component surface
    /// without duplicating legacy term data into every locale's `messages` map.
    pub fn resolve_template_message(
        &self,
        message_id: &str,
        args: &MessageArgs<'_>,
        form: Option<&TermForm>,
        gender: Option<GrammaticalGender>,
    ) -> Option<String> {
        if let Some((term, implied_form)) = Self::term_message_parts(message_id) {
            let effective_form = form.cloned().or(implied_form).unwrap_or(TermForm::Long);
            if let Some(value) = self.resolved_general_term(&term, &effective_form, gender) {
                return Some(value);
            }
        }

        self.resolve_message(message_id, args)
    }

    fn term_message_parts(message_id: &str) -> Option<(GeneralTerm, Option<TermForm>)> {
        let key = message_id.strip_prefix("term.")?;
        let (term_key, implied_form) = key
            .strip_suffix("-long")
            .map_or((key, None), |base| (base, Some(TermForm::Long)));
        let term = Self::parse_general_term(term_key)
            .unwrap_or_else(|| GeneralTerm::Unknown(term_key.to_string()));
        Some((term, implied_form))
    }

    pub(super) fn resolve_message_text(
        &self,
        message_id: &str,
        count: Option<u64>,
        gender: Option<GrammaticalGender>,
    ) -> Option<String> {
        let args = MessageArgs {
            count,
            gender: gender.as_ref().map(Self::gender_selector_key),
            ..MessageArgs::default()
        };

        self.resolve_message(message_id, &args)
    }
}
