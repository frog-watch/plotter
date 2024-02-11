#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use frogwatch_plotter::TimeSeriesPlot;
use std::io::{self, Read};
use std::{
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use frogwatch_plotter::FFTPlot;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let app = frogwatch_plotter::PlotApp::new();

    let timeseries_ref = app.timeseries.clone();
    let fft_ref = app.fft.clone();

    thread::spawn(move || {
        fn handle_client(
            mut stream: TcpStream,
            monitor: Arc<Mutex<TimeSeriesPlot>>,
        ) -> io::Result<()> {
            let mut buffer = [0; 2048]; // Buffer to store the data
            let mut parse_buffer = String::with_capacity(4096);

            let mut t = 0.0;

            // Read data from the stream into the buffer
            while let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read == 0 {
                    break; // Connection was closed
                }

                // Convert the bytes to a string and print
                let received_data = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

                parse_buffer += &received_data; // Append the received data to the parse_buffer

                let use_all = parse_buffer.ends_with("\n");

                let leftover = {
                    let mut lines: Vec<&str> = parse_buffer.lines().collect();

                    let leftover = if !use_all {
                        lines.pop().map(|l| l.to_string())
                    } else {
                        None
                    };

                    for line in lines {
                        let parts = line
                            .trim_end_matches('\n')
                            .trim_matches(|c| c == '[' || c == ']')
                            .split(',')
                            .collect::<Vec<&str>>();
                        // println!("Received: {:?}", parts);
                        for p in parts {
                            let x = match p.trim().parse::<f64>() {
                                Ok(value) => value,
                                _ => {
                                    log::warn!("Failed to parse {}", p);
                                    continue;
                                }
                            };

                            // if x.abs() > 1000.0 {
                            //     error!("Skipping large value {}", x);
                            //     continue;
                            // }

                            monitor.lock().unwrap().add(t, x);
                            t += 1.0;
                        }
                    }

                    leftover
                };

                parse_buffer.clear();
                if let Some(last_line) = leftover {
                    parse_buffer += last_line.as_str();
                }
            }

            Ok(())
        }

        // Step 1: Bind to a TCP port
        let listener = TcpListener::bind("127.0.0.1:12345").unwrap();
        println!("Listening on 127.0.0.1:12345");

        // Step 2: Listen for incoming connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    // Handle the client in a function
                    handle_client(stream, timeseries_ref.clone()).ok();
                }
                Err(e) => {
                    // Handle error in connection
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    thread::spawn(move || {
        fn handle_client(mut stream: TcpStream, monitor: Arc<Mutex<FFTPlot>>) -> io::Result<()> {
            let mut buffer = [0; 80920]; // Buffer to store the data
            let mut parse_buffer = String::with_capacity(80920);

            // Read data from the stream into the buffer
            while let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read == 0 {
                    break; // Connection was closed
                }

                // Convert the bytes to a string and print
                let received_data = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

                parse_buffer += &received_data; // Append the received data to the parse_buffer

                let use_all = parse_buffer.ends_with("\n");

                let leftover = {
                    let mut lines: Vec<&str> = parse_buffer.lines().collect();

                    let leftover = if !use_all {
                        lines.pop().map(|l| l.to_string())
                    } else {
                        None
                    };

                    for line in lines {
                        let mut vec: Vec<f64> = Vec::with_capacity(1000);
                        let parts = line
                            .trim_end_matches('\n')
                            .trim_matches(|c| c == '[' || c == ']')
                            .split(',')
                            .collect::<Vec<&str>>();
                        // println!("Received: {:?}", parts);
                        for p in parts {
                            let x = match p.trim().parse::<f64>() {
                                Ok(value) => value,
                                _ => {
                                    log::warn!("Failed to parse {}", p);
                                    continue;
                                }
                            };

                            vec.push(x);
                        }
                        monitor.lock().unwrap().add(vec);
                    }

                    leftover
                };

                parse_buffer.clear();
                if let Some(last_line) = leftover {
                    parse_buffer += last_line.as_str();
                }
            }

            Ok(())
        }

        // Step 1: Bind to a TCP port
        let listener = TcpListener::bind("127.0.0.1:12346").unwrap();
        println!("Listening on 127.0.0.1:12346");

        // Step 2: Listen for incoming connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    // Handle the client in a function
                    handle_client(stream, fft_ref.clone()).ok();
                }
                Err(e) => {
                    // Handle error in connection
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    });

    // let native_options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default()
    //         .with_inner_size([1000.0, 600.0])
    //         .with_min_inner_size([800.0, 400.0])
    //         .with_icon(
    //             // NOE: Adding an icon is optional
    //             eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
    //                 .unwrap(),
    //         ),
    //     ..Default::default()
    // };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Frogwatch Plotter",
        native_options,
        Box::new(|_cc| Box::new(app)),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(frogwatch_plotter::TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
