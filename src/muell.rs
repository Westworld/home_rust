extern crate ical;

use std::io::BufReader;
use std::fs::File;
use std::time::{Duration};
use chrono::prelude::*;
use std::thread::sleep;


fn run_muell(tx: &std::sync::mpsc::Sender<crate::Mymessage>) {
    let path: &str;
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        path = "/home/pi/abfuhrtermine.ics";
    }
    else {
        path = "abfuhrtermine.ics";
    }

    let buf: BufReader<File>;
    let f = File::open(path);
    match f {
        Ok (file) => buf = BufReader::new(file),
        Err(e) => { eprintln!("{:?}", e); return;}   
    }

    let local  = Local::now();
    let dt_heute: String = local.format("%Y%m%d").to_string();
    let tommorow = local + Duration::from_secs(60 * 60 * 24);
    let dt_morgen: String = tommorow.format("%Y%m%d").to_string();
    // println!("heute: {}, morgen: {}", dt_heute, dt_morgen);

    let reader = ical::IcalParser::new(buf);

    for line in reader {
        //println!("prop: {:?}", line);
         match line {
            Ok(t) => {
                let events = t.events;

                for event in events {
                    let props = event.properties;

                    let mut date: String = "".to_string();
                    let mut summary: String= "".to_string();

                    for prop in props {
                        if prop.name == "SUMMARY" {
                            summary = prop.value.unwrap();
                        } else {
                            if prop.name == "DTSTART" {
                                date = prop.value.unwrap();
                                //date = date;
                                date = date[..8].to_string();
                                #[cfg(debug_assertions)]
                                println!("date: {:?}", date);
                            }   
                        }                     
                    }

                    if date.is_empty() && summary.is_empty() {
                        crate::send_message("display/muell", "-".to_string(), tx);
                    }
                    else 
                    {
                        // wenn date = heute und heute vor 12 Uhr oder date = morgen
                        if (date == dt_heute) || (date == dt_morgen) {
                            #[cfg(debug_assertions)]
                            println!("treffer: {:?}", date);
                            if date == dt_heute && local.hour() >= 10 {
                                crate::send_message("display/muell", "-".to_string(), tx);
                            } else 
                            {
                                crate::send_message("display/muell", summary, tx);
                            }     
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}


pub fn do_muell(tx: std::sync::mpsc::Sender<crate::Mymessage>) {
    sleep(Duration::from_secs(15));
    loop {

        let local: DateTime<Local> = Local::now();
        if local.hour() == 1 {
            run_muell(&tx);
            sleep(Duration::from_secs(60*60)); // sleep 1 hour
        }
        if local.hour() == 10 {
            run_muell(&tx);
            sleep(Duration::from_secs(60*60)); // sleep 1 hour
        } 
        sleep(Duration::from_secs(60*10)); // sleep 10 minute
    }

}


#[cfg(test)]
mod tests {

    #[test]
    fn testmuell() {

        let (_tx, _rx) = crate::mpsc::channel();
        super::run_muell(&_tx );

    }  
}