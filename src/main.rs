
//use std::io::Empty;

use std::thread;
use rumqttc::{Client, MqttOptions, QoS};
use std::time::{Duration};
use std::thread::sleep;
use std::sync::mpsc;

// compile: cargo build
// test: cargo run
// test:  cargo test -- --nocapture muell::tests::testmuell
// final: cargo build --release     // binary in target/release
// kill old running build on raspi first
// ps ax | grep -v grep | grep http_test
// sudo nohup /home/pi/rust/home_rust/target/release/http_test &

// cp /home/pi/rust/home_rust/target/release/http_test /home/pi/http_test_backup


pub struct Mymessage {
    pub topic: String,
    pub payload: String,
}

pub mod http;
pub mod fritz;
pub mod sonnen;
pub mod wallbox;
pub mod wandler;
pub mod wetter;
pub mod strom;
pub mod udplog;
pub mod muell;

fn send_message(the_topic: &str, the_payload:String, tx: &std::sync::mpsc::Sender<crate::Mymessage>) {
    let hostname = env!("HOSTNAME");
    if hostname != "Thomas_test" {
        let answ1 = crate::Mymessage {
            topic: String::from(the_topic),
            payload: the_payload,
        };
        if let Err(_) =  tx.send(answ1) {/* nothing */}; 
    }
    else {
        println!("{}", the_payload);
    }

   
}

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
    
    let _handle = thread::spawn( || {
        udplog::do_udp();
    });

    let tx2 = tx.clone();
    let _handle2 = thread::spawn( || {
            fritz::do_fritz(tx2);
        });

    let tx3 = tx.clone();
    let _handle3 = thread::spawn( || {
            sonnen::do_sonnen(tx3);
        });

    let tx4 = tx.clone();
    let _handle4 = thread::spawn( || {
            wallbox::do_wallbox(tx4);
        });

    let tx5 = tx.clone();
    let _handle5 = thread::spawn( || {
            wandler::do_wandler(tx5);
        });


    let tx6 = tx.clone();
    let _handle6 = thread::spawn( || {
            wetter::do_wetter(tx6);
    });

    let tx7 = tx.clone();
    let _handle7 = thread::spawn( || {
            strom::do_strom(tx7);
    });
   
    let tx8 = tx.clone();
    let _handle8 = thread::spawn( || {
            muell::do_muell(tx8);
    });

    loop {
        sleep(Duration::from_millis(100));

        if let Ok(_notification) = connection.recv() {
            #[cfg(debug_assertions)]
            println!("Notification = {_notification:?}");
        }

        match rx.try_recv() {
            Ok(msg) => {
                if cfg!(debug_assertions) {
                    let topic: String = "Debug/".to_string()+&msg.topic;
                    client.try_publish(topic, QoS::AtLeastOnce, true, msg.payload.as_bytes()).unwrap()
                } else {
                    if let Err(_) =  client.try_publish(msg.topic, QoS::AtLeastOnce, true, msg.payload.as_bytes()) {
                        //nothing;
                    }

                }
                
            },
   
            Err(std::sync::mpsc::TryRecvError::Empty) => continue,

            Err(_) => break,
        }
    }

}




