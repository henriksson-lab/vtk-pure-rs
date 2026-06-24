//! ODE (Ordinary Differential Equation) integrators.
//!
//! Used internally by stream tracers and particle advection.

/// Euler method: simplest first-order integrator.
pub fn euler_step(pos: [f64; 3], velocity_fn: &dyn Fn([f64; 3]) -> [f64; 3], dt: f64) -> [f64; 3] {
    let v = velocity_fn(pos);
    [pos[0] + v[0] * dt, pos[1] + v[1] * dt, pos[2] + v[2] * dt]
}

/// RK2 (midpoint method): second-order integrator.
pub fn rk2_step(pos: [f64; 3], velocity_fn: &dyn Fn([f64; 3]) -> [f64; 3], dt: f64) -> [f64; 3] {
    let k1 = velocity_fn(pos);
    let mid = [
        pos[0] + k1[0] * dt * 0.5,
        pos[1] + k1[1] * dt * 0.5,
        pos[2] + k1[2] * dt * 0.5,
    ];
    let k2 = velocity_fn(mid);
    [
        pos[0] + k2[0] * dt,
        pos[1] + k2[1] * dt,
        pos[2] + k2[2] * dt,
    ]
}

/// RK4 (classical Runge-Kutta): fourth-order integrator.
pub fn rk4_step(pos: [f64; 3], velocity_fn: &dyn Fn([f64; 3]) -> [f64; 3], dt: f64) -> [f64; 3] {
    let k1 = velocity_fn(pos);
    let p2 = [
        pos[0] + k1[0] * dt * 0.5,
        pos[1] + k1[1] * dt * 0.5,
        pos[2] + k1[2] * dt * 0.5,
    ];
    let k2 = velocity_fn(p2);
    let p3 = [
        pos[0] + k2[0] * dt * 0.5,
        pos[1] + k2[1] * dt * 0.5,
        pos[2] + k2[2] * dt * 0.5,
    ];
    let k3 = velocity_fn(p3);
    let p4 = [
        pos[0] + k3[0] * dt,
        pos[1] + k3[1] * dt,
        pos[2] + k3[2] * dt,
    ];
    let k4 = velocity_fn(p4);

    [
        pos[0] + (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]) * dt / 6.0,
        pos[1] + (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]) * dt / 6.0,
        pos[2] + (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]) * dt / 6.0,
    ]
}

/// Integrate a trajectory using RK4, returning all positions.
pub fn integrate_trajectory(
    start: [f64; 3],
    velocity_fn: &dyn Fn([f64; 3]) -> [f64; 3],
    dt: f64,
    max_steps: usize,
) -> Vec<[f64; 3]> {
    let mut positions = Vec::with_capacity(max_steps + 1);
    let mut pos = start;
    positions.push(pos);
    for _ in 0..max_steps {
        pos = rk4_step(pos, velocity_fn, dt);
        positions.push(pos);
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    // Constant velocity field: v = (1, 0, 0)
    fn const_vel(p: [f64; 3]) -> [f64; 3] {
        let _ = p;
        [1.0, 0.0, 0.0]
    }

    #[test]
    fn euler_constant() {
        let p = euler_step([0.0; 3], &const_vel, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rk2_constant() {
        let p = rk2_step([0.0; 3], &const_vel, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rk4_constant() {
        let p = rk4_step([0.0; 3], &const_vel, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-10);
    }

    // Circular orbit: v = (-y, x, 0) at unit circle speed
    fn circular_vel(p: [f64; 3]) -> [f64; 3] {
        [-p[1], p[0], 0.0]
    }

    #[test]
    fn rk4_circular_orbit() {
        // Start at (1,0,0), integrate one full period (2*pi)
        let dt = 0.01;
        let steps = (2.0 * std::f64::consts::PI / dt) as usize;
        let trajectory = integrate_trajectory([1.0, 0.0, 0.0], &circular_vel, dt, steps);

        // Should return close to start after one full orbit
        let last = trajectory.last().unwrap();
        assert!((last[0] - 1.0).abs() < 0.01, "x: {}", last[0]);
        assert!(last[1].abs() < 0.01, "y: {}", last[1]);
    }

    #[test]
    fn trajectory_length() {
        let traj = integrate_trajectory([0.0; 3], &const_vel, 0.1, 10);
        assert_eq!(traj.len(), 11); // start + 10 steps
    }
}
