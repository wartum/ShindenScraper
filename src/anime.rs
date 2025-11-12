use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub struct Anime {
    pub title: String,
    pub anime_type: String,
    pub status: String,
    pub emmision_date: String,
    pub genres: String,
    pub target_groups: String,
    pub shinden_url: String,
    pub ogladajanime_url: String,
}

impl PartialOrd for Anime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.emmision_date
                .cmp(&other.emmision_date)
                .then(self.title.cmp(&other.title)),
        )
    }
}

impl Ord for Anime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.emmision_date
            .cmp(&other.emmision_date)
            .then(self.title.cmp(&other.title))
    }
}

impl Display for Anime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r##"{}
- Status: {}
- Typ: {}
- Grupa docelowa: {}
- Gatunek: {}
- Data emisji: {}
- Shinden: {}"##,
            self.title,
            self.status,
            self.anime_type,
            self.target_groups,
            self.genres,
            self.emmision_date,
            self.shinden_url
        )
    }
}

impl Anime {
    pub fn to_csv(&self) -> String {
        format!(
            r##""{}", "{}", "{}", "{}", "{}", "{}", "{}", "{}""##,
            self.status,
            self.anime_type,
            self.emmision_date,
            self.target_groups,
            self.genres,
            self.title,
            self.ogladajanime_url,
            self.shinden_url,
        )
    }
}
