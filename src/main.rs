#[macro_use]
extern crate conrod;
use conrod::{widget, Borderable, Colorable, Positionable, Sizeable, Widget};
use conrod::backend::glium::glium;
use conrod::backend::glium::glium::{DisplayBuild, Surface};


struct AppState {
	board: [[bool; 8]; 8],
}

impl AppState {
	fn new() -> AppState {
		AppState {
			board: [ [true, true, true, true, true, true, true, true],
						   [true, false, false, false, false, false, false, true],
						   [true, false, true, false, true, true, true, true],
						   [true, false, true, false, true, true, true, true],
						   [true, false, false, false, true, true, true, true],
						   [true, true, true, true, true, true, true, true],
						   [true, true, false, true, false, false, false, true],
						   [true, true, true, true, true, true, true, true] ],
		}
	}
}


fn main() {
	const WIDTH: u32 = 400;
	const HEIGHT: u32 = 400;
	
	let mut app_state = AppState::new();

	// Build the window.
	let display = glium::glutin::WindowBuilder::new()
		.with_vsync()
		.with_dimensions(WIDTH, HEIGHT)
		.with_title("Game of Life")
		.with_multisampling(4)
		.build_glium()
		.unwrap();

	// construct our `Ui`.
	let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

	// Generate the widget identifiers.
	widget_ids!(struct Ids { 
		title, 
		info,
		board
	});
	let ids = Ids::new(ui.widget_id_generator());

	// Add a `Font` to the `Ui`'s `font::Map` from file.
	const FONT_PATH: &'static str =
		concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/NotoSans/NotoSans-Regular.ttf");
	ui.fonts.insert_from_file(FONT_PATH).unwrap();

	// A type used for converting `conrod::render::Primitives` into `Command`s that can be used
	// for drawing to the glium `Surface`.
	let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

	// The image map describing each of our widget->image mappings (in our case, none).
	let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

	// Poll events from the window.
	let mut last_update = std::time::Instant::now();
	let mut ui_needs_update = true;
	'main: loop {

		// We don't want to loop any faster than 60 FPS, so wait until it has been at least
		// 16ms since the last yield.
		let sixteen_ms = std::time::Duration::from_millis(16);
		let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
		if duration_since_last_update < sixteen_ms {
			std::thread::sleep(sixteen_ms - duration_since_last_update);
		}

		// Collect all pending events.
		let mut events: Vec<_> = display.poll_events().collect();

		// If there are no events and the `Ui` does not need updating, wait for the next event.
		if events.is_empty() && !ui_needs_update {
			events.extend(display.wait_events().next());
		}

		// Reset the needs_update flag and time this update.
		ui_needs_update = false;
		last_update = std::time::Instant::now();

		// Handle all events.
		for event in events {

			// Use the `winit` backend feature to convert the winit event to a conrod one.
			if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
				ui.handle_event(event);
				ui_needs_update = true;
			}

			match event {
				// Break from the loop upon `Escape`.
				glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
				glium::glutin::Event::Closed =>
					break 'main,
				_ => {},
			}
		}

		// Instantiate all widgets in the GUI.
		{
			let ui = &mut ui.set_widgets();

			// "Hello World!" in the middle of the screen.
			widget::Text::new("Game of Life")
				.mid_top_of(ui.window)
				.color(conrod::color::WHITE)
				.font_size(32)
				.set(ids.title, ui);
			widget::Text::new("Start")
				.down(5.0)
				.color(conrod::color::WHITE)
				.font_size(8)
				.set(ids.info, ui);

			// A demonstration using widget_matrix to easily draw a matrix of any kind of widget.
			let (cols, rows) = (8, 8);
			let mut elements = widget::Matrix::new(cols, rows)
				.down(20.0)
				.w_h(260.0, 260.0)
				.set(ids.board, ui);

			// The `Matrix` widget returns an `Elements`, which can be used similar to an `Iterator`.
			while let Some(elem) = elements.next(ui) {
				let (col, row) = (elem.col, elem.row);

				// Color effect for fun.
				let (r, g, b, a) = (
					0.5 + (elem.col as f32 / cols as f32) / 2.0,
					0.75,
					1.0 - (elem.row as f32 / rows as f32) / 2.0,
					1.0
				);

				// We can use `Element`s to instantiate any kind of widget we like.
				// The `Element` does all of the positioning and sizing work for us.
				// Here, we use the `Element` to `set` a `Toggle` widget for us.
				let toggle = widget::Toggle::new(app_state.board[col][row])
					.rgba(r, g, b, a)
					.border(1.0);
				if let Some(new_value) = elem.set(toggle, ui).last() {
					app_state.board[col][row] = new_value;
				}
			}

			
		}

		// Render the `Ui` and then display it on the screen.
		if let Some(primitives) = ui.draw_if_changed() {
			renderer.fill(&display, primitives, &image_map);
			let mut target = display.draw();
			target.clear_color(0.0, 0.0, 0.0, 1.0);
			renderer.draw(&display, &mut target, &image_map).unwrap();
			target.finish().unwrap();
		}
	}
}
