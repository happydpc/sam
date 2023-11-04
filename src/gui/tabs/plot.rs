use std::cell::RefCell;
use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use egui::Color32;
use egui::Rect;
use egui::Vec2;
use egui_gizmo::Gizmo;
use egui_gizmo::GizmoMode;
use egui_gizmo::GizmoVisuals;
use nalgebra::UnitQuaternion;
use nalgebra::Vector3;

use crate::data_source::DataSource;
use crate::settings::AppSettings;
use crate::state::VehicleState;

use crate::gui::map::*;
use crate::gui::maxi_grid::*;
use crate::gui::misc::*;
use crate::gui::plot::*;

const R: Color32 = Color32::from_rgb(0xfb, 0x49, 0x34);
const G: Color32 = Color32::from_rgb(0xb8, 0xbb, 0x26);
const B: Color32 = Color32::from_rgb(0x83, 0xa5, 0x98);
const R1: Color32 = Color32::from_rgb(0xcc, 0x24, 0x1d);
const G1: Color32 = Color32::from_rgb(0x98, 0x97, 0x1a);
const B1: Color32 = Color32::from_rgb(0x45, 0x85, 0x88);
const O: Color32 = Color32::from_rgb(0xfa, 0xbd, 0x2f);
const O1: Color32 = Color32::from_rgb(0xd6, 0x5d, 0x0e);
const BR: Color32 = Color32::from_rgb(0x61, 0x48, 0x1c);
const P: Color32 = Color32::from_rgb(0xb1, 0x62, 0x86);
const C: Color32 = Color32::from_rgb(0x68, 0x9d, 0x6a);

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectedPlot {
    Orientation,
    VerticalSpeed,
    Altitude,
    Gyroscope,
    Accelerometers,
    Magnetometer,
    Pressures,
    Temperatures,
    Power,
    Runtime,
    Signal,
    Map,
}

impl std::fmt::Display for SelectedPlot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SelectedPlot::Orientation    => write!(f, "Orientation"),
            SelectedPlot::VerticalSpeed  => write!(f, "Vertical Speed & Accel."),
            SelectedPlot::Altitude       => write!(f, "Altitude"),
            SelectedPlot::Gyroscope      => write!(f, "Gyroscope"),
            SelectedPlot::Accelerometers => write!(f, "Accelerometers"),
            SelectedPlot::Magnetometer   => write!(f, "Magnetometer"),
            SelectedPlot::Pressures      => write!(f, "Pressures"),
            SelectedPlot::Temperatures   => write!(f, "Temperatures"),
            SelectedPlot::Power          => write!(f, "Power"),
            SelectedPlot::Runtime        => write!(f, "Runtime"),
            SelectedPlot::Signal         => write!(f, "Signal"),
            SelectedPlot::Map            => write!(f, "Map"),
        }
    }
}

pub struct PlotTab {
    maxi_grid_state: MaxiGridState,
    dropdown_selected_plot: SelectedPlot,

    shared_plot: Rc<RefCell<SharedPlotState>>,
    orientation_plot: PlotState,
    vertical_speed_plot: PlotState,
    altitude_plot: PlotState,
    gyroscope_plot: PlotState,
    accelerometer_plot: PlotState,
    magnetometer_plot: PlotState,
    barometer_plot: PlotState,
    temperature_plot: PlotState,
    power_plot: PlotState,
    runtime_plot: PlotState,
    signal_plot: PlotState,

    map: MapState,
}

impl PlotTab {
    pub fn init(settings: &AppSettings) -> Self {
        let shared_plot = Rc::new(RefCell::new(SharedPlotState::new()));

        let orientation_plot = PlotState::new("Orientation", (Some(-180.0), Some(540.0)), shared_plot.clone())
            .line("Roll (Z) [°]", B, |vs| vs.euler_angles().map(|a| a.z))
            .line("Pitch (X) [°]", R, |vs| vs.euler_angles().map(|a| a.x))
            .line("Yaw (Y) [°]", G, |vs| vs.euler_angles().map(|a| a.y))
            .line("Roll (True) (Z) [°]", B1, |vs| vs.true_euler_angles().map(|a| a.z))
            .line("Pitch (True) (X) [°]", R1, |vs| vs.true_euler_angles().map(|a| a.x))
            .line("Yaw (True) (Y) [°]", G1, |vs| vs.true_euler_angles().map(|a| a.y));

        let vertical_speed_plot = PlotState::new("Vert. Speed & Accel.", (None, None), shared_plot.clone())
            .line("Vertical Accel [m/s²]", O1, |vs| vs.vertical_accel)
            .line("Vertical Accel (Filt.) [m/s²]", O, |vs| vs.vertical_accel_filtered)
            .line("Vario [m/s]", B, |vs| vs.vertical_speed)
            .line("True Vertical Accel [m/s²]", G, |vs| vs.true_vertical_accel)
            .line("True Vario [m/s]", B1, |vs| vs.true_vertical_speed);

        let altitude_plot = PlotState::new("Altitude (ASL)", (None, None), shared_plot.clone())
            .line("Altitude (Ground) [m]", BR, |vs| vs.altitude_ground)
            .line("Altitude (Baro) [m]", B1, |vs| vs.altitude_baro)
            .line("Altitude [m]", B, |vs| vs.altitude)
            .line("Altitude (GPS) [m]", G, |vs| vs.altitude_gps);

        let gyroscope_plot = PlotState::new("Gyroscope", (Some(-10.0), Some(10.0)), shared_plot.clone())
            .line("Gyro (X) [°/s]", R, |vs| vs.gyroscope.map(|a| a.x))
            .line("Gyro (Y) [°/s]", G, |vs| vs.gyroscope.map(|a| a.y))
            .line("Gyro (Z) [°/s]", B, |vs| vs.gyroscope.map(|a| a.z));

        let accelerometer_plot = PlotState::new("Accelerometers", (Some(-10.0), Some(10.0)), shared_plot.clone())
            .line("Accel 2 (X) [m/s²]", R1, |vs| vs.accelerometer2.map(|a| a.x))
            .line("Accel 2 (Y) [m/s²]", G1, |vs| vs.accelerometer2.map(|a| a.y))
            .line("Accel 2 (Z) [m/s²]", B1, |vs| vs.accelerometer2.map(|a| a.z))
            .line("Accel 1 (X) [m/s²]", R, |vs| vs.accelerometer1.map(|a| a.x))
            .line("Accel 1 (Y) [m/s²]", G, |vs| vs.accelerometer1.map(|a| a.y))
            .line("Accel 1 (Z) [m/s²]", B, |vs| vs.accelerometer1.map(|a| a.z));

        let magnetometer_plot = PlotState::new("Magnetometer", (None, None), shared_plot.clone())
            .line("Mag (X) [µT]", R, |vs| vs.magnetometer.map(|a| a.x))
            .line("Mag (Y) [µT]", G, |vs| vs.magnetometer.map(|a| a.y))
            .line("Mag (Z) [µT]", B, |vs| vs.magnetometer.map(|a| a.z));

        let barometer_plot = PlotState::new("Pressures", (None, None), shared_plot.clone())
            .line("Barometer [bar]", C, |vs| vs.pressure_baro.map(|p| p / 1000.0))
            .line("Drogue Cartridge [bar]", R1, |vs| vs.drogue_cartridge_pressure)
            .line("Drogue Chamber [bar]", G1, |vs| vs.drogue_chamber_pressure)
            .line("Main Cartridge [bar]", R, |vs| vs.main_cartridge_pressure)
            .line("Main Chamber [bar]", G, |vs| vs.main_chamber_pressure);

        let temperature_plot = PlotState::new("Temperatures", (Some(25.0), Some(35.0)), shared_plot.clone())
            .line("Baro. Temp. [°C]", C, |vs| vs.temperature_baro)
            .line("Core Temp. [°C]", B, |vs| vs.temperature_core);

        let power_plot = PlotState::new("Power", (Some(0.0), Some(9.0)), shared_plot.clone())
            .line("Arm Voltage [V]", O, |vs| vs.arm_voltage)
            .line("Battery Voltage [V]", G, |vs| vs.battery_voltage)
            .line("Current [A]", O1, |vs| vs.current)
            .line("Charge Voltage [V]", B, |vs| vs.charge_voltage)
            .line("Breakwire Open?", R, |vs| vs.breakwire_open.map(|bw| bw.then(|| 1.0).unwrap_or(0.0)));

        let runtime_plot = PlotState::new("Runtime", (Some(0.0), Some(100.0)), shared_plot.clone())
            .line("CPU Util. [%]", O, |vs| vs.cpu_utilization.map(|u| u as f32))
            .line("Heap Util. [%]", G, |vs| vs.heap_utilization.map(|u| u as f32));

        let signal_plot = PlotState::new("Signal", (Some(-100.0), Some(10.0)), shared_plot.clone())
            .line("GCS RSSI [dBm]", B, |vs| vs.gcs_lora_rssi.map(|x| x as f32 / -2.0))
            .line("GCS Signal RSSI [dBm]", B1, |vs| vs.gcs_lora_rssi_signal.map(|x| x as f32 / -2.0))
            .line("GCS SNR [dB]", C, |vs| vs.gcs_lora_snr.map(|x| x as f32 / 4.0))
            .line("Vehicle RSSI [dBm]", P, |vs| vs.vehicle_lora_rssi.map(|x| x as f32 / -2.0))
            .line("HDOP", R, |vs| vs.hdop.map(|x| x as f32 / 100.0))
            .line("# Satellites", G, |vs| vs.num_satellites.map(|x| x as f32));

        let map = MapState::new(settings.mapbox_access_token.clone());

        Self {
            maxi_grid_state: MaxiGridState::default(),
            dropdown_selected_plot: SelectedPlot::Orientation,
            shared_plot,
            orientation_plot,
            vertical_speed_plot,
            altitude_plot,
            gyroscope_plot,
            accelerometer_plot,
            magnetometer_plot,
            barometer_plot,
            temperature_plot,
            power_plot,
            runtime_plot,
            signal_plot,
            map,
        }
    }

    fn all_plots(&mut self, f: impl FnOnce(&mut PlotState) + Copy) {
        for plot in [
            &mut self.orientation_plot,
            &mut self.vertical_speed_plot,
            &mut self.altitude_plot,
            &mut self.gyroscope_plot,
            &mut self.accelerometer_plot,
            &mut self.magnetometer_plot,
            &mut self.barometer_plot,
            &mut self.temperature_plot,
            &mut self.power_plot,
            &mut self.runtime_plot,
            &mut self.signal_plot,
        ] {
            (f)(plot);
        }
    }

    pub fn reset(&mut self, keep_position: bool) {
        let now = Instant::now();
        self.all_plots(|plot| plot.reset(now, keep_position));
        self.map.reset();
    }

    pub fn show_all(&mut self) {
        self.all_plots(|plot| plot.show_all());
    }

    pub fn push_vehicle_state(&mut self, time: &Instant, vs: &VehicleState) {
        self.all_plots(|plot| plot.push(*time, vs));
        self.map.push(*time, vs);
    }

    fn plot_gizmo(
        &mut self,
        ui: &mut egui::Ui,
        viewport: Rect,
        orientation: UnitQuaternion<f32>,
        colors: (Color32, Color32, Color32)
    ) {
        // We can't disable interaction with the gizmo, so we disable the entire UI
        // when the user gets too close. TODO: upstream way to disable interaction?
        let enabled = !ui.rect_contains_pointer(viewport);

        // use top right of plot for indicator, space below for plot
        let viewport = Rect::from_two_pos(viewport.lerp_inside(Vec2::new(0.4, 0.55)), viewport.right_top());

        let fade_to_color = Color32::BLACK;
        ui.visuals_mut().widgets.noninteractive.weak_bg_fill = fade_to_color;

        // square viewport
        let viewport_square_side = f32::min(viewport.width(), viewport.height());
        let viewport = viewport.shrink2((viewport.size() - Vec2::splat(viewport_square_side))*0.5);

        let view = UnitQuaternion::from_euler_angles(-90.0f32.to_radians(), 180f32.to_radians(), 0.0f32.to_radians());

        let visuals = GizmoVisuals {
            x_color: colors.0,
            y_color: colors.1,
            z_color: colors.2,
            inactive_alpha: 1.0,
            highlight_alpha: 1.0,
            stroke_width: 3.0,
            gizmo_size: viewport_square_side * 0.4,
            ..Default::default()
        };

        let gizmo = Gizmo::new("My gizmo")
            .mode(GizmoMode::Translate)
            .viewport(viewport)
            .orientation(egui_gizmo::GizmoOrientation::Local)
            .model_matrix(orientation.to_homogeneous())
            .view_matrix(view.to_homogeneous())
            .visuals(visuals);

        ui.add_enabled_ui(enabled, |ui| {
            gizmo.interact(ui);
        });
    }

    fn plot_orientation(&mut self, ui: &mut egui::Ui, data_source: &mut dyn DataSource) {
        let mut viewport = ui.cursor();
        viewport.set_width(ui.available_width());
        viewport.set_height(ui.available_height());

        let orientation = data_source.vehicle_states()
            .rev()
            .find_map(|(_, vs)| vs.orientation)
            .unwrap_or(UnitQuaternion::new(Vector3::new(0.0, 0.0, 0.0)));
        let true_orientation = data_source.vehicle_states()
            .rev()
            .find_map(|(_, vs)| vs.true_orientation);

        ui.plot_telemetry(&self.orientation_plot);

        if let Some(orientation) = true_orientation {
            self.plot_gizmo(ui, viewport, orientation, (R1, G1, B1));
        }

        self.plot_gizmo(ui, viewport, orientation, (R, G, B));
    }

    pub fn main_ui(&mut self, ui: &mut egui::Ui, data_source: &mut dyn DataSource) {
        self.shared_plot.borrow_mut().set_end(data_source.end());

        if ui.available_width() > 1000.0 {
            MaxiGrid::new((4, 3), ui, self.maxi_grid_state.clone())
                .cell("Orientation", |ui| self.plot_orientation(ui, data_source))
                .cell("Vert. Speed & Accel", |ui| ui.plot_telemetry(&self.vertical_speed_plot))
                .cell("Altitude (ASL)", |ui| ui.plot_telemetry(&self.altitude_plot))
                .cell("Position", |ui| ui.map(&self.map))
                .cell("Gyroscope", |ui| ui.plot_telemetry(&self.gyroscope_plot))
                .cell("Accelerometers", |ui| ui.plot_telemetry(&self.accelerometer_plot))
                .cell("Magnetometer", |ui| ui.plot_telemetry(&self.magnetometer_plot))
                .cell("Pressures", |ui| ui.plot_telemetry(&self.barometer_plot))
                .cell("Temperature", |ui| ui.plot_telemetry(&self.temperature_plot))
                .cell("Power", |ui| ui.plot_telemetry(&self.power_plot))
                .cell("Runtime", |ui| ui.plot_telemetry(&self.runtime_plot))
                .cell("Signal", |ui| ui.plot_telemetry(&self.signal_plot));
        } else {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().combo_width = ui.available_width();
                    egui::ComboBox::from_id_source("plot_selector")
                        .selected_text(format!("{}", self.dropdown_selected_plot))
                        .show_ui(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for p in [
                                SelectedPlot::Orientation,
                                SelectedPlot::VerticalSpeed,
                                SelectedPlot::Altitude,
                                SelectedPlot::Gyroscope,
                                SelectedPlot::Accelerometers,
                                SelectedPlot::Magnetometer,
                                SelectedPlot::Pressures,
                                SelectedPlot::Temperatures,
                                SelectedPlot::Power,
                                SelectedPlot::Runtime,
                                SelectedPlot::Signal,
                                SelectedPlot::Map
                            ] {
                                ui.selectable_value(&mut self.dropdown_selected_plot, p, format!("{}", p));
                            }
                        });
                });

                match self.dropdown_selected_plot {
                    SelectedPlot::Orientation    => self.plot_orientation(ui, data_source),
                    SelectedPlot::VerticalSpeed  => ui.plot_telemetry(&self.vertical_speed_plot),
                    SelectedPlot::Altitude       => ui.plot_telemetry(&self.altitude_plot),
                    SelectedPlot::Gyroscope      => ui.plot_telemetry(&self.gyroscope_plot),
                    SelectedPlot::Accelerometers => ui.plot_telemetry(&self.accelerometer_plot),
                    SelectedPlot::Magnetometer   => ui.plot_telemetry(&self.magnetometer_plot),
                    SelectedPlot::Pressures      => ui.plot_telemetry(&self.barometer_plot),
                    SelectedPlot::Temperatures   => ui.plot_telemetry(&self.temperature_plot),
                    SelectedPlot::Power          => ui.plot_telemetry(&self.power_plot),
                    SelectedPlot::Runtime        => ui.plot_telemetry(&self.runtime_plot),
                    SelectedPlot::Signal         => ui.plot_telemetry(&self.signal_plot),
                    SelectedPlot::Map            => ui.map(&self.map),
                }
            });
        }
    }

    pub fn bottom_bar_ui(&mut self, ui: &mut egui::Ui, _data_source: &mut dyn DataSource) {
        ui.toggle_button(&mut self.shared_plot.borrow_mut().show_stats, "📈 Show Stats", "📉 Hide Stats");
    }

    pub fn apply_settings(&mut self, settings: &AppSettings) {
        self.map.set_access_token(settings.mapbox_access_token.clone());
    }
}
