use crate::brokerage::{Brokerage, Event};
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
    fn on_event(&mut self, _event: Event) -> Result<(), Self::Error> {
        Ok(())
    }
    fn at_open(&mut self, _brokerage: &mut Brokerage, _market: &Market) -> Result<(), Self::Error> {
        Ok(())
    }
    fn during_regular_hours(
        &mut self,
        _brokerage: &mut Brokerage,
        _market: &Market,
    ) -> Result<(), Self::Error> {
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
