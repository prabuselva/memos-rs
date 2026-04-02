use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Filter {
    must: Vec<Condition>,
}

#[derive(Debug, Serialize)]
struct Condition {
    key: String,
    match_value: MatchCondition,
}

#[derive(Debug, Serialize)]
struct MatchCondition {
    match_value: Match,
}

#[derive(Debug, Serialize)]
struct Match {
    value: String,
}

fn main() {
    let filter = Filter {
        must: vec![Condition {
            key: "user_id".to_string(),
            match_value: MatchCondition {
                match_value: Match {
                    value: "test-user".to_string(),
                },
            },
        }],
    };

    println!("{}", serde_json::to_string_pretty(&filter).unwrap());
}
