use crate::Result;
use crate::anime::*;
use reqwest::redirect::Policy;
use reqwest::{Client, ClientBuilder, header};
use scraper::{Html, Selector};

const CURRENT_SEASON_URL: &str = "https://shinden.pl/series/season/current";

pub async fn scrape() -> Result<Vec<Anime>> {
    let client = build_http_client(true)?;
    let all_anime_list_rsp = client
        .get(CURRENT_SEASON_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let document = Html::parse_document(&all_anime_list_rsp);
    let selector = Selector::parse("li.title > h3.box-title > a").unwrap();

    let mut all_anime = Vec::new();
    for element in document.select(&selector) {
        let url = format!("https://shinden.pl{}", element.attr("href").unwrap());
        if let Ok(anime) = scrape_anime_details(&url, &client).await {
            println!("{anime}\n=====");
            all_anime.push(anime);
        }
    }
    Ok(all_anime)
}

fn build_http_client(redirect: bool) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Accept-Language",
        header::HeaderValue::from_static("pl,en-US;q=0.7,en;q=0.3"),
    );
    headers.insert(
        "User-Agent",
        header::HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64; rv:143.0) Gecko/20100101 Firefox/143.0",
        ),
    );

    if redirect {
        ClientBuilder::new()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .map_err(|e| e.into())
    } else {
        ClientBuilder::new()
            .cookie_store(true)
            .default_headers(headers)
            .redirect(Policy::none())
            .build()
            .map_err(|e| e.into())
    }
}

async fn scrape_anime_details(url: &str, client: &Client) -> Result<Anime> {
    let anime_page_rsp = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let document = Html::parse_document(&anime_page_rsp);

    let title_selector = Selector::parse("h1.page-title > span.title").unwrap();
    let info_selector = Selector::parse("dl.info-aside-list > dd").unwrap();
    let tags_selector = Selector::parse("ul.tags > li > a.button-with-tip").unwrap();

    let mut title_iterator = document.select(&title_selector);
    let mut info_iterator = document.select(&info_selector);
    let tags_iterator = document.select(&tags_selector);

    let title = title_iterator
        .next()
        .ok_or("No title found")?
        .inner_html()
        .trim()
        .replace(",", "")
        .to_string();
    let anime_type = info_iterator
        .next()
        .ok_or("No type found")?
        .inner_html()
        .trim()
        .to_string();
    let status = info_iterator
        .next()
        .ok_or("No status found")?
        .inner_html()
        .trim()
        .to_string();
    let emmision_date = make_date(
        info_iterator
            .next()
            .ok_or("No emission date found")?
            .inner_html()
            .trim(),
    );

    let mut genres = String::new();
    let mut target_groups = String::new();
    for element in tags_iterator {
        if element.attr("href").unwrap_or("").contains("/genre/") {
            genres.push_str(format!("{} ", element.inner_html().trim()).as_str());
        } else if element.attr("href").unwrap_or("").contains("/targetgroup/") {
            target_groups.push_str(format!("{} ", element.inner_html().trim()).as_str());
        }
    }

    let noredirect_client = build_http_client(false)?;
    let ogladajanime_url = make_ogladajanime_url(&title, &noredirect_client).await;

    Ok(Anime {
        title,
        anime_type,
        status,
        emmision_date,
        genres,
        target_groups,
        shinden_url: url.to_owned(),
        ogladajanime_url,
    })
}

fn is_char_allowed_in_url(c: &char) -> bool {
    let blacklisted_chars = ['(', ')', '[', ']', '{', '}', '!', '?', ':', ';', '"', '\''];
    c.is_ascii() && !blacklisted_chars.contains(c)
}

async fn make_ogladajanime_url(title: &str, client: &Client) -> String {
    let title = title.replace(" ", "-").replace("/", "-");
    let title: String = title.chars().filter(is_char_allowed_in_url).collect();
    let url = format!(
        "https://ogladajanime.pl/anime/{}",
        title.to_ascii_lowercase().trim_end_matches('.')
    );

    if let Ok(res) = validate_ogladajanime_url(&url, client).await {
        if res {
            return url.to_string();
        }
    }

    return "".to_string();
}

async fn validate_ogladajanime_url(url: &str, client: &Client) -> Result<bool> {
    let rsp = client.get(url).send().await?;

    if rsp.status() == 200 {
        return Ok(true);
    } else {
        return Ok(false);
    }
}

fn make_date(date: &str) -> String {
    if date.chars().filter(|c| *c == '.').count() < 2 {
        format! {"??.{}", date}
    } else {
        date.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scrape_anime() {
        let url = "https://shinden.pl/series/68750-boku-no-hero-academia-final-season";
        let client = build_http_client(true).unwrap();
        let anime = scrape_anime_details(url, &client).await.unwrap();
        assert_eq!(anime.title, "Boku no Hero Academia: Final Season");
        assert_eq!(anime.anime_type, "TV");
        assert_eq!(anime.status, "Zakończone");
        assert_eq!(anime.emmision_date, "04.10.2025");
        assert_eq!(anime.genres.trim(), "Akcja Fantasy");
        assert_eq!(anime.target_groups.trim(), "Shounen");
        assert_eq!(anime.shinden_url, url);
    }

    #[tokio::test]
    async fn make_ogladajanime_url() {
        let client = build_http_client(false).unwrap();

        assert_eq!(
            super::make_ogladajanime_url("Fate/strange Fake", &client).await,
            "https://ogladajanime.pl/anime/fate-strange-fake"
        );
        assert_eq!(
            super::make_ogladajanime_url("[Oshi no Ko] 3rd Season", &client).await,
            "https://ogladajanime.pl/anime/oshi-no-ko-3rd-season"
        );
        assert_eq!(
            super::make_ogladajanime_url("Cardfight!! Vanguard: Divinez Genma Seisen-hen", &client)
                .await,
            "https://ogladajanime.pl/anime/cardfight-vanguard-divinez-genma-seisen-hen"
        );
        assert_eq!(
            super::make_ogladajanime_url("jujutsu-kaisen-3rd-season", &client).await,
            ""
        );
    }
}
