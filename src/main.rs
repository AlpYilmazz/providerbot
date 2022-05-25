
use std::{io::BufReader, fs::File, str::FromStr, env};

use clitrs::args::CliArgs;
use reqwest::Response;
use roux::{Reddit, Me, util::RouxError, Subreddit, subreddit::responses::{submissions, SubmissionsData}, responses::BasicThing};
use serde::Deserialize;
use tokio::task::{JoinHandle, JoinError};


#[derive(Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct Login {
    pub user: LoginUser,
    pub user_agent: String,
    pub client_id: String,
    pub client_secret: String,
}

impl Login {
    pub fn from(path: &str) -> Self {
        let login_file = File::open(path).expect("File not found");
        let login_rdr = BufReader::new(login_file);
        serde_json::from_reader(login_rdr).expect("Could not deserialize")
    }
}

#[derive(Debug)]
pub struct ParseError;

#[derive(Debug)]
pub enum RedditSort {
    // TODO: Top needs time period
    Best, Hot, New, Top, Controversial, Rising
}

impl FromStr for RedditSort {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "best" => Ok(RedditSort::Best),
            "hot" => Ok(RedditSort::Hot),
            "new" => Ok(RedditSort::New),
            "top" => Ok(RedditSort::Top),
            "controversial" => Ok(RedditSort::Controversial),
            "rising" => Ok(RedditSort::Rising),
            _ => Err(ParseError)
        }
    }
}

#[derive(Debug)]
pub enum Provide {
    BotCall(String),
    Nothing,
}

#[derive(Debug)]
pub struct Command {
    // Functionality
    pub provide: Provide,
    // View
    pub subreddit: String,
    pub sort: RedditSort,
    pub n: u32,
}

pub struct ProviderBot {
    login: Login,
    me: Me,
}

impl ProviderBot {
    pub async fn awake(login: Login) -> Result<Self, RouxError> {
        let me = Self::login_reddit(&login).await?;
        Ok(Self {
            login,
            me,
        })
    }

    async fn login_reddit(login: &Login) -> Result<Me, RouxError> {
        Reddit::new(&login.user_agent, &login.client_id, &login.client_secret)
            .username(&login.user.username)
            .password(&login.user.password)
            .login()
            .await
    }

    pub async fn do_the_thing(&self, command: Command) -> Result<(), RouxError> {
        println!("Providing\n{:?}", command);

        match &command.provide {
            Provide::BotCall(bot_name) => {
                let get_fullname = |t: &BasicThing<SubmissionsData>| t.data.name.clone();
                let sub = Subreddit::new(&command.subreddit);
                let mut submissions = None;
                match &command.sort {
                    RedditSort::Hot => submissions = Some(sub.hot(command.n, None).await?),
                    _ => todo!("Only hot feed sort is implemented"),
                }
                let submissions: Vec<String> = submissions.unwrap()
                        .data.children
                        .iter().map(get_fullname)
                        .collect();
                
                // let mut handles = Vec::with_capacity(submissions.len());
                for submission in &submissions {
                    // tokio::spawn();
                    self.me.comment(bot_name, submission).await?;
                }

                // for handle in handles {
                //     handle.await.unwrap()?;
                // }
            },
            Provide::Nothing => {},
        }
        
        Ok(())
    }
}


async fn reddit_test_1() {
    let nth: usize = 10;
    let sub = Subreddit::new("sandboxtest");
    let hot = sub.hot(nth as u32, None).await.unwrap();
    
    let a = &hot.data.children.get(nth-1).unwrap();
    dbg!(&a.data);
    dbg!(&a.kind);
    
    let id = a.data.id.clone();
    let c0 = sub.article_comments(&id, Some(1), Some(1)).await.unwrap();
    let c = &c0.data.children.get(0).unwrap();
    dbg!(&c.data);
    dbg!(&c.kind);
}


#[tokio::main]
async fn main() {

    // cargo run -- --subreddit=random -p botcall -n 10 -n 12
    
    let mut args = CliArgs::new();
    args.with("--subreddit/-r = s")
        .with("--provide/-p = s")
        .with("--sort/-s = s? ::>hot")
        .with("--n-posts/-n = i")
        .parse_cmd().expect("Args parse error");
    dbg!(&args);

    let login = Login::from("login.json");
    let provider_bot = ProviderBot::awake(login).await.unwrap();

    let command = Command {
        provide: Provide::Nothing, // Provide::BotCall("redditMP4bot".to_string()),
        subreddit: "random".to_string(),
        sort: RedditSort::from_str("hot").expect("Unknown type"),
        n: 1,
    };

    provider_bot.do_the_thing(command).await.unwrap();
}
