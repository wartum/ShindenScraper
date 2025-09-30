mod anime;
mod scrape;

use anime::Anime;
use scrape::scrape;
use std::{fs::File, io::Write};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut all_anime = scrape().await?;
    all_anime.sort();
    save(&all_anime);
    Ok(())
}

fn save(all_anime: &[Anime]) {
    let mut file = File::create("anime_season.csv").unwrap();
    _ = file.write_all("Status, Typ, Data emisji, Grupa docelowa, Gatunek, Tytuł, Link do Ogladajanime, Link do Shinden\n".as_bytes());
    for anime in all_anime.iter() {
        _ = file.write_all(anime.to_csv().as_bytes());
        _ = file.write_all(b"\n");
    }
}
