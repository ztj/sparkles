#![deny(
    rust_2018_idioms,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    future_incompatible,
    clippy::pedantic
)]

use minifb::{Key, Window, WindowOptions};
use rand::seq::SliceRandom;

mod sparkles {
    #[derive(Default, Debug)]
    pub enum Sparkle {
        Bright,
        #[default]
        Dim,
    }

    #[derive(Default)]
    pub struct Sparkles {
        lights: [[std::sync::Mutex<bool>; 8]; 8],
    }

    impl Sparkles {
        pub fn display(&self, mut update: impl FnMut(usize, usize, Sparkle)) {
            for (x, row) in self.lights.iter().enumerate() {
                for (y, light) in row.iter().enumerate() {
                    let sparkle = if *light.lock().unwrap() {
                        Sparkle::Bright
                    } else {
                        Sparkle::Dim
                    };
                    update(x, y, sparkle);
                }
            }
        }

        pub fn toggle(&self, x: usize, y: usize) {
            let mut light = self.lights[x][y].lock().unwrap();
            *light = !*light;
        }
    }
}

fn main() {
    // Create our sparkle board
    let sparkles = std::sync::Arc::new(sparkles::Sparkles::default());

    // Start a thread that will continuously alter the state of the board at random
    {
        let sparkles = sparkles.clone();
        std::thread::spawn(move || loop {
            let x = rand::random::<usize>() % 8;
            let y = rand::random::<usize>() % 8;
            sparkles.toggle(x, y);
            let delay = std::time::Duration::from_millis(rand::random::<u64>() % 950 + 50);
            std::thread::sleep(delay);
        });
    }

    // Create a window to display the board
    let mut buffer: Vec<u32> = vec![0; 8 * 8];
    let mut window = Window::new(
        "Sparkles",
        8,
        8,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X32,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to open Window");

    // Display loop for the board
    loop {
        sparkles.display(|x, y, sparkle| {
            // Choosing random bright colors to help visually demonstrate the update frequency
            static BRIGHT_COLORS: &[u32] = &[0x00_00_FF, 0x00_FF_00, 0xFF_00_00];
            let bright_color = *(BRIGHT_COLORS.choose(&mut rand::thread_rng()).unwrap());
            let color = match sparkle {
                sparkles::Sparkle::Bright => bright_color,
                sparkles::Sparkle::Dim => 0x00_00_00,
            };
            buffer[x * 8 + y] = color;
        });
        window
            .update_with_buffer(&buffer, 8, 8)
            .expect("Unable to update Window");
        if window.is_key_down(Key::Escape) || !window.is_open() {
            std::process::exit(0);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
