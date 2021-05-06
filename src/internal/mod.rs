pub(super) mod archetypes;
pub(super) mod components;
pub(super) mod core;
pub(super) mod entities;
pub(super) mod group_actions;
pub(super) mod groups;
pub(super) mod system;
pub(super) mod system_state;

// TODO: change to pub(super) if possible
pub(crate) mod main;

// TODO: use one letter convention everywhere for closure params ?
// TODO: don't put "s" before Facade and System (check it's ok everywhere)
// TODO: put "s" if multiple params in macros
// TODO: refactor code to avoid line breaks when not method chaining
// TODO: docstrings starts with infinitive verb
// TODO: put maximum of external logic in internal module
// TODO: choose carefully "component_type" or "type" name
