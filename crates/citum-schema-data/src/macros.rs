/*
SPDX-License-Identifier: MIT OR Apache-2.0
SPDX-FileCopyrightText: © 2023-2026 Bruce D'Arcus
*/

/// Generates a string-backed enum that gracefully captures unknown variants.
///
/// The macro handles custom `serde::Deserialize` and `serde::Serialize` to ensure
/// unknown string values are captured into an `Unknown(String)` variant instead
/// of failing the parse. It also configures `schemars` and `specta` to skip the
/// `Unknown` variant, maintaining a strictly closed public schema for producers.
#[macro_export]
macro_rules! tolerant_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident = $val:expr
            ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "bindings", derive(specta::Type))]
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                #[cfg_attr(any(feature = "schema", feature = "bindings"), serde(rename = $val))]
                $variant,
            )+
            #[doc = "Fallback for forward-compatibility."]
            #[cfg_attr(feature = "schema", schemars(skip))]
            #[cfg_attr(feature = "bindings", specta(skip))]
            Unknown(String),
        }

        impl $name {
            #[doc = "Returns the string value associated with this variant."]
            #[must_use]
            pub fn as_str(&self) -> &str {
                match self {
                    $( Self::$variant => $val, )+
                    Self::Unknown(s) => s.as_str(),
                }
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct Visitor;
                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("a string")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok(match value {
                            $( $val => $name::$variant, )+
                            _ => $name::Unknown(value.to_owned()),
                        })
                    }
                }
                deserializer.deserialize_str(Visitor)
            }
        }
    }
}
