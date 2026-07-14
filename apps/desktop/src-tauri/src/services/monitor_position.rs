use crate::domain::SavedPosition;
use tauri::{AppHandle, Manager, PhysicalPosition, Position};

const SAFE_MARGIN: i32 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkArea {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn restore_mascot_position(
    app: &AppHandle,
    saved: Option<&SavedPosition>,
) -> Result<(i32, i32), String> {
    let window = app
        .get_webview_window("mascot")
        .ok_or_else(|| "mascot window is unavailable".to_string())?;
    let size = window.outer_size().map_err(|error| error.to_string())?;
    let monitors = app.available_monitors().map_err(|error| error.to_string())?;
    let areas: Vec<(Option<String>, WorkArea)> = monitors
        .iter()
        .map(|monitor| {
            let area = monitor.work_area();
            (
                monitor.name().cloned(),
                WorkArea {
                    x: area.position.x,
                    y: area.position.y,
                    width: area.size.width,
                    height: area.size.height,
                },
            )
        })
        .collect();

    let preferred_area = saved.and_then(|position| {
        areas
            .iter()
            .find(|(name, _)| name == &position.monitor_name)
            .map(|(_, area)| *area)
    });
    let primary_area = app
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .map(|monitor| {
            let area = monitor.work_area();
            WorkArea {
                x: area.position.x,
                y: area.position.y,
                width: area.size.width,
                height: area.size.height,
            }
        })
        .or_else(|| areas.first().map(|(_, area)| *area))
        .ok_or_else(|| "no monitor is available".to_string())?;

    let candidate = saved.and_then(|position| {
        let area = preferred_area.or_else(|| {
            areas
                .iter()
                .map(|(_, area)| *area)
                .find(|area| point_is_in_area(position.x, position.y, *area))
        })?;
        Some(clamp_to_area(
            position.x,
            position.y,
            size.width,
            size.height,
            area,
        ))
    });
    let position = candidate.unwrap_or_else(|| {
        bottom_right_position(size.width, size.height, primary_area, SAFE_MARGIN)
    });
    window
        .set_position(Position::Physical(PhysicalPosition::new(
            position.0, position.1,
        )))
        .map_err(|error| error.to_string())?;
    Ok(position)
}

pub fn bottom_right_position(
    window_width: u32,
    window_height: u32,
    area: WorkArea,
    margin: i32,
) -> (i32, i32) {
    let right = area.x.saturating_add(area.width as i32);
    let bottom = area.y.saturating_add(area.height as i32);
    (
        right
            .saturating_sub(window_width as i32)
            .saturating_sub(margin),
        bottom
            .saturating_sub(window_height as i32)
            .saturating_sub(margin),
    )
}

pub fn clamp_to_area(
    x: i32,
    y: i32,
    window_width: u32,
    window_height: u32,
    area: WorkArea,
) -> (i32, i32) {
    let max_x = area
        .x
        .saturating_add(area.width as i32)
        .saturating_sub(window_width as i32);
    let max_y = area
        .y
        .saturating_add(area.height as i32)
        .saturating_sub(window_height as i32);
    (x.clamp(area.x, max_x.max(area.x)), y.clamp(area.y, max_y.max(area.y)))
}

fn point_is_in_area(x: i32, y: i32, area: WorkArea) -> bool {
    x >= area.x
        && y >= area.y
        && x < area.x.saturating_add(area.width as i32)
        && y < area.y.saturating_add(area.height as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_negative_monitor_coordinates() {
        let area = WorkArea {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1040,
        };
        assert_eq!(clamp_to_area(-2000, 900, 560, 360, area), (-1920, 680));
    }

    #[test]
    fn resets_to_primary_work_area_bottom_right() {
        let area = WorkArea {
            x: 0,
            y: 0,
            width: 1920,
            height: 1040,
        };
        assert_eq!(bottom_right_position(560, 360, area, 24), (1336, 656));
    }

    #[test]
    fn keeps_window_visible_when_larger_than_work_area() {
        let area = WorkArea {
            x: 100,
            y: 50,
            width: 320,
            height: 200,
        };
        assert_eq!(clamp_to_area(999, 999, 560, 360, area), (100, 50));
    }
}
