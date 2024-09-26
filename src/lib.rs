use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VorlagenTitel {
    lang_key: String,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Staende {
    pub ja_staende_ganz: u8,
    pub nein_staende_ganz: u8,
    pub anzahl_staende_ganz: u8,
    pub ja_staende_halb: u8,
    pub nein_staende_halb: u8,
    pub anzahl_staende_halb: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resultat {
    pub gebiet_ausgezaehlt: bool,
    pub ja_stimmen_in_prozent: f64,
    pub ja_stimmen_absolut: u32,
    pub nein_stimmen_absolut: u32,
    pub stimmbeteiligung_in_prozent: f64,
    pub eingelegte_stimmzettel: u32,
    pub anzahl_stimmberechtigte: u32,
    pub gueltige_stimmen: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bezirk {
    pub geo_levelnummer: String,
    pub geo_levelname: String,
    pub resultat: Resultat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Gemeinde {
    pub geo_levelnummer: String,
    pub geo_levelname: String,
    pub geo_level_parentnummer: String,
    pub resultat: Resultat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Kanton {
    pub geo_levelnummer: String,
    pub geo_levelname: String,
    pub resultat: Resultat,
    pub bezirke: Option<Vec<Bezirk>>,
    pub gemeinden: Option<Vec<Gemeinde>>,
    pub zaehlkreise: Option<Vec<Gemeinde>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Vorlage {
    pub vorlagen_id: u32,
    pub reihenfolge_anzeige: u32,
    pub vorlagen_titel: Vec<VorlagenTitel>,
    pub vorlage_beendet: bool,
    pub provisorisch: bool,
    pub vorlage_angenommen: bool,
    pub vorlagen_art_id: u32,
    pub hauptvorlagen_id: u32,
    pub reserve_indo_text: Option<String>,
    pub doppeltes_mehr: bool,
    pub staende: Staende,
    pub resultat: Resultat,
    pub kantone: Vec<Kanton>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Schweiz {
    pub geo_levelnummer: u8,
    pub geo_levelname: String,
    pub noch_keine_information: bool,
    pub vorlagen: Vec<Vorlage>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub abstimmtag: String,
    pub timestamp: String,
    pub schweiz: Schweiz,
}

pub async fn get_data(url: &str) -> Result<Data, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let data: Data = serde_json::from_str(&response)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let url = "https://ogd-static.voteinfo-app.ch/v1/ogd/sd-t-17-02-20240922-eidgAbstimmung.json";
        let out = get_data(url).await;
        out.unwrap();
    }
}
