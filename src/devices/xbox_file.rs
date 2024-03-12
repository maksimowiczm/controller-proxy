use crate::controller_state::ControllerState;
use crate::devices::Device;
use filepath::FilePath;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::mem::transmute;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{io, mem};
use tokio::io::{AsyncRead, ReadBuf};

pub struct XboxFile {
    state: ControllerState,
    event_file: File,
    dead_zone: i16,
}

impl XboxFile {
    pub async fn create() -> Result<(Self, std::path::PathBuf), Box<dyn std::error::Error>> {
        let re_xbox = Regex::new(r"Microsoft (X-Box|Xbox)")?;
        let xbox = Device::from_proc_file(re_xbox)?;
        let event_file = xbox.get_event_handler()?;
        let path = event_file.path()?;
        Ok((
            XboxFile {
                event_file,
                state: ControllerState::default(),
                dead_zone: 25,
            },
            path,
        ))
    }

    fn read_buffer(&mut self) -> io::Result<[u8; XBOX_PACKET_SIZE]> {
        let mut buffer = [0u8; XBOX_PACKET_SIZE];
        self.event_file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn update(&mut self, packet: &XboxPacket) -> bool {
        if packet.code != 1 && packet.code != 4 {
            return false;
        }

        if let Ok(mut value) = i16::try_from(packet.value * 255 / i16::MAX as i32) {
            // apply dead zone
            if -self.dead_zone < value && value < self.dead_zone {
                value = 0;
            }

            // update only if value changes
            return if packet.code == 1 && self.state.left_thumb != value {
                self.state.left_thumb = value;
                true
            } else if packet.code == 4 && self.state.right_thumb != value {
                self.state.right_thumb = value;
                true
            } else {
                false
            };
        }

        false
    }
}

#[repr(C)]
#[derive(Debug)]
struct XboxPacket {
    pub tv_sec: u64,
    pub tv_usec: u64,
    pub ev_type: u16,
    pub code: u16,
    pub value: i32,
}
const XBOX_PACKET_SIZE: usize = mem::size_of::<XboxPacket>();

// todo make it async
impl AsyncRead for XboxFile {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let buffer = self.read_buffer()?;

        unsafe {
            let mut packet: XboxPacket = transmute(buffer);

            // skip "trash?" packets
            while packet.code == 0 || !self.update(&packet) {
                packet = transmute(self.read_buffer()?);

                if packet.code != 0 {
                    log::debug!("{:?}", packet);
                }
            }

            let ptr = &self.state as *const ControllerState as *const u8;

            log::debug!("{:?}", self.state);
            buf.put_slice(std::slice::from_raw_parts(
                ptr,
                mem::size_of::<ControllerState>(),
            ))
        }

        Poll::Ready(Ok(()))
    }
}
