//! Delivery is a microservice.

extern crate delivery_lib;
extern crate stq_logging;

fn main() {
    let config = delivery_lib::config::Config::new().expect("Can't load app config!");

    // Prepare logger
    stq_logging::init(config.graylog.as_ref());

    delivery_lib::start_server(config, &None, || ());
}
