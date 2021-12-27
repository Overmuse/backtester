use crate::brokerage::actor::Event;
use crate::brokerage::handle::Brokerage;
use crate::markets::handle::Market;

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait Strategy {
    type Error;

    async fn initialize(&mut self) {}

    async fn before_open(
        &mut self,
        brokerage: Brokerage,
        market: Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn on_event(&mut self, event: Event) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn at_open(&mut self, brokerage: Brokerage, market: Market) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn during_regular_hours(
        &mut self,
        brokerage: Brokerage,
        market: Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn at_close(&mut self, brokerage: Brokerage, market: Market) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn after_close(
        &mut self,
        brokerage: Brokerage,
        market: Market,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
