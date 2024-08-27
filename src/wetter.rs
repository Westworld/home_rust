use std::thread::sleep;
use std::time::{Duration};

use serde::{Deserialize, Serialize};
use serde_json::{Result, Value, json};

#[derive(Serialize, Deserialize)]
struct Wetter {
 current: WCurrent,
}

#[derive(Serialize, Deserialize)]
struct WCurrent {
    temp: f64,
    wind_speed: f64,
    weather: Vec<WMain>,
} 

#[derive(Serialize, Deserialize)]
struct WMain {
    main: String,
    description: String,
    icon: String,
}

#[derive(Serialize, Deserialize)]
struct Wrain {
    "1h": f64,
 }

#[derive(Serialize, Deserialize)]
struct WCurrentNew {
    temp: i32,
    wind_speed: i32,
    rain: u32,
    main: String,
    description: String,
    icon: String,
} 

fn get_wetter_sub(current: WCurrent) -> WCurrentNew  {
    let newcurrent = WCurrentNew {
        wind_speed: (current.wind_speed * 3.6).round() as i32,
        temp: current.temp.round() as i32,
        rain: 0,
        main: current.weather[0].main.clone(),
        description: current.weather[0].description.clone(),
        icon: current.weather[0].icon.clone(),
    };
    return newcurrent;


    /*

	if (data.get("rain") is None):
		now["rain"] = 0
	else:
		rain = data.get("rain")
		now["rain"] = rain.get("1h")
	dt = data.get("dt",0)
	now["day"] = datetime.fromtimestamp(dt).strftime('%a')
	now["hour"] = datetime.fromtimestamp(dt).strftime('%H:%M')
	return now

    */
}

fn parse_wetter(tx: &std::sync::mpsc::Sender<crate::Mymessage>, answer: String) {
    let w_main = WMain {
        main: "".to_string(),
        description: "".to_string(),
        icon: "".to_string(),
    };
    let vw_main: Vec<WMain> = vec![w_main];
    let w_current = WCurrent {
        temp: -75.0,
        wind_speed: 0.0,
        weather: vw_main,
    };
    let wetter = Wetter {
        current: w_current,
    };

    if answer.starts_with("{") {
        let parsed: Wetter = match serde_json::from_str (&answer) {
            Ok (p) => p,
            Err (_) => wetter,
        };

    let the_current = parsed.current;
    let new_current = get_wetter_sub(the_current);

        let j = serde_json::to_string(&new_current).unwrap();
        println!("current: {}", j);


        /* 
        for topiccheck in check {
            let answ1 = crate::Mymessage {
                topic: String::from("HomeServer/Batterie/".to_owned()+topiccheck),
                payload: parsed[topiccheck].to_string(),
            };
            if let Err(_) =  tx.send(answ1) {/* nothing */};
        }
        */

    }

}


fn get_wetter(tx: &std::sync::mpsc::Sender<crate::Mymessage>)  {
    let w_key  = env!("w_key");
    let w_lat  = env!("w_lat");
    let w_lon  = env!("w_lon");
    let url = format!("https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&appid={}&units=metric&lang=DE",w_lat, w_lon, w_key);

    let answer : String = crate::http::get_request(&url.as_str(), 30);

    #[cfg(debug_assertions)]
    println!("wetter {}", answer);

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

        let (tx, rx) = crate::mpsc::channel();

        super::parse_wetter(&tx, answer);

        /* 
        let bytesin:i32;
        let bytesout:i32;
        (bytesin, bytesout) = super::do_parse(&result);

        println!("parse: {} {}", bytesin, bytesout);

        assert_eq!(bytesin, 8757);
        assert_eq!(bytesout, 1007);
        */
    }  
}