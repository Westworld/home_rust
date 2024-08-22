use quick_xml::events::Event;
use quick_xml::reader::Reader;
//use std::io::Empty;
use std::thread::sleep;
use std::thread;
use rumqttc::{Client, MqttOptions, QoS};
use std::time::{Duration};
use std::sync::mpsc;


// compile: cargo build
// test: cargo run
// final: cargo build --release     // binary in target/release

struct Mymessage {
    topic: String,
    payload: String,
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


/*/
async fn get_request() -> String {
    let response = match reqwest::get("https://www.fruityvice.com/api/fruit/appple").await {
        Ok(answer) => answer,
        Err(_) =>  return String::new(),
    };
 
    let status = response.status().as_u16();
    if status != 200 {
        return String::new()
    }
    else {
        let body: String = match response.text().await {
            Ok(answer) => answer,
            Err(_) => String::new(),
        };
        return body
    }
}
*/

fn post_request() -> String {
    let body:&'static str = "<?xml version=\"1.0\"?><s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\"
        s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">
        <s:Body><u:GetAddonInfos xmlns:u=\"urn:schemas-upnp-org:service:WANCommonInterfaceConfig:1\">
        </u:GetAddonInfos></s:Body></s:Envelope>";
    let client = reqwest::blocking::Client::new();
    let response = match client.post("http://fritz.box:49000/igdupnp/control/WANCommonIFC1")
        .header("Content-Type", "text/xml")
        .header("SOAPACTION", "urn:schemas-upnp-org:service:WANCommonInterfaceConfig:1#GetAddonInfos")
        .body(body)
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

/*
fn do_fritz(client: Client)  {
    let result:  String = post_request();
    //println!("Body main:\n{}", result);

    let bytesin:i32;
    let bytesout:i32;
    (bytesin, bytesout) = do_parse(&result);
    println!("in/out    {}  -  {}", bytesin, bytesout);
    client.try_publish("hello/bytesin".to_string(), QoS::AtLeastOnce, true, bytesin.to_string().as_bytes()).unwrap();
    client.try_publish("hello/bytesout".to_string(), QoS::AtLeastOnce, true, bytesout.to_string().as_bytes()).unwrap();
}
    */

fn do_fritz(tx: std::sync::mpsc::Sender<Mymessage>) {
    loop {
        let result:  String = post_request();
        //println!("Body main:\n{}", result);

        let bytesin:i32;
        let bytesout:i32;
        (bytesin, bytesout) = do_parse(&result);

        let answ1 = Mymessage {
            topic: String::from("hello/bytesin"),
            payload: bytesin.to_string(),
        };
        let answ2 = Mymessage {
            topic: String::from("hello/bytesout"),
            payload: bytesout.to_string(),
        };
        if let Err(_) =  tx.send(answ1) {/* nothing */};
        if let Err(_) =  tx.send(answ2) {/* nothing */};

        sleep(Duration::from_secs(20));
    }
}

fn main()  {
    let mut mqttoptions = MqttOptions::new("test-1", "192.168.0.46", 1883);
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_credentials("Enzel", "hausen");

    let (client, mut connection) = Client::new(mqttoptions, 10);
    let (tx, rx) = mpsc::channel();
    
        let tx2 = tx.clone();
        let _handle = thread::spawn( || {
            do_fritz(tx2);
        });


    loop {
        sleep(Duration::from_millis(100));

        if let Ok(notification) = connection.recv() {
            println!("Notification = {notification:?}");
        }

        match rx.try_recv() {
            Ok(msg) => client.try_publish(msg.topic, QoS::AtLeastOnce, true, msg.payload.as_bytes()).unwrap(),
   
            Err(std::sync::mpsc::TryRecvError::Empty) => continue,

            Err(_) => break,
        }
    }

}
