use std::thread::sleep;
use std::time::{Duration};

fn get_wallbox(tx: &std::sync::mpsc::Sender<crate::Mymessage>)  {

    // http://192.168.0.63/proxy.php/192.168.189.12:8080/api/v1/status
    let host1:  &str;
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        #[cfg(debug_assertions)]
        println!("hostname '{}' {}",hostname, hostname.len());
        host1 =  "http://192.168.189.3/api/status?filter=car,amp,alw,eto,nrg,psm,frc";
    }
    else {
        host1 = "http://192.168.0.63/proxy.php/192.168.189.3/api/status?filter=car,amp,alw,eto,nrg,psm,frc";
    }

    let answer : String = crate::http::get_request(host1);
    #[cfg(debug_assertions)]
    println!("wallbox {}", answer);

    let check = vec!["car", "amp", "alw", "eto", "psm", "frc", "nrg"];
    
    if answer.starts_with("{\"") {
        let parsed = json::parse(answer.as_str()).unwrap();
        for topiccheck in check {
            let  thepayload: String ;
            if topiccheck == "nrg" {
                let mut total: i32 = parsed["nrg"][11].as_i32().unwrap();
                total += parsed["nrg"][12].as_i32().unwrap();
                total += parsed["nrg"][13].as_i32().unwrap();
                thepayload = total.to_string();
            }
            else {
                thepayload = parsed[topiccheck].to_string();
            }
            let answ1 = crate::Mymessage {
                topic: String::from("HomeServer/Wallbox/".to_owned()+topiccheck),
                payload: thepayload,
            };
            if let Err(_) =  tx.send(answ1) {/* nothing */};
        }

    }
}


pub fn do_wallbox(tx: std::sync::mpsc::Sender<crate::Mymessage>) {
    sleep(Duration::from_secs(5));
    loop {
        sleep(Duration::from_secs(60));
        
        get_wallbox(&tx);
    }

}