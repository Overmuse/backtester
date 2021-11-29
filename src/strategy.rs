use crate::brokerage::{Brokerage, Event};
use crate::markets::market::Market;

#[allow(unused_variables)]
pub trait Strategy {
    type Error;

    fn initialize(&mut self) {}

    fn before_open(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn on_event(&mut self, event: Event) -> Result<(), Self::Error> {
        Ok(())
    }

    fn at_open(&mut self, brokerage: &mut Brokerage, market: &Market) -> Result<(), Self::Error> {
        Ok(())
    }

    fn during_regular_hours(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn at_close(&mut self, brokerage: &mut Brokerage, market: &Market) -> Result<(), Self::Error> {
        Ok(())
    }

    fn after_close(
        &mut self,
        brokerage: &mut Brokerage,
        market: &Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
