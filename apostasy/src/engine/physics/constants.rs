pub mod source_physics {
    /// Gravity acceleration in sqr(m/s)
    pub const GRAVITY: f32 = -15.0;

    /// Ground acceleration in sqr(m/s)
    pub const GROUND_ACCELERATION: f32 = 25.0;

    /// Air acceleration in m/s
    pub const AIR_ACCELERATION: f32 = 8.0;

    /// Ground friction for control
    pub const GROUND_FRICTION: f32 = 12.0;

    /// Stop speed threshold in m/s below this, friction halts movement completely
    pub const STOP_SPEED: f32 = 1.0;

    /// Maximum ground speed in m/s sprinting speed
    pub const MAX_GROUND_SPEED: f32 = 12.0;

    /// Jump speed in m/s
    pub const JUMP_SPEED: f32 = 6.6;

    /// Bunny hop momentum preservation factor
    pub const BUNNY_HOP_FACTOR: f32 = 0.95;

    /// Grace period for movement preservation after jump (in seconds)
    pub const JUMP_GRACE_PERIOD: f32 = 0.2;

    /// Coyote time allows jumping for this long after leaving ground (in seconds)
    pub const COYOTE_TIME: f32 = 0.15;

    /// Maximum air speed
    pub const MAX_AIR_SPEED: f32 = 15.0;

    /// Physics object linear damping
    pub const PHYSICS_LINEAR_DAMPING: f32 = 0.05;

    /// Physics object angular damping
    pub const PHYSICS_ANGULAR_DAMPING: f32 = 0.1;

    /// Resting threshold for physics objects
    pub const RESTING_THRESHOLD: f32 = 0.1;

    /// Terminal velocity for falling objects
    pub const TERMINAL_VELOCITY: f32 = 3500.0;
}

