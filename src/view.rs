use std::io;

use chrono::Timelike;

use crate::font;
use crate::term;
use crate::time;

const RESET: term::Paint = term::Paint {
    color: term::Color::Reset,
    ground: term::Ground::Back,
};

//  H       :   M       :   S
// ...|...|...|...|...|...|...|...
// ...|...|...|...|...|...|...|...
// ...|...|...|...|...|...|...|...
// ...|...|...|...|...|...|...|...
// ...|...|...|...|...|...|...|...
//
//           ....-..-..
//           Y    M  S
#[derive(Clone, Debug)]
pub struct Clock<'tz> {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    date: time::Date<'tz>,
    time: time::Time,
    zone: &'tz str,
    color: term::Paint,
    second: bool,
    military: bool,
}

impl<'tz> Clock<'tz> {

    pub fn start(
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        zone: &'tz str,
        color: term::Color,
        second: bool,
        military: bool,
    ) -> io::Result<Self> {
        Ok(Clock {
            x, y,
            w, h,
            date: time::Date::default(),
            time: time::Time::blank(second, military),
            zone,
            color: term::Paint { color, ground: term::Ground::Back },
            second,
            military,
        })
    }

    pub fn toggle_second(&mut self) {
        self.second = !self.second;
    }

    pub fn toggle_military(&mut self) {
        self.military = !self.military;
    }

    pub fn set_color(&mut self, color: term::Color) {
        self.color = term::Paint { color, ground: term::Ground::Back };
    }

    pub fn center(&mut self, (w, h): (u16, u16)) {
        self.x = w / 2 - self.width() / 2;
        self.y = h / 2 - self.height() / 2;
    }

    pub fn reset<W: io::Write>(&mut self, mut out: W) -> io::Result<()> {
        self.date = time::Date::default();
        self.time = time::Time::blank(self.second, self.military);
        write!(out, "{}{}", RESET, term::CLEAR)
    }

    /// Best effort real-time synchronization.
    pub fn sync(&self) {
        let start = chrono::Local::now().nanosecond() as u64;
        let delay = std::time::Duration::from_nanos(1_000_000_000 - start);
        std::thread::sleep(delay);
    }

    pub fn draw<W: io::Write>(&mut self, mut out: W) -> io::Result<()> {

        let (date, time) = time::now(&self.zone, self.second, self.military);
        let draw = self.time ^ time;

        for digit in 0..self.digits() {

            let dx = self.x + ((font::W + 1) * self.w * digit as u16);
            let dy = self.y;

            let mut mask = 0b1_000_000_000_000_000u16;

            for i in 0..15 {
                mask >>= 1; if draw[digit] & mask == 0 { continue }
                let color = if time[digit] & mask > 0 { self.color } else { RESET };
                let width = self.w as usize;
                let x = i % font::W * self.w + dx;
                let y = i / font::W * self.h + dy;
                for j in 0..self.h {
                    let goto = term::Move(x, y + j);
                    write!(out, "{}{}{:3$}", color, goto, " ", width)?;
                }
            }
        }

        if date != self.date {
            let date_x = self.x + self.width() / 2 - date.width() / 2;
            let date_y = self.y + self.height() + 1;
            let goto = term::Move(date_x, date_y);
            write!(out, "{}{}{}", RESET, goto, date)?;
        }

        out.flush()?;
        self.date = date;
        self.time = time;
        Ok(())
    }

    fn digits(&self) -> usize {
        time::Time::width(self.second, self.military)
    }

    pub fn width(&self) -> u16 {
        (self.w * (font::W + 1)) * self.digits() as u16 - 1
    }

    pub fn height(&self) -> u16 {
        (self.h * font::H)
    }
}

