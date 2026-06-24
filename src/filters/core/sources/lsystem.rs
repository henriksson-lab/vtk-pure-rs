//! L-system turtle graphics to polyline geometry.

use crate::data::{CellArray, Points, PolyData};

/// L-system rule: character -> replacement string.
pub struct LRule {
    pub from: char,
    pub to: String,
}

/// Generate L-system string after N iterations.
pub fn lsystem_generate(axiom: &str, rules: &[LRule], iterations: usize) -> String {
    let mut current = axiom.to_string();
    for _ in 0..iterations {
        let mut next = String::new();
        for ch in current.chars() {
            if let Some(rule) = rules.iter().find(|r| r.from == ch) {
                next.push_str(&rule.to);
            } else {
                next.push(ch);
            }
        }
        current = next;
    }
    current
}

/// Interpret L-system string as turtle graphics, producing polylines.
/// F = forward, + = turn left, - = turn right, [ = push state, ] = pop state.
pub fn lsystem_to_polydata(instructions: &str, step: f64, angle_deg: f64) -> PolyData {
    let angle = angle_deg.to_radians();
    let mut pts = Points::<f64>::new();
    let mut lines = CellArray::new();
    let mut x = 0.0f64;
    let mut y = 0.0f64;
    let mut dir = std::f64::consts::FRAC_PI_2; // start pointing up
    let mut stack: Vec<(f64, f64, f64)> = Vec::new();
    let mut current_line: Vec<i64> = Vec::new();

    let start = pts.len();
    pts.push([x, y, 0.0]);
    current_line.push(start as i64);

    for ch in instructions.chars() {
        match ch {
            'F' | 'G' => {
                x += step * dir.cos();
                y += step * dir.sin();
                let idx = pts.len();
                pts.push([x, y, 0.0]);
                current_line.push(idx as i64);
            }
            '+' => dir += angle,
            '-' => dir -= angle,
            '[' => {
                stack.push((x, y, dir));
                if current_line.len() >= 2 {
                    lines.push_cell(&current_line);
                }
                current_line.clear();
                let idx = pts.len();
                pts.push([x, y, 0.0]);
                current_line.push(idx as i64);
            }
            ']' => {
                if current_line.len() >= 2 {
                    lines.push_cell(&current_line);
                }
                current_line.clear();
                if let Some((sx, sy, sd)) = stack.pop() {
                    x = sx;
                    y = sy;
                    dir = sd;
                    let idx = pts.len();
                    pts.push([x, y, 0.0]);
                    current_line.push(idx as i64);
                }
            }
            _ => {}
        }
    }
    if current_line.len() >= 2 {
        lines.push_cell(&current_line);
    }

    let mut result = PolyData::new();
    result.points = pts;
    result.lines = lines;
    result
}

/// Generate Koch snowflake.
pub fn koch_snowflake(iterations: usize, step: f64) -> PolyData {
    let rules = [LRule {
        from: 'F',
        to: "F+F--F+F".to_string(),
    }];
    let s = lsystem_generate("F--F--F", &rules, iterations);
    lsystem_to_polydata(&s, step, 60.0)
}

/// Generate Hilbert curve.
pub fn hilbert_curve(iterations: usize, step: f64) -> PolyData {
    let rules = [
        LRule {
            from: 'A',
            to: "-BF+AFA+FB-".to_string(),
        },
        LRule {
            from: 'B',
            to: "+AF-BFB-FA+".to_string(),
        },
    ];
    let s = lsystem_generate("A", &rules, iterations);
    lsystem_to_polydata(&s, step, 90.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_generate() {
        let rules = [LRule {
            from: 'F',
            to: "F+F".to_string(),
        }];
        let s = lsystem_generate("F", &rules, 2);
        assert_eq!(s, "F+F+F+F");
    }
    #[test]
    fn test_koch() {
        let k = koch_snowflake(1, 1.0);
        assert!(k.points.len() > 3);
        assert!(k.lines.num_cells() >= 1);
    }
    #[test]
    fn test_hilbert() {
        let h = hilbert_curve(2, 1.0);
        assert!(h.points.len() > 4);
    }
}
