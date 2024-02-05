use bevy::reflect::Reflect;
use bevy::ecs::event::Event;
use bevy::ecs::event::{EventReader, EventWriter};
use bevy::ecs::system::{ResMut};
use bevy::input::{ButtonState, Input, Axis};
use bevy::prelude::DetectChangesMut;
use crate::axis::{Axis3ds, Axis3dsType};
use crate::button::{Button3ds, Button3dsType};
/// 3ds event for when the "value" on the axis changes
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Axis3dsChangedEvent {
    /// The type of the triggered axis.
    pub axis_type: Axis3dsType,
    /// The value of the axis.
    pub value: f32,
}

impl Axis3dsChangedEvent {
    /// Creates a [`Axis3dsChangedEvent`].
    pub fn new(axis_type: Axis3dsType, value: f32) -> Self {
        Self {
            axis_type,
            value,
        }
    }
}


/// A 3ds button input event, that only gets sent when the button state has changed.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct Button3dsChangedEvent {
    /// The 3ds button assigned to the event.
    pub button_type: Button3dsType,
    /// The pressed state of the button.
    pub state: ButtonState,
}

impl Button3dsChangedEvent {
    /// Creates a [`Button3dsChangedEvent`].
    pub fn new(button_type: Button3dsType, state: ButtonState) -> Self {
        Self {
            button_type,
            state,
        }
    }
}


/// A 3ds button input event from ctru event system.
#[derive(Event, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct CtruButtonChangedEvent {
    /// The 3ds button assigned to the event.
    pub button_type: Button3dsType,
    /// The pressed state of the button.
    pub state: ButtonState,
}

impl CtruButtonChangedEvent {
    /// Creates a [`CtruButtonChangedEvent`].
    pub fn new(button_type: Button3dsType, state: ButtonState) -> Self {
        Self {
            button_type,
            state,
        }
    }
}


/// A 3ds event.
///
/// This event type is used over the
/// [`Button3dsChangedEvent`] and [`Axis3dsChangedEvent`] when
/// the in-frame relative ordering of events is important.
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum Event3ds {
    /// A button of the 3ds has been triggered.
    Button(CtruButtonChangedEvent),
    /// An axis of the 3ds has been triggered.
    Axis(Axis3dsChangedEvent),
}

impl From<CtruButtonChangedEvent> for Event3ds {
    fn from(value: CtruButtonChangedEvent) -> Self {
        Self::Button(value)
    }
}

impl From<Axis3dsChangedEvent> for Event3ds {
    fn from(value: Axis3dsChangedEvent) -> Self {
        Self::Axis(value)
    }
}


/// Splits the [`Event3ds`] event stream into it's component events.
pub fn event_system_3ds(
    mut events_3ds: EventReader<Event3ds>,
    mut button_events: EventWriter<CtruButtonChangedEvent>,
    mut axis_events: EventWriter<Axis3dsChangedEvent>,
    mut button_input: ResMut<Input<Button3ds>>,
) {
    button_input.bypass_change_detection().clear();
    for event_3ds in events_3ds.read() {
        match event_3ds {
            Event3ds::Button(button_event) => button_events.send(button_event.clone()),
            Event3ds::Axis(axis_event) => axis_events.send(axis_event.clone()),
        }
    }
}


/// Uses [`Axis3dsChangedEvent`]s to update the relevant [`Input`] and [`Axis`] values.
pub fn axis_3ds_event_system(
    mut axis_3ds: ResMut<Axis<Axis3ds>>,
    mut axis_events: EventReader<Axis3dsChangedEvent>,
) {
    for axis_event in axis_events.read() {
        let axis = Axis3ds::new(axis_event.axis_type);
        //if we want to put thresholds for udpating axis, we would use AxisSettings.filter here
        // after having fetched the old value from the axis resource that matches this axis_type.
        axis_3ds.set(axis, axis_event.value);
        // TODO: decide if we want to send this as an event
    }
}


/// Uses [`Button3dsChangedEvent`]s to update the relevant [`Input`] values.
pub fn button_3ds_event_system(
    mut button_changed_events: EventReader<CtruButtonChangedEvent>,
    mut button_input: ResMut<Input<Button3ds>>,
    mut button_input_events: EventWriter<Button3dsChangedEvent>,
) {
    for button_event in button_changed_events.read() {
        let button = Button3ds::new(button_event.button_type);
        if !button_event.state.is_pressed() {
            // Check if button was previously pressed
            if button_input.pressed(button) { //todo this if statement is redundant because ctru
                //already checks that button wasn't pressed in previous frame
                button_input_events.send(Button3dsChangedEvent {
                    button_type: button.button_type,
                    state: ButtonState::Released,
                });
            }
            // We don't have to check if the button was previously pressed here
            // because that check is performed within Input<T>::release()
            button_input.release(button);
        } else if button_event.state.is_pressed() {
            // Check if button was previously not pressed
            if !button_input.pressed(button) { // same as the if statement above
                button_input_events.send(Button3dsChangedEvent {
                    button_type: button.button_type,
                    state: ButtonState::Pressed,
                });
            }
            button_input.press(button);
        };
    }
}
