use serialport::{DataBits, StopBits, Parity};
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration};
use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

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

    enum Findtag {
        Start,
        End,
        Kauf,
        Verkauf,
        Leistung
    }


    fn send_message(the_topic: &str, the_payload:String, tx: &std::sync::mpsc::Sender<crate::Mymessage>) {
        let answ1 = crate::Mymessage {
            topic: String::from(the_topic),
            payload: the_payload,
        };
        if let Err(_) =  tx.send(answ1) {/* nothing */};    
    }


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
                            if counter == 7 {
                                gesamtwert -= 50.0;
                            }
                            if counter == 14 {
                                gesamtwert -= 20.0;
                            }
                            if counter < 4 && gesamtwert < 100.0 {
                                gesamtwert = 0.0;
                            }
                            if gesamtwert < 0.0 {
                                gesamtwert = 0.0;
                            }
                            #[cfg(debug_assertions)]
                            println!("S{}: {}", counter, gesamtwert);

                            let topic = format!("HomeServer/Einzel/{}/state", &rooms[(counter) as usize]);
                            send_message(&topic, gesamtwert.to_string(),  tx);

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

                send_message("HomeServer/Einzel/Gesamt", gesamt.to_string(),  tx);
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

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

fn findflag(welches: Findtag, buffer: &Vec<u8>) -> i32 {
    let vergleich: Vec<u8> = match welches {
        Findtag::Start =>    [0x1b,0x1b,0x1b,0x1b,0x01,0x01,0x01,0x01].to_vec(),
        Findtag::End =>      [0x1b,0x1b,0x1b,0x1b,0x1a].to_vec(),
        Findtag::Kauf =>     [0x77,0x07,0x01,0x00,0x01,0x08,0x00,0xff].to_vec(),
        Findtag::Verkauf =>  [0x77,0x07,0x01,0x00,0x02,0x08,0x00,0xff].to_vec(),
        Findtag::Leistung => [0x77,0x07,0x01,0x00,0x10,0x07,0x00,0xff].to_vec(),
    };

    let pos = find_subsequence(buffer, &vergleich);
    if pos.is_none() {
        return -1;
    }
    else {
        return pos.unwrap() as i32;
    }
}

fn hex_to_int(data: &[u8], bits: u8) -> i64 {
    if bits == 8 {
        let rgba: i8 = data[0] as i8;
        return rgba as i64;
    } else {
        if bits == 16 {
            let rgba: u16 = ((data[0] as u16) << 8) | (data[1] as u16)  ;
            let result: i16 = rgba as i16;
            return result as i64;
        } else {
            if bits == 32 {
                let rgba = ((data[0] as u32) << 24) | ((data[1] as u32) << 16) | ((data[2] as u32) << 8) | (data[3] as u32);
                let result: i32 = rgba as i32;
                return result as i64;
            } else {
                if bits == 64 {
                    let rgba: u64 =((data[0] as u64) << 56) |  ((data[1] as u64) << 48) | ((data[2] as u64) << 40) | ((data[3] as u64) << 32) | ((data[4] as u64) << 24) | ((data[5] as u64) << 16) | ((data[6] as u64) << 8) |(data[7] as u64);
                    let result: i64 = rgba as i64;
                    return result;
                } else {
                    println!("invalid bit lenght: {}", bits);
                    return 0;
                }
            }
        }
    }
}

fn get_smart_value(data: &Vec<u8>) -> f64 {

    let length = (data[0] & 0x0F) -1;
    let sign: u8 = data[0] >> 4;

    #[cfg(debug_assertions)]
    println!("length = {}, sign = {}, data = {:02X?}", length, sign, &data);
    if sign == 5 {
        // mit vorzeichen
        if length == 2 { 
            let result: i64 = hex_to_int(&data[1..=2], 16);
            return result as f64;
        }
        else {
            if length == 4 {
                let result: i64 = hex_to_int(&data[1..=4], 32);
                return result as f64;
            } else {
                if length == 8 {
                    let result: i64 = hex_to_int(&data[1..=8], 64);
                    return result as f64;
                } else {
                    println!("unbekannte länge = {}, sign = {}", length, sign);
                    return 0.0;
                }
            }
        }


    } else {
        // ohne vorzeichen
        
        if length == 2 { 
            let result: u64 = hex_to_int(&data[1..=2], 16) as u64;
            return result as f64;
        }
        else {
            if length == 4 {
                let result: u64 = hex_to_int(&data[1..=4], 32) as u64;
                return result as f64;
            } else {
                if length == 8 {
                    let result: u64 = hex_to_int(&data[1..=8], 64) as u64;
                    return result as f64;
                } else {
                    println!("unbekannte länge = {}, sign = {}", length, sign);
                    return 0.0;
                }
            }
        }


    }
    
}

fn parse_smartmeter(data: &Vec<u8>, tx: &std::sync::mpsc::Sender<crate::Mymessage>) -> (f64, f64, f64) {
    let kauf: f64;
    let pos = findflag(Findtag::Kauf, data);
    if pos < 0 {
        return (0.0, 0.0, 0.0);
    }    
    else {
        let start: usize = (pos+8+10).try_into().unwrap();
        let end: usize = start+8;
        let value: Vec<u8> = data[start..end].to_vec();
        kauf = get_smart_value(&value) / 10000.0;
    }  

    let verkauf: f64;
    let pos = findflag(Findtag::Verkauf, data);
    if pos < 0 {
        return (0.0, 0.0, 0.0);
    }    
    else {
        let start: usize = (pos+8+6).try_into().unwrap();
        let end: usize = start+8;
        let value: Vec<u8> = data[start..end].to_vec();
        verkauf = get_smart_value(&value) / 10000.0;
    }  
    
    let leistung: f64;
    let pos = findflag(Findtag::Leistung, data);
    if pos < 0 {
        return (0.0, 0.0, 0.0);
    }    
    else {
        let start: usize = (pos+8+6).try_into().unwrap();
        let end: usize = start+8;
        let value: Vec<u8> = data[start..end].to_vec();
        leistung = get_smart_value(&value);
    }                                

    send_message("HomeServer/Strom/Kauf", kauf.to_string(),  tx);
    send_message("HomeServer/Strom/Verkauf", verkauf.to_string(),  tx);
    send_message("HomeServer/Strom/Leistung", leistung.to_string(),  tx);

    return (kauf, verkauf, leistung);
}

fn get_smartmeter() -> Vec<u8> {
    let port_name ="/dev/ttyUSB0";
    let baud_rate = 9600;
    let mut ergebnis: Vec<u8> = vec![];

    let port = serialport::new(port_name, baud_rate)
        .data_bits(DataBits::Eight)
        .stop_bits(StopBits::One)
        .parity(Parity::None)
        .timeout(Duration::from_millis(1000))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            let mut startflag = false;
            let mut endflag = false;
            let mut anfang: usize = 0;
            let mut ende: usize = 0;
            
            while !endflag {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => {
                        //#[cfg(debug_assertions)]
                        //println!("smartmeter gelesen: {} - {:02X?}",  t, &serial_buf[..t]);

                        ergebnis.extend(&serial_buf[..t]);

                        // find start/end
                        if !startflag {
                            let pos = findflag(Findtag::Start, &ergebnis);
                            if pos >= 0 {
                                startflag = true;
                                anfang = pos as usize;
                            }
                        }
                        if startflag && !endflag {
                            let pos = findflag(Findtag::End, &ergebnis[anfang..].to_vec());
                            if pos >= 0 {
                                endflag = true;
                                ende = (pos+5) as usize;
                            }
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            let realend= anfang+ende;
            return ergebnis[anfang..realend].to_vec();

        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            return ergebnis;
        }
    }
}
   

pub fn do_strom(tx: std::sync::mpsc::Sender<crate::Mymessage>) { // (tx: std::sync::mpsc::Sender<crate::Mymessage>) {
    // alle 30 sec Einzelzähler
    // alle 15 sec Stromzähler

    //sleep(Duration::from_secs(15));

    let mut lastday: u32;
    let mut file: std::fs::File;

    // this is only to init it, to avoid rust complaining about none initialisied
    let local: DateTime<Local> = Local::now();

    // new day
    lastday = local.day();

    let the_date: String = format!("{}", local.format("%Y-%b"));
    let the_url_date: String = format!("{}", local.format("%Y%m%d"));
    let path: String;
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        path = "/home/pi/Strom/Day/".to_string()+&the_date;
    }
    else {
        path = "/Users/thomas/documents/rust/Strom/Day/".to_string()+&the_date;
    }  
    let path1 = format!("{}/Strom_{}.csv", path, the_url_date);
    
    match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&path1){
            Ok(t) => {
                file = t;
            }
            Err(e) => {
                eprintln!("{:?} {}", e, path1);
                return;
            }               
        }



    loop {
        let local: DateTime<Local> = Local::now();

        if lastday != local.day() {
            // new day
            lastday = local.day();

            let the_date: String = format!("{}", local.format("%Y-%b"));
            let the_url_date: String = format!("{}", local.format("%Y%m%d"));
            let path: String;
            let hostname = env!("HOSTNAME");
            if hostname != "Thomas_test" {
                path = "/home/pi/Strom/Day/".to_string()+&the_date;
            }
            else {
                path = "/Users/thomas/documents/rust/Strom/Day/".to_string()+&the_date;
            }  
            let path1 = format!("{}/Strom_{}.csv", path, the_url_date);
            
            match OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(&path1){
                    Ok(t) => {
                        file = t;
                    }
                    Err(e) => {
                        eprintln!("{:?} {}", e, path1);
                        return;
                    }               
                }
        }


        let second = local.second();
        if (second == 0) || (second == 30) {
            let einzel_daten = get_einzel();
            parse_einzel(einzel_daten, &tx);

            let smart_daten = get_smartmeter();
            //fs::write("smartmeter.txt", &smart_daten).unwrap();
            //println!("smartmeter geschrieben");
            let (kauf, verkauf, leistung) = parse_smartmeter(&smart_daten, &tx);
            #[cfg(debug_assertions)]
            println!("Kauf: {}, Verkauf: {}, Leistung: {}", kauf, verkauf, leistung);
            sleep(Duration::from_millis(1000));
        }

        if (second == 15) || (second == 45) {
            let smart_daten = get_smartmeter();
            //fs::write("smartmeter.txt", &smart_daten).unwrap();
            //println!("smartmeter geschrieben");
            let (kauf, verkauf, leistung) = parse_smartmeter(&smart_daten, &tx);
            #[cfg(debug_assertions)]
            println!("Kauf: {}, Verkauf: {}, Leistung: {}", kauf, verkauf, leistung);

            let dt_string = local.format("%d/%m/%Y %H:%M:%S");
            if (kauf !=0.0) && (verkauf != 0.0) {
                if let Err(e) = writeln!(file, "{}\t\t{}\t{}\t{}", dt_string, kauf, verkauf, leistung) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
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
        
        let (tx, rx) = crate::mpsc::channel();

        super::parse_einzel(result, &tx);

    }  

    #[test]
    fn testsmart() {
        let file_path = "smartmeter.txt";
        let result = std::fs::read(file_path)
        .expect("Should have been able to read the file");

        let (tx, rx) = crate::mpsc::channel();

        let (kauf, verkauf, leistung) = super::parse_smartmeter(&result, &tx);
        assert_eq!(kauf, 1811.0676);
        assert_eq!(verkauf, 2543.317);
        assert_eq!(leistung, -2172.0);
    }      
}
