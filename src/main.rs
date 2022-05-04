use std::{env, io};
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;
use std::time::Duration;
use actix_web::{App, dev, Error, get, http, HttpRequest, HttpResponse, HttpServer, Result, web};
use actix_web::client::{Client, Connector};
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound, ErrorUriTooLong};
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use dotenv::dotenv;
use lazy_static::lazy_static;
use log::info;
use log::LevelFilter::Info;
use regex::{escape, Regex};
use serde::Deserialize;

lazy_static! {
    static ref ENABLE_REGEX: bool = env_lookup("ENABLE_REGEX", false);
    static ref MAX_PATTERN_LEN: i32 = env_lookup("MAX_PATTER_LEN", 70);

    static ref ENABLE_CUSTOM_HOSTS: bool = env_lookup("ENABLE_CUSTOM_HOSTS", false);

    static ref TAG_PATTERN: Regex = Regex::new(r"(?P<major>\d+)([.-_](?P<minor>\d+)([.-_](?P<patch>\d+))?([.-_]?(?P<pre>[\w\d]+))?)?").unwrap();
    static ref REPLACE_PATTERN: Regex = Regex::new(r"\{\w*?}").unwrap();

    static ref USER_AGENT: String = format!("smartrelease/{}", env_lookup::<String>("CARGO_PKG_VERSION", "".to_string()));
}

#[derive(Deserialize)]
struct Query {
    /// Normally unmatched wildcards are remaining unedited in the pattern
    /// but with this option enabled, these unmatched wildcard are cleared cut out
    clear_unknown: Option<bool>,

    /// Reverses the asset order provided by the api
    reverse: Option<bool>,

    // The following fields are alternatives when a wildcard could not be matched
    major: Option<String>,
    minor: Option<String>,
    patch: Option<String>,
    pre: Option<String>,
    tag: Option<String>
}

#[derive(Deserialize)]
struct Assets {
    name: String,
    browser_download_url: String
}

#[derive(Deserialize)]
struct GitHub {
    tag_name: String,
    assets: Vec<Assets>
}

#[get("/github/{user}/{repo}/{pattern}")]
async fn github(
    web::Path((user, repo, pattern)): web::Path<(String, String, String)>,
    query: web::Query<Query>
) -> Result<HttpResponse> {
    request_github(user.as_str(), repo.as_str(), pattern.as_str(), query).await
}

#[derive(Deserialize)]
struct Gitea {
    tag_name: String,
    assets: Vec<Assets>
}

#[get("/gitea/{user}/{repo}/{pattern}")]
async fn gitea(
    web::Path((user, repo, pattern)): web::Path<(String, String, String)>,
    query: web::Query<Query>
) -> Result<HttpResponse> {
    request_gitea("gitea.com", user.as_str(), repo.as_str(), pattern.as_str(), query).await
}

#[get("/custom/{host}/{platform}/{user}/{repo}/{pattern}")]
async fn custom(
    web::Path((host, platform, user, repo, pattern)): web::Path<(String, String, String, String, String)>,
    query: web::Query<Query>
) -> Result<HttpResponse> {
    if *ENABLE_CUSTOM_HOSTS {
        match platform.as_str() {
            "gitea" => request_gitea(host.as_str(), user.as_str(), repo.as_str(), pattern.as_str(), query).await,
            _ => Err(ErrorBadRequest("Invalid host"))
        }
    } else {
        Err(ErrorNotFound("Custom hosts are disabled"))
    }
}

async fn request_github(user: &str, repo: &str, pattern: &str, query: web::Query<Query>) -> Result<HttpResponse> {
    if let Some(err) = pre_check(pattern) {
        return Err(err)
    }

    let mut res = client()
        .get(format!("https://api.github.com/repos/{}/{}/releases/latest", user, repo))
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", USER_AGENT.as_str())
        .send()
        .await?;
    let mut result = res.json::<GitHub>().await?;

    process(&pattern, &mut result.assets, query.into_inner(), &result.tag_name)
}

async fn request_gitea(host: &str, user: &str, repo: &str, pattern: &str, query: web::Query<Query>) -> Result<HttpResponse> {
    if let Some(err) = pre_check(pattern) {
        return Err(err)
    }

    let mut res = client()
        .get(format!("{}://{}/api/v1/repos/{}/{}/releases?limit=1", if env_lookup("HTTPS_ONLY", true) { "HTTPS" } else { "HTTP" }, host, user, repo))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::USER_AGENT, USER_AGENT.as_str())
        .send()
        .await?;
    let mut result = res.json::<[Gitea; 1]>().await?;

    return process(pattern, &mut result[0].assets, query.into_inner(), &result[0].tag_name )
}

fn redirect_error<B>(res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    if res.request().uri().path() == "/favicon" {
        return Ok(ErrorHandlerResponse::Response(res))
    }

    let split_path: Vec<&str> = res.request().uri().path().split("/").collect();

    info!("{} {}: got {} ({})",
        ip(res.request()),
        res.request().path(),
        res.status().as_u16(),
        res.response().error().map_or_else(|| String::new(), |v| v.to_string()));

    if split_path.len() >= 4 {
        let location = match *split_path.get(1).unwrap() {
            "github" => format!("https://github.com/{}/{}/releases/latest",
                                *split_path.get(2).unwrap(),
                                *split_path.get(3).unwrap()),
            "gitea" => format!("https://gitea.com/{}/{}/releases",
                               *split_path.get(2).unwrap(),
                               *split_path.get(3).unwrap()),
            _ => "".to_string()
        };

        if location != "" {
            return Ok(ErrorHandlerResponse::Response(
                res.into_response(
                    HttpResponse::Found()
                        .header(http::header::LOCATION, location)
                        .finish()
                        .into_body()
                )
            ))
        }
    }

    Ok(ErrorHandlerResponse::Response(res))
}

fn client() -> Client {
    return Client::builder()
        .timeout(Duration::from_secs(5))
        .connector(
            Connector::new()
                .timeout(Duration::from_secs(3))
                .finish()
        )
        .finish();
}

fn env_lookup<F: FromStr>(name: &str, default: F) -> F {
    if let Ok(envvar) = env::var(name) {
        envvar.parse::<F>().unwrap_or(default)
    } else {
        default
    }
}

fn ip(request: &HttpRequest) -> String {
    request
        .connection_info().realip_remote_addr().unwrap()
        .rsplit_once(":").unwrap().0.to_string()
}

fn pre_check(pattern: &str) -> Option<Error> {
    // if MAX_PATTERN_LEN is -1 or below the len checking is disabled
    if *MAX_PATTERN_LEN > -1 && REPLACE_PATTERN.replace_all(pattern, "").len() > *MAX_PATTERN_LEN as usize {
        return Some(ErrorUriTooLong(format!("Pattern / last url path must not exceed {} characters", *MAX_PATTERN_LEN)))
    }
    None
}

fn process(pattern: &str, assets: &mut Vec<Assets>, query: Query, tag_name: &String) -> Result<HttpResponse> {
    let re: Regex;
    let mut replaced = replace(
        pattern.to_string(),
        tag_name.to_string(),
        [
            ("major", query.major),
            ("minor", query.minor),
            ("patch", query.patch),
            ("pre", query.pre),
            ("tag", query.tag)
        ].iter().cloned().collect(),
        query.clear_unknown.unwrap_or(true)
    );
    if !*ENABLE_REGEX {
        replaced = escape(replaced.as_str())
    }

    match Regex::new(replaced.as_str()) {
        Ok(r) => re = r,
        Err(e) => {
            return Err(ErrorInternalServerError(e));
        }
    }

    if query.reverse.unwrap_or(false) {
        assets.reverse()
    }

    for asset in assets {
        if re.is_match(asset.name.as_str()) {
            return Ok(HttpResponse::Found().set_header(http::header::LOCATION, format!("{}", asset.browser_download_url)).finish())
        }
    }

    Err(ErrorNotFound("No matching asset was found"))
}

fn replace(pattern: String, tag: String, alternatives: HashMap<&str, Option<String>>, clear_unknown: bool) -> String {
    let mut result = pattern;

    if let Some(regex_match) = TAG_PATTERN.captures(tag.as_str()) {
        for name in ["major", "minor", "patch", "pre"] {
            if let Some(named) = regex_match.name(name) {
                result = result.replace(format!("{{{}}}", name).as_str(), named.as_str());
            }
        }
    }

    result = result.replace("{tag}", tag.as_str());

    for alternative in alternatives {
        if let Some(value) = alternative.1 {
            result = result.replace(format!("{{{}}}", alternative.0).as_str(), value.as_str());
        }
    }

    if clear_unknown {
        result = REPLACE_PATTERN.replace_all(result.as_str(), "").to_string()
    }

    result
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "actix_server=warn")
    }

    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] - {}: {}",
                buf.timestamp(),
                record.level(),
                buf.style().value(record.args())
            )
        })
        .filter_level(Info)
        .init();

    let host = env_lookup::<String>("HOST", "0.0.0.0".to_string());
    let port = env_lookup::<i16>("PORT", 8080);

    let server = HttpServer::new(|| {
        App::new()
            .service(github)
            .service(gitea)
            .service(custom)
            .service(
                web::resource("/").route(web::get().to(|| async {
                    HttpResponse::Found()
                        .header(http::header::LOCATION, "https://github.com/ByteDream/smartrelease")
                        .finish()
                })
            ))
            .wrap(
                ErrorHandlers::new()
                    .handler(http::StatusCode::BAD_REQUEST, redirect_error)
                    .handler(http::StatusCode::NOT_FOUND, redirect_error)
                    .handler(http::StatusCode::GATEWAY_TIMEOUT, redirect_error)
            )
    })
        .bind(format!("{}:{}", host, port))?
        .run();

    info!(
        "Started server on {}:{} with regex {} and a max pattern len of {}",
        host,
        port,
        if *ENABLE_REGEX { "enabled" } else { "disabled" },
        *MAX_PATTERN_LEN
    );

    server.await
}
