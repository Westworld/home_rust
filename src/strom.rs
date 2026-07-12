use serialport::{DataBits, Parity};
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration};
use chrono::prelude::*;
//use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::fs;

// SML Protocol http://www.stefan-weigert.de/php_loader/sml.php
// https://wiki.volkszaehler.org/hardware/channels/meters/power/edl-ehz/emh-ehz-h1


/*
Found 3 ports:
  /dev/ttyUSB0
    Type: USB
    VID:0403 PID:6001
     Serial Number: A105FRRC
      Manufacturer: FTDI
           Product: FT232R USB UART
  /dev/ttyACM0
    Type: USB
    VID:16c0 PID:0483
     Serial Number: 1187120
      Manufacturer: Teensyduino
           Product: USB Serial
  /dev/ttyAMA10
    Type: Unknown

    */



fn parse_einzel(block: String, tx: &std::sync::mpsc::Sender<crate::Mymessage>) {
    // zerlege block in Zeilen
    let rooms = vec!["Start", "Herd1", "Herd2", "Herd3", "Spuelmaschine", "Trockner", "Waschmaschine", "Kueche", "Wohnzimmer", "Schlafzimmer","leer", "Billard", "Bad", "Keller", "Heizung", "Arbeitszimmer", "Flur_DG" ];

    let v: Vec<&str> = block.split("\r\n").collect();
    if v.len() > 1 {
        let start: Option<usize> = v.iter().position(|&r| r == "start");
        let end: Option<usize> = v.iter().position(|&r| r == "end");
        if start.is_some() && end.is_some() {
            #[cfg(debug_assertions)]
            println!("start: {:?}, end: {:?}", start, end);
            let start= start.unwrap();
            let end = end.unwrap();
            if (end-start) == 17 {
                let mut gesamt: f64 = 0.0;
                let mut counter: u8 = 1;
                for number in (start+1)..end {
                    #[cfg(debug_assertions)]
                    println!("line: {}", v[number]);
                    let v2: Vec<&str> = v[number].split(":").collect();
                    if v2.len() != 2 {
                        #[cfg(debug_assertions)]
                        println!("falsche größe von v2: {:?}", v2.len());
                        break;
                    }
                    let control: String = format!("S{}", counter);
                    if control == v2[0] {
                        let v3: Vec<&str> = v2[1].split(",").collect();
                        if v3.len() != 6 {
                            println!("falsche größe von v3: {:?}", v3.len());
                            break;                            
                        } else {
                            let mut gesamtwert: f64 = 0.0;
                            for wert in v3 {
                                let total: f64 = wert.parse().unwrap();
                                gesamtwert += total;
                            }
                            gesamtwert /= 6.0;
                            gesamtwert = gesamtwert * 25.0 / 1000.0;
                            gesamtwert = (gesamtwert * 230.0).round();
                           // #Arbeitszimmer liefert manchmal zu hohe Werte, daher auf 40 begrenzen
                            if (counter == 15) && (gesamtwert > 500.0) {
                                gesamtwert = 40.0;
                            }
                            // Wohnzimmer, Keller, Billard, Arbeitszimmer, Heizung, minus 20
                            if (counter == 8) || (counter == 11) || (counter == 13) || (counter == 14) || (counter == 15)  {
                                gesamtwert -= 20.0;
                            }                           
                            if counter == 7 {
                                gesamtwert -= 50.0;
                            }
                            if counter < 4 && gesamtwert < 100.0 {
                                gesamtwert = 0.0;
                            }
                            if gesamtwert < 10.0 {
                                gesamtwert = 0.0;
                            }
                            #[cfg(debug_assertions)]
                            println!("S{}: {}", counter, gesamtwert);

                            let topic = format!("HomeServer/Einzel/{}/state", &rooms[(counter) as usize]);
                            crate::send_message(&topic, gesamtwert.to_string(),  tx);

                            gesamt += gesamtwert;
                        }
                    }    
                    else {
                        println!("control fehler: {:?} {}", control, v2[0]);
                        break;
                    }
                    counter = counter+1;
                }
                #[cfg(debug_assertions)]
                println!("gesamt {}", gesamt);

                crate::send_message("HomeServer/Einzel/Gesamt", gesamt.to_string(),  tx);
            }
            else {
                println!("falsche Anzahl Zeilen zwischen Start und End");
            }
        }
        else {
            #[cfg(debug_assertions)]
            println!("fehler, es fehlt Start oder End");
        }
    }
    else {
        #[cfg(debug_assertions)]
        println!("zerlegen, nur eine zeile");
    }

    // erste Zeile muss start sein, letzte end
    // dazwischen S1 bis S16
    // pro Zeile 6 Messwerte, Mittelwert bilden
}

fn get_einzel() -> String {
    let port_name ="/dev/ttyACM0";
    let baud_rate = 115200;

    let port = serialport::new(port_name, baud_rate)
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .timeout(Duration::from_millis(1000))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            //println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);

            match port.write("\n".as_bytes()) {
                Ok(_) => (),
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => eprintln!("{:?}", e),
            }
            std::thread::sleep(Duration::from_millis(1000));

            match port.read(serial_buf.as_mut_slice()) {
                    Ok(_) => {
                        let s = match String::from_utf8(serial_buf) {
                            Ok(v) => v,
                            Err(_) => return "".to_string(),
                        };
                        

                        return s;
                    }
                    Err(_) => return "".to_string(),
                }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            return "".to_string();
        }
    }
}



pub fn do_strom(tx: std::sync::mpsc::Sender<crate::Mymessage>) { // (tx: std::sync::mpsc::Sender<crate::Mymessage>) {
    // alle 30 sec Einzelzähler
    // alle 15 sec Stromzähler

    //sleep(Duration::from_secs(15));

    // this is only to init it, to avoid rust complaining about none initialisied
    let local: DateTime<Local> = Local::now();

    let the_date: String = format!("{}", local.format("%Y-%b"));
   // let the_url_date: String = format!("{}", local.format("%Y%m%d"));
    let path: String;
    let hostname = env!("APP_HOSTNAME");
    if hostname != "Thomas_test" {
        path = "/home/pi/Strom/Day/".to_string()+&the_date;
    }
    else {
        path = "/Users/thomas/documents/rust/Strom/Day/".to_string()+&the_date;
    }  
    //let path1 = format!("{}/Strom_{}.csv", path, the_url_date);

    if !Path::new(&path).exists() {
        fs::create_dir_all(&path).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });  
    }


    loop {
        let local: DateTime<Local> = Local::now();


        let second = local.second();
        if (second == 0) || (second == 30) {
            let einzel_daten = get_einzel();
            parse_einzel(einzel_daten, &tx);

            sleep(Duration::from_millis(1000));
        }


        sleep(Duration::from_millis(200));
    }

}


#[cfg(test)]
mod tests {

    #[test]
    fn testeinzel() {
        let file_path = "einzel.txt";
        let result = std::fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
        
        let (_tx, _rx) = crate::mpsc::channel();

        super::parse_einzel(result, &_tx);

    }  

    #[test]
    fn testsmart() {
        let file_path = "smartmeter.txt";
        let result = std::fs::read(file_path)
        .expect("Should have been able to read the file");

        let (_tx, _rx) = crate::mpsc::channel();

        let (kauf, verkauf, leistung) = super::parse_smartmeter(&result, &_tx);
        assert_eq!(kauf, 1811.0676);
        assert_eq!(verkauf, 2543.317);
        assert_eq!(leistung, -2172.0);
    }      
}
