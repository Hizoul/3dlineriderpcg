use bevy::{prelude::Vec3, render::color::Color};

// Simulation Related
pub const DIRECTION_FORWARD: i64 = 0;
pub const DIRECTION_RIGHT: i64 = 1;
pub const DIRECTION_LEFT: i64 = 2;
pub const DIRECTION_BACK: i64 = 3;

pub const ACTION_STRAIGHT: i64 = 0;
pub const ACTION_RIGHT: i64 = 1;
pub const ACTION_LEFT: i64 = 2;
pub const ACTION_UP: i64 = 3;
pub const ACTION_DOWN: i64 = 4;
pub const ACTION_STRAIGHT_DOWN: i64 = 5;
pub const ACTION_STRAIGHT_EMPTY: i64 = 6;
pub const ACTION_RIGHT_EMPTY: i64 = 7;
pub const ACTION_LEFT_EMPTY: i64 = 8;
pub const ACTION_UP_EMPTY: i64 = 9;
pub const ACTION_DOWN_EMPTY: i64 = 10;
pub const ACTION_STRAIGHT_DOWN_EMPTY: i64 = 11;
pub const ACTION_STRAIGHT_BOOST: i64 = 12;
pub const ACTION_RIGHT_BOOST: i64 = 13;
pub const ACTION_LEFT_BOOST: i64 = 14;
pub const ACTION_UP_BOOST: i64 = 15;
pub const ACTION_DOWN_BOOST: i64 = 16;
pub const ACTION_STRAIGHT_DOWN_BOOST: i64 = 17;
pub const ACTION_STRAIGHT_DAMPEN: i64 = 18;
pub const ACTION_RIGHT_DAMPEN: i64 = 19;
pub const ACTION_LEFT_DAMPEN: i64 = 20;
pub const ACTION_UP_DAMPEN: i64 = 21;
pub const ACTION_DOWN_DAMPEN: i64 = 22;
pub const ACTION_STRAIGHT_DOWN_DAMPEN: i64 = 23;

pub const TP_EMPTY: u8 = 0;
pub const TP_NORMAL: u8 = 1;
pub const TP_ACCELERATE: u8 = 2;
pub const TP_DECELERATE: u8 = 3;
pub const TP_GOAL: u8 = 4;

// Env related
pub const PREV_POINTS_ZEROED: [Vec3; 4] = [Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO];
pub const ACTION_TYPE_STATIC: u8 = 0;
pub const ACTION_TYPE_STATIC_WITH_EMPTY: u8 = 1;
pub const ACTION_TYPE_STATIC_WITH_BOOST: u8 = 2;
pub const ACTION_TYPE_FREE_POINTS: u8 = 3;
pub const ACTION_TYPE_FREE_POINTS_WITH_TP: u8 = 4;
pub const ACTION_TYPE_FREE_POINTS_RELATIVE: u8 = 5;
pub const ACTION_TYPE_FREE_POINTS_WITH_TP_RELATIVE: u8 = 6;
pub const ACTION_TYPE_RADIAL: u8 = 7;
pub const ACTION_TYPE_RADIAL_WITH_TP: u8 = 8;
pub const OBSERVATION_TYPE_BUILD_POINTS: u8 = 0;
pub const OBSERVATION_TYPE_3D_VIEW: u8 = 1;
pub const OBSERVATION_TYPE_3D_VIEW_ONEHOT: u8 = 2;
pub const OBSERVATION_TYPE_GOAL_AND_LAST_POINT: u8 = 3;
pub const OBSERVATION_TYPE_SLIDING_WINDOW: u8 = 4;
pub const TARGET_STATIC_START_AND_END: u8 = 0;
pub const TARGET_RANDOM_START_AND_END: u8 = 1;
pub const TARGET_STATIC_START_RANDOM_END: u8 = 2;
pub const TARGET_RANDOM_START_STATIC_END: u8 = 3;
pub const TARGET_ABOVE_START: u8 = 4;
pub const TARGET_SAME_HEIGHT_AS_START: u8 = 5;
pub const TARGET_LOOP_TRACK: u8 = 6;  // TODO
pub const TARGET_RANDOM_WITH_CHECKPOINT_BELOW: u8 = 7;
pub const TARGET_RANDOM_WITH_CHECKPOINT_ABOVE: u8 = 8;

pub const REWARD_GOAL_REACHED_BY_BALL: u8 = 0;
pub const REWARD_GOAL_REACHED_BY_TRACK: u8 = 1;
pub const REWARD_GOAL_REACHED_BY_BOTH_ONLY: u8 = 2;
pub const REWARD_GOING_UP: u8 = 3;
pub const REWARD_LONGEST_TRACK: u8 = 4;
pub const REWARD_SHORTEST_TRACK: u8 = 5;
pub const REWARD_SPEED_TOTAL: u8 = 6;
pub const REWARD_SPEED_AT_END: u8 = 7;
pub const REWARD_LOW_SPEED_AT_END: u8 = 8;
pub const REWARD_MOST_ROTATION: u8 = 9;
pub const REWARD_LEAST_ROTATION: u8 = 10;
pub const REWARD_TRACK_TOUCHES: u8 = 11;
pub const REWARD_AIR_TIME: u8 = 12;
pub const REWARD_VALID_ACTION_CHOSEN: u8 = 13;
pub const REWARD_SCOLD_INVALID_ACTION: u8 = 14;
pub const REWARD_DISTANCE_TO_GOAL_IN_SIMULATION: u8 = 15;
pub const REWARD_SIMULATE_INBETWEEN: u8 = 16;
pub const REWARD_FASTEST_GOAL_REACH: u8 = 17;
pub const REWARD_END_BUILD_PHASE_IF_TRACK_REACHES_GOAL: u8 = 18;
pub const REWARD_DISTANCE_TO_GOAL_IN_SIMULATION_IF_TRACK_REACHED_GOAL: u8 = 20;
pub const REWARD_DISTANCE_OF_TRACK_TO_GOAL_AT_END: u8 = 21;
pub const REWARD_TRACK_CLOSER_TO_GOAL_IN_STEP: u8 = 22;
pub const REWARD_SCOLD_PREMATURE_END: u8 = 23;
pub const REWARD_MIMIC_STRAIGHT_LINE_HEURISTIC: u8 = 24;
pub const REWARD_USING_BOOSTER_TYPE_TRACK: u8 = 25;
pub const REWARD_REACH_CHECKPOINT: u8 = 26;
pub const REWARD_TRACK_REACH_CHECKPOINT: u8 = 27;


pub const COLOR_GOAL: Color = Color::rgba(0.0, 1.0, 0.54, 0.35);
pub const COLOR_CHECKPOINT: Color = Color::rgba(0.11, 0.3, 0.0, 0.35);
pub const COLOR_SPAWN: Color = Color::rgba(0.98, 0.81, 0.63, 0.5);
pub const COLOR_RIDER: Color = Color::rgba(0.97, 0.57, 0.37, 1.0);
pub const COLOR_TRACK: Color = Color::rgba(0.0, 0.69, 0.79, 1.0);
pub const COLOR_TRACK_BOOST: Color = Color::rgba(0.49, 0.81, 0.71, 1.0);
pub const COLOR_TRACK_BRAKE: Color = Color::rgba(0.43, 0.1, 0.02, 1.0);