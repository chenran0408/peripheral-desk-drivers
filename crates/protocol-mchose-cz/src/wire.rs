//! The vendor's CZ report ID 0 protocol. Addresses are byte offsets in a profile.
use peripheral_hid_transport::WinTransport;
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, String>;

pub trait Io {
    fn packet(&mut self, cmd: u8, offset: u16, len: u8, write: Option<&[u8]>) -> Result<Vec<u8>>;
}
fn frame(cmd: u8, offset: u16, len: u8, data: Option<&[u8]>) -> [u8; 65] {
    let mut out = [0; 65];
    let [lo, hi] = offset.to_le_bytes();
    out[1..9].copy_from_slice(&[0x55, cmd, 0, 0, len, lo, hi, 0]);
    if let Some(bytes) = data {
        out[9..9 + bytes.len()].copy_from_slice(bytes);
    }
    out[4] = out[5..].iter().fold(0u8, |a, b| a.wrapping_add(*b));
    out
}
impl Io for WinTransport {
    fn packet(&mut self, cmd: u8, offset: u16, len: u8, write: Option<&[u8]>) -> Result<Vec<u8>> {
        let time = Duration::from_millis(1000);
        self.write(&frame(cmd, offset, len, write), time)?;
        let end = Instant::now() + time;
        loop {
            let left = end
                .checked_duration_since(Instant::now())
                .ok_or("键盘应答超时")?;
            let b = self.read(65, left)?;
            if b.len() < 9 || b[..3] != [0, 0xaa, cmd] || b[6..8] != offset.to_le_bytes() {
                continue;
            }
            let used = b[5];
            if used > len || (write.is_none() && used != len) {
                return Err("键盘返回的有效长度与请求不符".into());
            }
            return b
                .get(9..9 + usize::from(used))
                .map(<[u8]>::to_vec)
                .ok_or_else(|| "键盘应答不完整".into());
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub cmd: u8,
    pub addr: u16,
    pub len: usize,
}
impl Region {
    pub fn new(cmd: u8, addr: usize, len: usize) -> Self {
        Self {
            cmd,
            addr: addr as u16,
            len,
        }
    }
    pub fn read(self, io: &mut impl Io) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(self.len);
        for pos in (0..self.len).step_by(56) {
            data.extend(io.packet(
                self.cmd,
                self.addr + pos as u16,
                (self.len - pos).min(56) as u8,
                None,
            )?);
        }
        Ok(data)
    }
    fn write(self, io: &mut impl Io, bytes: &[u8]) -> Result<()> {
        let set = match self.cmd {
            5 => 6,
            8 => 9,
            160 => 161,
            162 => 163,
            164 => 165,
            166 => 167,
            _ => return Err("该区域不可写".into()),
        };
        for (i, c) in bytes.chunks(56).enumerate() {
            io.packet(set, self.addr + (i * 56) as u16, c.len() as u8, Some(c))?;
        }
        if self.read(io)? != bytes {
            return Err("读回内容与写入值不同".into());
        }
        Ok(())
    }
}
pub struct Edit {
    pub region: Region,
    pub old: Vec<u8>,
    pub new: Vec<u8>,
}
pub fn commit(io: &mut impl Io, edits: &[Edit]) -> Result<usize> {
    if edits
        .iter()
        .any(|e| e.old.len() != e.region.len || e.new.len() != e.region.len)
    {
        return Err("编辑的数据长度不匹配".into());
    }
    let mut written: Vec<&Edit> = Vec::new();
    for e in edits.iter().filter(|e| e.old != e.new) {
        written.push(e);
        if let Err(err) = e.region.write(io, &e.new) {
            let mut failures = Vec::new();
            for x in written.into_iter().rev() {
                if let Err(r) = x.region.write(io, &x.old) {
                    failures.push(r)
                }
            }
            return Err(if failures.is_empty() {
                format!("{err}。已恢复原值并读回确认。")
            } else {
                format!("{err}。恢复未完成：{}", failures.join(" / "))
            });
        }
    }
    Ok(written.len())
}
pub fn current(io: &mut impl Io) -> Result<usize> {
    let b = Region::new(4, 0, 6).read(io)?;
    let mut order = b[2..6].to_vec();
    order.sort_unstable();
    if order == [0, 1, 2, 3] && b[0] < b[1] && b[1] <= 4 {
        Ok(usize::from(b[2 + usize::from(b[0])]))
    } else if b[0] < 4 {
        Ok(usize::from(b[0]))
    } else {
        Err("键盘当前板载配置无效".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vendor_read_and_write_frames_have_adjacent_checksum_and_offset() {
        assert_eq!(
            &frame(160, 1040, 8, None)[..9],
            &[0, 85, 160, 0, 28, 8, 16, 4, 0]
        );
        assert_eq!(
            &frame(161, 8, 2, Some(&[160, 1]))[..11],
            &[0, 85, 161, 0, 171, 2, 8, 0, 0, 160, 1]
        );
    }
}
