use embedded_hal::i2c::I2c as BlockingI2c;
use embedded_hal_async::i2c::I2c as AsyncI2c;

/// Wrapper that makes blocking I2C appear async
pub struct BlockingI2cAdapter<I2C> {
    i2c: I2C,
}

impl<I2C> BlockingI2cAdapter<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }
}

impl<I2C> AsyncI2c for BlockingI2cAdapter<I2C>
where
    I2C: BlockingI2c,
{
    async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.read(address, buffer)
    }

    async fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(address, bytes)
    }

    async fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c.write_read(address, bytes, buffer)
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        for op in operations {
            match op {
                embedded_hal_async::i2c::Operation::Read(buf) => {
                    self.i2c.read(address, buf)?;
                }
                embedded_hal_async::i2c::Operation::Write(buf) => {
                    self.i2c.write(address, buf)?;
                }
            }
        }
        Ok(())
    }
}

// Add this trait bound to make the error type available
impl<I2C> embedded_hal_async::i2c::ErrorType for BlockingI2cAdapter<I2C>
where
    I2C: BlockingI2c,
{
    type Error = I2C::Error;
}

pub fn block_on_lis3dh<T>(
    future: impl core::future::Future<Output = Result<T, lis3dh_async::Error<esp_idf_hal::i2c::I2cError>>>
) -> anyhow::Result<T> {
    futures_lite::future::block_on(future)
        .map_err(|e| anyhow::anyhow!("LIS3DH error: {:?}", e))
}
