use time::macros::format_description;

pub fn ip_to_i32(ip: String) -> Option<u32> {
    let port_split: Vec<&str> = ip.split(":").collect();
    if port_split.len() < 1 {
        return None;
    }

    let ip_split: Vec<&str> = port_split[0].split(".").collect();
    if ip_split.len() < 4 {
        return None;
    }

    let mut out: u32 = 0;

    for i in 0..4 {
        match ip_split[i].parse::<u32>() {
            Ok(s) => out += s * (i as u32 + 1),
            Err(_) => return None,
        };
    }

    return Some(out);
}

pub const TIME_FORMAT: &[time::format_description::FormatItem<'_>] = format_description!("[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]");
