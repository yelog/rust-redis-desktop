use crate::connection::ConnectionConfig;

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Data,
    Terminal,
    Monitor,
    CommandStats,
    SlowLog,
    Clients,
    PubSub,
    Script,
}

#[derive(Clone, PartialEq)]
pub enum FormMode {
    New,
    Edit(ConnectionConfig),
}
