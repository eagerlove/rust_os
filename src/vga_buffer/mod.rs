/* Global println function implementation */

// special character
mod codepage437;

use core::fmt; // support formatting macros
use spin::Mutex; // set spinlock to imeplement safety interior mutability
use lazy_static::lazy_static; // lazily initalized variable 
use volatile::Volatile;// prevent compiler optimization of reading or writing buffer


// color enum
#[allow(dead_code)] // disable unused warning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
	Black = 0,
	Blue = 1,
	Green = 2,
	Cyan = 3,
	Red = 4,
	Magenta = 5,
	Brown = 6,
	LightGray = 7,
	DarkGray = 8,
	LightBlue = 9,
	LightGreen = 10,
	LightCyan = 11,
	LightRed = 12,
	Pink = 13,
	Yellow = 14,
	White = 15,
}

// character color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
	fn new(foreground: Color, background: Color) -> ColorCode {
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}

// character
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
	ascii_character: u8,
	color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// buffer struct
struct Buffer {
	chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// write character to screen
pub struct Writer {
	column_position: usize, // cursor position
	color_code: ColorCode, // color
	buffer: &'static mut Buffer, // 'static lifetime
}

impl Writer {
	// print character
	pub fn write_char(&mut self, character:char) {
		match character {
			'\n' => self.new_line(),
			'\t' => while self.column_position % 8 != 0 {
				self.write_byte(b' ');
			},
			_ => {
				// unkonwn character
				let byte = codepage437::encode(character).unwrap_or(6);
				self.write_byte(byte)
			},
		}
	}

	// print byte
	pub fn write_byte(&mut self, byte: u8) {
		// auto linefeed
		if self.column_position >= BUFFER_WIDTH {
			self.new_line();
		}

		let row = BUFFER_HEIGHT - 1;
		let col = self.column_position;

		let color_code = self.color_code;
		self.buffer.chars[row][col].write(ScreenChar {
			ascii_character: byte,
			color_code,
		});
		self.column_position += 1;

	}

	// print string
	pub fn write_string(&mut self, s: &str) {
		for c in s.chars() {
			self.write_char(c);
		}
	}

	// create a new row
	fn new_line(&mut self) {
		for row in 1..BUFFER_HEIGHT {
			for col in 0..BUFFER_WIDTH {
				let character = self.buffer.chars[row][col].read();
				self.buffer.chars[row - 1][col].write(character);
			}
		}
		self.clear_row(BUFFER_HEIGHT - 1);
		self.column_position = 0;
	}
	// delete whole row
	fn clear_row(&mut self, row: usize) {
		let blank = ScreenChar {
			ascii_character: b' ', // write space to clear character
			color_code: self.color_code,
		};
		for col in 0..BUFFER_WIDTH {
			self.buffer.chars[row][col].write(blank)
		}
	}
}

impl fmt::Write for Writer {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.write_string(s);
		Ok(()) // fmt::Result emum type == return 0
	}
}

// static global print interface: Write
lazy_static! {
	pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
		column_position: 0,
		color_code: ColorCode::new(Color::Green, Color::Black),
		buffer: unsafe { &mut *(0xb8000 as *mut Buffer)}, // create a buffer reference pointing to 0xb8000
	});
}

#[macro_export] // make macro available to the whole crate
macro_rules! print {
	($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export] // make macro available to the whole crate
macro_rules! println {
	() => (print!("\n"));
	($fmt:expr) => (print!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)] // hide the details from the generated documentation
pub fn _print(args: fmt::Arguments) {
	use core::fmt::Write;
	use x86_64::instructions::interrupts;

	// ensure that no interrupt can occur as long as the Mutex is locked
	interrupts::without_interrupts( || {
		WRITER.lock().write_fmt(args).unwrap()
	});
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
	use core::fmt::Write;
	use x86_64::instructions::interrupts; 
	let s = "Some test string that fits on a single line";
	// disable interrupts and lock writer to avoid race condition
	interrupts::without_interrupts(|| {
		let mut writer = WRITER.lock();
		writeln!(writer, "\n{}", s).expect("writeln failed");
		for (i, c) in s.chars().enumerate() {
		// BUFFER_HEIGHT - 2: last screen line will append a newline 
		let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
		assert_eq!(char::from(screen_char.ascii_character), c);
		}
	});

}