
use std::time::Duration;

use bevy::{
    prelude::*,
    time::TimeSystem,
    reflect::Reflect
};


#[derive(Default)]
pub struct FakeTimePlugin;

#[derive(Resource, Reflect, Debug, Clone)]
pub struct FakeTimeStepSize(pub u64);

impl Default for FakeTimeStepSize {
    fn default() -> FakeTimeStepSize {
        FakeTimeStepSize(40)
    }
}


impl Plugin for FakeTimePlugin {
    fn build(&self, app: &mut App) {
        app
        .register_type::<Timer>()
        .init_resource::<Time>()
        .insert_resource(Time::<Virtual>::from_max_delta(Duration::from_secs(6)))
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(2)))
        .init_resource::<FakeTimeStepSize>()
        .add_systems(First, fake_time_system.in_set(TimeSystem));
    }
}

fn fake_time_system(
    mut time: ResMut<Time>,
    mut v_time: ResMut<Time::<Virtual>>,
    time_step_size: Res<FakeTimeStepSize>,
    mut has_received_time: Local<bool>,
) {
    time.advance_by(Duration::from_millis(time_step_size.0));
    v_time.advance_by(Duration::from_millis(time_step_size.0));
    *has_received_time = true;
}
