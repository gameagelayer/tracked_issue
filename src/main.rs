use async_recursion::async_recursion;
use graphql_client::{GraphQLQuery, Response};

pub type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.docs.graphql",
    query_path = "graphql/query.graphql"
)]
pub struct IssueQuery;

const GITHUB_URL: &str = "https://api.github.com/graphql";
const ORG: &str = "k-nasa";
const REPO: &str = "wai";
const ROOT_ISSUE_NUMBER: i64 = 1486;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let token = env!("GITHUB_ACCESS_TOKEN");

    let client = reqwest::Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            ))
            .collect(),
        )
        .build()?;

    println!("graph LR");
    fetch_tracked_issue(&client, ROOT_ISSUE_NUMBER).await?;
    println!(
        "classDef CLOSED fill:#8256d0,color:#FFFFFF,stroke-width:0px;
        classDef OPEN fill:#347d39,color:#FFFFFF,stroke-width:0px;"
    );

    Ok(())
}

#[async_recursion]
async fn fetch_tracked_issue(
    client: &reqwest::Client,
    root_issue: i64,
) -> Result<(), anyhow::Error> {
    let v = issue_query::Variables {
        owner: ORG.into(),
        repository_name: REPO.into(),
        number: root_issue,
    };
    let request_body = IssueQuery::build_query(v);

    let res = client.post(GITHUB_URL).json(&request_body).send().await?;
    let response_body: Response<issue_query::ResponseData> = res.json().await?;
    for i in response_body
        .data
        .unwrap()
        .repository
        .unwrap()
        .issue
        .unwrap()
        .tracked_issues
        .nodes
        .unwrap()
    {
        let i = i.as_ref().unwrap();

        let state = match i.state {
            issue_query::IssueState::OPEN => "OPEN",
            issue_query::IssueState::CLOSED => "CLOSED",
            _ => "OTHER",
        };

        println!(
            "\t{} --> {}[\"{}\"]:::{}",
            root_issue, i.number, i.title, state
        );

        println!("click {} href \"{}\" _blank", i.number, i.url);
        fetch_tracked_issue(client, i.number).await?;
    }

    Ok(())
}
