use bevy::reflect::Reflect;
use bevy::ecs::event::Event;
use bevy::ecs::event::{EventReader, EventWriter};
use bevy::ecs::system::{ResMut};
use bevy::input::{ButtonState, Input, Axis};
use bevy::prelude::DetectChangesMut;
use crate::axis::{_3dsAxis, _3dsAxisType};
use crate::button::{_3dsButton, _3dsButtonType};
/// 3ds event for when the "value" on the axis changes
/// by an amount larger than the threshold defined in [`_3dsInputSettings`].
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct _3dsAxisChangedEvent {
    /// The type of the triggered axis.
    pub axis_type: _3dsAxisType,
    /// The value of the axis.
    pub value: f32,
}

impl _3dsAxisChangedEvent {
    /// Creates a [`_3dsAxisChangedEvent`].
    pub fn new(axis_type: _3dsAxisType, value: f32) -> Self {
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
pub struct _3dsButtonChangedEvent {
    /// The 3ds button assigned to the event.
    pub button_type: _3dsButtonType,
    /// The pressed state of the button.
    pub state: ButtonState,
}

impl _3dsButtonChangedEvent {
    /// Creates a [`_3dsButtonChangedEvent`].
    pub fn new(button_type: _3dsButtonType, state: ButtonState) -> Self {
        Self {
            button_type,
            state,
        }
    }
}

/// A 3ds event.
///
/// This event type is used over the
/// [`_3dsButtonChangedEvent`] and [`_3dsAxisChangedEvent`] when
/// the in-frame relative ordering of events is important.
#[derive(Event, Debug, Clone, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum _3dsEvent {
    /// A button of the 3ds has been triggered.
    Button(_3dsButtonChangedEvent),
    /// An axis of the 3ds has been triggered.
    Axis(_3dsAxisChangedEvent),
}

impl From<_3dsButtonChangedEvent> for _3dsEvent {
    fn from(value: _3dsButtonChangedEvent) -> Self {
        Self::Button(value)
    }
}

impl From<_3dsAxisChangedEvent> for _3dsEvent {
    fn from(value: _3dsAxisChangedEvent) -> Self {
        Self::Axis(value)
    }
}


/// Splits the [`_3dsEvent`] event stream into it's component events.
pub fn _3ds_event_system(
    mut _3ds_events: EventReader<_3dsEvent>,
    mut button_events: EventWriter<_3dsButtonChangedEvent>,
    mut axis_events: EventWriter<_3dsAxisChangedEvent>,
    mut button_input: ResMut<Input<_3dsButton>>,
) {
    button_input.bypass_change_detection().clear();
    for _3ds_event in _3ds_events.read() {
        match _3ds_event {
            _3dsEvent::Button(button_event) => button_events.send(button_event.clone()),
            _3dsEvent::Axis(axis_event) => axis_events.send(axis_event.clone()),
        }
    }
}


/// Uses [`_3dsAxisChangedEvent`]s to update the relevant [`Input`] and [`Axis`] values.
pub fn _3ds_axis_event_system(
    mut _3ds_axis: ResMut<Axis<_3dsAxis>>,
    mut axis_events: EventReader<_3dsAxisChangedEvent>,
) {
    for axis_event in axis_events.read() {
        let axis = _3dsAxis::new(axis_event.axis_type);
        //if we want to put thresholds for udpating axis, we would use AxisSettings.filter here
        // after having fetched the old value from the axis resource that matches this axis_type.
        _3ds_axis.set(axis, axis_event.value);
        // TODO: decide if we want to send this as an event
    }
}


/// Uses [`_3dsButtonChangedEvent`]s to update the relevant [`Input`] values.
pub fn _3ds_button_event_system(
    mut button_changed_events: EventReader<_3dsButtonChangedEvent>,
    mut button_input: ResMut<Input<_3dsButton>>,
    mut button_input_events: EventWriter<_3dsButtonChangedEvent>,
) {
    //TODO: remove all button state checking logic, because ctru already does that for us.
    for button_event in button_changed_events.read() {
        let button = _3dsButton::new(button_event.button_type);
        if !button_event.state.is_pressed() {
            // Check if button was previously pressed
            if button_input.pressed(button) {
                button_input_events.send(_3dsButtonChangedEvent {
                    button_type: button.button_type,
                    state: ButtonState::Released,
                });
            }
            // We don't have to check if the button was previously pressed here
            // because that check is performed within Input<T>::release()
            button_input.release(button);
        } else if button_event.state.is_pressed() {
            // Check if button was previously not pressed
            if !button_input.pressed(button) {
                button_input_events.send(_3dsButtonChangedEvent {
                    button_type: button.button_type,
                    state: ButtonState::Pressed,
                });
            }
            button_input.press(button);
        };
    }
}
