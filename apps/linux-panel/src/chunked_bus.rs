//! An SPI bus that writes in pieces spidev accepts.

use embedded_hal::spi::{ErrorType, SpiBus};

/// Bytes spidev takes in one write unless its `bufsiz` module parameter is raised.
pub const SPIDEV_CHUNK: usize = 4096;

/// Wraps `bus`, splitting each write into pieces of at most `chunk` bytes. Linux's spidev
/// refuses longer writes, and the panel drivers send up to 19,200 bytes at a time. The panel's
/// chip selects are GPIO lines the driver holds, so the pieces arrive as one transfer.
#[derive(Debug)]
pub struct ChunkedBus<B> {
    bus: B,
    chunk: usize,
}

impl<B> ChunkedBus<B> {
    /// Wraps `bus`, writing at most `chunk` bytes at a time (at least one).
    #[must_use]
    pub fn new(bus: B, chunk: usize) -> Self {
        Self {
            bus,
            chunk: chunk.max(1),
        }
    }
}

impl<B: ErrorType> ErrorType for ChunkedBus<B> {
    type Error = B::Error;
}

impl<B: SpiBus> SpiBus for ChunkedBus<B> {
    fn read(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        for piece in words.chunks_mut(self.chunk) {
            self.bus.read(piece)?;
        }
        Ok(())
    }

    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        for piece in words.chunks(self.chunk) {
            self.bus.write(piece)?;
        }
        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.bus.transfer(read, write)
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        self.bus.transfer_in_place(words)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.bus.flush()
    }
}

#[cfg(test)]
mod test {
    use std::convert::Infallible;

    use super::*;

    #[derive(Default)]
    struct Recorder(Vec<usize>);

    impl ErrorType for Recorder {
        type Error = Infallible;
    }

    impl SpiBus for Recorder {
        fn read(&mut self, _words: &mut [u8]) -> Result<(), Infallible> {
            Ok(())
        }

        fn write(&mut self, words: &[u8]) -> Result<(), Infallible> {
            self.0.push(words.len());
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

    #[test]
    fn splits_long_writes() {
        let mut bus = ChunkedBus::new(Recorder::default(), SPIDEV_CHUNK);
        assert_eq!(bus.write(&[0; 19_200]), Ok(()));
        assert_eq!(bus.write(&[0x10]), Ok(()));
        assert_eq!(bus.bus.0, [4096, 4096, 4096, 4096, 2816, 1]);
    }
}
