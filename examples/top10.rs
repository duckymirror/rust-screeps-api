//! Simple command line program to view the top 10 users in power processing and GCL.
//!
//! Logs in using the SCREEPS_API_USERNAME and SCREEPS_API_PASSWORD env variables.
extern crate dotenv;
extern crate fern;
extern crate log;
extern crate screeps_api;

use screeps_api::LeaderboardType::*;
use screeps_api::endpoints::leaderboard::page::LeaderboardPage;

/// Set up dotenv and retrieve a specific variable, informatively panicking if it does not exist.
fn env(var: &str) -> String {
    dotenv::dotenv().ok();
    match ::std::env::var(var) {
        Ok(value) => value,
        Err(e) => panic!("must have `{}` defined (err: {:?})", var, e),
    }
}

fn opt_env(var: &str, default: &str) -> String {
    dotenv::dotenv().ok();
    match ::std::env::var(var) {
        Ok(value) => value,
        Err(_) => default.to_owned(),
    }
}

/// Prints to stdout information derived from the leaderboard page result.
fn print_ranks(result: &LeaderboardPage) {
    for ranked_user in &result.ranks {
        match result
            .user_details
            .iter()
            .find(|x| x.0 == ranked_user.user_id)
        {
            Some(&(_, ref details)) => {
                println!("\t[{}] {} (GCL {})", ranked_user.rank, details.username, details.gcl_points);
            }
            None => {
                println!("\t[{}] {}", ranked_user.rank, ranked_user.user_id);
            }
        }
    }
}

fn main() {
    fern::Dispatch::new()
        .level(log::LogLevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let mut client = screeps_api::SyncConfig::new()
        .unwrap()
        .url(&opt_env("SCREEPS_API_URL", screeps_api::DEFAULT_OFFICIAL_API_URL))
        .build()
        .unwrap();

    client
        .login(env("SCREEPS_API_USERNAME"), env("SCREEPS_API_PASSWORD"))
        .unwrap();

    let top10gcl = client
        .leaderboard_page(GlobalControl, "2017-02", 10, 0)
        .unwrap();
    println!("Top 10 world leaderboard:");
    print_ranks(&top10gcl);

    let top10power = client
        .leaderboard_page(PowerProcessed, "2017-02", 10, 0)
        .unwrap();
    println!("Top 10 power leaderboard:");
    print_ranks(&top10power);
}
