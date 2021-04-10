use structopt::clap::arg_enum;

pub mod gal16v8;

arg_enum! {
    #[derive(Debug)]
    pub enum Device {
        Gal16V8,
    }
}
