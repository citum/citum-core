/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

//! Construction helpers for [`InputReference`].
//!
//! Hosts the transitional PascalCase constructors (kept until the
//! `csl26-1bdr` cutover lands), the snake_case `from_boxed_*` helpers, and the
//! private `from_known<T>` used by the serde dispatcher.

use serde::Deserialize;
use serde_json::Value as JsonValue;

use super::types::legal::{Brief, Hearing, LegalCase, Regulation, Statute, Treaty};
use super::types::specialized::{
    AudioVisualWork, Classic, Dataset, Event, Patent, Software, Standard,
};
use super::types::structural::{
    Collection, CollectionComponent, Monograph, Serial, SerialComponent,
};
use super::{ClassExtension, InputReference, UnknownClassData};

macro_rules! boxed_reference_constructor {
    ($fn_name:ident, $ty:ty, $variant:ident) => {
        fn $fn_name(reference: Box<$ty>) -> Self {
            Self {
                extension: ClassExtension::$variant(reference),
            }
        }
    };
}

impl InputReference {
    pub(super) fn from_known<T>(
        class_extension: impl FnOnce(Box<T>) -> ClassExtension,
        value: JsonValue,
    ) -> Result<Self, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Defensive: the dispatcher in `deserialize_reference_body` only ever
        // hands us a `JsonValue::Object`; reject anything else with a schema
        // error rather than letting `from_value` produce an opaque type error.
        if !matches!(&value, JsonValue::Object(_)) {
            return Err(<serde_json::Error as serde::de::Error>::custom(
                "reference body must be a JSON object",
            ));
        }
        let inner = serde_json::from_value(value)?;
        Ok(Self {
            extension: class_extension(Box::new(inner)),
        })
    }

    // ────────────────────────────────────────────────────────────────────
    // Transitional PascalCase constructors.
    //
    // These exist to keep call sites that used to construct an `InputReference`
    // enum variant working unchanged through the discriminator cutover
    // (bean: csl26-1bdr). They are NOT the long-term public surface and
    // SHOULD NOT be relied upon by new code — use a `Box::new(<Class> { … })`
    // value plus a future snake_case factory once one is exposed.
    //
    // TODO(csl26-1bdr): replace these 19 functions with idiomatic snake_case
    // constructors and drop the transitional set. Tracked by parent epic.
    // ────────────────────────────────────────────────────────────────────

    /// Construct a monograph reference (transitional; see block note above).
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Monograph(reference: Box<Monograph>) -> Self {
        Self::from_boxed_monograph(reference)
    }

    /// Construct a collection-component reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn CollectionComponent(reference: Box<CollectionComponent>) -> Self {
        Self::from_boxed_collection_component(reference)
    }

    /// Construct a serial-component reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn SerialComponent(reference: Box<SerialComponent>) -> Self {
        Self::from_boxed_serial_component(reference)
    }

    /// Construct a collection reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Collection(reference: Box<Collection>) -> Self {
        Self::from_boxed_collection(reference)
    }

    /// Construct a serial reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Serial(reference: Box<Serial>) -> Self {
        Self::from_boxed_serial(reference)
    }

    /// Construct a legal-case reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn LegalCase(reference: Box<LegalCase>) -> Self {
        Self::from_boxed_legal_case(reference)
    }

    /// Construct a statute reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Statute(reference: Box<Statute>) -> Self {
        Self::from_boxed_statute(reference)
    }

    /// Construct a treaty reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Treaty(reference: Box<Treaty>) -> Self {
        Self::from_boxed_treaty(reference)
    }

    /// Construct a hearing reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Hearing(reference: Box<Hearing>) -> Self {
        Self::from_boxed_hearing(reference)
    }

    /// Construct a regulation reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Regulation(reference: Box<Regulation>) -> Self {
        Self::from_boxed_regulation(reference)
    }

    /// Construct a brief reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Brief(reference: Box<Brief>) -> Self {
        Self::from_boxed_brief(reference)
    }

    /// Construct a classic reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Classic(reference: Box<Classic>) -> Self {
        Self::from_boxed_classic(reference)
    }

    /// Construct a patent reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Patent(reference: Box<Patent>) -> Self {
        Self::from_boxed_patent(reference)
    }

    /// Construct a dataset reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Dataset(reference: Box<Dataset>) -> Self {
        Self::from_boxed_dataset(reference)
    }

    /// Construct a standard reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Standard(reference: Box<Standard>) -> Self {
        Self::from_boxed_standard(reference)
    }

    /// Construct a software reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Software(reference: Box<Software>) -> Self {
        Self::from_boxed_software(reference)
    }

    /// Construct an event reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Event(reference: Box<Event>) -> Self {
        Self::from_boxed_event(reference)
    }

    /// Construct an audio-visual reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn AudioVisual(reference: Box<AudioVisualWork>) -> Self {
        Self::from_boxed_audio_visual(reference)
    }

    /// Construct an unknown-class reference.
    #[allow(non_snake_case, reason = "Transitional compatibility constructor")]
    #[must_use]
    pub fn Unknown(reference: Box<UnknownClassData>) -> Self {
        Self {
            extension: ClassExtension::Unknown(reference),
        }
    }

    boxed_reference_constructor!(from_boxed_monograph, Monograph, Monograph);
    boxed_reference_constructor!(
        from_boxed_collection_component,
        CollectionComponent,
        CollectionComponent
    );
    boxed_reference_constructor!(
        from_boxed_serial_component,
        SerialComponent,
        SerialComponent
    );
    boxed_reference_constructor!(from_boxed_collection, Collection, Collection);
    boxed_reference_constructor!(from_boxed_serial, Serial, Serial);
    boxed_reference_constructor!(from_boxed_legal_case, LegalCase, LegalCase);
    boxed_reference_constructor!(from_boxed_statute, Statute, Statute);
    boxed_reference_constructor!(from_boxed_treaty, Treaty, Treaty);
    boxed_reference_constructor!(from_boxed_hearing, Hearing, Hearing);
    boxed_reference_constructor!(from_boxed_regulation, Regulation, Regulation);
    boxed_reference_constructor!(from_boxed_brief, Brief, Brief);
    boxed_reference_constructor!(from_boxed_classic, Classic, Classic);
    boxed_reference_constructor!(from_boxed_patent, Patent, Patent);
    boxed_reference_constructor!(from_boxed_dataset, Dataset, Dataset);
    boxed_reference_constructor!(from_boxed_standard, Standard, Standard);
    boxed_reference_constructor!(from_boxed_software, Software, Software);
    boxed_reference_constructor!(from_boxed_event, Event, Event);
    boxed_reference_constructor!(from_boxed_audio_visual, AudioVisualWork, AudioVisual);
}
