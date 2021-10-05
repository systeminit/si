#[macro_use]
mod cfg;

cfg_feature! {
    #![feature = "server"]
    pub mod server;
}
