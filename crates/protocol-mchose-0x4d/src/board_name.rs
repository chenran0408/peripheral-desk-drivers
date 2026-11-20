use super::*;

const NAME_BYTES: usize = 60;
const READ_NAME: u16 = 0x0905;
const WRITE_NAME: u16 = 0x0910;

fn read(t: &mut WinTransport, profile: u8) -> Result<Vec<u8>, String> {
    let mut name = Vec::with_capacity(NAME_BYTES);
    while name.len() < NAME_BYTES {
        let n = (NAME_BYTES - name.len()).min(23);
        let bytes = read_reply(
            t,
            READ_NAME,
            &[profile, name.len() as u8, n as u8],
            &mut None,
            |frame| Some(payload(frame, READ_NAME)?.get(..n)?.to_vec()),
        )?;
        name.extend(bytes);
    }
    Ok(name)
}

fn write(t: &mut WinTransport, profile: u8, raw: &[u8]) -> Result<(), String> {
    for (n, chunk) in raw.chunks(20).enumerate() {
        let pos = n * 20;
        let ctl = (chunk.len() as u8) << 2
            | u8::from(pos == 0)
            | (u8::from(pos + chunk.len() == raw.len()) << 1);
        let mut data = vec![profile, pos as u8, ctl];
        data.extend_from_slice(chunk);
        settings::send_check(t, WRITE_NAME, &data)?;
    }
    Ok(())
}

fn parts(raw: &[u8]) -> Result<(&str, &str), String> {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let full =
        std::str::from_utf8(&raw[..end]).map_err(|_| "板载配置名使用未知字符编码，已保留原值")?;
    let suffix = ["$0$", "$1$", "$2$"]
        .iter()
        .filter_map(|&sep| full.find(sep))
        .min()
        .unwrap_or(full.len());
    Ok(full.split_at(suffix))
}

pub(super) fn handle(t: &mut WinTransport, req: &Value, count: u8) -> Result<Value, String> {
    let profile = req["profile"]
        .as_u64()
        .filter(|&n| n < u64::from(count))
        .ok_or("请选择设备提供的板载配置")? as u8;
    let baseline = read(t, profile)?;
    let (old, suffix) = parts(&baseline)?;
    let limit = NAME_BYTES - suffix.len();
    let name = if req["op"] == "set-board-name" {
        let text = req["name"].as_str().ok_or("请填写配置名称")?;
        if text.trim().is_empty()
            || text.contains('\0')
            || text.len() > limit
            || ["$0$", "$1$", "$2$"].iter().any(|s| text.contains(s))
        {
            return Err(format!(
                "请使用 1–{limit} 个 UTF-8 字节的名称，避开官方模板分隔符"
            ));
        }
        if text != old {
            let old_string = [old, suffix].concat().into_bytes();
            let new_string = [text, suffix].concat().into_bytes();
            // Empty original names need a NUL byte to issue an actual restore to the device.
            settings::verified(old_string, new_string, |bytes| {
                write(t, profile, if bytes.is_empty() { &[0] } else { bytes })?;
                let readback = read(t, profile)?;
                let end = readback
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(readback.len());
                Ok(readback[..end].to_vec())
            })?;
        }
        text
    } else {
        old
    };
    Ok(json!({"profile":profile,"name":name,"max_bytes":limit}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rename_keeps_official_template_reference_outside_the_editable_name() {
        assert_eq!(
            parts("办公$2$profile-id\0余量".as_bytes()).unwrap(),
            ("办公", "$2$profile-id")
        );
        assert_eq!(parts(b"custom\0\0").unwrap(), ("custom", ""));
        assert!(parts(&[0xff, 0]).is_err());
    }
}
