crate mod command;
mod plugins;

// Plugins
mod infractions_plugin;

crate use plugins::Plugins;

crate use infractions_plugin::InfractionsPlugin;
