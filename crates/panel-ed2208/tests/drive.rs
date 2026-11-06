//! The driver against a mock bus that records every byte, pin change and delay.

#![allow(clippy::expect_used)] // Tests may panic on failed assumptions.

use std::cell::RefCell;
use std::convert::Infallible;
use std::rc::Rc;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{self, InputPin, OutputPin};
use embedded_hal::spi::{self, SpiBus};
use frame_core::{Glass, PanelSize};
use panel_ed2208::{Ed2208, PAGE_LEN, PanelError, Pins};

/// One write on the bus: whether the chip was selected, whether DC said data, and the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Write {
    selected: bool,
    data: bool,
    bytes: Vec<u8>,
}

/// The shared state of the mock board.
struct Board {
    cs_low: bool,
    dc_high: bool,
    resets: usize,
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
            selected: board.cs_low,
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
            "cs" => board.cs_low = !high,
            "dc" => board.dc_high = high,
            _ => {
                if !high {
                    board.resets += 1;
                }
            }
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

/// A healthy panel: ready at once, then busy for 19 s from 3 ms after a refresh command.
fn healthy(since: Option<u64>) -> bool {
    since.is_some_and(|ms| (3..19_003).contains(&ms))
}

type Driver = Ed2208<Bus, Pin, Busy, Delay>;

fn panel(busy_low: fn(Option<u64>) -> bool) -> (Driver, Shared) {
    let board: Shared = Rc::new(RefCell::new(Board {
        cs_low: false,
        dc_high: false,
        resets: 0,
        writes: Vec::new(),
        clock_ms: 0,
        refresh_at: None,
        busy_low,
    }));
    let pins = Pins {
        cs: Pin(Rc::clone(&board), "cs"),
        dc: Pin(Rc::clone(&board), "dc"),
        rst: Pin(Rc::clone(&board), "rst"),
        busy: Busy(Rc::clone(&board)),
    };
    let driver = Ed2208::new(Bus(Rc::clone(&board)), pins, Delay(Rc::clone(&board)));
    (driver, board)
}

/// The commands sent, each with the length of its data. Panics on a write with the chip not
/// selected.
fn commands(writes: &[Write]) -> Vec<(u8, usize)> {
    let mut out: Vec<(u8, usize)> = Vec::new();
    for write in writes {
        assert!(write.selected, "wrote with the chip not selected");
        if write.data {
            if let Some(last) = out.last_mut() {
                last.1 += write.bytes.len();
            }
        } else {
            out.push((write.bytes[0], 0));
        }
    }
    out
}

#[test]
fn wakes_with_a_reset_the_sequence_and_power_on() {
    let (mut driver, board) = panel(healthy);
    driver.wake().expect("wakes");
    let board = board.borrow();
    assert_eq!(board.resets, 1);
    let sent = commands(&board.writes);
    assert_eq!(sent.len(), 20);
    assert_eq!(sent[0], (0xaa, 6));
    assert!(sent.contains(&(0x61, 4)));
    assert_eq!(sent[19], (0x04, 0));
    let resolution: &Write = board
        .writes
        .iter()
        .find(|write| write.data && write.bytes.len() == 4 && write.bytes[0] == 0x03)
        .expect("the resolution went out");
    // 800 by 480.
    assert_eq!(resolution.bytes, [0x03, 0x20, 0x01, 0xe0]);
    assert!(!board.cs_low && !board.dc_high);
}

#[test]
fn draws_the_page_in_chunks_and_refreshes() {
    let (mut driver, board) = panel(healthy);
    let mut page: Vec<u8> = vec![0x11; PAGE_LEN];
    page[PAGE_LEN - 1] = 0x35;
    driver.wake().expect("wakes");
    board.borrow_mut().writes.clear();
    let took: u32 = driver.show(&page).expect("shows");
    assert!((18_000..20_000).contains(&took), "{took} ms");
    let board = board.borrow();
    assert_eq!(commands(&board.writes), vec![(0x10, 192_000), (0x12, 1)]);
    let sent: Vec<u8> = board
        .writes
        .iter()
        .filter(|write| write.data && write.bytes.len() > 1)
        .flat_map(|write| write.bytes.clone())
        .collect();
    assert_eq!(sent, page);
    assert!(board.writes.iter().all(|write| write.bytes.len() <= 6_400));
}

#[test]
fn refuses_a_page_of_the_wrong_length() {
    let (mut driver, board) = panel(healthy);
    assert_eq!(
        driver.draw(&[0; 960_000]),
        Err(PanelError::WrongLength(960_000))
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
fn gives_up_on_a_controller_that_stays_busy() {
    let (mut driver, _) = panel(|_| true);
    assert_eq!(driver.wake(), Err(PanelError::StayedBusy));
}

#[test]
fn sleeps_with_power_off_then_deep_sleep() {
    let (mut driver, board) = panel(healthy);
    driver.sleep().expect("sleeps");
    assert_eq!(commands(&board.borrow().writes), vec![(0x02, 1), (0x07, 1)]);
    assert_eq!(board.borrow().writes[3].bytes, [0xa5]);
}

#[test]
fn is_glass_for_the_small_panel() {
    fn shown<G: Glass>(glass: &mut G, page: &[u8]) -> Result<u32, G::Error> {
        glass.wake()?;
        let took: u32 = glass.show(page)?;
        glass.sleep()?;
        Ok(took)
    }
    assert_eq!(Driver::PANEL, PanelSize::Inches7Point3);
    assert_eq!(
        frame_core::panel_bytes::len(PanelSize::Inches7Point3),
        PAGE_LEN
    );
    let (mut driver, _) = panel(healthy);
    assert!(shown(&mut driver, &[0x11; PAGE_LEN]).is_ok());
}
