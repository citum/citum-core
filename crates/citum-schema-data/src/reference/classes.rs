/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus and Citum contributors
*/

//! Reference class discriminator types and shared dispatch helpers.

#[cfg(feature = "bindings")]
use specta::Type;

use serde::{Deserialize, Serialize};

use super::input::UnknownClassData;
use super::types::legal::{Brief, Hearing, LegalCase, Regulation, Statute, Treaty};
use super::types::specialized::{
    AudioVisualWork, Classic, Dataset, Event, Patent, Software, Standard,
};
use super::types::structural::{
    Collection, CollectionComponent, Monograph, Serial, SerialComponent,
};

/// Typed class discriminator returned by [`super::InputReference::class`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "bindings", derive(Type))]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceClass {
    /// A monograph, such as a book or report.
    Monograph,
    /// A component of a larger monographic collection.
    CollectionComponent,
    /// A component of a larger serial publication.
    SerialComponent,
    /// A collection of works, such as an anthology or proceedings.
    Collection,
    /// A serial publication, such as a journal or newspaper.
    Serial,
    /// A legal case.
    LegalCase,
    /// A statute or legislative act.
    Statute,
    /// An international treaty or agreement.
    Treaty,
    /// A legislative or administrative hearing.
    Hearing,
    /// An administrative regulation.
    Regulation,
    /// A legal brief or filing.
    Brief,
    /// A classic work with standard citation forms.
    Classic,
    /// A patent.
    Patent,
    /// A research dataset.
    Dataset,
    /// A technical standard or specification.
    Standard,
    /// Software or source code.
    Software,
    /// An event such as a conference, performance, or broadcast.
    Event,
    /// An audio-visual work.
    AudioVisual,
    /// A class string not known by this version.
    ///
    /// `#[serde(skip)]`: an `Unknown(...)` variant cannot serialize directly
    /// through the derived `Serialize` impl on `ReferenceClass`. The class
    /// string is preserved on the wire via [`UnknownClassData::class`] inside
    /// the surrounding `InputReference`, not via this enum standalone.
    #[serde(skip)]
    Unknown(String),
}

impl ReferenceClass {
    /// Known class names in their wire-format spelling.
    pub const KNOWN: &'static [&'static str] = &[
        "monograph",
        "collection-component",
        "serial-component",
        "collection",
        "serial",
        "legal-case",
        "statute",
        "treaty",
        "hearing",
        "regulation",
        "brief",
        "classic",
        "patent",
        "dataset",
        "standard",
        "software",
        "event",
        "audio-visual",
    ];

    /// Return the known discriminator for a wire-format class name.
    #[must_use]
    pub(crate) fn from_known_name(class: &str) -> Option<Self> {
        match class {
            "monograph" => Some(Self::Monograph),
            "collection-component" => Some(Self::CollectionComponent),
            "serial-component" => Some(Self::SerialComponent),
            "collection" => Some(Self::Collection),
            "serial" => Some(Self::Serial),
            "legal-case" => Some(Self::LegalCase),
            "statute" => Some(Self::Statute),
            "treaty" => Some(Self::Treaty),
            "hearing" => Some(Self::Hearing),
            "regulation" => Some(Self::Regulation),
            "brief" => Some(Self::Brief),
            "classic" => Some(Self::Classic),
            "patent" => Some(Self::Patent),
            "dataset" => Some(Self::Dataset),
            "standard" => Some(Self::Standard),
            "software" => Some(Self::Software),
            "event" => Some(Self::Event),
            "audio-visual" => Some(Self::AudioVisual),
            _ => None,
        }
    }

    /// Return this class's wire-format spelling.
    #[cfg(feature = "schema")]
    #[must_use]
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Monograph => "monograph",
            Self::CollectionComponent => "collection-component",
            Self::SerialComponent => "serial-component",
            Self::Collection => "collection",
            Self::Serial => "serial",
            Self::LegalCase => "legal-case",
            Self::Statute => "statute",
            Self::Treaty => "treaty",
            Self::Hearing => "hearing",
            Self::Regulation => "regulation",
            Self::Brief => "brief",
            Self::Classic => "classic",
            Self::Patent => "patent",
            Self::Dataset => "dataset",
            Self::Standard => "standard",
            Self::Software => "software",
            Self::Event => "event",
            Self::AudioVisual => "audio-visual",
            Self::Unknown(class) => class,
        }
    }
}

/// Class-specific overlay stored inside [`super::InputReference`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub enum ClassExtension {
    /// Monograph-specific payload.
    Monograph(Box<Monograph>),
    /// Collection-component-specific payload.
    CollectionComponent(Box<CollectionComponent>),
    /// Serial-component-specific payload.
    SerialComponent(Box<SerialComponent>),
    /// Collection-specific payload.
    Collection(Box<Collection>),
    /// Serial-specific payload.
    Serial(Box<Serial>),
    /// Legal-case-specific payload.
    LegalCase(Box<LegalCase>),
    /// Statute-specific payload.
    Statute(Box<Statute>),
    /// Treaty-specific payload.
    Treaty(Box<Treaty>),
    /// Hearing-specific payload.
    Hearing(Box<Hearing>),
    /// Regulation-specific payload.
    Regulation(Box<Regulation>),
    /// Brief-specific payload.
    Brief(Box<Brief>),
    /// Classic-work-specific payload.
    Classic(Box<Classic>),
    /// Patent-specific payload.
    Patent(Box<Patent>),
    /// Dataset-specific payload.
    Dataset(Box<Dataset>),
    /// Standard-specific payload.
    Standard(Box<Standard>),
    /// Software-specific payload.
    Software(Box<Software>),
    /// Event-specific payload.
    Event(Box<Event>),
    /// Audio-visual-specific payload.
    AudioVisual(Box<AudioVisualWork>),
    /// Unknown-class payload.
    Unknown(Box<UnknownClassData>),
}

impl ClassExtension {
    /// Return the typed discriminator corresponding to this extension.
    #[must_use]
    pub(crate) fn reference_class(&self) -> ReferenceClass {
        match self {
            Self::Monograph(_) => ReferenceClass::Monograph,
            Self::CollectionComponent(_) => ReferenceClass::CollectionComponent,
            Self::SerialComponent(_) => ReferenceClass::SerialComponent,
            Self::Collection(_) => ReferenceClass::Collection,
            Self::Serial(_) => ReferenceClass::Serial,
            Self::LegalCase(_) => ReferenceClass::LegalCase,
            Self::Statute(_) => ReferenceClass::Statute,
            Self::Treaty(_) => ReferenceClass::Treaty,
            Self::Hearing(_) => ReferenceClass::Hearing,
            Self::Regulation(_) => ReferenceClass::Regulation,
            Self::Brief(_) => ReferenceClass::Brief,
            Self::Classic(_) => ReferenceClass::Classic,
            Self::Patent(_) => ReferenceClass::Patent,
            Self::Dataset(_) => ReferenceClass::Dataset,
            Self::Standard(_) => ReferenceClass::Standard,
            Self::Software(_) => ReferenceClass::Software,
            Self::Event(_) => ReferenceClass::Event,
            Self::AudioVisual(_) => ReferenceClass::AudioVisual,
            Self::Unknown(data) => ReferenceClass::Unknown(data.class.clone()),
        }
    }

    /// Return the wire-format class string for this extension.
    #[must_use]
    pub(crate) fn class_name(&self) -> &str {
        match self {
            Self::Monograph(_) => "monograph",
            Self::CollectionComponent(_) => "collection-component",
            Self::SerialComponent(_) => "serial-component",
            Self::Collection(_) => "collection",
            Self::Serial(_) => "serial",
            Self::LegalCase(_) => "legal-case",
            Self::Statute(_) => "statute",
            Self::Treaty(_) => "treaty",
            Self::Hearing(_) => "hearing",
            Self::Regulation(_) => "regulation",
            Self::Brief(_) => "brief",
            Self::Classic(_) => "classic",
            Self::Patent(_) => "patent",
            Self::Dataset(_) => "dataset",
            Self::Standard(_) => "standard",
            Self::Software(_) => "software",
            Self::Event(_) => "event",
            Self::AudioVisual(_) => "audio-visual",
            Self::Unknown(data) => &data.class,
        }
    }
}

macro_rules! class_dispatch {
    (
        $extension:expr,
        |$reference:ident| $body:expr,
        audio_visual($audio_visual:ident) => $audio_visual_body:expr,
        unknown($unknown:pat) => $unknown_body:expr
    ) => {
        match $extension {
            ClassExtension::Monograph($reference) => $body,
            ClassExtension::CollectionComponent($reference) => $body,
            ClassExtension::SerialComponent($reference) => $body,
            ClassExtension::Collection($reference) => $body,
            ClassExtension::Serial($reference) => $body,
            ClassExtension::LegalCase($reference) => $body,
            ClassExtension::Statute($reference) => $body,
            ClassExtension::Treaty($reference) => $body,
            ClassExtension::Hearing($reference) => $body,
            ClassExtension::Regulation($reference) => $body,
            ClassExtension::Brief($reference) => $body,
            ClassExtension::Classic($reference) => $body,
            ClassExtension::Patent($reference) => $body,
            ClassExtension::Dataset($reference) => $body,
            ClassExtension::Standard($reference) => $body,
            ClassExtension::Software($reference) => $body,
            ClassExtension::Event($reference) => $body,
            ClassExtension::AudioVisual($audio_visual) => $audio_visual_body,
            ClassExtension::Unknown($unknown) => $unknown_body,
        }
    };

    ($extension:expr, |$reference:ident| $body:expr, unknown($unknown:pat) => $unknown_body:expr) => {
        match $extension {
            ClassExtension::Monograph($reference) => $body,
            ClassExtension::CollectionComponent($reference) => $body,
            ClassExtension::SerialComponent($reference) => $body,
            ClassExtension::Collection($reference) => $body,
            ClassExtension::Serial($reference) => $body,
            ClassExtension::LegalCase($reference) => $body,
            ClassExtension::Statute($reference) => $body,
            ClassExtension::Treaty($reference) => $body,
            ClassExtension::Hearing($reference) => $body,
            ClassExtension::Regulation($reference) => $body,
            ClassExtension::Brief($reference) => $body,
            ClassExtension::Classic($reference) => $body,
            ClassExtension::Patent($reference) => $body,
            ClassExtension::Dataset($reference) => $body,
            ClassExtension::Standard($reference) => $body,
            ClassExtension::Software($reference) => $body,
            ClassExtension::Event($reference) => $body,
            ClassExtension::AudioVisual($reference) => $body,
            ClassExtension::Unknown($unknown) => $unknown_body,
        }
    };
}

pub(crate) use class_dispatch;
