use crate::Value;

/// Defines the interface for message factories.
///
/// The callback functions are called when the specific event occurs during message processing.
///
pub trait MessageFactory {
    /// Called when a \<template> processing is started.
    /// * `id` is the template id;
    /// * `name` is the template name.
    fn start_template(&mut self, id: u32, name: &str);

    /// Called when a \<template> processing is finished.
    fn stop_template(&mut self);

    /// Called when a field element is processed.
    /// * `id` is the field instruction id;
    /// * `name` is the field name;
    /// * `value` is the field value which is optional.
    fn set_value(&mut self, id: u32, name: &str, value: Option<Value>);

    /// Called when a \<sequence> element processing is started.
    /// * `id` is the sequence instruction id; can be `0` if id is not specified;
    /// * `name` is the sequence name;
    /// * `length` is the sequence length.
    fn start_sequence(&mut self, id: u32, name: &str, length: u32);

    /// Called when a sequence item processing is started.
    /// * `index` is the sequence item index.
    fn start_sequence_item(&mut self, index: u32);

    /// Called when a sequence item processing is finished.
    fn stop_sequence_item(&mut self);

    /// Called when a \<sequence> processing is finished.
    fn stop_sequence(&mut self);

    /// Called when a \<group> element processing is started.
    /// * `name` is the group name.
    fn start_group(&mut self, name: &str);

    /// Called when a \<group> element processing is finished.
    fn stop_group(&mut self);

    /// Called when a template reference (\<templateRef>) processing is started.
    /// * `name` is the template name;
    /// * `dynamic` is `true` if the template reference is dynamic.
    fn start_template_ref(&mut self, name: &str, dynamic: bool);

    /// Called when a template reference (\<templateRef>) processing is finished.
    fn stop_template_ref(&mut self);
}
