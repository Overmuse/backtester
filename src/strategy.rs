use crate::brokerage::Brokerage;
use crate::markets::market::Market;

pub trait Strategy {
    type Error;

    fn initialize(&mut self) {}
    fn before_open(
        &mut self,
        _brokerage: &mut Brokerage,
        _market: &Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn at_open(&mut self, _brokerage: &mut Brokerage, _market: &Market) -> Result<(), Self::Error> {
        Ok(())
    }
    fn on_data(&mut self, _brokerage: &mut Brokerage, _market: &Market) -> Result<(), Self::Error> {
        Ok(())
    }
    fn at_close(
        &mut self,
        _brokerage: &mut Brokerage,
        _market: &Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn after_close(
        &mut self,
        _brokerage: &mut Brokerage,
        _market: &Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
