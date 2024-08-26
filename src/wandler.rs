
use std::thread::sleep;
use std::time::{Duration};
use chrono::prelude::*;
use std::path::Path;
use std::fs;
use chrono::TimeDelta;
//use chrono::DateTime;

fn calc_pv(wert: &str) -> i32 {
    let mut whfloat: f64 = wert.parse().unwrap();
    whfloat /= 0.65535;
    let wh:i32 = whfloat as i32;
    return wh;
}

fn send_message(the_topic: &str, the_payload:String, tx: &std::sync::mpsc::Sender<crate::Mymessage>) {
    let answ1 = crate::Mymessage {
        topic: String::from(the_topic),
        payload: the_payload,
    };
    if let Err(_) =  tx.send(answ1) {/* nothing */};    
}

fn get_wandler_hour_sub(url: String, path: String) ->f64 {
    let answer : String = crate::http::get_request(&url, 25);
    #[cfg(debug_assertions)]
    println!("{} {}", url, answer);

    if answer != "" {
        fs::write(path, &answer).expect("Unable to write file");
    }
    else {
        #[cfg(debug_assertions)]
        println!("error url request {}", url);
    }

    let v: Vec<&str> = answer.split('\r').collect();
    if v.len() > 1 {
        let cells: Vec<&str> = v[1].split(';').collect(); 
        if cells.len()>4 {
            let total: f64 = cells[4].parse().unwrap();
            return total;
        }
    }

    return -1.0;
}

fn get_wandler_month_sub(url: String, path: String) {
    let answer : String = crate::http::get_request(&url, 25);
    #[cfg(debug_assertions)]
    println!("{} {}", url, answer);

    if answer != "" {
        fs::write(path, &answer).expect("Unable to write file");
    }
    else {
        #[cfg(debug_assertions)]
        println!("error url request {}", url);
    }
}

fn get_wandler_hour(tx: &std::sync::mpsc::Sender<crate::Mymessage>, local: &DateTime<Local> )  {
    //let the_date: String = local.year().to_string()+"-"+local.month()
    let the_date: String = format!("{}", local.format("%Y-%b"));
    let the_url_date: String = format!("{}", local.format("%Y%m%d"));

    #[cfg(debug_assertions)]
    println!("date: {} {}", the_date, the_url_date);
    let path: String;
    let url: String;
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        path = "/home/pi/Strom/Day/".to_string()+&the_date;
        url = "http://192.168.189.".to_string();
    }
    else {
        path = "/Users/thomas/documents/rust/Strom/Day/".to_string()+&the_date;
        url = "http://192.168.0.63/proxy.php/192.168.189.".to_string();
    }

    if !Path::new(&path).exists() {
        fs::create_dir_all(&path).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });  
    }
    let url1 = format!("{}11/{}.CSV", url, the_url_date);
    let path1 = format!("{}/Haus_{}.csv", path, the_url_date);
    let haus = get_wandler_hour_sub(url1, path1);

    let url2 = format!("{}8/{}.CSV", url, the_url_date);
    let path2 = format!("{}/Garage_{}.csv", path, the_url_date);
    let garage = get_wandler_hour_sub(url2, path2);

    #[cfg(debug_assertions)]
    println!("Wandler: {} {} {}", haus, garage, haus+garage);

    send_message("HomeServer/Strom/HausDaily", haus.to_string(), tx);
    send_message("HomeServer/Strom/GarageDaily", garage.to_string(), tx);
    let total:f64 = ((garage+haus) * 100.0).round() / 100.0;
    send_message("HomeServer/Strom/ProduktionDaily", total.to_string(), tx);    
}

fn get_wandler_day(inlocal: &DateTime<Local> )  {
    let local: DateTime<Local>  = *inlocal - TimeDelta::try_minutes(60).unwrap();
    let the_date: String = format!("{}", local.format("%Y-%b"));
    let the_url_date: String = format!("{}", local.format("%Y%m"));

    #[cfg(debug_assertions)]
    println!("date: {} {}", the_date, the_url_date);
    let path: String;
    let url: String;
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        path = "/home/pi/Strom/Month/".to_string()+&the_date;
        url = "http://192.168.189.".to_string();
    }
    else {
        path = "/Users/thomas/documents/rust/Strom/Month/".to_string()+&the_date;
        url = "http://192.168.0.63/proxy.php/192.168.189.".to_string();
    }

    if !Path::new(&path).exists() {
        fs::create_dir_all(&path).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });  
    }

    let url1 = format!("{}11/{}.CSV", url, the_url_date);
    let path1 = format!("{}/Haus_{}.csv", path, the_url_date);
    get_wandler_month_sub(url1, path1);

    let url2 = format!("{}8/{}.CSV", url, the_url_date);
    let path2 = format!("{}/Garage_{}.csv", path, the_url_date);
    get_wandler_month_sub(url2, path2);

  }

fn get_wandler_month(inlocal: &DateTime<Local> )  {
    let local: DateTime<Local>  = *inlocal - TimeDelta::try_minutes(60).unwrap();
    let the_date: String = format!("{}", local.format("%Y"));
    let the_url_date: String = format!("{}", local.format("%Y"));

    #[cfg(debug_assertions)]
    println!("date: {} {}", the_date, the_url_date);
    let path: String;
    let url: String;
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        path = "/home/pi/Strom/Year/".to_string();
        url = "http://192.168.189.".to_string();
    }
    else {
        path = "/Users/thomas/documents/rust/Strom/Year/".to_string()+&the_date;
        url = "http://192.168.0.63/proxy.php/192.168.189.".to_string();
    }

    #[cfg(debug_assertions)]
    println!("day: {} {}", path, url);

    if !Path::new(&path).exists() {
        fs::create_dir_all(&path).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });     
    }

    let url1 = format!("{}11/{}.CSV", url, the_url_date);
    let path1 = format!("{}/Haus_{}.csv", path, the_url_date);
    get_wandler_month_sub(url1, path1);

    let url2 = format!("{}8/{}.CSV", url, the_url_date);
    let path2 = format!("{}/Garage_{}.csv", path, the_url_date);
    get_wandler_month_sub(url2, path2);
  }

fn get_wandler(tx: &std::sync::mpsc::Sender<crate::Mymessage>) -> i32 {
    let host1:  &str;  
    let host2:  &str;     
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
            #[cfg(debug_assertions)]
            println!("hostname '{}' {}",hostname, hostname.len());
            host1 =  "http://192.168.189.11/realtime.csv";
            host2 =  "http://192.168.189.8/realtime.csv";
    }
    else {
            host1 = "http://192.168.0.63/proxy.php/192.168.189.11/realtime.csv";
            host2 = "http://192.168.0.63/proxy.php/192.168.189.8/realtime.csv";
    }

    let answer : String = crate::http::get_request(host1, 25);
    #[cfg(debug_assertions)]
    println!("wandler11 {}", answer);
    let answer2 : String = crate::http::get_request(host2, 25);
    #[cfg(debug_assertions)]
    println!("wandler8 {}", answer2);

    // 1724604051;13804;9290;160;313;97;3459;4
    let v: Vec<&str> = answer.split(';').collect();
    // 1724604450;10902;10863;9281;178;164;416;161;3188;4
    let v2: Vec<&str> = answer2.split(';').collect();

    if v.len() < 8 {
        #[cfg(debug_assertions)]
        println!("wandler11 zu kurz {}", v.len()); 
        return 1;       
    }

    if v2.len() < 10 {
        #[cfg(debug_assertions)]
        println!("wandler8 zu kurz {}", v.len());  
        return 1;      
    }

    let wh = calc_pv(v[5]);
    let  wg: i32 = calc_pv(v2[7]);
    let wg_nraw: f64 = v2[4].parse().unwrap();
    let wg_sraw: f64 = v2[5].parse().unwrap();
    let mut wg_snraw: f64 = wg_nraw + wg_sraw;
    if wg_snraw == 0.0 {
        wg_snraw = 1.0;
    }
    let wg_nproz = wg_nraw/wg_snraw;
    let wg_sproz = wg_sraw/wg_snraw;
    let wg_n:i32 = (wg as f64 * wg_nproz) as i32;
    let wg_s:i32 = (wg as f64 * wg_sproz) as i32;

    if (wg > 0) && (wg < 10) {
        #[cfg(debug_assertions)]
        println!("wg < 10 {}", wg); 
        return 1;
    }

    let total:i32 = wg+wh;

    send_message("HomeServer/Strom/Dach", wh.to_string(), tx);
    send_message("HomeServer/Strom/Garage", wg.to_string(), tx);
    send_message("HomeServer/Strom/GarageN", wg_n.to_string(), tx);
    send_message("HomeServer/Strom/GarageS", wg_s.to_string(), tx);
    send_message("HomeServer/Strom/Produktion", total.to_string(), tx);
     
    return total;
}


pub fn do_wandler(tx: std::sync::mpsc::Sender<crate::Mymessage>) {
    // alle 30 sec hole aktuelle Wandlerwerte, 120 wenn ergebnis 0 (=nacht)
    // alle 60 min hole Wandlerhour (ändert sich alle 5 min)
    // alle 24 hour, 0:05 hole Wandlermonth von -60 min damit vormonat, wichtig am Monatsersten
    // alle 1.  monat hole Wandleryear vom vortag

    //let local: DateTime<Local> = Local::now();
    //get_wandler_day(&local);  // nur für Test am Start

    let mut last_hour: u32 = 0;
    let mut last_day: u32 = 0;
    let mut last_month: u32 = 0;

    sleep(Duration::from_secs(15));
    loop {
        sleep(Duration::from_secs(30));
        
        if get_wandler(&tx) == 0 {
            sleep(Duration::from_secs(90));
        }

        let local: DateTime<Local> = Local::now();
        if local.hour() != last_hour {
            last_hour = local.hour();
            get_wandler_hour(&tx, &local);
        }
        if local.day() != last_day {
            last_day = local.day();
            get_wandler_day(&local);
        }        
        if local.month() != last_month {
            last_month = local.month();
            get_wandler_month(&local);
        }  
    }

}



#[cfg(test)]
mod tests {


    #[test]
    fn testsplit() {
        let answer = "1724604051;13804;9290;160;313;97;3459;4".to_string();
        let v: Vec<&str> = answer.split(';').collect();
        assert_eq!(v.len() >= 8, true);

        let dummy = v[5];
        let mut whfloat: f64 = dummy.parse().unwrap();
        whfloat /= 0.65535;
        let wh:i32 = whfloat as i32;

        println!("wh: {} whfloat: {}", wh, whfloat);
        assert_eq!(whfloat, 148.0);

    }  

    #[test]
    fn testtime() {


    }
}
