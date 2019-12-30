#[macro_use]
extern crate mysql;

use mysql::chrono::NaiveDateTime;
use serde_json::Value;
use std::env;

#[derive(Debug)]
pub struct FeedbackEvent {
    id: i32,
    datetime: NaiveDateTime,
    round_id: i32,
    key_name: String,
    key_type: String,
    version: i32,
    json: Value,
}

#[derive(Debug)]
pub struct FeedbackEventNormalised {
    datetime: NaiveDateTime,
    round_id: i32,
    category_primary: String,
    category_secondary: String,
    category_tertiary: String,
    version: i32,
    value: String,
}

fn main() -> Result<(), std::io::Error> {
    let database_url =
        env::var("FEEDBACK_DATABASE_URL").expect("Could not read FEEDBACK_DATABASE_URL variable");
    let feedback_table_name =
        env::var("FEEDBACK_TABLE_NAME").expect("Could not read FEEDBACK_TABLE_NAME variable");
    let feedback_normalized_table_name = env::var("FEEDBACK_NORMALISED_TABLE_NAME")
        .expect("Could not read FEEDBACK_NORMALISED_TABLE_NAME variable");
    // let database_url = "mysql://root:password@localhost:3306/ss13";
    // let feedback_table_name = "erro_feedback";
    // let feedback_normalized_table_name = "erro_feedback_normalized";
    let pool = mysql::Pool::new(database_url).unwrap();

    let events: Vec<FeedbackEvent> = pool
        .prep_exec(
            format!(
                "SELECT * FROM {} WHERE `round_id` NOT IN (SELECT `round_id` FROM {})",
                feedback_table_name, feedback_normalized_table_name
            ),
            (),
        )
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (id, datetime, round_id, key_name, key_type, version, json) =
                        mysql::from_row(row);
                    let json: String = json;
                    let json = serde_json::from_str(&json).unwrap();
                    FeedbackEvent {
                        id: id,
                        datetime: datetime,
                        round_id: round_id,
                        key_name: key_name,
                        key_type: key_type,
                        version: version,
                        json: json,
                    }
                })
                .collect()
        })
        .unwrap();

    let mut normalised_events = Vec::new();
    for event in &events {
        normalised_events.extend(match event.key_type.as_str() {
            "amount" => process_amount(event),
            "tally" => process_tally(event),
            "associative" => process_associative(event),
            "nested tally" => process_nested_tally(event),
            "text" => process_text(event),
            _ => panic!("Unexpected key_type: {}", event.key_type),
        });
    }

    let mut stmt = pool
        .prepare(format!(
            "INSERT INTO {} (datetime, round_id, category_primary, category_secondary, category_tertiary, version, data) VALUES
                (:datetime, :round_id, :category_primary, :category_secondary, :category_tertiary, :version, :data)",
            feedback_normalized_table_name
        )).unwrap();

    for event in normalised_events.iter() {
        stmt.execute(params! {
            "datetime" => event.datetime,
            "round_id" => event.round_id,
            "category_primary" => &event.category_primary,
            "category_secondary" => &event.category_secondary,
            "category_tertiary" => &event.category_tertiary,
            "version" => event.version,
            "data" => &event.value,
        })
        .unwrap();
    }

    // println!("Hello, world! {:?}", normalised_events);

    Ok(())
}

pub fn process_amount(event: &FeedbackEvent) -> Vec<FeedbackEventNormalised> {
    vec![FeedbackEventNormalised {
        datetime: event.datetime,
        round_id: event.round_id,
        category_primary: event.key_name.clone(),
        category_secondary: "".to_string(),
        category_tertiary: "".to_string(),
        version: event.version,
        value: event.json["data"].to_string(),
    }]
}

pub fn process_tally(event: &FeedbackEvent) -> Vec<FeedbackEventNormalised> {
    let mut normalised_events = vec![];
    for (key, value) in event.json["data"].as_object().unwrap() {
        normalised_events.push(FeedbackEventNormalised {
            datetime: event.datetime,
            round_id: event.round_id,
            category_primary: event.key_name.clone(),
            category_secondary: key.to_string(),
            category_tertiary: "".to_string(),
            version: event.version,
            value: value.to_string(),
        });
    }

    normalised_events
}

pub fn process_nested_tally(event: &FeedbackEvent) -> Vec<FeedbackEventNormalised> {
    let mut normalised_events = vec![];
    for (key, value) in event.json["data"].as_object().unwrap() {
        for (nested_key, nested_value) in value.as_object().unwrap() {
            normalised_events.push(FeedbackEventNormalised {
                datetime: event.datetime,
                round_id: event.round_id,
                category_primary: event.key_name.clone(),
                category_secondary: key.to_string(),
                category_tertiary: nested_key.to_string(),
                version: event.version,
                value: nested_value.to_string(),
            });
        }
    }

    normalised_events
}

pub fn process_associative(event: &FeedbackEvent) -> Vec<FeedbackEventNormalised> {
    println!("Received associative value, ignoring. Associative events will need a more manual approach. {:?}", event.key_name);
    vec![]
}

pub fn process_text(event: &FeedbackEvent) -> Vec<FeedbackEventNormalised> {
    let mut normalised_events = vec![];
    if event.json["data"].is_array() {
        for value in event.json["data"].as_array().unwrap() {
            normalised_events.push(FeedbackEventNormalised {
                datetime: event.datetime,
                round_id: event.round_id,
                category_primary: event.key_name.clone(),
                category_secondary: "".to_string(),
                category_tertiary: "".to_string(),
                version: event.version,
                value: value.to_string(),
            });
        }
    } else {
        normalised_events.push(FeedbackEventNormalised {
            datetime: event.datetime,
            round_id: event.round_id,
            category_primary: event.key_name.clone(),
            category_secondary: "".to_string(),
            category_tertiary: "".to_string(),
            version: event.version,
            value: event.json["data"].to_string(),
        });
    }

    normalised_events
}
