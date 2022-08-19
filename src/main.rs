use chrono::{DateTime, Utc};
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, USER_AGENT},
    StatusCode,
};
use serde_json::json;
use std::{env, io, process::ExitCode};

const PAGINATION_LIMIT: i64 = 100;
const API_ENDPOINT: &str = "https://api.github.com/graphql";

struct Issue {
    number: u64,
    created_at: DateTime<Utc>,
    closed_at: DateTime<Utc>,
}

#[derive(Debug)]
enum RepoError {
    ConnectionError(reqwest::Error),
}

fn post_request(
    client: &Client,
    query: &str,
    token: &str,
) -> Result<(String, StatusCode), RepoError> {
    let resp = client
        .post(API_ENDPOINT)
        .header(USER_AGENT, "gh-issue-count-history")
        .header(AUTHORIZATION, format!("bearer {}", token))
        .body(json!({ "query": query }).to_string())
        .send()
        .map_err(|err| RepoError::ConnectionError(err))?;
    let status = resp.status();
    Ok((
        resp.text().map_err(|err| RepoError::ConnectionError(err))?,
        status,
    ))
}

fn process_repo(owner: &str, name: &str, token: &str) -> Result<(), RepoError> {
    let client = Client::new();
    let first_query = format!(
        r#"
    query {{
        repository(name:"{}",owner:"{}") {{
          issues(first:1) {{
            edges {{
              cursor,
              node {{
                number,
                createdAt,
                closedAt
              }}
            }}
          }}
        }}
      }}"#,
        name, owner
    );
    println!("{}", first_query);
    println!("{:?}", post_request(&client, &first_query, token)?);
    Ok(())
    /*
    let regular_query = format!(
        r#"
    query {{
        repository(name:"{}",owner:"{}") {{
          issues(first:{},after:{}) {{
            edges {{
              cursor,
              node {{
                number,
                createdAt,
                closedAt
              }}
            }}
          }}
        }}
      }}"#,
        name, owner, PAGINATION_LIMIT, current_cursor
    );
    println!("Hello, world!");*/
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        let (owner, name) = (args[1].as_str(), args[2].as_str());
        let mut token = String::new();
        eprint!("Please enter your GitHub personal access token: ");
        if let Err(err) = io::stdin().read_line(&mut token) {
            eprintln!("Failed to read token from stdin: {}", err);
            return ExitCode::FAILURE;
        }
        match process_repo(owner, name, token.trim()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Processing repository failed with error: {:?}", err);
                ExitCode::FAILURE
            }
        }
    } else {
        eprintln!("Usage: {} <repo-owner> <repo-name>", args[0]);
        ExitCode::FAILURE
    }
}
