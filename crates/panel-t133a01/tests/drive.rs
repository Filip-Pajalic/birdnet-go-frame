//! The driver against a mock bus that records every byte, pin change and delay.

#![allow(clippy::expect_used)] // Tests may panic on failed assumptions.

use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{self, InputPin, OutputPin};
use embedded_hal::spi::{self, SpiBus};
use panel_t133a01::{PanelError, Pins, T133a01};

/// One write on the bus: the chips selected, whether DC said data, and the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Write {
    cs0: bool,
    cs1: bool,
    data: bool,
    bytes: Vec<u8>,
}

/// The shared state of the mock board.
struct Board {
    cs0_low: bool,
    cs1_low: bool,
    dc_high: bool,
    rst_low_ever: bool,
    writes: Vec<Write>,
    /// Milliseconds of delay asked for so far.
    clock_ms: u64,
    /// When the last refresh command went out, by the clock.
    refresh_at: Option<u64>,
    /// How BUSY answers, from the milliseconds since the refresh command (or `None` before).
    busy_low: fn(Option<u64>) -> bool,
}

type Shared = Rc<RefCell<Board>>;

struct Bus(Shared);
struct Pin(Shared, &'static str);
struct Busy(Shared);
struct Delay(Shared);

impl spi::ErrorType for Bus {
    type Error = Infallible;
}

impl SpiBus for Bus {
    fn read(&mut self, _words: &mut [u8]) -> Result<(), Infallible> {
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Infallible> {
        let mut board = self.0.borrow_mut();
        let write = Write {
            cs0: board.cs0_low,
            cs1: board.cs1_low,
            data: board.dc_high,
            bytes: words.to_vec(),
        };
        if !write.data && write.bytes == [0x12] {
            board.refresh_at = Some(board.clock_ms);
        }
        board.writes.push(write);
        Ok(())
    }

    fn transfer(&mut self, _read: &mut [u8], _write: &[u8]) -> Result<(), Infallible> {
        Ok(())
    }

    fn transfer_in_place(&mut self, _words: &mut [u8]) -> Result<(), Infallible> {
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
}

impl digital::ErrorType for Pin {
    type Error = Infallible;
}

impl OutputPin for Pin {
    fn set_low(&mut self) -> Result<(), Infallible> {
        self.set(false);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Infallible> {
        self.set(true);
        Ok(())
    }
}

impl Pin {
    fn set(&self, high: bool) {
        let mut board = self.0.borrow_mut();
        match self.1 {
            "cs0" => board.cs0_low = !high,
            "cs1" => board.cs1_low = !high,
            "dc" => board.dc_high = high,
            _ => board.rst_low_ever |= !high,
        }
    }
}

impl digital::ErrorType for Busy {
    type Error = Infallible;
}

impl InputPin for Busy {
    fn is_high(&mut self) -> Result<bool, Infallible> {
        self.is_low().map(|low| !low)
    }

    fn is_low(&mut self) -> Result<bool, Infallible> {
        let board = self.0.borrow();
        let since: Option<u64> = board.refresh_at.map(|at| board.clock_ms - at);
        Ok((board.busy_low)(since))
    }
}

impl DelayNs for Delay {
    fn delay_ns(&mut self, nanoseconds: u32) {
        self.0.borrow_mut().clock_ms += u64::from(nanoseconds) / 1_000_000;
    }

    fn delay_ms(&mut self, milliseconds: u32) {
        self.0.borrow_mut().clock_ms += u64::from(milliseconds);
    }
}

/// A healthy panel: ready at once, then busy for 28 s from 3 ms after a refresh command.
fn healthy(since: Option<u64>) -> bool {
    since.is_some_and(|ms| (3..28_003).contains(&ms))
}

fn panel(busy_low: fn(Option<u64>) -> bool) -> (T133a01<Bus, Pin, Busy, Delay>, Shared) {
    let board: Shared = Rc::new(RefCell::new(Board {
        cs0_low: false,
        cs1_low: false,
        dc_high: false,
        rst_low_ever: false,
        writes: Vec::new(),
        clock_ms: 0,
        refresh_at: None,
        busy_low,
    }));
    let pins = Pins {
        cs0: Pin(Rc::clone(&board), "cs0"),
        cs1: Pin(Rc::clone(&board), "cs1"),
        dc: Pin(Rc::clone(&board), "dc"),
        rst: Pin(Rc::clone(&board), "rst"),
        busy: Busy(Rc::clone(&board)),
    };
    let driver = T133a01::new(Bus(Rc::clone(&board)), pins, Delay(Rc::clone(&board)));
    (driver, board)
}

/// The commands sent, with the chips that took each and the length of its data.
fn commands(writes: &[Write]) -> Vec<(u8, bool, bool, usize)> {
    let mut out: Vec<(u8, bool, bool, usize)> = Vec::new();
    for write in writes {
        if write.data {
            if let Some(last) = out.last_mut() {
                last.3 += write.bytes.len();
            }
        } else {
            out.push((write.bytes[0], write.cs0, write.cs1, 0));
        }
    }
    out
}

#[test]
fn wakes_with_fuglerammes_sequence() {
    let (mut driver, board) = panel(healthy);
    driver.wake().expect("wakes");
    let board = board.borrow();
    assert!(board.rst_low_ever, "reset pulsed");
    let sent = commands(&board.writes);
    assert_eq!(sent.len(), 16 + 2);
    // The first is for the first controller only, with nine bytes of data.
    assert_eq!(sent[0], (0x74, true, false, 9));
    // The resolution goes to both.
    assert!(sent.contains(&(0x61, true, true, 4)));
    // Then the PLL and power on, to both.
    assert_eq!(sent[16], (0x30, true, true, 1));
    assert_eq!(sent[17], (0x04, true, true, 0));
    // Every chip select is released afterwards.
    assert!(!board.cs0_low && !board.cs1_low && !board.dc_high);
}

#[test]
fn draws_each_half_to_its_own_controller_and_refreshes() {
    let (mut driver, board) = panel(healthy);
    let mut page: Vec<u8> = vec![0x11; T133a01::<Bus, Pin, Busy, Delay>::PAGE_LEN];
    page[480_000..].fill(0x33);
    driver.wake().expect("wakes");
    board.borrow_mut().writes.clear();
    let took: u32 = driver.show(&page).expect("shows");
    assert!((27_000..29_000).contains(&took), "{took} ms");
    let board = board.borrow();
    let sent = commands(&board.writes);
    assert_eq!(
        sent,
        vec![
            (0xe0, true, true, 1),
            (0x10, true, false, 480_000),
            (0xe0, true, true, 1),
            (0x10, false, true, 480_000),
            (0x12, true, true, 1),
        ]
    );
    let left: Vec<u8> = board
        .writes
        .iter()
        .filter(|write| write.data && write.cs0 && !write.cs1)
        .flat_map(|write| write.bytes.clone())
        .collect();
    assert!(left.iter().all(|&byte| byte == 0x11));
    let right_chunks: Vec<&Write> = board
        .writes
        .iter()
        .filter(|write| write.data && write.cs1 && !write.cs0)
        .collect();
    assert!(right_chunks.iter().all(|write| write.bytes.len() <= 19_200));
    assert!(
        right_chunks
            .iter()
            .all(|write| write.bytes.iter().all(|&b| b == 0x33))
    );
}

#[test]
fn refuses_a_page_of_the_wrong_length() {
    let (mut driver, board) = panel(healthy);
    assert_eq!(
        driver.draw(&[0; 192_000]),
        Err(PanelError::WrongLength(192_000))
    );
    assert!(board.borrow().writes.is_empty());
}

#[test]
fn notices_refreshes_that_are_not_real() {
    let (mut driver, _) = panel(|_| false);
    assert_eq!(driver.refresh(), Err(PanelError::RefreshNotStarted));

    let (mut driver, _) = panel(|since| since.is_some_and(|ms| ms < 500));
    assert_eq!(driver.refresh(), Err(PanelError::RefreshTooShort(500)));

    let (mut driver, _) = panel(|since| since.is_some());
    assert_eq!(driver.refresh(), Err(PanelError::RefreshTimedOut));
}

#[test]
fn gives_up_on_controllers_that_stay_busy() {
    let (mut driver, _) = panel(|_| true);
    assert_eq!(driver.wake(), Err(PanelError::StayedBusy));
}

#[test]
fn sleeps_with_power_off() {
    let (mut driver, board) = panel(healthy);
    driver.sleep().expect("sleeps");
    assert_eq!(
        commands(&board.borrow().writes),
        vec![(0x02, true, true, 1)]
    );
    let (_bus, _pins, _delay) = driver.release();
}
