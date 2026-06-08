/// Normalize phone number to E.164 format
/// Strips spaces, dashes, brackets. Prepends +91 for 10-digit Indian numbers.
pub fn normalize_phone(input: &str) -> Option<String> {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();

    let cleaned = digits.replace(' ', "");

    if cleaned.starts_with('+') {
        // Already has country code
        if cleaned.len() >= 8 && cleaned.len() <= 16 {
            return Some(cleaned);
        }
    } else if cleaned.len() == 10 {
        // Indian 10-digit number
        return Some(format!("+91{}", cleaned));
    } else if cleaned.len() == 12 && cleaned.starts_with("91") {
        // 91XXXXXXXXXX
        return Some(format!("+{}", cleaned));
    }

    None
}

/// Check if a string looks like a valid E.164 phone number
pub fn is_valid_e164(phone: &str) -> bool {
    phone.starts_with('+')
        && phone.len() >= 8
        && phone.len() <= 16
        && phone[1..].chars().all(|c| c.is_ascii_digit())
}
