use embassy_stm32::pac;
use defmt::*;

pub fn check_enabled_clocks(){
    // DIAGNOSTIC: dump every RCC clock-enable register so we can see if
    // anything is still clocked that we didn't expect at this point.
    info!("Clock enabled peripherals:");
    let ahb1enr_reg = pac::RCC.ahb1enr().read();
    info!("GPDMA1: {}", ahb1enr_reg.gpdma1en());
    // let ahb2enr_reg = pac::RCC.ahb2enr().read();
    let ahb4enr_reg = pac::RCC.ahb4enr().read();
    info!("ADC4: {}", ahb4enr_reg.adc4en());
    let ahb5enr_reg = pac::RCC.ahb5enr().read();
    info!("RADIO: {}", ahb5enr_reg.radioen());


    let apb1enr1_reg = pac::RCC.apb1enr1().read();
    info!("I2C1: {}", apb1enr1_reg.i2c1en());
    info!("I2C2: {}", apb1enr1_reg.i2c2en());
    info!("USART2: {}", apb1enr1_reg.usart2en());
    info!("USART3: {}", apb1enr1_reg.usart3en());
    info!("TIM2: {}", apb1enr1_reg.tim2en());
    info!("TIM3: {}", apb1enr1_reg.tim3en());
    info!("TIM4: {}", apb1enr1_reg.tim4en());
    info!("SPI2: {}", apb1enr1_reg.spi2en());

    let apb1enr2_reg = pac::RCC.apb1enr2().read();
    info!("I2C4: {}", apb1enr2_reg.i2c4en());
    info!("LPTIM2: {}", apb1enr2_reg.lptim2en());
    let apb2enr_reg = pac::RCC.apb2enr().read();
    info!("TIM17: {}", apb2enr_reg.tim17en());
    info!("TIM16: {}", apb2enr_reg.tim16en());
    info!("USART1: {}", apb2enr_reg.usart1en());
    info!("SPI1: {}", apb2enr_reg.spi1en());
    info!("TIM1: {}", apb2enr_reg.tim1en());
    let apb7enr_reg = pac::RCC.apb7enr().read();
    info!("VREF: {}", apb7enr_reg.vrefen());
    info!("LPTIM1: {}", apb7enr_reg.lptim1en());
    info!("I2C3: {}", apb7enr_reg.i2c3en());
    info!("LPUART1: {}", apb7enr_reg.lpuart1en());
    info!("SPI3: {}", apb7enr_reg.spi3en());
}
