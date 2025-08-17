use crate::app::control_primitives::KinState;
/// Computes position, velocity, and acceleration using a 5th degree polynomial trajectory
///
/// This function uses a 5th degree polynomial with the following boundary conditions:
/// - f(0) = A, f(1) = B
/// - f'(0) = 0, f'(1) = 0
/// - f''(0) = 0, f''(1) = 0
///
/// # Arguments
/// * `start_pos` - Starting position (A)
/// * `end_pos` - Ending position (B)
/// * `duration` - Total duration of the trajectory
/// * `start_time` - Start time of the trajectory
/// * `current_time` - Current time to evaluate at
///
/// # Returns
/// * `KinState` containing position, velocity, and acceleration
///
/// # Panics
/// * If duration is zero or negative
/// * If current_time is before start_time or after start_time + duration
pub fn trajectory_5th_degree(
    start_pos: f32,
    end_pos: f32,
    duration_s: f32,
    elapsed_s: f32,
) -> KinState {

    // Normalize time to [0, 1]
    let t = elapsed_s / duration_s;
    // Polynomial coefficients for f(t) = at⁵ + bt⁴ + ct³ + dt² + et + f
    let a = 6.0 * (end_pos - start_pos);
    let b = -15.0 * (end_pos - start_pos);
    let c = 10.0 * (end_pos - start_pos);
    let d = 0.0;
    let e = 0.0;
    let f = start_pos;

    // Calculate position: f(t) = at⁵ + bt⁴ + ct³ + dt² + et + f
    //let position = 0.0;
    let position = a * (t*t*t*t*t) + b * (t*t*t*t) + c * (t*t*t) + d * (t*t) + e * t + f;
    // Calculate velocity: f'(t) = 5at⁴ + 4bt³ + 3ct² + 2dt + e
    // Note: we need to scale by 1/duration to get velocity in original time units

    //let velocity_normalized = 0.0;
    let velocity_normalized = 5.0 * a * (t*t*t*t) + 4.0 * b * (t*t*t) + 3.0 * c * (t*t) + 2.0 * d * t + e;
    let velocity = velocity_normalized / duration_s;

    // Calculate acceleration: f''(t) = 20at³ + 12bt² + 6ct + 2d
    // Note: we need to scale by 1/duration² to get acceleration in original time units
    //let acceleration_normalized = 0.0;
    let acceleration_normalized = 20.0 * a * (t*t*t) + 12.0 * b * (t*t) + 6.0 * c * t + 2.0 * d;
    let acceleration = acceleration_normalized / (duration_s * duration_s);

    KinState {
        pos: position,
        speed: velocity,
        accel: acceleration,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_conditions() {
        let start_pos = 10.0;
        let end_pos = 20.0;
        let duration = Microseconds(2_000_000); // 2 seconds
        let start_time = Microseconds(0); // 1 second
        let end_time = Microseconds(2_000_000); // 1 second

        // Test at start
        let result_start = trajectory_5th_degree(start_pos, end_pos, duration, start_time);
        assert!((result_start.pos - start_pos).abs() < 1e-6);
        assert!(result_start.speed.abs() < 1e-6);
        assert!(result_start.accel.abs() < 1e-6);

        // Test at end
        let result_end = trajectory_5th_degree(start_pos, end_pos, duration, end_time);
        assert!((result_end.pos - end_pos).abs() < 1e-6);
        assert!(result_end.speed.abs() < 1e-6);
        assert!(result_end.accel.abs() < 1e-6);
    }

    #[test]
    fn test_midpoint() {
        let start_pos = 0.0;
        let end_pos = 10.0;
        let duration = Microseconds(1_000_000); // 1 second

        // Test at midpoint
        let result_mid = trajectory_5th_degree(start_pos, end_pos, duration, Microseconds(500_000));
        // At t=0.5, the 5th degree polynomial should give us position = 5.0
        assert!((result_mid.pos - 5.0).abs() < 1e-6);
    }
}