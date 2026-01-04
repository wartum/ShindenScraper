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
    save(&all_anime)
}

fn save(all_anime: &[Anime]) -> Result<()> {
    let mut file = File::create("anime_season.csv").unwrap();
    file.write_all(Anime::get_csv_headers().as_bytes())?;
    for anime in all_anime.iter() {
        if anime.anime_type == "TV" {
            file.write_all(anime.to_csv().as_bytes())?;
            file.write_all(b"\n")?;
        }
    }

    Ok(())
}
