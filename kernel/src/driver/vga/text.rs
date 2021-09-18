use core::fmt;

pub struct Text {
	row: u8,
	column: u8,
}

impl Text {
	const WIDTH: u8 = 80;
	const HEIGHT: u8 = 24;
	const BUFFER: *mut u16 = 0x0b8000 as *mut u16;

	pub fn new() -> Self {
		Self { row: 0, column: 0 }
	}

	unsafe fn write_byte(&mut self, b: u8, fg: u8, bg: u8, x: u8, y: u8) {
		let i = isize::from(Self::WIDTH) * isize::from(y) + isize::from(x);
		let v = u16::from(b) | (u16::from(fg & 0xf | bg << 4) << 8);
		core::ptr::write_volatile(Self::BUFFER.offset(i), v);
	}

	fn put_byte(&mut self, b: u8, fg: u8, bg: u8) {
		if b == b'\n' {
			self.column = 0;
			self.row += 1;
		} else {
			// SAFETY: x and y are in range
			unsafe {
				self.write_byte(b, fg, bg, self.column, self.row);
			}
			self.column += 1;
			if self.column >= Self::WIDTH {
				self.column = 0;
				self.row += 1;
			}
		}
		if self.row >= Self::HEIGHT {
			self.row = 0;
		}
	}

	pub fn write_str(&mut self, s: &[u8], fg: u8, bg: u8) {
		for b in s.iter().copied() {
			self.put_byte(b, fg, bg);
		}
	}

	pub fn write_num(&mut self, mut n: i128, base: u8, fg: u8, bg: u8) -> Result<(), InvalidBase> {
		// Implementation stolen from https://stackoverflow.com/a/23840699/7327379
		(2 <= base && base < 36).then(|| ()).ok_or(InvalidBase)?;

		let mut t;

		let mut buf = [0; 128];
		let mut i = 0;

		while {
			let base = i128::from(base);
			t = n;
			n /= base;
			let d = (35 + (t - n * base)) as usize;
			buf[i] = b"zyxwvutsrqponmlkjihgfedcba9876543210123456789abcdefghijklmnopqrstuvwxyz"[d];
			i += 1;
			n != 0
		} {}

		if t < 0 {
			buf[i] = b'-';
			i += 1;
		}

		for b in buf[..i].iter().rev().copied() {
			self.put_byte(b, fg, bg);
		}

		Ok(())
	}
}

impl fmt::Write for Text {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.write_str(s.as_bytes(), 0xf, 0x0);
		Ok(())
	}
}

pub struct InvalidBase;
