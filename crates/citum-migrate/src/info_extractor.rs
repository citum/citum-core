use citum_schema::{CitationField, StyleInfo, StyleLink, StylePerson, StyleSource};
use csl_legacy::model::Info as LegacyInfo;

/// Converts a CSL 1.0 `Info` struct into a Citum `StyleInfo` with provenance.
pub struct InfoExtractor;

impl InfoExtractor {
    #[must_use]
    pub fn extract(legacy: &LegacyInfo) -> StyleInfo {
        let fields = legacy
            .fields
            .iter()
            .filter_map(|f| parse_field(f))
            .collect::<Vec<_>>();

        let original_authors = legacy
            .authors
            .iter()
            .map(|p| StylePerson {
                name: p.name.clone(),
                email: p.email.clone(),
                uri: p.uri.clone(),
            })
            .collect::<Vec<_>>();

        let links = legacy
            .links
            .iter()
            .map(|l| StyleLink {
                href: l.href.clone(),
                rel: l.rel.clone(),
            })
            .collect::<Vec<_>>();

        let source = if legacy.id.is_empty() {
            None
        } else {
            Some(StyleSource {
                csl_id: legacy.id.clone(),
                adapted_by: Some("citum-migrate".to_string()),
                license: legacy.rights.clone(),
                original_authors,
                links,
            })
        };

        StyleInfo {
            title: Some(legacy.title.clone()).filter(|s| !s.is_empty()),
            id: None, // Citum styles get their own ID separately
            description: legacy.summary.clone(),
            default_locale: None, // populated elsewhere
            fields,
            source,
            short_name: None, // set manually on well-known styles
            edition: None,    // set manually on well-known styles
        }
    }
}

fn parse_field(s: &str) -> Option<CitationField> {
    match s {
        "anthropology" => Some(CitationField::Anthropology),
        "biology" => Some(CitationField::Biology),
        "botany" => Some(CitationField::Botany),
        "chemistry" => Some(CitationField::Chemistry),
        "communications" => Some(CitationField::Communications),
        "engineering" => Some(CitationField::Engineering),
        "geography" => Some(CitationField::Geography),
        "geology" => Some(CitationField::Geology),
        "history" => Some(CitationField::History),
        "humanities" => Some(CitationField::Humanities),
        "law" => Some(CitationField::Law),
        "linguistics" => Some(CitationField::Linguistics),
        "literature" => Some(CitationField::Literature),
        "math" => Some(CitationField::Math),
        "medicine" => Some(CitationField::Medicine),
        "philosophy" => Some(CitationField::Philosophy),
        "physics" => Some(CitationField::Physics),
        "political-science" => Some(CitationField::PoliticalScience),
        "psychology" => Some(CitationField::Psychology),
        "science" => Some(CitationField::Science),
        "social-science" => Some(CitationField::SocialScience),
        "sociology" => Some(CitationField::Sociology),
        "theology" => Some(CitationField::Theology),
        "zoology" => Some(CitationField::Zoology),
        _ => None,
    }
}
