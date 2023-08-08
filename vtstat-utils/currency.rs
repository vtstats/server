pub fn currency_symbol_to_code(i: &str) -> Option<&str> {
    match i {
        "$" => Some("USD"),
        "€" => Some("EUR"),
        "¥" => Some("JPY"),
        "£" => Some("GBP"),
        "A$" => Some("AUD"),
        "CA$" => Some("CAD"),
        "HK$" => Some("HKD"),
        "NZ$" => Some("NZD"),
        "₩" => Some("KRW"),
        "MX$" => Some("MXN"),
        "₹" => Some("INR"),
        "R$" => Some("BRL"),
        "NT$" => Some("TWD"),
        "₪" => Some("ILS"),
        "₱" => Some("PHP"),
        // "F CFA" => Some("CFA"),
        i if i.len() == 3 && i.bytes().all(|c| c.is_ascii_uppercase()) => Some(i),
        _ => None,
    }
}
