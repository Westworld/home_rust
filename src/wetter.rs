//use core::time;
//use tokio::time::error::Elapsed;
use std::thread::sleep;
use std::time::{Duration};
use serde::{Deserialize, Serialize};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct WCurrentNew {
    temp: i32,
    wind_speed: i32,
    rain: u32,
    main: String,
    description: String,
    icon: String,
    day: String,
    hour: String,
} 

fn send_message(the_topic: &str, the_payload:String, tx: &std::sync::mpsc::Sender<crate::Mymessage>) {
    let answ1 = crate::Mymessage {
        topic: String::from(the_topic),
        payload: the_payload,
    };
    if let Err(_) =  tx.send(answ1) {/* nothing */};    
}

fn get_wetter_sub(current: json::JsonValue) -> WCurrentNew  {

    let mut newcurrent = WCurrentNew {
        wind_speed: 0,
        temp: 0,
        rain: 0,
        main: "".to_string(),
        description: "".to_string(),
        icon: "".to_string(),
        day: "".to_string(),
        hour: "".to_string(),
    };

    let result: Option<f64> =  current["temp"].as_f64();
    if let Some(i) = result {
        newcurrent.temp = i as i32;
    } 

    let result: Option<f64> =  current["wind_speed"].as_f64();
    if let Some(i) = result {
        newcurrent.wind_speed = (i * 3.6) as i32;
    } 

    let result: Option<f64> =  current["rain"]["1h"].as_f64();
    if let Some(i) = result {
        newcurrent.rain = i as u32;
    } 
    let result: Option<f64> =  current["snow"]["1h"].as_f64();
    if let Some(i) = result {
        newcurrent.rain = i as u32;
    } 

    newcurrent.main = current["weather"][0]["main"].to_string();
    newcurrent.description = current["weather"][0]["description"].to_string();
    newcurrent.icon = current["weather"][0]["icon"].to_string();

        // Convert the timestamp string into an i64
    let timestamp: i64;
    let result = current["dt"].as_i64();
    if let Some(i) = result {
        timestamp = i;
    } else {
        timestamp = 0;
    }

    let utc = DateTime::from_timestamp(timestamp, 0).unwrap();
    let local: DateTime<Local> = DateTime::from(utc);

    //let local: DateTime<Local> = Local::now();
    newcurrent.day = format!("{}", local.format("%a"));
    newcurrent.hour = format!("{}", local.format("%H:%M"));

    return newcurrent;

}

fn parse_wetter(tx: &std::sync::mpsc::Sender<crate::Mymessage>, answer: String) {

    if answer.starts_with("{") {

        let parsed: json::JsonValue = match json::parse (&answer) {
            Ok (p) => p,
            Err (_) => return,
        };

        if !parsed["current"].is_null() {
            let new_current = get_wetter_sub(parsed["current"].clone());
        
            #[cfg(debug_assertions)]
            let serialized = serde_json::to_string(&new_current).unwrap();
            #[cfg(debug_assertions)]
            println!("current: {}", serialized);

            send_message("HomeServer/Wetter/temp", new_current.temp.to_string(), tx);
            send_message("HomeServer/Wetter/wind", new_current.wind_speed.to_string(), tx);
            send_message("HomeServer/Wetter/main", new_current.main, tx);
            send_message("HomeServer/Wetter/description", new_current.description, tx);
            send_message("HomeServer/Wetter/icon", new_current.icon, tx);
            send_message("HomeServer/Wetter/rain", new_current.rain.to_string(), tx);

            let mut rain_forecast: Vec<f64> = Vec::new();
            let minutely = parsed["minutely"].clone();
            if minutely.is_array() {
                 for ele in minutely.members() {
                    rain_forecast.push(ele["precipitation"].as_f64().expect("rain no number"));
                }               
            }
    
            let rain_forecast_str = format!("{:?}", rain_forecast);
            send_message("HomeServer/Wetter/rainForecast", rain_forecast_str, tx);

            let hourly = parsed["hourly"].clone();
            if hourly.is_array() {
                let new_current = get_wetter_sub(hourly[1].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour1", serialized, tx);
 
                let new_current = get_wetter_sub(hourly[4].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour2", serialized, tx);

                let new_current = get_wetter_sub(hourly[7].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour3", serialized, tx);

                let new_current = get_wetter_sub(hourly[10].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour4", serialized, tx);

                let new_current = get_wetter_sub(hourly[13].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour5", serialized, tx);                                                 

                let new_current = get_wetter_sub(hourly[16].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour6", serialized, tx);

                let new_current = get_wetter_sub(hourly[19].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour7", serialized, tx);

                let new_current = get_wetter_sub(hourly[22].clone());
                let serialized = serde_json::to_string(&new_current).unwrap();
                send_message("HomeServer/Wetter/hour8", serialized, tx);                                                 
 
                let mut temp_forecast: Vec<f64> = Vec::new();
                for ele in hourly.members() {
                    temp_forecast.push(ele["temp"].as_f64().expect("temp no number"));
                }               
                let temp_forecast_str = format!("{:?}", temp_forecast);
                send_message("HomeServer/Wetter/tempForecast", temp_forecast_str, tx);
            }
            

            return;
        }
    }
    



}


fn get_wetter(tx: &std::sync::mpsc::Sender<crate::Mymessage>)  {
    let w_key  = env!("w_key");
    let w_lat  = env!("w_lat");
    let w_lon  = env!("w_lon");
    let url = format!("https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&appid={}&units=metric&lang=DE",w_lat, w_lon, w_key);

    let answer : String = crate::http::get_request(&url.as_str(), 30);

    parse_wetter(tx, answer);
}

pub fn do_wetter(tx: std::sync::mpsc::Sender<crate::Mymessage>) {
    sleep(Duration::from_secs(100)); 
    loop {
        get_wetter(&tx);
        sleep(Duration::from_secs(1800));  // 30 min        
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn testwetter() {
        let file_path = "wetter.json";
        let answer = std::fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

        let (_tx, _rx) = crate::mpsc::channel();

        super::parse_wetter(&_tx, answer);

    }  
}