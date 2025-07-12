use eframe::{egui, HardwareAcceleration};
use eframe::egui::Theme;
use egui_plot::{Legend, Line, Plot, PlotBounds, PlotPoint, PlotPoints, Points, Arrows};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use eframe::egui::{Vec2b, ViewportBuilder};
use bjr_telemetry::{TelemetryData, TelemetryPacket};
use strum::{EnumCount, VariantNames};
use rfd::FileDialog;
use csv::Writer;
use std::fs::File;
use std::io;

const MAX_POINTS: usize = 1000;

struct TelemetryHolder {
    pub data: VecDeque<TelemetryPacket>,
}

impl TelemetryHolder {
    fn new() -> Self {
        TelemetryHolder {
            data: VecDeque::with_capacity(MAX_POINTS)
        }
    }

    fn add_packet(&mut self, packet: TelemetryPacket) {
        if self.data.len() == MAX_POINTS {
            self.data.pop_front();
        }
        self.data.push_back(packet)
    }
}

struct TelemetryApp {
    telemetry_data: Arc<Mutex<TelemetryHolder>>,
    lines: Vec<Vec<PlotPoint>>,
    line_visibility: [bool;TelemetryData::COUNT],
    reset_view: bool,
    update_view: bool,
    reset_ball_view: bool,
}
fn get_ball_position_and_velocity(telemetry_data: &TelemetryHolder) -> Option<(PlotPoint, PlotPoint)> {
    // Get the latest packet
    let latest_packet = telemetry_data.data.back()?;

    // Find indices for BallXPos, BallYPos, BallXVel, BallYVel
    let ball_x_pos_idx = TelemetryData::VARIANTS.iter().position(|&name| name == "BallXPos")?;
    let ball_y_pos_idx = TelemetryData::VARIANTS.iter().position(|&name| name == "BallYPos")?;
    let ball_x_vel_idx = TelemetryData::VARIANTS.iter().position(|&name| name == "BallXVel")?;
    let ball_y_vel_idx = TelemetryData::VARIANTS.iter().position(|&name| name == "BallYVel")?;

    // Extract values
    let ball_x_pos = latest_packet.data[ball_x_pos_idx]?.get_printable_value();
    let ball_y_pos = latest_packet.data[ball_y_pos_idx]?.get_printable_value();
    let ball_x_vel = latest_packet.data[ball_x_vel_idx]?.get_printable_value();
    let ball_y_vel = latest_packet.data[ball_y_vel_idx]?.get_printable_value();

    Some((
        PlotPoint::new(ball_x_pos, ball_y_pos),
        PlotPoint::new(ball_x_vel, ball_y_vel)
    ))
}

impl TelemetryApp {
    fn new(telemetry_data: Arc<Mutex<TelemetryHolder>>) -> Self {
        Self {
            telemetry_data,
            line_visibility: [true;TelemetryData::COUNT],
            reset_view: false,
            update_view: true,
            reset_ball_view: false,
            lines: std::iter::repeat(Vec::with_capacity(MAX_POINTS)).take(TelemetryData::COUNT).collect(),
        }
    }

    fn get_update_button_text(&self) -> &'static str {
        if self.update_view {
            "Pause"
        } else {
            "Run"
        }
    }

    fn update_button_clicked(&mut self){
        if self.update_view {
            self.update_view = false;
        } else {
            self.update_view = true;
        }
    }

    fn flush_buffer(&mut self){
        let mut telemetry_data = self.telemetry_data.lock().unwrap();
        telemetry_data.data.clear();
    }

    fn export_data(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("CSV file", &["csv"])
            .set_file_name("data_export.csv")
            .save_file()
        {
            // Open file
            let file = File::create(path).unwrap();
            let mut wtr = Writer::from_writer(file);

            // Write header
            let mut record:[String;27] = [const{String::new()};27];
            record[0] = "Packet ID".to_string();
            record[1] = "Timestamp [s]".to_string();
            let mut idx = 2;
            for field_name in TelemetryData::VARIANTS {
                record[idx]=field_name.to_string();
                idx+=1;
            }
            wtr.write_record(&record).unwrap();

            // Write data rows
            let telemetry_data = self.telemetry_data.lock().unwrap();
            for packet in &telemetry_data.data {
                let mut record:[String;27] = [const{String::new()};27];
                record[0] = packet.packet_id.to_string();
                record[1] = packet.timestamp.to_string();
                let mut idx = 2;
                for data in packet.data{
                    if let Some(data) = data {
                        record[idx]=data.get_printable_value().to_string();
                    }
                    idx+=1;
                }
                wtr.write_record(&record).unwrap();
            }
            // Flush writer
            wtr.flush().unwrap();
        }
    }

}

impl eframe::App for TelemetryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("checkbox_panel").show(ctx, |ui| {
                ui.heading("Line Visibility");
                ui.add_space(10.0);

                for (i, visible) in self.line_visibility.iter_mut().enumerate() {
                    ui.checkbox(visible, TelemetryData::VARIANTS[i]);
                }
                ui.add_space(20.0);

                if ui.button("Reset view").clicked() {
                    self.reset_view = true;
                }

                if ui.button("Reset ball view").clicked() {
                    self.reset_ball_view = true;
                }

                if ui.button(self.get_update_button_text()).clicked() {
                    self.update_button_clicked();
                }
                if ui.button("Flush").clicked() {
                    self.flush_buffer();
                }
                if ui.button("Export").clicked() {
                    self.export_data();
                }
            });

            let telemetry_data = self.telemetry_data.lock().unwrap();

            // Create a vertical layout for the two plots
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical(|ui| {
                    // First plot - Time series data
                    let max_time_plot_height = 400.0;
                    let time_plot_height = (ui.available_height() * 0.5).min(max_time_plot_height);
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), time_plot_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            Plot::new("telemetry_plot")
                                .legend(Legend::default())
                                .show(ui, |plot_ui| {
                                    if self.update_view {
                                        self.lines = std::iter::repeat(Vec::with_capacity(crate::MAX_POINTS)).take(TelemetryData::COUNT).collect();
                                        for packet in &telemetry_data.data {
                                            let x = packet.timestamp as f64 / 1000000.0;
                                            for idx in 0..TelemetryData::COUNT {
                                                if self.line_visibility[idx] {
                                                    if let Some(data) = packet.data[idx] {
                                                        self.lines[idx].push(PlotPoint { x, y: data.get_printable_value() })
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    for (idx, line) in self.lines.iter().enumerate() {
                                        if self.line_visibility[idx] {
                                            plot_ui.line(Line::new("", PlotPoints::Owned(line.clone()) ).name(TelemetryData::VARIANTS[idx]));
                                        }
                                    }
                                    if self.reset_view{
                                        plot_ui.set_auto_bounds(Vec2b::new(true,true));
                                        self.reset_view = false;
                                    }
                                });
                        }
                    );

                    ui.separator();

                    let available_size = ui.available_size();
                    let square_size = available_size.x.min(available_size.y);
                    // Second plot - Ball position and velocity
                    ui.allocate_ui_with_layout(
                        egui::vec2(square_size, square_size),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.label("Ball Position & Velocity");
                            Plot::new("ball_plot")
                                .legend(Legend::default())
                                .view_aspect(1.0)
                                .data_aspect(1.0)
                                .show(ui, |plot_ui| {
                                    if let Some((position, velocity)) = get_ball_position_and_velocity(&telemetry_data) {
                                        // Draw the ball position as a point
                                        plot_ui.points(
                                            Points::new("Ball Position", PlotPoints::Owned(vec![position]))
                                                .radius(5.0)
                                                .color(egui::Color32::RED)
                                        );

                                        // Draw the velocity vector as an arrow
                                        let arrow_end = PlotPoint::new(
                                            position.x + velocity.x * 0.1, // Scale the velocity vector for visibility
                                            position.y + velocity.y * 0.1
                                        );

                                        plot_ui.arrows(
                                            Arrows::new("", PlotPoints::Owned(vec![position]), PlotPoints::Owned(vec![arrow_end]))
                                                .name("Ball Velocity")
                                                .color(egui::Color32::BLUE)
                                        );

                                        // Draw a trajectory trail (last few positions)
                                        let trail_points: Vec<PlotPoint> = telemetry_data.data
                                            .iter()
                                            .rev()
                                            .take(50) // Show last 50 positions
                                            .filter_map(|packet| {
                                                let ball_x_pos_idx = TelemetryData::VARIANTS.iter().position(|&name| name == "BallXPos")?;
                                                let ball_y_pos_idx = TelemetryData::VARIANTS.iter().position(|&name| name == "BallYPos")?;
                                                let x = packet.data[ball_x_pos_idx]?.get_printable_value();
                                                let y = packet.data[ball_y_pos_idx]?.get_printable_value();
                                                Some(PlotPoint::new(x, y))
                                            })
                                            .collect();

                                        if !trail_points.is_empty() {
                                            plot_ui.line(
                                                Line::new("", PlotPoints::Owned(trail_points))
                                                    .name("Ball Trail")
                                                    .color(egui::Color32::GRAY)
                                                    .width(1.0)
                                            );
                                        }
                                    }

                                    if self.reset_ball_view {
                                        plot_ui.set_auto_bounds(Vec2b::new(true, true));
                                        self.reset_ball_view = false;
                                    }
                                });
                        }
                    );
                });
            });
        });

        // Request a repaint
        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let telemetry_data = Arc::new(Mutex::new(TelemetryHolder::new()));
    let telemetry_data_clone = Arc::clone(&telemetry_data);

    // Spawn a thread to handle TCP connections
    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        for mut maybe_stream in listener.incoming() {
            let mut stream = maybe_stream.unwrap();
            loop {
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        if let Ok(packet) = postcard::from_bytes::<TelemetryPacket>(&buffer[..n]) {
                            let mut data = telemetry_data_clone.lock().unwrap();
                            data.add_packet(packet);
                        }
                    }
                    Err(_) => break, // Error occurred, break the loop
                }
            }
        }
        // close the socket server
        drop(listener);
    });

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(egui::vec2(800.0, 800.0)), // Made taller for two plots
        vsync: false,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: HardwareAcceleration::Required,
        renderer: Default::default(),
        run_and_return: false,
        event_loop_builder: None,
        window_builder: None,
        shader_version: None,
        centered: false,
        persist_window: false,
        persistence_path: None,
        dithering: false,
    };

    eframe::run_native(
        "Telemetry Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(TelemetryApp::new(telemetry_data)))),
    )
        .map_err(|e| e.into())
}