use std::thread::sleep;
use std::time::{Duration};


fn find_sonnen(tx: &std::sync::mpsc::Sender<crate::Mymessage>) -> i32 {
// test 4 and 12 first
// then 1-255
// return 0 if not found

    if get_sonnen(tx,4) {
        return 4;
    }
    if get_sonnen(tx,12) {
        return 12;
    }
    
    for number in 1..255 {
        if get_sonnen(tx,number) {
            return number;
        }        
    }
    return 0;
}


fn get_sonnen(tx: &std::sync::mpsc::Sender<crate::Mymessage>, subid:i32) -> bool {

    // http://192.168.0.63/proxy.php/192.168.189.12:8080/api/v1/status
    let host1:  &str;
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        #[cfg(debug_assertions)]
        println!("hostname '{}' {}",hostname, hostname.len());
        host1 =  "http://192.168.189.";
    }
    else {
        host1 = "http://192.168.0.63/proxy.php/192.168.189.";
    }
    let host2 =  ":8080/api/v1/status" ;

    let thehost = host1.to_owned()+&subid.to_string()+host2;

    let answer : String = crate::http::get_request(&thehost.as_str(), 2);
    #[cfg(debug_assertions)]
    println!("sonnen ip: {} - {}",thehost, answer);

    let check = vec!["Production_W", "Consumption_W", "Pac_total_W", "Uac", "USOC"];
    
    if answer.starts_with("{\"") {
        let parsed = json::parse(answer.as_str()).unwrap();
        for topiccheck in check {
            if parsed[topiccheck].is_null() {
                return false;
            }
            let answ1 = crate::Mymessage {
                topic: String::from("HomeServer/Batterie/".to_owned()+topiccheck),
                payload: parsed[topiccheck].to_string(),
            };
            if let Err(_) =  tx.send(answ1) {/* nothing */};
        }

        return true;
    }
    else {
        return false;
    }
}


pub fn do_sonnen(tx: std::sync::mpsc::Sender<crate::Mymessage>) {
    let mut subid:i32 = find_sonnen(&tx);

    loop {
        sleep(Duration::from_secs(60));
        
        if get_sonnen(&tx, subid) {

        }
        else {
            subid = find_sonnen(&tx);
        }
    }

}