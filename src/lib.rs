use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};

async fn get_latest_url(url: &str) -> Result<String> {
    #[derive(Serialize, Deserialize)]
    struct Resource {
        coverage: String,
        url: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Resources {
        resources: Vec<Resource>,
    }

    #[derive(Serialize, Deserialize)]
    struct Results {
        result: Resources,
    }

    let response = reqwest::get(url).await?.text().await?;
    let results: Results = serde_json::from_str(&response)?;
    let resources = results.result.resources;
    let resource = resources.iter().max_by(|a, b| a.coverage.cmp(&b.coverage));
    if let Some(resource) = resource {
        Ok(resource.url.clone())
    } else {
        Err(anyhow::Error::msg("no resources found"))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    DE,
    FR,
    IT,
    RM,
    EN,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct IssueTitle {
    #[serde(rename = "langKey")]
    lang: Lang,
    text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    #[serde(rename = "gebietAusgezaehlt")]
    pub count_completed: bool,
    #[serde(rename = "jaStimmenAbsolut")]
    pub yes_votes: u32,
    #[serde(rename = "neinStimmenAbsolut")]
    pub no_votes: u32,
    #[serde(rename = "eingelegteStimmzettel")]
    pub cast_ballot_papers: u32,
    #[serde(rename = "anzahlStimmberechtigte")]
    pub eligible_voters: u32,
}

impl Outcome {
    pub fn yes_ratio(&self) -> f64 {
        1.0 / self.valid_votes() as f64 * self.yes_votes as f64
    }

    pub fn no_ratio(&self) -> f64 {
        1.0 / self.valid_votes() as f64 * self.no_votes as f64
    }

    pub fn valid_votes(&self) -> u32 {
        self.yes_votes + self.no_votes
    }

    pub fn invalid_votes(&self) -> u32 {
        self.cast_ballot_papers - self.valid_votes()
    }

    pub fn valid_votes_ratio(&self) -> f64 {
        1.0 / self.cast_ballot_papers as f64 * self.valid_votes() as f64
    }

    pub fn invalid_votes_ratio(&self) -> f64 {
        1.0 / self.cast_ballot_papers as f64 * self.invalid_votes() as f64
    }

    pub fn turnout(&self) -> f64 {
        1.0 / self.eligible_voters as f64 * self.valid_votes() as f64
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct District {
    #[serde(rename = "geoLevelnummer")]
    pub geo_levelnumber: String,
    #[serde(rename = "geoLevelname")]
    pub geo_levelname: String,
    #[serde(rename = "resultat")]
    pub outcome: Outcome,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Commune {
    #[serde(rename = "geoLevelnummer")]
    pub geo_levelnumber: String,
    #[serde(rename = "geoLevelname")]
    pub geo_levelname: String,
    #[serde(rename = "geoLevelParentnummer")]
    pub geo_level_parentnumber: String,
    #[serde(rename = "resultat")]
    pub outcome: Outcome,
}

type Constituency = Commune;

pub mod national {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OutcomeCantons {
        #[serde(rename = "jaStaendeGanz")]
        pub yes_full_cantons: u8,
        #[serde(rename = "neinStaendeGanz")]
        pub no_full_cantons: u8,
        #[serde(rename = "anzahlStaendeGanz")]
        pub full_canton_count: u8,
        #[serde(rename = "jaStaendeHalb")]
        pub yes_half_cantons: u8,
        #[serde(rename = "neinStaendeHalb")]
        pub no_half_cantons: u8,
        #[serde(rename = "anzahlStaendeHalb")]
        pub half_canton_count: u8,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Canton {
        #[serde(rename = "geoLevelnummer")]
        pub geo_levelnumber: String,
        #[serde(rename = "geoLevelname")]
        pub geo_levelname: String,
        #[serde(rename = "resultat")]
        pub outcome: Outcome,
        #[serde(rename = "bezirke")]
        pub districts: Option<Vec<District>>,
        #[serde(rename = "gemeinden")]
        pub communes: Option<Vec<Commune>>,
        #[serde(rename = "zaehlkreise")]
        pub constituencies: Option<Vec<Constituency>>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Issue {
        #[serde(rename = "vorlagenId")]
        pub issue_id: u32,
        #[serde(rename = "reihenfolgeAnzeige")]
        pub display_order: u32,
        #[serde(rename = "vorlagenTitel")]
        pub issue_title: Vec<IssueTitle>,
        #[serde(rename = "vorlageBeendet")]
        pub issue_completed: bool,
        #[serde(rename = "provisorisch")]
        pub provisional: bool,
        #[serde(rename = "vorlageAngenommen")]
        pub issue_accepted: bool,
        #[serde(rename = "vorlagenArtId")]
        pub issue_type_id: u32,
        #[serde(rename = "hauptvorlagenId")]
        pub main_issue_id: u32,
        #[serde(rename = "reserveInfoText")]
        pub reserve_info_text: Option<String>,
        #[serde(rename = "doppeltesMehr")]
        pub double_majority: bool,
        #[serde(rename = "staende")]
        pub outcome_cantons: OutcomeCantons,
        #[serde(rename = "resultat")]
        pub outcome: Outcome,
        #[serde(rename = "kantone")]
        pub cantons: Vec<Canton>,
    }

    impl Issue {
        pub fn get_title(&self, lang: Lang) -> Option<&str> {
            self.issue_title.iter().find_map(|title| {
                if title.lang == lang && !title.text.chars().all(char::is_whitespace) {
                    Some(title.text.as_str())
                } else {
                    None
                }
            })
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Country {
        #[serde(rename = "geoLevelnummer")]
        pub geo_levelnumber: u8,
        #[serde(rename = "geoLevelname")]
        pub geo_levelname: String,
        #[serde(rename = "nochKeineInformation")]
        pub no_infos_yet: bool,
        #[serde(rename = "vorlagen")]
        pub issues: Vec<Issue>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Data {
        pub abstimmtag: String,
        pub timestamp: String,
        #[serde(rename = "schweiz")]
        pub country: Country,
    }

    pub async fn get_data_by_url(url: &str) -> Result<Data> {
        let response = reqwest::get(url).await?.text().await?;
        let data: Data = serde_json::from_str(&response)?;
        Ok(data)
    }

    pub async fn get_latest() -> Result<Data> {
        let url = "https://ckan.opendata.swiss/api/3/action/package_show?id=echtzeitdaten-am-abstimmungstag-zu-eidgenoessischen-abstimmungsvorlagen";
        let url = get_latest_url(url).await?;
        let response = reqwest::get(url).await?.text().await?;
        let data: Data = serde_json::from_str(&response)?;
        Ok(data)
    }
}

pub mod cantonal {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Issue {
        #[serde(rename = "vorlagenId")]
        pub issue_id: u32,
        #[serde(rename = "reihenfolgeAnzeige")]
        pub display_order: u32,
        #[serde(rename = "vorlagenTitel")]
        pub issue_title: Vec<IssueTitle>,
        #[serde(rename = "vorlageBeendet")]
        pub issue_completed: bool,
        #[serde(rename = "vorlageAngenommen")]
        pub issue_accepted: bool,
        #[serde(rename = "vorlagenArtId")]
        pub issue_type_id: u32,
        #[serde(rename = "hauptvorlagenId")]
        pub main_issue_id: Option<u32>,
        #[serde(rename = "resultat")]
        pub outcome: Outcome,
        #[serde(rename = "bezirke")]
        pub districts: Option<Vec<District>>,
        #[serde(rename = "gemeinden")]
        pub communes: Option<Vec<Commune>>,
        #[serde(rename = "zaehlkreise")]
        pub constituencies: Option<Vec<Constituency>>,
    }

    impl Issue {
        pub fn get_title(&self, lang: Lang) -> Option<&str> {
            self.issue_title.iter().find_map(|title| {
                if title.lang == lang && !title.text.chars().all(char::is_whitespace) {
                    Some(title.text.as_str())
                } else {
                    None
                }
            })
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Canton {
        #[serde(rename = "geoLevelnummer")]
        pub geo_levelnumber: u8,
        #[serde(rename = "geoLevelname")]
        pub geo_levelname: String,
        #[serde(rename = "nochKeineInformation")]
        pub no_infos_yet: bool,
        #[serde(rename = "vorlagen")]
        pub issues: Vec<Issue>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Data {
        pub abstimmtag: String,
        pub timestamp: String,
        #[serde(rename = "kantone")]
        pub kantone: Vec<Canton>,
    }

    pub async fn get_data_by_url(url: &str) -> Result<Data> {
        let response = reqwest::get(url).await?.text().await?;
        let data: Data = serde_json::from_str(&response)?;
        Ok(data)
    }

    pub async fn get_latest() -> Result<Data> {
        let url = "https://ckan.opendata.swiss/api/3/action/package_show?id=echtzeitdaten-am-abstimmungstag-zu-kantonalen-abstimmungsvorlagen";
        let url = get_latest_url(url).await?;
        let response = reqwest::get(url).await?.text().await?;
        let data: Data = serde_json::from_str(&response)?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn national_by_url() {
        let url =
            "https://ogd-static.voteinfo-app.ch/v1/ogd/sd-t-17-02-20240922-eidgAbstimmung.json";
        let out = national::get_data_by_url(url).await;
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn national_latest() {
        let out = national::get_latest().await;
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn cantonal_by_url() {
        let url =
            "https://ogd-static.voteinfo-app.ch/v1/ogd/sd-t-17-02-20240922-kantAbstimmung.json";
        let out = cantonal::get_data_by_url(url).await;
        assert!(out.is_ok());
    }

    #[tokio::test]
    async fn cantonal_latest() {
        let out = cantonal::get_latest().await;
        assert!(out.is_ok());
    }
}
