use ambient_api::prelude::*;
use ambient_example_messaging::messages::Local;

#[main]
pub fn main() {
    // This module will receive messages from `server.rs`, and respond to them.
    Local::subscribe(move |source, data| {
        println!("{source:?}: {data:?}");
        if let Some(id) = source.local() {
            Local::new("Hi, back!").send_local(id);
        }
    });
}
