
//use std::io::Empty;

use std::thread;
use rumqttc::{Client, MqttOptions, QoS};
use std::time::{Duration};
use std::thread::sleep;
use std::sync::mpsc;

// compile: cargo build
// test: cargo run
// final: cargo build --release     // binary in target/release

pub struct Mymessage {
    pub topic: String,
    pub payload: String,
}

pub mod http;
pub mod fritz;
pub mod sonnen;

fn main()  {
    let mqttclient: String;
    let hostname = env!("HOSTNAME");  // compile time!!!
    if hostname != "Thomas_test" {
         mqttclient = hostname.to_string();
    }
    else {
        mqttclient = "MQTT_".to_owned()+env!("HOSTNAME");
    }
    
    let mqttuser  = env!("MQTT_user");
    let mqttpass  = env!("MQTT_password");

    let mut mqttoptions = MqttOptions::new(mqttclient, "192.168.0.46", 1883);
    mqttoptions
        .set_keep_alive(Duration::from_secs(5))
        .set_credentials(mqttuser, mqttpass);

    let (client, mut connection) = Client::new(mqttoptions, 10);
    let (tx, rx) = mpsc::channel();
    
    let tx2 = tx.clone();
    let _handle = thread::spawn( || {
            fritz::do_fritz(tx2);
        });

    let tx3 = tx.clone();
    let _handle = thread::spawn( || {
            sonnen::do_sonnen(tx3);
        });

    loop {
        sleep(Duration::from_millis(100));

        if let Ok(notification) = connection.recv() {
            #[cfg(debug_assertions)]
            println!("Notification = {notification:?}");
        }

        match rx.try_recv() {
            Ok(msg) => {
                if cfg!(debug_assertions) {
                    let topic: String = "Debug/".to_string()+&msg.topic;
                    client.try_publish(topic, QoS::AtLeastOnce, true, msg.payload.as_bytes()).unwrap()
                } else {
                    client.try_publish(msg.topic, QoS::AtLeastOnce, true, msg.payload.as_bytes()).unwrap()
                }
                
            },
   
            Err(std::sync::mpsc::TryRecvError::Empty) => continue,

            Err(_) => break,
        }
    }

}




