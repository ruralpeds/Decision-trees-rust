use serde::{Deserialize, Serialize};

/// A reference to evidence supporting a clinical recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub ref_type: EvidenceType,
    pub title: String,
    pub url: Option<String>,
    pub pmid: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    PeerReviewedJournal,
    Guideline,
    SocietyStatement,
    ClinicalTrial,
    SystematicReview,
    MetaAnalysis,
    UpToDate,
    CDC,
    FDA,
    AAP,
    ExpertOpinion,
}

/// A reference to a formal clinical guideline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineRef {
    pub organization: String,
    pub title: String,
    pub url: Option<String>,
    pub publication_year: Option<u16>,
    pub section: Option<String>,
}

impl EvidenceRef {
    pub fn new(ref_type: EvidenceType, title: impl Into<String>) -> Self {
        Self {
            ref_type,
            title: title.into(),
            url: None,
            pmid: None,
            doi: None,
            publication_year: None,
        }
    }

    pub fn with_pmid(mut self, pmid: impl Into<String>) -> Self {
        self.pmid = Some(pmid.into());
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_doi(mut self, doi: impl Into<String>) -> Self {
        self.doi = Some(doi.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_ref_creation() {
        let evidence = EvidenceRef::new(
            EvidenceType::PeerReviewedJournal,
            "Respiratory Distress in Neonates",
        )
        .with_pmid("12345678")
        .with_url("https://pubmed.ncbi.nlm.nih.gov/12345678");

        assert_eq!(evidence.pmid, Some("12345678".to_string()));
        assert!(evidence.url.is_some());
    }

    #[test]
    fn test_guideline_ref() {
        let guideline = GuidelineRef {
            organization: "American Academy of Pediatrics".to_string(),
            title: "Neonatal Respiratory Distress Management".to_string(),
            url: Some("https://aap.org/...".to_string()),
            publication_year: Some(2023),
            section: Some("Section 3.2".to_string()),
        };

        assert_eq!(guideline.organization, "American Academy of Pediatrics");
    }
}
