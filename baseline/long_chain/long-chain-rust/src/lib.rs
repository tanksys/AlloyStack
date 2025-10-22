use std::collections::HashMap;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use serde_json::json;
use redis::Commands;

type Error = Box<dyn std::error::Error>;

const REDIS_BASE_URL: &str = "redis.openfaas-yjn.svc.cluster.local:6379";
const KEY: &str = "key";

pub fn handle(_body: Vec<u8>) -> Result<Vec<u8>, Error> {
    let client = redis::Client::open(format!("redis://{}/", REDIS_BASE_URL))?;
    let mut con = client.get_connection()?;
    let event: HashMap<String, serde_json::Value> = serde_json::from_slice(&_body)?;

    let now_n = &event["now_n"].as_u64().unwrap();
    let tot_n = &event["tot_n"].as_u64().unwrap();
    let mut timestamp = event["timestamp"].as_f64().unwrap();
    let mut timestamp_end: f64 = 0.0;
    // let mut timestamp_now: f64 = 0.0;
    let mut size_of_output: usize = 0;

    if *now_n == 1 {
        let start: SystemTime = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH);
        timestamp = since_the_epoch.unwrap().as_secs_f64() * 1000.0;

        let data_size = &event["data_size"].as_u64().unwrap();
        let data_size_usize = *data_size as usize;
        let input_data = "a".repeat(data_size_usize);
        let _ = put_object(&(KEY.to_string()), &input_data, &mut con);
    } else if *now_n == *tot_n {
        let output: String = get_object(&(KEY.to_string()), &mut con)?;
        size_of_output = output.len();

        let end: SystemTime = SystemTime::now();
        let since_the_epoch = end
            .duration_since(UNIX_EPOCH);
        timestamp_end = since_the_epoch.unwrap().as_secs_f64() * 1000.0;
    } else {
        let output: String = get_object(&(KEY.to_string()), &mut con)?;
        let _ = put_object(&(KEY.to_string()), &output, &mut con);
    }

    // let now: SystemTime = SystemTime::now();
    // let since_the_epoch = now.duration_since(UNIX_EPOCH);
    // timestamp_now = since_the_epoch.unwrap().as_secs_f64() * 1000.0;

    let rsp = json!({
        "timestamp_start": timestamp,
        "timestamp_end": timestamp_end,
        // "timestamp_now": timestamp_now,
        "output_size": size_of_output,
    });

    Ok(rsp.to_string().into_bytes())
}


fn put_object(key: &String, val: &String, con: &mut redis::Connection) -> Result<(), Error> {
    con.set(key, val)?;
    Ok(())
}

fn get_object(key: &String, con: &mut redis::Connection) -> Result<String, Error> {
    Ok(con.get(key)?)
}
