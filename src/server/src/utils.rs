extern crate bincode;
extern crate common;
extern crate log4rs;
extern crate log_panics;
extern crate simple_error;

use log;
use log4rs::{append, config, encode};
use simple_error::SimpleError as Error;
use std::env;
use std::error;

use rand::{self, distributions, Rng};

pub fn init_logger() -> Result<(), Box<dyn error::Error>> {
    log_panics::init();

    let logfile = append::file::FileAppender::builder()
        .encoder(Box::new(encode::pattern::PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S)} {l} {t}] {m}{n}",
        )))
        .build("/var/log/rc.server.log")?;

    let config = config::Config::builder()
        .appender(config::Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            config::Root::builder()
                .appender("logfile")
                .build(log::LevelFilter::Debug),
        )?;

    log4rs::init_config(config)?;

    Ok(())
}

const DEFAULT_PORT: u16 = 20301;

pub struct Config {
    token: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        let port: u16 = match env::var("RC_PORT") {
            Ok(value) => match value.parse::<u16>() {
                Ok(res) => res,
                Err(_) => {
                    return Err(Error::new(
                        "Invalid integer value for RC_PORT environment variable.",
                    ))
                }
            },
            Err(_) => {
                debug!(
                    "RC_PORT environment variable missing. Use default {} port.",
                    DEFAULT_PORT
                );
                DEFAULT_PORT
            }
        };

        let token = match env::var("RC_TOKEN") {
            Ok(res) => res,
            Err(_) => {
                let tmp = Config::tmp_token(64);
                warn!(
                    "RC_TOKEN environment variable missing. Using temporary token:\n{}",
                    tmp
                );
                tmp
            }
        };

        Ok(Config {
            token: token,
            port: port,
        })
    }

    pub fn is_valid_token(&self, token: String) -> bool {
        self.token == token
    }

    fn tmp_token(size: u8) -> String {
        rand::thread_rng()
            .sample_iter(distributions::Alphanumeric)
            .take(size as usize)
            .collect()
    }
}
