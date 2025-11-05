use redis::Commands;
use std::time::SystemTime;
use std::collections::HashMap;
use serde_json::json;

type Error = Box<dyn std::error::Error>;

const REDIS_BASE_URL: &str = "redis.openfaas-yjn.svc.cluster.local:6379";
// const DATA_SIZE: usize = 1024*4;
const KEY: &str = "key";

pub fn handle(_body: Vec<u8>) -> Result<Vec<u8>, Error> {
    let client = redis::Client::open(format!("redis://{}/", REDIS_BASE_URL))?;
    let mut con = client.get_connection()?;
    let event: HashMap<String, serde_json::Value> = serde_json::from_slice(&_body)?;

    let data_size = &event["data_size"].as_u64().unwrap();
    let data_size_usize = *data_size as usize;
    let input_data = "a".repeat(data_size_usize);

    let start: SystemTime = SystemTime::now();
    println!("####start:{:?}", start);

    let _ = put_object(&(KEY.to_string()), &input_data, &mut con);
    println!("####write over");

    let _: String  = get_object(&(KEY.to_string()), &mut con)?;
    println!("####read over");

    let end: SystemTime = SystemTime::now();
    println!("####end:{:?}", end);
    let cost_time = end.duration_since(start).unwrap().as_secs_f64() * 1000.0;

    let rsp = json!({
        "cost_time": cost_time,
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