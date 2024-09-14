use time::macros::format_description;

pub fn ip_to_u32(ip: String) -> Option<u32> {
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
            Ok(s) => out += s * 2_u32.pow(8 * (3 - i) as u32),
            Err(_) => return None,
        };
    }

    return Some(out);
}

pub fn u32_to_ip(ip: u32) -> String {
    return format!(
        "{}.{}.{}.{}",
        ip / (256 * 256 * 256) % 256,
        ip / (256 * 256) % 256,
        ip / 256 % 256,
        ip % 256,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost() {
        let ip_str = String::from("127.0.0.1");

        let ip_dec = ip_to_u32(ip_str.clone()).unwrap();

        assert_eq!(ip_dec, 2130706433); // magic number localhost to dec
        assert_eq!(u32_to_ip(ip_dec), ip_str);
    }

    #[test]
    fn test_ignores_port() {
        let ip_str = String::from("127.0.0.1:8080");

        let ip_dec = ip_to_u32(ip_str.clone()).unwrap();

        assert_eq!(ip_dec, 2130706433); // magic number localhost to dec
    }

    #[test]
    fn test_private_ip() {
        let ip_str = String::from("192.168.66.133");

        let ip_dec = ip_to_u32(ip_str.clone()).unwrap();

        assert_eq!(ip_dec, 3232252549); // magic number of the ip in dec
        assert_eq!(u32_to_ip(ip_dec), ip_str);
    }
}

pub const TIME_FORMAT: &[time::format_description::FormatItem<'_>] = format_description!(
    "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second]"
);
