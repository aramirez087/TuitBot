//! Vendored `x-client-transaction` implementation used to generate
//! `x-client-transaction-id` headers for X web requests.
//!
//! The crates.io release of `x-client-transaction` enables `reqwest` default
//! TLS features in its published manifest, which diverges from this
//! workspace's Rustls-only graph during `cargo package` verification. Keeping
//! the implementation in-tree makes the published package resolve the same
//! dependency graph as local builds.

pub(crate) use transaction::ClientTransaction;

mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub(crate) enum Error {
        #[error("Mismatched interpolation arguments")]
        MismatchedArguments,

        #[error("Request error: {0}")]
        Request(#[from] reqwest::Error),

        #[error("Parse error: {0}")]
        Parse(String),

        #[error("Missing key: {0}")]
        MissingKey(String),

        #[error("Base64 error: {0}")]
        Base64(#[from] base64::DecodeError),
    }
}

mod cubic_curve {
    /// Cubic curve implementation for animation interpolation.
    pub(super) struct Cubic {
        curves: Vec<f64>,
    }

    impl Cubic {
        /// Create a new cubic curve with the given control points.
        pub(super) fn new(curves: Vec<f64>) -> Self {
            Self { curves }
        }

        /// Calculate a value on the cubic curve for the given time parameter.
        pub(super) fn get_value(&self, time: f64) -> f64 {
            let mut start_gradient = 0.0;
            let mut end_gradient = 0.0;
            let start = 0.0;
            let mut mid = 0.0;
            let end = 1.0;

            if time <= 0.0 {
                if self.curves[0] > 0.0 {
                    start_gradient = self.curves[1] / self.curves[0];
                } else if self.curves[1] == 0.0 && self.curves[2] > 0.0 {
                    start_gradient = self.curves[3] / self.curves[2];
                }
                return start_gradient * time;
            }

            if time >= 1.0 {
                if self.curves[2] < 1.0 {
                    end_gradient = (self.curves[3] - 1.0) / (self.curves[2] - 1.0);
                } else if self.curves[2] == 1.0 && self.curves[0] < 1.0 {
                    end_gradient = (self.curves[1] - 1.0) / (self.curves[0] - 1.0);
                }
                return 1.0 + end_gradient * (time - 1.0);
            }

            let mut start_value = start;
            let mut end_value = end;

            while start_value < end_value {
                mid = (start_value + end_value) / 2.0;
                let x_est = Self::calculate(self.curves[0], self.curves[2], mid);
                if (time - x_est).abs() < 0.00001 {
                    return Self::calculate(self.curves[1], self.curves[3], mid);
                }
                if x_est < time {
                    start_value = mid;
                } else {
                    end_value = mid;
                }
            }
            Self::calculate(self.curves[1], self.curves[3], mid)
        }

        /// Helper function to calculate points on the curve.
        fn calculate(a: f64, b: f64, m: f64) -> f64 {
            3.0 * a * (1.0 - m) * (1.0 - m) * m + 3.0 * b * (1.0 - m) * m * m + m * m * m
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_cubic_curve() {
            let cubic = Cubic::new(vec![0.1, 0.2, 0.3, 0.4]);
            let value = cubic.get_value(0.5);
            assert!(value > 0.0);
        }
    }
}

mod interpolate {
    use super::error::Error;

    /// Interpolate between two lists of numerical values.
    pub(super) fn interpolate(
        from_list: &[f64],
        to_list: &[f64],
        factor: f64,
    ) -> Result<Vec<f64>, Error> {
        if from_list.len() != to_list.len() {
            return Err(Error::MismatchedArguments);
        }

        let mut out = Vec::with_capacity(from_list.len());
        for idx in 0..from_list.len() {
            out.push(interpolate_num(from_list[idx], to_list[idx], factor));
        }
        Ok(out)
    }

    /// Interpolate between two numerical values.
    fn interpolate_num(from_val: f64, to_val: f64, factor: f64) -> f64 {
        from_val * (1.0 - factor) + to_val * factor
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_interpolate() {
            let from = vec![0.0, 10.0, 20.0];
            let to = vec![100.0, 110.0, 120.0];
            let result = interpolate(&from, &to, 0.5).unwrap();
            assert_eq!(result, vec![50.0, 60.0, 70.0]);
        }

        #[test]
        fn test_interpolate_error() {
            let from = vec![0.0, 10.0];
            let to = vec![100.0, 110.0, 120.0];
            let result = interpolate(&from, &to, 0.5);
            assert!(result.is_err());
        }
    }
}

mod rotation {
    use std::f64::consts::PI;

    /// Convert rotation from degrees to a 2D rotation matrix.
    pub(super) fn convert_rotation_to_matrix(degrees: f64) -> Vec<f64> {
        let radians = degrees * PI / 180.0;
        let cos = radians.cos();
        let sin = radians.sin();
        vec![cos, -sin, sin, cos]
    }

    /// Convert rotation from degrees to a 2D transformation matrix.
    #[allow(dead_code)]
    pub(super) fn convert_rotation_to_transform_matrix(degrees: f64) -> Vec<f64> {
        let radians = degrees * PI / 180.0;
        let cos = radians.cos();
        let sin = radians.sin();
        vec![cos, sin, -sin, cos, 0.0, 0.0]
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_rotation_matrix() {
            let matrix = convert_rotation_to_matrix(90.0);
            assert!((matrix[0] - 0.0).abs() < 0.00001);
            assert!((matrix[1] - (-1.0)).abs() < 0.00001);
            assert!((matrix[2] - 1.0).abs() < 0.00001);
            assert!((matrix[3] - 0.0).abs() < 0.00001);
        }

        #[test]
        fn test_transform_matrix() {
            let matrix = convert_rotation_to_transform_matrix(90.0);
            assert!((matrix[0] - 0.0).abs() < 0.00001);
            assert!((matrix[1] - 1.0).abs() < 0.00001);
            assert!((matrix[2] - (-1.0)).abs() < 0.00001);
            assert!((matrix[3] - 0.0).abs() < 0.00001);
            assert_eq!(matrix[4], 0.0);
            assert_eq!(matrix[5], 0.0);
        }
    }
}

mod utils {
    use super::error::Error;
    use base64::{engine::general_purpose, Engine as _};
    use regex::Regex;
    use reqwest::blocking::Client;
    use scraper::{Html, Selector};
    use std::collections::HashMap;

    /// Handle X migration redirect and get the home page.
    pub(super) fn handle_x_migration(client: &Client) -> Result<Html, Error> {
        let migration_regex = Regex::new(
            r"(https?://(?:www\.)?(twitter|x)\.com(/x)?/migrate([/?])?tok=[a-zA-Z0-9%\-_]+)+",
        )
        .expect("migration regex is valid");

        let response = client.get("https://x.com").send()?;
        let html = Html::parse_document(&response.text()?);

        let meta_selector =
            Selector::parse("meta[http-equiv='refresh']").expect("meta selector is valid");
        if let Some(meta) = html.select(&meta_selector).next() {
            let content = meta.value().attr("content").unwrap_or("");
            if let Some(captures) = migration_regex.captures(content) {
                if let Some(url) = captures.get(0) {
                    let migration_response = client.get(url.as_str()).send()?;
                    return Ok(Html::parse_document(&migration_response.text()?));
                }
            }
        }

        let form_selector1 = Selector::parse("form[name='f']").expect("form selector is valid");
        let form_selector2 = Selector::parse("form[action='https://x.com/x/migrate']")
            .expect("migrate form selector is valid");

        let form = html
            .select(&form_selector1)
            .next()
            .or_else(|| html.select(&form_selector2).next());

        if let Some(form) = form {
            let url = form
                .value()
                .attr("action")
                .unwrap_or("https://x.com/x/migrate");
            let method = form.value().attr("method").unwrap_or("POST");

            let input_selector = Selector::parse("input").expect("input selector is valid");
            let mut request_payload = HashMap::new();

            for input in form.select(&input_selector) {
                if let Some(name) = input.value().attr("name") {
                    if let Some(value) = input.value().attr("value") {
                        request_payload.insert(name, value);
                    }
                }
            }

            let migration_response = match method.to_uppercase().as_str() {
                "POST" => client.post(url).form(&request_payload).send()?,
                _ => client.get(url).query(&request_payload).send()?,
            };

            return Ok(Html::parse_document(&migration_response.text()?));
        }

        Ok(html)
    }

    /// Check if a number is odd.
    pub(super) fn is_odd(num: i32) -> f64 {
        if num % 2 == 1 {
            -1.0
        } else {
            0.0
        }
    }

    /// Round a number in JavaScript style (ROUND_HALF_UP).
    pub(super) fn js_round(num: f64) -> f64 {
        let decimal_part = num - num.trunc();
        if decimal_part == -0.5 {
            num.ceil()
        } else {
            num.round()
        }
    }

    /// Convert a float to a hexadecimal string.
    pub(super) fn float_to_hex(x: f64) -> String {
        if x == 0.0 {
            return "0".to_string();
        }

        let mut result = String::new();
        let mut quotient = x.floor() as i64;
        let mut fraction = x - quotient as f64;

        let parse_digit = |value: i64| {
            if value > 9 {
                std::char::from_u32((value as u32) + 55).expect("hex digit is valid")
            } else {
                std::char::from_digit(value as u32, 10).expect("decimal digit is valid")
            }
        };

        if quotient == 0 {
            result.push('0');
        } else {
            while quotient > 0 {
                let remainder = quotient % 16;
                quotient /= 16;

                result.insert(0, parse_digit(remainder));
            }
        }

        if fraction > 0.0 {
            result.push('.');

            while fraction > 0.0 {
                fraction *= 16.0;
                let integer = fraction.floor() as i64;
                fraction -= integer as f64;

                result.push(parse_digit(integer));

                if result.len() > 20 {
                    break;
                }
            }
        }

        result
    }

    /// Base64 encode a byte array.
    pub(super) fn base64_encode<T: AsRef<[u8]>>(data: T) -> String {
        general_purpose::STANDARD.encode(data)
    }

    /// Base64 decode a string.
    pub(super) fn base64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_is_odd() {
            assert_eq!(is_odd(1), -1.0);
            assert_eq!(is_odd(2), 0.0);
            assert_eq!(is_odd(3), -1.0);
            assert_eq!(is_odd(4), 0.0);
        }

        #[test]
        fn test_js_round() {
            assert_eq!(js_round(0.0), 0.0);
            assert_eq!(js_round(0.4), 0.0);
            assert_eq!(js_round(0.5), 1.0);
            assert_eq!(js_round(0.6), 1.0);
            assert_eq!(js_round(1.5), 2.0);

            assert_eq!(js_round(-0.0), 0.0);
            assert_eq!(js_round(-0.4), 0.0);
            assert_eq!(js_round(-0.5), -0.0);
            assert_eq!(js_round(-0.6), -1.0);
            assert_eq!(js_round(-1.5), -1.0);
        }

        #[test]
        fn test_float_to_hex() {
            assert_eq!(float_to_hex(10.0), "A");
            assert_eq!(float_to_hex(16.0), "10");
            assert_eq!(float_to_hex(0.5), "0.8");
        }

        #[test]
        fn test_base64() {
            let original = "hello world";
            let encoded = base64_encode(original);
            let decoded = base64_decode(&encoded).unwrap();
            assert_eq!(decoded, original.as_bytes());
        }
    }
}

mod transaction {
    use super::cubic_curve::Cubic;
    use super::error::Error;
    use super::interpolate::interpolate;
    use super::rotation::convert_rotation_to_matrix;
    use super::utils::{
        base64_decode, base64_encode, float_to_hex, handle_x_migration, is_odd, js_round,
    };
    use regex::Regex;
    use reqwest::blocking::Client;
    use scraper::{ElementRef, Html, Selector};
    use sha2::{Digest, Sha256};
    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    static ON_DEMAND_FILE_REGEX: OnceLock<Regex> = OnceLock::new();
    static INDICES_REGEX: OnceLock<Regex> = OnceLock::new();

    fn on_demand_file_regex() -> &'static Regex {
        ON_DEMAND_FILE_REGEX.get_or_init(|| {
            Regex::new(r#"['|\"]ondemand\.s['|\"]:\s*['|\"]([\w]*)['|\"]"#)
                .expect("ondemand regex is valid")
        })
    }

    fn indices_regex() -> &'static Regex {
        INDICES_REGEX.get_or_init(|| {
            Regex::new(r"(\(\w{1}\[(\d{1,2})\],\s*16\))+").expect("indices regex is valid")
        })
    }

    pub(crate) struct ClientTransaction {
        additional_random_number: u8,
        default_keyword: String,
        key_bytes: Vec<u8>,
        animation_key: String,
    }

    impl ClientTransaction {
        pub(crate) fn new(client: &Client) -> Result<Self, Error> {
            let home_page = handle_x_migration(client)?;

            let (row_index, key_bytes_indices) = Self::get_indices(&home_page, client)?;
            let key = Self::get_key(&home_page)?;
            let key_bytes = Self::get_key_bytes(&key)?;
            let animation_key =
                Self::get_animation_key(&key_bytes, &home_page, row_index, &key_bytes_indices)?;

            Ok(Self {
                additional_random_number: 3,
                default_keyword: String::from("obfiowerehiring"),
                key_bytes,
                animation_key,
            })
        }

        fn get_indices(home_page: &Html, client: &Client) -> Result<(usize, Vec<usize>), Error> {
            let mut key_byte_indices = Vec::new();

            let html_content = home_page.html();
            let on_demand_file = on_demand_file_regex()
                .captures(&html_content)
                .ok_or_else(|| Error::Parse("Couldn't find ondemand file".into()))?;

            let on_demand_file_url = format!(
                "https://abs.twimg.com/responsive-web/client-web/ondemand.s.{}a.js",
                on_demand_file
                    .get(1)
                    .expect("ondemand capture exists")
                    .as_str()
            );

            let on_demand_response = client.get(&on_demand_file_url).send()?;
            let on_demand_content = on_demand_response.text()?;

            for captures in indices_regex().captures_iter(&on_demand_content) {
                if let Some(index_match) = captures.get(2) {
                    if let Ok(index) = index_match.as_str().parse::<usize>() {
                        key_byte_indices.push(index);
                    }
                }
            }

            if key_byte_indices.is_empty() {
                return Err(Error::Parse("Couldn't get KEY_BYTE indices".into()));
            }

            Ok((key_byte_indices[0], key_byte_indices[1..].to_vec()))
        }

        fn get_key(page: &Html) -> Result<String, Error> {
            let selector = Selector::parse("[name='twitter-site-verification']")
                .expect("verification selector is valid");
            let element = page
                .select(&selector)
                .next()
                .ok_or_else(|| Error::MissingKey("Couldn't get key from the page source".into()))?;

            let key = element
                .value()
                .attr("content")
                .ok_or_else(|| Error::MissingKey("Missing content attribute".into()))?;

            Ok(key.to_string())
        }

        fn get_key_bytes(key: &str) -> Result<Vec<u8>, Error> {
            Ok(base64_decode(key)?)
        }

        fn get_frames(page: &Html) -> Vec<ElementRef<'_>> {
            let selector =
                Selector::parse("[id^='loading-x-anim']").expect("animation selector is valid");
            page.select(&selector).collect()
        }

        fn get_2d_array(
            key_bytes: &[u8],
            page: &Html,
            frames: Option<Vec<ElementRef<'_>>>,
        ) -> Result<Vec<Vec<i32>>, Error> {
            let frames = frames.unwrap_or_else(|| Self::get_frames(page));

            let frame_index = (key_bytes[5] % 4) as usize;
            if frame_index >= frames.len() {
                return Err(Error::Parse("Invalid frame index".into()));
            }

            let frame = frames[frame_index];

            let mut outer_children = frame.children();
            let first_child = outer_children
                .next()
                .ok_or_else(|| Error::Parse("No first child in frame".into()))?;
            let first_child = ElementRef::wrap(first_child)
                .ok_or_else(|| Error::Parse("First child is not an element".into()))?;

            let mut inner_children = first_child.children();
            let path_node = inner_children
                .nth(1)
                .ok_or_else(|| Error::Parse("No second child in an inner group".into()))?;
            let path_elem = ElementRef::wrap(path_node)
                .ok_or_else(|| Error::Parse("Second child is not an element".into()))?;

            let d_attr = path_elem
                .value()
                .attr("d")
                .ok_or_else(|| Error::Parse("Missing 'd' attribute".into()))?;
            let d_content = d_attr
                .get(9..)
                .ok_or_else(|| Error::Parse("Path data too short".into()))?;

            let segments = d_content.split('C');

            let mut result = Vec::new();
            for segment in segments {
                let numbers: Vec<i32> = segment
                    .replace(|c: char| !c.is_ascii_digit() && c != '-', " ")
                    .split_whitespace()
                    .filter_map(|value| value.parse::<i32>().ok())
                    .collect();

                result.push(numbers);
            }

            Ok(result)
        }

        fn solve(value: f64, min_val: f64, max_val: f64, rounding: bool) -> f64 {
            let result = value * (max_val - min_val) / 255.0 + min_val;
            if rounding {
                result.floor()
            } else {
                (result * 100.0).round() / 100.0
            }
        }

        fn animate(frames: &[i32], target_time: f64) -> String {
            let from_color: Vec<f64> = frames[..3]
                .iter()
                .map(|&value| value as f64)
                .chain(std::iter::once(1.0))
                .collect();
            let to_color: Vec<f64> = frames[3..6]
                .iter()
                .map(|&value| value as f64)
                .chain(std::iter::once(1.0))
                .collect();
            let from_rotation = vec![0.0];
            let to_rotation = vec![Self::solve(frames[6] as f64, 60.0, 360.0, true)];

            let curves: Vec<f64> = frames[7..]
                .iter()
                .enumerate()
                .map(|(idx, &value)| Self::solve(value as f64, is_odd(idx as i32), 1.0, false))
                .collect();

            let cubic = Cubic::new(curves);
            let value = cubic.get_value(target_time);

            let color = interpolate(&from_color, &to_color, value).expect("color lengths match");
            let color: Vec<f64> = color.iter().map(|&value| value.clamp(0.0, 255.0)).collect();

            let rotation =
                interpolate(&from_rotation, &to_rotation, value).expect("rotation lengths match");
            let matrix = convert_rotation_to_matrix(rotation[0]);

            let mut str_arr = Vec::new();

            for value in &color[..color.len() - 1] {
                str_arr.push(format!("{:x}", value.round() as i32));
            }

            for value in matrix {
                let rounded = (value * 100.0).round() / 100.0;
                let abs_value = rounded.abs();
                let hex_value = float_to_hex(abs_value);

                if hex_value.starts_with('.') {
                    str_arr.push(format!("0{}", hex_value.to_lowercase()));
                } else if hex_value.is_empty() {
                    str_arr.push("0".to_string());
                } else {
                    str_arr.push(hex_value.to_lowercase());
                }
            }

            str_arr.push("0".to_string());
            str_arr.push("0".to_string());

            let animation_key = str_arr.join("");
            animation_key.replace(['.', '-'], "")
        }

        fn get_animation_key(
            key_bytes: &[u8],
            page: &Html,
            row_index: usize,
            key_bytes_indices: &[usize],
        ) -> Result<String, Error> {
            let total_time = 4096.0;

            let row_index_value = (key_bytes[row_index] % 16) as usize;

            let frame_time = key_bytes_indices
                .iter()
                .map(|&index| (key_bytes[index] % 16) as f64)
                .fold(1.0, |acc, value| acc * value);

            let frame_time = js_round(frame_time / 10.0) * 10.0;

            let arr = Self::get_2d_array(key_bytes, page, None)?;

            if row_index_value >= arr.len() {
                return Err(Error::Parse("Invalid row index".into()));
            }

            let frame_row = &arr[row_index_value];

            let target_time = frame_time / total_time;
            let animation_key = Self::animate(frame_row, target_time);

            Ok(animation_key)
        }

        pub(crate) fn generate_transaction_id(
            &self,
            method: &str,
            path: &str,
        ) -> Result<String, Error> {
            let time_now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is after unix epoch")
                .as_secs()
                .saturating_sub(1682924400) as u32;

            let time_now_bytes = [
                (time_now & 0xFF) as u8,
                ((time_now >> 8) & 0xFF) as u8,
                ((time_now >> 16) & 0xFF) as u8,
                ((time_now >> 24) & 0xFF) as u8,
            ];

            let hash_input = format!(
                "{}!{}!{}{}{}",
                method, path, time_now, self.default_keyword, self.animation_key
            );

            let mut hasher = Sha256::new();
            hasher.update(hash_input.as_bytes());
            let hash_result = hasher.finalize();

            let hash_bytes: Vec<u8> = hash_result[..16].to_vec();
            let random_num = rand::random::<u8>();

            let mut bytes_arr = Vec::with_capacity(
                self.key_bytes.len() + time_now_bytes.len() + hash_bytes.len() + 1,
            );
            bytes_arr.extend_from_slice(&self.key_bytes);
            bytes_arr.extend_from_slice(&time_now_bytes);
            bytes_arr.extend_from_slice(&hash_bytes);
            bytes_arr.push(self.additional_random_number);

            let mut out = vec![random_num];
            out.extend(bytes_arr.iter().map(|&byte| byte ^ random_num));

            let encoded = base64_encode(&out);
            let result = encoded.trim_end_matches('=');

            Ok(result.to_string())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use reqwest::blocking::Client;

        #[test]
        #[ignore = "Ignoring network-dependent test"]
        fn test_transaction_id_generation() -> Result<(), Error> {
            let client = Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36")
                .build()?;

            let transaction = ClientTransaction::new(&client)?;
            let transaction_id =
                transaction.generate_transaction_id("GET", "/i/api/1.1/jot/client_event.json")?;

            assert!(!transaction_id.is_empty());

            Ok(())
        }
    }
}
