use super::{I2c, RegisterInterface, bisync, only_async, only_sync};
use crate::{
    CtrlMode, DeviceMode, FT6336U_I2C_ADDRESS, Ft6336uError, Ft6336uInterface, Ft6336uLowLevel,
    GestureId, GestureMode, PowerModeEnum, TouchData, TouchEvent, TouchStatus,
};

#[bisync]
impl<I2CBus, E> RegisterInterface for Ft6336uInterface<I2CBus>
where
    I2CBus: I2c<Error = E>,
    E: core::fmt::Debug,
{
    type AddressType = u8;
    type Error = Ft6336uError<E>;

    async fn read_register(
        &mut self,
        address: u8,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c_bus
            .write_read(FT6336U_I2C_ADDRESS, &[address], data)
            .await
            .map_err(Ft6336uError::I2c)
    }

    async fn write_register(
        &mut self,
        address: u8,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let mut buffer = [0u8; 5];
        buffer[0] = address;
        buffer[1..1 + data.len()].copy_from_slice(data);
        self.i2c_bus
            .write(FT6336U_I2C_ADDRESS, &buffer[..1 + data.len()])
            .await
            .map_err(Ft6336uError::I2c)
    }
}

pub struct Ft6336u<
    I2CImpl: RegisterInterface<AddressType = u8, Error = Ft6336uError<I2CBusErr>>,
    I2CBusErr: core::fmt::Debug,
> {
    pub ll: Ft6336uLowLevel<I2CImpl>,
    touch_data: TouchData,
    _marker: core::marker::PhantomData<I2CBusErr>,
}

impl<I2CBus, E> Ft6336u<Ft6336uInterface<I2CBus>, E>
where
    I2CBus: I2c<Error = E>,
    E: core::fmt::Debug,
{
    pub fn new(i2c: I2CBus) -> Self {
        Self {
            ll: Ft6336uLowLevel::new(Ft6336uInterface::new(i2c)),
            touch_data: TouchData::default(),
            _marker: core::marker::PhantomData,
        }
    }
}

pub trait CurrentFt6336uDriverInterface<E>:
    RegisterInterface<AddressType = u8, Error = Ft6336uError<E>>
{
}

impl<T, E> CurrentFt6336uDriverInterface<E> for T
where
    T: RegisterInterface<AddressType = u8, Error = Ft6336uError<E>>,
    E: core::fmt::Debug,
{
}

include!("bisync_helpers.rs");

impl<I2CImpl, I2CBusErr> Ft6336u<I2CImpl, I2CBusErr>
where
    I2CImpl: CurrentFt6336uDriverInterface<I2CBusErr>,
    I2CBusErr: core::fmt::Debug,
{
    // === Device Mode (0x00) ===

    #[bisync]
    pub async fn read_device_mode(&mut self) -> Result<DeviceMode, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.device_mode();
        let reg = read_internal(&mut op).await?;
        Ok(reg.mode())
    }

    #[bisync]
    pub async fn write_device_mode(
        &mut self,
        mode: DeviceMode,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.device_mode();
        write_internal(&mut op, |r| r.set_mode(mode)).await
    }

    // === Gesture ID (0x01) ===

    #[bisync]
    pub async fn read_gesture_id(&mut self) -> Result<GestureId, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.gesture_id();
        let reg = read_internal(&mut op).await?;
        Ok(reg.gesture())
    }

    // === Touch Detection Status (0x02) ===

    #[bisync]
    pub async fn read_touch_count(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.td_status();
        let reg = read_internal(&mut op).await?;
        Ok(reg.touch_count())
    }

    // === Touch Point Data (0x03-0x0E, block repeated for 2 points) ===

    #[bisync]
    pub async fn read_touch_x(&mut self, point: usize) -> Result<u16, Ft6336uError<I2CBusErr>> {
        let mut block = self.ll.tp(point);
        let mut op = block.x_event();
        let reg = read_internal(&mut op).await?;
        Ok(reg.x())
    }

    #[bisync]
    pub async fn read_touch_y(&mut self, point: usize) -> Result<u16, Ft6336uError<I2CBusErr>> {
        let mut block = self.ll.tp(point);
        let mut op = block.y_id();
        let reg = read_internal(&mut op).await?;
        Ok(reg.y())
    }

    #[bisync]
    pub async fn read_touch_event(
        &mut self,
        point: usize,
    ) -> Result<TouchEvent, Ft6336uError<I2CBusErr>> {
        let mut block = self.ll.tp(point);
        let mut op = block.x_event();
        let reg = read_internal(&mut op).await?;
        Ok(reg.event())
    }

    #[bisync]
    pub async fn read_touch_id(&mut self, point: usize) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut block = self.ll.tp(point);
        let mut op = block.y_id();
        let reg = read_internal(&mut op).await?;
        Ok(reg.id())
    }

    #[bisync]
    pub async fn read_touch_weight(
        &mut self,
        point: usize,
    ) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut block = self.ll.tp(point);
        let mut op = block.weight();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn read_touch_area(
        &mut self,
        point: usize,
    ) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut block = self.ll.tp(point);
        let mut op = block.misc();
        let reg = read_internal(&mut op).await?;
        Ok(reg.area())
    }

    // === Threshold (0x80) ===

    #[bisync]
    pub async fn read_touch_threshold(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.threshold();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_touch_threshold(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.threshold();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === Filter Coefficient (0x85) ===

    #[bisync]
    pub async fn read_filter_coefficient(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.filter_coefficient();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_filter_coefficient(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.filter_coefficient();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === Ctrl (0x86) ===

    #[bisync]
    pub async fn read_ctrl_mode(&mut self) -> Result<CtrlMode, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.ctrl();
        let reg = read_internal(&mut op).await?;
        Ok(reg.mode())
    }

    #[bisync]
    pub async fn write_ctrl_mode(
        &mut self,
        mode: CtrlMode,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.ctrl();
        write_internal(&mut op, |r| r.set_mode(mode)).await
    }

    // === Time Enter Monitor (0x87) ===

    #[bisync]
    pub async fn read_time_enter_monitor(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.time_enter_monitor();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_time_enter_monitor(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.time_enter_monitor();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === Active Mode Rate (0x88) ===

    #[bisync]
    pub async fn read_active_rate(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.active_mode_rate();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_active_rate(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.active_mode_rate();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === Monitor Mode Rate (0x89) ===

    #[bisync]
    pub async fn read_monitor_rate(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.monitor_mode_rate();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_monitor_rate(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.monitor_mode_rate();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === Frequency Hopping Enable (0x8B) ===

    #[bisync]
    pub async fn read_freq_hopping_en(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.freq_hopping_en();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_freq_hopping_en(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.freq_hopping_en();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === Gesture Parameters (0x91-0x96) ===

    #[bisync]
    pub async fn read_radian_value(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.radian_value();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_radian_value(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.radian_value();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    #[bisync]
    pub async fn read_offset_left_right(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.offset_left_right();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_offset_left_right(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.offset_left_right();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    #[bisync]
    pub async fn read_offset_up_down(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.offset_up_down();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_offset_up_down(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.offset_up_down();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    #[bisync]
    pub async fn read_distance_left_right(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.distance_left_right();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_distance_left_right(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.distance_left_right();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    #[bisync]
    pub async fn read_distance_up_down(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.distance_up_down();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_distance_up_down(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.distance_up_down();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    #[bisync]
    pub async fn read_distance_zoom(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.distance_zoom();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_distance_zoom(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.distance_zoom();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === System Information (0x9F-0xBC) ===

    #[bisync]
    pub async fn read_cipher_mid(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.cipher_mid();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn read_cipher_low(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.cipher_low();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn read_library_version(&mut self) -> Result<u16, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.library_version();
        let reg = read_internal(&mut op).await?;
        Ok(reg.version())
    }

    #[bisync]
    pub async fn read_chip_id(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.chip_id();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn read_g_mode(&mut self) -> Result<GestureMode, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.g_mode();
        let reg = read_internal(&mut op).await?;
        Ok(reg.mode())
    }

    #[bisync]
    pub async fn write_g_mode(
        &mut self,
        mode: GestureMode,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.g_mode();
        write_internal(&mut op, |r| r.set_mode(mode)).await
    }

    #[bisync]
    pub async fn read_power_mode(&mut self) -> Result<PowerModeEnum, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.power_mode();
        let reg = read_internal(&mut op).await?;
        Ok(reg.mode())
    }

    #[bisync]
    pub async fn write_power_mode(
        &mut self,
        mode: PowerModeEnum,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.power_mode();
        write_internal(&mut op, |r| r.set_mode(mode)).await
    }

    #[bisync]
    pub async fn read_firmware_id(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.firmware_id();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn read_focaltech_id(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.focaltech_id();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn read_release_code_id(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.release_code_id();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn read_face_dec_mode(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.face_dec_mode();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_face_dec_mode(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.face_dec_mode();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    #[bisync]
    pub async fn read_state(&mut self) -> Result<u8, Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.state();
        let reg = read_internal(&mut op).await?;
        Ok(reg.value())
    }

    #[bisync]
    pub async fn write_state(
        &mut self,
        val: u8,
    ) -> Result<(), Ft6336uError<I2CBusErr>> {
        let mut op = self.ll.state();
        write_internal(&mut op, |r| r.set_value(val)).await
    }

    // === Scan (reads all touch points efficiently) ===

    #[bisync]
    pub async fn scan(&mut self) -> Result<TouchData, Ft6336uError<I2CBusErr>> {
        let touch_count = self.read_touch_count().await?;
        self.touch_data.touch_count = touch_count;

        if touch_count == 0 {
            self.touch_data.points[0].status = TouchStatus::Release;
            self.touch_data.points[1].status = TouchStatus::Release;
        } else if touch_count == 1 {
            // Read YId register (single 16-bit read gives us both y and id)
            let y_id_reg = {
                let mut block = self.ll.tp(0);
                let mut op = block.y_id();
                read_internal(&mut op).await?
            };
            let id = y_id_reg.id() as usize;
            if id < 2 {
                let prev_status = self.touch_data.points[id].status;
                self.touch_data.points[id].status = match prev_status {
                    TouchStatus::Release => TouchStatus::Touch,
                    _ => TouchStatus::Stream,
                };

                // Read XEvent register (single 16-bit read gives us x)
                let x_event_reg = {
                    let mut block = self.ll.tp(0);
                    let mut op = block.x_event();
                    read_internal(&mut op).await?
                };
                self.touch_data.points[id].x = x_event_reg.x();
                self.touch_data.points[id].y = y_id_reg.y();

                let other_id = id ^ 1;
                self.touch_data.points[other_id].status = TouchStatus::Release;
            }
        } else {
            // Touch point 1
            let y_id_reg1 = {
                let mut block = self.ll.tp(0);
                let mut op = block.y_id();
                read_internal(&mut op).await?
            };
            let id1 = y_id_reg1.id() as usize;
            if id1 < 2 {
                let prev_status = self.touch_data.points[id1].status;
                self.touch_data.points[id1].status = match prev_status {
                    TouchStatus::Release => TouchStatus::Touch,
                    _ => TouchStatus::Stream,
                };
                let x_event_reg1 = {
                    let mut block = self.ll.tp(0);
                    let mut op = block.x_event();
                    read_internal(&mut op).await?
                };
                self.touch_data.points[id1].x = x_event_reg1.x();
                self.touch_data.points[id1].y = y_id_reg1.y();
            }

            // Touch point 2
            let y_id_reg2 = {
                let mut block = self.ll.tp(1);
                let mut op = block.y_id();
                read_internal(&mut op).await?
            };
            let id2 = y_id_reg2.id() as usize;
            if id2 < 2 {
                let prev_status = self.touch_data.points[id2].status;
                self.touch_data.points[id2].status = match prev_status {
                    TouchStatus::Release => TouchStatus::Touch,
                    _ => TouchStatus::Stream,
                };
                let x_event_reg2 = {
                    let mut block = self.ll.tp(1);
                    let mut op = block.x_event();
                    read_internal(&mut op).await?
                };
                self.touch_data.points[id2].x = x_event_reg2.x();
                self.touch_data.points[id2].y = y_id_reg2.y();
            }
        }

        Ok(self.touch_data)
    }
}
