use std::time::{Duration};

pub struct MyHeaders {
    pub key: String,
    pub value: String,
}

pub fn get_request(the_url: &str, timeout: u64) -> String {
    let client = reqwest::blocking::Client::new();
    let timeout_duration = Duration::from_secs(timeout);

    let response = match client.get(the_url)
        .timeout(timeout_duration)
        .send() {
        Ok(answer) => answer,
        Err(_) =>  return String::new(),
    };
 
    let status = response.status().as_u16();
    if status != 200 {
        return String::new()
    }
    else {
        let body: String = match response.text() {
            Ok(answer) => answer,
            Err(_) => String::new(),
        };
        return body
    }
}


pub fn post_request(the_body: &'static str, the_url: &'static str, header1: MyHeaders, header2: MyHeaders) -> String {
    /*let headers: Vec<MyHeaders> = vec![
            MyHeaders{key: "1".to_string(), value: "a".to_string()}, 
            MyHeaders{key: "2".to_string(), value: "b".to_string()}]; */

    let client = reqwest::blocking::Client::new();
    let response = match client.post(the_url)
        .header(header1.key,header1.value)
        .header(header2.key, header2.value)
        .body(the_body)
        .send() {
        Ok(answer) => answer,
        Err(_) =>  return String::new(),
    };
 
    let status = response.status().as_u16();
    if status != 200 {
        return String::new()
    }
    else {
        let body: String = match response.text() {
            Ok(answer) => answer,
            Err(_) => String::new(),
        };
        return body
    }
}
