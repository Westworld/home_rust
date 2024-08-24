use quick_xml::events::Event;
use quick_xml::reader::Reader;
//use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration};

pub struct Mymessage {
    pub topic: String,
    pub payload: String,
}

fn do_parse(xml: &str) -> (i32, i32) {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut bytesin:i32 = -1;
    let mut bytesout:i32 = -1;
    let mut tagname : String = "".to_string();

    // The `Reader` does not implement `Iterator` because it outputs borrowed data (`Cow`s)
    loop {
        // NOTE: this is the generic case when we don't know about the input BufRead.
        // when the input is a &str or a &[u8], we don't actually need to use another
        // buffer, we could directly call `reader.read_event()`
        match reader.read_event_into(&mut buf) {
        Err(_) => bytesin = -1,
        // exits the loop when reaching end of file
        Ok(Event::Eof) => break,

        Ok(Event::Start(e)) => {
            match e.name().as_ref() {
                b"NewByteSendRate" => tagname = "send".to_string(),
                b"NewByteReceiveRate" => tagname = "received".to_string(),
                _ => tagname = "".to_string(),
            }
        },

        Ok(Event::Text(e)) => {
        // let val:i32 = e.unescape().unwrap().parse::<i32>().unwrap();
        match e.unescape().unwrap().parse::<i32>() {
            Ok(val) => {
                match tagname.as_str() {
                    "received" => bytesin = val * 8 / 1024,
                    "send" => bytesout = val * 8 / 1024,
                    _ => (),  // nothing
                }
            },
            Err(_) => break,
        }

        }
        // There are several other `Event`s we do not consider here
        _ => (),
    }
    buf.clear();
    }

    (bytesin, bytesout)
}


pub fn do_fritz(tx: std::sync::mpsc::Sender<Mymessage>) {
    loop {
        let body:&'static str = "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"
        s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
        <s:Body><u:GetAddonInfos xmlns:u=\"urn:schemas-upnp-org:service:WANCommonInterfaceConfig:1\">
        </u:GetAddonInfos></s:Body></s:Envelope>";
        let url:&'static str = "http://fritz.box:49000/igdupnp/control/WANCommonIFC1";
        let header1: crate::http::MyHeaders = 
            crate::http::MyHeaders{
                key: "Content-Type".to_string(), 
                value: "text/xml".to_string()};
        let header2: crate::http::MyHeaders = 
                crate::http::MyHeaders{
                    key: "SOAPACTION".to_string(), 
                    value: "urn:schemas-upnp-org:service:WANCommonInterfaceConfig:1#GetAddonInfos".to_string()};

        let result:  String = crate::http::post_request(body, url, header1, header2);
        //println!("Body main:\n{}", result);

        let bytesin:i32;
        let bytesout:i32;
        (bytesin, bytesout) = do_parse(&result);

        let answ1 = Mymessage {
            topic: String::from("HomeServer/Internet/DownloadRate"),
            payload: bytesin.to_string(),
        };
        let answ2 = Mymessage {
            topic: String::from("HomeServer/Internet/UploadRate"),
            payload: bytesout.to_string(),
        };
        if let Err(_) =  tx.send(answ1) {/* nothing */};
        if let Err(_) =  tx.send(answ2) {/* nothing */};

        #[cfg(debug_assertions)]
        println!("Fritz: {} {}", bytesin, bytesout);

        sleep(Duration::from_secs(15));
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn example() {
        let result = 2+2;
        assert_eq!(result, 4);
    }

    #[test]
    fn testxml() {
        let file_path = "xmltest.txt";
        let result = std::fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

        println!("xml: {}", result);

        let bytesin:i32;
        let bytesout:i32;
        (bytesin, bytesout) = super::do_parse(&result);

        println!("parse: {} {}", bytesin, bytesout);

        assert_eq!(bytesin, 8757);
        assert_eq!(bytesout, 1007);
    }  
}
